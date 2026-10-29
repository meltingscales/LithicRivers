//! Corpse selection and corpse looting modals.

use crate::app_state::*;
use crate::ui::centered_rect;
use crate::App;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Render corpse looting modals when active
pub(crate) fn render_corpse_looting_modals(f: &mut Frame, app: &mut App) {
    let looting_state = app.panels.corpse_looting.clone();
    match looting_state {
        CorpseLootingState::SelectingCorpse {
            adjacent_entities,
            selected_corpse,
        } => {
            render_corpse_selection_modal(f, app, &adjacent_entities, selected_corpse);
        }
        CorpseLootingState::LootingCorpse {
            entity,
            selected_loot_item,
            selected_player_item,
            loot_panel_focus,
        } => {
            render_corpse_loot_modal(
                f,
                app,
                entity,
                selected_loot_item,
                selected_player_item,
                loot_panel_focus,
            );
        }
        CorpseLootingState::None => {
            // No modal to render
        }
    }
}

/// Render the corpse selection modal when multiple corpses are nearby
fn render_corpse_selection_modal(
    f: &mut Frame,
    app: &mut App,
    adjacent_entities: &[hecs::Entity],
    selected_corpse: usize,
) {
    use lithicrivers_core::components::{EntityKind, Inventory as InvComp};
    use ratatui::{
        style::Modifier,
        text::{Line, Span},
        widgets::{Clear, List, ListItem},
    };

    // Create modal area (centered, 50% width, 40% height)
    let area = centered_rect(50, 40, f.area());

    // Clear the background
    f.render_widget(Clear, area);

    // Main modal block
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Line::from(" Choose Corpse to Loot "))
        .title_alignment(Alignment::Center)
        .style(Style::default());

    let inner = block.inner(area);
    f.render_widget(block, area);
    app.ui
        .click_targets
        .push(area, crate::ui::click_targets::ClickTarget::ModalArea);

    // Create list of corpses
    let corpse_items: Vec<ListItem> = adjacent_entities
        .iter()
        .enumerate()
        .filter_map(|(i, &entity)| {
            // Get corpse info
            if let (Ok(entity_kind), Ok(inv)) = (
                app.core.game.world.get::<&EntityKind>(entity),
                app.core.game.world.get::<&InvComp>(entity),
            ) {
                if *entity_kind == EntityKind::Corpse {
                    let is_selected = selected_corpse == i;
                    let style = if is_selected {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };

                    // Show corpse name and item count
                    let item_count = inv.slots.len();
                    let text = format!("Corpse ({} items)", item_count);

                    return Some(ListItem::new(Line::from(Span::styled(text, style))));
                }
            }
            None
        })
        .collect();

    let list =
        List::new(corpse_items).highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_widget(list, inner);
    app.ui.click_targets.rows(
        inner,
        0,
        adjacent_entities.len(),
        crate::ui::click_targets::ClickTarget::CorpsePickerRow,
    );

    // Controls at bottom
    let controls = Paragraph::new("↑↓: Select | Enter: Loot | Esc: Cancel")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));

    if inner.height > 1 {
        let controls_area = Rect {
            x: inner.x,
            y: inner.y + inner.height - 1,
            width: inner.width,
            height: 1,
        };
        f.render_widget(controls, controls_area);
    }
}

/// Render the corpse looting modal with inventory transfer interface
fn render_corpse_loot_modal(
    f: &mut Frame,
    app: &mut App,
    corpse_entity: hecs::Entity,
    selected_loot_item: usize,
    selected_player_item: usize,
    loot_panel_focus: bool,
) {
    use lithicrivers_core::components::{itemkind_name, Inventory as InvComp};
    use ratatui::{
        style::Modifier,
        text::{Line, Span},
        widgets::{Clear, List, ListItem},
    };

    // Get corpse inventory
    let corpse_inv = if let Ok(inv) = app.core.game.world.get::<&InvComp>(corpse_entity) {
        inv.clone()
    } else {
        return; // Corpse no longer exists
    };

    // Get player inventory
    let player_inv = if let Some(player_entity) = app.core.game.get_player_entity() {
        if let Ok(inv) = app.core.game.world.get::<&InvComp>(player_entity) {
            inv.clone()
        } else {
            return; // Player no longer exists
        }
    } else {
        return;
    };

    // Chests and generators share this window, with items moving both ways
    let block = app
        .core
        .game
        .world
        .get::<&lithicrivers_core::factory::FactoryBlock>(corpse_entity)
        .ok()
        .map(|b| b.kind);
    let is_chest = block.is_some();
    use lithicrivers_core::factory::FactoryKind;
    let (title, items_title) = match block {
        Some(FactoryKind::BurnerGenerator) => {
            let buffer_kj = app
                .core
                .game
                .world
                .get::<&lithicrivers_core::power::Generator>(corpse_entity)
                .map_or(0, |g| g.buffer_j / 1000);
            let network = app.core.game.res.power.network_of(corpse_entity);
            let grid = match network {
                Some(n) => format!("network {} / {} kW", n.supply_w / 1000, n.demand_w / 1000),
                None => "no power pole in reach".to_string(),
            };
            (
                format!(" Burner Generator - {grid} "),
                format!(
                    " Fuel ({}/{}) - {buffer_kj} kJ left ",
                    corpse_inv.slots.len(),
                    corpse_inv.max_slots
                ),
            )
        }
        Some(kind @ (FactoryKind::AutoPlanter | FactoryKind::AutoHarvester)) => {
            use lithicrivers_core::machines::{action_cost, Machine, MACHINE_CAPACITY};
            let world = &app.core.game.world;
            let progress = world
                .get::<&Machine>(corpse_entity)
                .map_or(0, |m| m.progress * 100 / action_cost(kind));
            let consumer = world
                .get::<&lithicrivers_core::power::PowerConsumer>(corpse_entity)
                .ok()
                .map(|c| *c);
            let powered = app.core.game.res.power.network_of(corpse_entity).is_some();
            let status = match consumer {
                Some(c) if c.active && powered => format!(
                    "working {progress}%, power {}%",
                    c.satisfaction_permille / 10
                ),
                Some(c) if c.active => "no power pole in reach".to_string(),
                _ => "idle".to_string(),
            };
            let facing = world
                .get::<&lithicrivers_core::factory::FactoryBlock>(corpse_entity)
                .map(|b| b.facing.name())
                .unwrap_or("north");
            let held: u32 = corpse_inv.slots.iter().map(|s| s.qty).sum();
            if kind == FactoryKind::AutoPlanter {
                (
                    format!(" Auto-Planter - {status} "),
                    format!(" Acorns ({held}/{MACHINE_CAPACITY}) "),
                )
            } else {
                (
                    format!(" Auto-Harvester facing {facing} - {status} "),
                    format!(" Output ({held}/{MACHINE_CAPACITY}) "),
                )
            }
        }
        Some(_) => (
            " Chest ".to_string(),
            format!(
                " Chest Items ({}/{}) ",
                corpse_inv.slots.len(),
                corpse_inv.max_slots
            ),
        ),
        None => (" Looting Corpse ".to_string(), " Corpse Items ".to_string()),
    };

    // Create modal area (centered, 70% width, 60% height)
    let area = centered_rect(70, 60, f.area());

    // Clear the background
    f.render_widget(Clear, area);

    // Main modal block
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Line::from(title))
        .title_alignment(Alignment::Center)
        .style(Style::default());

    let inner = block.inner(area);
    f.render_widget(block, area);
    app.ui
        .click_targets
        .push(area, crate::ui::click_targets::ClickTarget::ModalArea);

    // Split into two columns
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);

    // Left side: Corpse inventory
    let corpse_block = Block::default()
        .borders(Borders::ALL)
        .title(Line::from(items_title))
        .border_style(if loot_panel_focus {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });

    let corpse_items: Vec<ListItem> = corpse_inv
        .slots
        .iter()
        .enumerate()
        .map(|(i, stack)| {
            let is_selected = loot_panel_focus && selected_loot_item == i;
            let style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(Span::styled(
                format!("{:2} x {}", stack.qty, itemkind_name(stack.kind)),
                style,
            )))
        })
        .collect();

    let container_rows = corpse_inv.slots.len();
    let corpse_list = List::new(corpse_items).block(corpse_block);
    f.render_widget(corpse_list, chunks[0]);
    app.ui
        .click_targets
        .rows(inset(chunks[0]), 0, container_rows, |row| {
            crate::ui::click_targets::ClickTarget::TransferRow {
                container: true,
                row,
            }
        });

    // Right side: the player's items, only what a machine takes
    let rows = app.core.game.get_player_entity().map(|p| {
        crate::app::item_filter::PlayerRows::for_container(&app.core.game.world, p, corpse_entity)
    });
    let shown: Vec<usize> = rows.as_ref().map_or_else(Vec::new, |r| r.slots.clone());
    let player_title = rows
        .as_ref()
        .map_or_else(|| " Your Items ".to_string(), |r| r.title());
    let player_block = Block::default()
        .borders(Borders::ALL)
        .title(Line::from(player_title))
        .border_style(if !loot_panel_focus {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });

    let player_items: Vec<ListItem> = shown
        .iter()
        .filter_map(|&slot| player_inv.slots.get(slot))
        .enumerate()
        .map(|(i, stack)| {
            let is_selected = !loot_panel_focus && selected_player_item == i;
            let style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(Span::styled(
                format!("{:2} x {}", stack.qty, itemkind_name(stack.kind)),
                style,
            )))
        })
        .collect();

    let player_rows = player_items.len();
    let player_list = List::new(player_items).block(player_block);
    f.render_widget(player_list, chunks[1]);
    app.ui
        .click_targets
        .rows(inset(chunks[1]), 0, player_rows, |row| {
            crate::ui::click_targets::ClickTarget::TransferRow {
                container: false,
                row,
            }
        });

    // Controls at bottom
    let take_all = app
        .core
        .config_manager
        .get_printable_key_for_keybind("ui", "TAKE_ALL");
    let controls = Paragraph::new(if is_chest {
        format!("←→ Panel | ↑↓ Select | Enter Move | {take_all} Take All | Esc Close")
    } else {
        format!("←→ Panel | ↑↓ Select | Enter Take | {take_all} Take All | Esc Close")
    })
    .alignment(Alignment::Center)
    .style(Style::default().fg(Color::Gray));

    if inner.height > 1 {
        let controls_area = Rect {
            x: inner.x,
            y: inner.y + inner.height - 1,
            width: inner.width,
            height: 1,
        };
        f.render_widget(controls, controls_area);
    }
}

/// The inside of a bordered box.
fn inset(rect: Rect) -> Rect {
    Rect::new(
        rect.x + 1,
        rect.y + 1,
        rect.width.saturating_sub(2),
        rect.height.saturating_sub(2),
    )
}

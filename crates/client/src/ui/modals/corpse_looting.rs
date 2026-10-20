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
    let area = centered_rect(50, 40, f.size());

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

    // Chests share this window, with items moving both ways
    let is_chest = app
        .core
        .game
        .world
        .get::<&lithicrivers_core::factory::FactoryBlock>(corpse_entity)
        .is_ok();
    let (title, items_title) = if is_chest {
        (
            " Chest ".to_string(),
            format!(
                " Chest Items ({}/{}) ",
                corpse_inv.slots.len(),
                corpse_inv.max_slots
            ),
        )
    } else {
        (" Looting Corpse ".to_string(), " Corpse Items ".to_string())
    };

    // Create modal area (centered, 70% width, 60% height)
    let area = centered_rect(70, 60, f.size());

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

    let corpse_list = List::new(corpse_items).block(corpse_block);
    f.render_widget(corpse_list, chunks[0]);

    // Right side: Player inventory
    let player_block = Block::default()
        .borders(Borders::ALL)
        .title(Line::from(" Your Items "))
        .border_style(if !loot_panel_focus {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });

    let player_items: Vec<ListItem> = player_inv
        .slots
        .iter()
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

    let player_list = List::new(player_items).block(player_block);
    f.render_widget(player_list, chunks[1]);

    // Controls at bottom
    let controls = Paragraph::new(if is_chest {
        "←→: Switch Panel | ↑↓: Select | Enter: Move Item | Esc: Close"
    } else {
        "←→: Switch Panel | ↑↓: Select | Enter: Take Item | Esc: Close"
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

//! The furnace window: its slots (fuel, for a crude furnace), input and output, with smelting
//! progress and fuel energy or power, next to the player's items.

use crate::ui::centered_rect;
use crate::App;
use lithicrivers_core::components::{itemkind_name, Inventory};
use lithicrivers_core::factory::FactoryBlock;
use lithicrivers_core::furnace::{self, Furnace, Slot};
use lithicrivers_core::power::PowerConsumer;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

fn list<'a>(title: String, rows: Vec<String>, focused: bool) -> List<'a> {
    let border = if focused {
        Color::Yellow
    } else {
        Color::DarkGray
    };
    List::new(rows.into_iter().map(ListItem::new).collect::<Vec<_>>())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(border)),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
}

pub(crate) fn render_furnace_window(f: &mut Frame, app: &App) {
    let Some(window) = app.panels.furnace_window else {
        return;
    };
    let world = &app.core.game.world;
    let Ok(furnace) = world.get::<&Furnace>(window.entity).map(|f| *f) else {
        return;
    };
    let facing = world
        .get::<&FactoryBlock>(window.entity)
        .map_or("north", |b| b.facing.name());
    let electric = furnace::is_electric(window.kind);
    let status = match furnace.ready_recipe() {
        Some(_) if furnace.lit => format!("smelting {}%", furnace.progress_percent()),
        Some(_) if electric => "no power".to_string(),
        Some(_) => "out of fuel".to_string(),
        None => "idle".to_string(),
    };
    let slots = Slot::for_kind(window.kind);
    // Only what the selected slot takes
    let player = app.core.game.get_player_entity();
    let rows = player
        .map(|p| crate::app::item_filter::PlayerRows::for_furnace_slot(world, p, window.slot));
    let stacks = player
        .and_then(|p| world.get::<&Inventory>(p).ok().map(|inv| inv.slots.clone()))
        .unwrap_or_default();
    let player_items: Vec<String> = rows
        .as_ref()
        .map_or_else(Vec::new, |r| r.slots.clone())
        .into_iter()
        .filter_map(|i| stacks.get(i))
        .map(|s| format!("{:>2} x {}", s.qty, itemkind_name(s.kind)))
        .collect();
    let player_title = rows.map_or_else(|| " Your Items ".to_string(), |r| r.title());
    let slot_rows: Vec<String> = slots
        .iter()
        .map(|&slot| {
            let contents = furnace.slot(slot).map_or("(empty)".to_string(), |s| {
                format!("{} x {}", s.qty, itemkind_name(s.kind))
            });
            format!("{:<7}{contents}", format!("{}:", slot.name()))
        })
        .collect();

    let area = centered_rect(70, 60, f.size());
    f.render_widget(Clear, area);
    let outer = Block::default()
        .borders(Borders::ALL)
        .title(format!(
            " {} facing {facing} - {status} ",
            window.kind.item().map_or("Furnace", itemkind_name)
        ))
        .title_alignment(Alignment::Center);
    let inner = outer.inner(area);
    f.render_widget(outer, area);
    app.ui
        .click_targets
        .push(area, crate::ui::click_targets::ClickTarget::ModalArea);
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(2)])
        .split(inner);
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[0]);

    let slot_index = slots.iter().position(|&s| s == window.slot).unwrap_or(0);
    let mut slot_state = ListState::default();
    slot_state.select(Some(slot_index));
    let slots_title = if electric {
        let power = world
            .get::<&PowerConsumer>(window.entity)
            .map_or(0, |c| c.satisfaction_permille / 10);
        format!(" Slots - power {power}% ")
    } else {
        format!(" Slots - {} kJ of fuel ", furnace.energy_left_j() / 1000)
    };
    f.render_stateful_widget(
        list(slots_title, slot_rows, !window.player_focus),
        columns[0],
        &mut slot_state,
    );
    let first = slot_state.offset();
    app.ui.click_targets.rows(
        crate::ui::click_targets::inset(columns[0]),
        first,
        slots.len().saturating_sub(first),
        crate::ui::click_targets::ClickTarget::FurnaceSlot,
    );
    let player_count = player_items.len();
    let mut player_state = ListState::default();
    player_state.select((!player_items.is_empty()).then_some(window.player_item));
    f.render_stateful_widget(
        list(player_title, player_items, window.player_focus),
        columns[1],
        &mut player_state,
    );
    let first = player_state.offset();
    app.ui.click_targets.rows(
        crate::ui::click_targets::inset(columns[1]),
        first,
        player_count.saturating_sub(first),
        crate::ui::click_targets::ClickTarget::FurnacePlayerRow,
    );
    let key = |c, a| app.core.config_manager.get_printable_key_for_keybind(c, a);
    let hint = format!(
        "{}/{}: panel  {}/{}: select  {}: move one item  {}: close",
        key("movement", "MOVE_WEST"),
        key("movement", "MOVE_EAST"),
        key("movement", "MOVE_NORTH"),
        key("movement", "MOVE_SOUTH"),
        key("ui", "MENU_ACTIVATE"),
        key("ui", "CLOSE_HELP_MENU"),
    );
    f.render_widget(
        Paragraph::new(Line::from(hint))
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center),
        rows[1],
    );
}

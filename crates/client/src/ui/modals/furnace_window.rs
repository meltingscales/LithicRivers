//! The furnace window: fuel, input and output slots with smelting progress and fuel energy,
//! next to the player's items.

use crate::ui::centered_rect;
use crate::App;
use lithicrivers_core::components::{itemkind_name, Inventory};
use lithicrivers_core::factory::FactoryBlock;
use lithicrivers_core::furnace::{Furnace, Slot};
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
    let status = match furnace.ready_recipe() {
        Some(recipe) if furnace.lit => format!(
            "smelting {}%",
            furnace.progress * 100 / recipe.steps().max(1)
        ),
        Some(_) => "out of fuel".to_string(),
        None => "idle".to_string(),
    };
    let player_items: Vec<String> = app
        .core
        .game
        .get_player_entity()
        .and_then(|p| world.get::<&Inventory>(p).ok().map(|inv| inv.slots.clone()))
        .unwrap_or_default()
        .iter()
        .map(|s| format!("{:>2} x {}", s.qty, itemkind_name(s.kind)))
        .collect();
    let slot_rows: Vec<String> = Slot::ALL
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
        .title(format!(" Crude Furnace facing {facing} - {status} "))
        .title_alignment(Alignment::Center);
    let inner = outer.inner(area);
    f.render_widget(outer, area);
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(2)])
        .split(inner);
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[0]);

    let slot_index = Slot::ALL
        .iter()
        .position(|&s| s == window.slot)
        .unwrap_or(0);
    let mut slot_state = ListState::default();
    slot_state.select(Some(slot_index));
    let slots_title = format!(" Slots - {} kJ of fuel ", furnace.energy_left_j() / 1000);
    f.render_stateful_widget(
        list(slots_title, slot_rows, !window.player_focus),
        columns[0],
        &mut slot_state,
    );
    let mut player_state = ListState::default();
    player_state.select((!player_items.is_empty()).then_some(window.player_item));
    f.render_stateful_widget(
        list(
            " Your Items ".to_string(),
            player_items,
            window.player_focus,
        ),
        columns[1],
        &mut player_state,
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

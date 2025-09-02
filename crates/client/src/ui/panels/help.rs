use crate::input::format_keycode;
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::collections::HashMap;

pub fn render_help_panel(f: &mut Frame, app: &mut crate::App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(Span::styled(
        "Controls",
        Style::default().fg(Color::Cyan),
    )));
    lines.push(Line::from(""));
    let movement_keys_list = vec![
        app.config_manager
            .get_printable_key_for_keybind("movement", "MOVE_NORTHWEST"),
        app.config_manager
            .get_printable_key_for_keybind("movement", "MOVE_NORTH"),
        app.config_manager
            .get_printable_key_for_keybind("movement", "MOVE_NORTHEAST"),
        app.config_manager
            .get_printable_key_for_keybind("movement", "MOVE_WEST"),
        app.config_manager
            .get_printable_key_for_keybind("movement", "WAIT"),
        app.config_manager
            .get_printable_key_for_keybind("movement", "MOVE_EAST"),
        app.config_manager
            .get_printable_key_for_keybind("movement", "MOVE_SOUTHWEST"),
        app.config_manager
            .get_printable_key_for_keybind("movement", "MOVE_SOUTH"),
        app.config_manager
            .get_printable_key_for_keybind("movement", "MOVE_SOUTHEAST"),
    ];
    // custom movement help rendering
    lines.push(Line::from(Span::raw(
        "Movement Directions: Numpad by default",
    )));
    lines.push(Line::from(Span::raw("")));
    lines.push(Line::from(Span::raw(" NW N NE ")));
    lines.push(Line::from(Span::raw(" W  .  E ")));
    lines.push(Line::from(Span::raw(" SW S SE ")));
    lines.push(Line::from(Span::raw("")));
    lines.push(Line::from(Span::raw("Movement Keys: ")));
    lines.push(Line::from(format!(
        " {}  {}  {} ",
        movement_keys_list[0], movement_keys_list[1], movement_keys_list[2]
    )));
    lines.push(Line::from(format!(
        " {}  {}  {} ",
        movement_keys_list[3], movement_keys_list[4], movement_keys_list[5]
    )));
    lines.push(Line::from(format!(
        " {}  {}  {} ",
        movement_keys_list[6], movement_keys_list[7], movement_keys_list[8]
    )));
    lines.push(Line::from(Span::raw("")));
    // Group keybinds by category
    let mut categorized_binds: HashMap<String, Vec<(String, Vec<KeyCode>)>> = HashMap::new();
    for (key, value) in &app.keybinds.map {
        let parts: Vec<&str> = key.split(':').collect();
        if parts.len() == 2 {
            let category = parts[0].to_string();
            let action = parts[1].to_string();
            categorized_binds
                .entry(category)
                .or_default()
                .push((action, value.clone()));
        }
    }
    // Define the order of categories
    let categories = vec!["viewport", "scale", "action", "ui", "inventory"];
    for category in categories {
        if let Some(binds) = categorized_binds.get(category) {
            let category_title = category.replace('_', " ");
            lines.push(Line::from(Span::raw(format!(
                "{} ",
                heck::AsTitleCase(&category_title)
            ))));
            for (action, keys) in binds {
                let key_str = keys
                    .iter()
                    .map(|k| format_keycode(k))
                    .collect::<Vec<_>>()
                    .join(", ");
                lines.push(Line::from(Span::raw(format!("  {}: {}", action, key_str))));
            }
            lines.push(Line::from(""));
        }
    }
    let block = Block::default().borders(Borders::ALL).title("Help");
    let _inner = block.inner(area);
    let p = Paragraph::new(lines)
        .alignment(Alignment::Left)
        .block(block);
    f.render_widget(p, area);
}

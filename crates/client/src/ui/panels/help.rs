use crossterm::event::KeyCode;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use std::collections::HashMap;

/// Format a KeyCode as a human-readable string
pub fn format_keycode(kc: &KeyCode) -> String {
    match kc {
        KeyCode::Char(c) => c.to_string(),
        KeyCode::BackTab => "Shift+Tab".to_string(),
        KeyCode::Backspace => "Backspace".to_string(),
        KeyCode::Tab => "Tab".to_string(),
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Esc => "Esc".to_string(),
        _ => format!("{:?}", kc),
    }
}

pub fn render_help_panel(f: &mut Frame, app: &mut crate::App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(Span::styled(
        "Controls",
        Style::default().fg(Color::Cyan),
    )));
    lines.push(Line::from(""));
    let movement_keys_list = vec![
        app.core
            .config_manager
            .get_printable_key_for_keybind("movement", "MOVE_NORTHWEST"),
        app.core
            .config_manager
            .get_printable_key_for_keybind("movement", "MOVE_NORTH"),
        app.core
            .config_manager
            .get_printable_key_for_keybind("movement", "MOVE_NORTHEAST"),
        app.core
            .config_manager
            .get_printable_key_for_keybind("movement", "MOVE_WEST"),
        app.core
            .config_manager
            .get_printable_key_for_keybind("movement", "WAIT"),
        app.core
            .config_manager
            .get_printable_key_for_keybind("movement", "MOVE_EAST"),
        app.core
            .config_manager
            .get_printable_key_for_keybind("movement", "MOVE_SOUTHWEST"),
        app.core
            .config_manager
            .get_printable_key_for_keybind("movement", "MOVE_SOUTH"),
        app.core
            .config_manager
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

    // Build mode keys visual layout
    let build_keys_list = vec![
        app.core
            .config_manager
            .get_printable_key_for_keybind("build", "BREAK_NORTHWEST"),
        app.core
            .config_manager
            .get_printable_key_for_keybind("build", "BREAK_NORTH"),
        app.core
            .config_manager
            .get_printable_key_for_keybind("build", "BREAK_NORTHEAST"),
        app.core
            .config_manager
            .get_printable_key_for_keybind("build", "BREAK_WEST"),
        app.core
            .config_manager
            .get_printable_key_for_keybind("build", "BREAK_CENTER"),
        app.core
            .config_manager
            .get_printable_key_for_keybind("build", "BREAK_EAST"),
        app.core
            .config_manager
            .get_printable_key_for_keybind("build", "BREAK_SOUTHWEST"),
        app.core
            .config_manager
            .get_printable_key_for_keybind("build", "BREAK_SOUTH"),
        app.core
            .config_manager
            .get_printable_key_for_keybind("build", "BREAK_SOUTHEAST"),
    ];
    lines.push(Line::from(Span::raw(format!(
        "Build Mode Keys: Use '{}' to cycle Movement --> Break --> Place",
        app.core
            .config_manager
            .get_printable_key_for_keybind("build", "TOGGLE_BREAK_PLACE_MODE")
    ))));
    lines.push(Line::from(Span::raw("")));
    lines.push(Line::from(Span::raw(" NW N NE ")));
    lines.push(Line::from(Span::raw(" W  .  E ")));
    lines.push(Line::from(Span::raw(" SW S SE ")));
    lines.push(Line::from(Span::raw("")));
    lines.push(Line::from(Span::raw("Break/Place Keys: ")));
    lines.push(Line::from(format!(
        " {}  {}  {} ",
        build_keys_list[0], build_keys_list[1], build_keys_list[2]
    )));
    lines.push(Line::from(format!(
        " {}  {}  {} ",
        build_keys_list[3], build_keys_list[4], build_keys_list[5]
    )));
    lines.push(Line::from(format!(
        " {}  {}  {} ",
        build_keys_list[6], build_keys_list[7], build_keys_list[8]
    )));
    lines.push(Line::from(Span::raw("")));
    // Group keybinds by category
    let mut categorized_binds: HashMap<String, Vec<(String, Vec<KeyCode>)>> = HashMap::new();
    for (key, value) in app.ui.keybinds.get_map() {
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
    let categories = vec![
        "viewport",
        "scale",
        "action",
        "build",
        "hotbar",
        "ui",
        "inventory",
    ];
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
    let inner = block.inner(area);
    let p = Paragraph::new(lines)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false })
        .scroll((app.panels.help.scroll, 0));
    f.render_widget(p, inner);
    f.render_widget(block, area);
}

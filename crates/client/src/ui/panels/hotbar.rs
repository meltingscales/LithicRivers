use crate::App;
use lithicrivers_core::components::{itemkind_sprite_name, ItemKind};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_hotbar_panel(f: &mut Frame, app: &mut App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();

    // Create hotbar slots (F1-F12)
    let mut slot_spans = Vec::new();

    for i in 0..12 {
        let slot_number = i + 1;
        let is_selected = app.panels.build.selected_hotbar_slot == i;

        // Style for selected vs unselected slots
        let style = if is_selected {
            Style::default().fg(Color::Yellow).bg(Color::DarkGray)
        } else {
            Style::default().fg(Color::White).bg(Color::Black)
        };

        // Create slot display (F1, F2, etc.)
        // let slot_text = format!("F{}", slot_number);
        let slot_text = app
            .core
            .config_manager
            .get_keybind_str("hotbar", &format!("HOTBAR_SLOT_{}", slot_number))
            .to_string();

        // Get assigned block content
        let content = if let Some(block_kind) = app.panels.build.hotbar_assignments[i] {
            format!("[{}]", item_glyph(app, block_kind))
        } else {
            "[ ]".to_string() // Empty slot
        };

        slot_spans.push(Span::styled(format!("{}{}", slot_text, content), style));

        // Add spacing between slots except for the last one
        if i < 11 {
            slot_spans.push(Span::raw(" "));
        }
    }

    // Create the hotbar line
    lines.push(Line::from(slot_spans));

    let content = Paragraph::new(lines)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Line::from(format!(
                    "Hotbar ({}-{}) | {} ({} to turn)",
                    app.core
                        .config_manager
                        .get_keybind_str("hotbar", "HOTBAR_SLOT_1"),
                    app.core
                        .config_manager
                        .get_keybind_str("hotbar", "HOTBAR_SLOT_12"),
                    placement_label(app),
                    app.core
                        .config_manager
                        .get_printable_key_for_keybind("build", "ROTATE_PLACEMENT")
                )))
                .style(Style::default().fg(Color::White)),
        );
    f.render_widget(content, area);
}

/// Which way the next placement goes: the facing, or for stairs, up or down.
fn placement_label(app: &crate::App) -> String {
    let build = &app.panels.build;
    if build.hotbar_assignments[build.selected_hotbar_slot]
        == Some(lithicrivers_core::components::ItemKind::Stairs)
    {
        return if build.stairs_down {
            "stairs down".to_string()
        } else {
            "stairs up".to_string()
        };
    }
    format!(
        "facing {} {}",
        build.placement_facing.arrow(),
        build.placement_facing.name()
    )
}

/// An item's 1x1 glyph, from its sprite in the items category ('?' if it has none).
pub(crate) fn item_glyph(app: &mut crate::App, kind: ItemKind) -> char {
    app.core
        .sprite_loader
        .load_sprite(itemkind_sprite_name(kind), "items")
        .sprites
        .first()
        .and_then(|s| s.chars().next())
        .unwrap_or('?')
}

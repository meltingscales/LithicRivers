use crate::App;
use lithicrivers_core::components::itemkind_sprite_name;
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
        let slot_text = format!("F{}", slot_number);

        // Get assigned block content
        let content = if let Some(block_kind) = app.panels.build.hotbar_assignments[i] {
            // Get the actual 1x1 sprite from SpriteLoader
            let sprite_name = itemkind_sprite_name(block_kind);
            // Parse category and name from sprite_name (format: "category/name")
            if let Some(slash_pos) = sprite_name.find('/') {
                let (category, name) = sprite_name.split_at(slash_pos);
                let name = &name[1..]; // Remove the '/'
                let sprite_data = app.core.sprite_loader.load_sprite(name, category);
                // Get the first character from the 1x1 sprite (sprites[0])
                let sprite_char = sprite_data
                    .sprites
                    .get(0)
                    .and_then(|s| s.chars().next())
                    .unwrap_or('?');
                format!("[{}]", sprite_char)
            } else {
                "[?]".to_string() // Invalid sprite name format
            }
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
                .title("Hotbar (F1-F12)")
                .style(Style::default().fg(Color::White)),
        );
    f.render_widget(content, area);
}

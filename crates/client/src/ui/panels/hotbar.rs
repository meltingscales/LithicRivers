use crate::App;
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

        // Add slot content (placeholder for now - could show item icons/names later)
        let content = "[ ]"; // Empty slot placeholder

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

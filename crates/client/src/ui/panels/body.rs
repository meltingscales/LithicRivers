use crate::rendering_helpers::build_body_ascii;
use lithicrivers_core::model::body::{Body, BodyPart, BodyPartState};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_body_panel(f: &mut Frame, app: &mut crate::App, area: Rect) {
    // Split area: left ASCII overview, right list
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(28), Constraint::Min(20)])
        .split(area);

    // Build from ECS
    let mut list_lines: Vec<Line> = Vec::new();
    let mut ascii_lines_opt: Option<Vec<Line<'static>>> = None;
    if let Some(e) = app.core.game.get_player_entity() {
        if let Ok(body) = app.core.game.world.get::<&Body>(e) {
            // Build list
            let mut parts: Vec<&BodyPart> = body.parts.values().collect();
            parts.sort_by_key(|p| p.part_type as i64);
            for p in parts {
                let (label, color) = match p.state {
                    BodyPartState::Missing => ("Missing", Color::DarkGray),
                    BodyPartState::Damaged => ("Damaged", Color::Yellow),
                    BodyPartState::Functional => ("Functional", Color::Green),
                    BodyPartState::Enhanced => ("Enhanced", Color::Cyan),
                };
                list_lines.push(Line::from(Span::styled(
                    format!("{:>15}: {} ({})", p.name, label, p.integrity),
                    Style::default().fg(color),
                )));
            }
            // Build ASCII from borrowed body
            ascii_lines_opt = Some(build_body_ascii(&*body));
        }
    }
    if list_lines.is_empty() {
        list_lines.push(Line::from(Span::raw("(No Body data)")));
    }

    // Left: ASCII overview
    let ascii_block = Block::default().borders(Borders::ALL).title("Body");
    let ascii_inner = ascii_block.inner(chunks[0]);
    let ascii_lines = ascii_lines_opt.unwrap_or_else(|| vec![Line::from(Span::raw("(No Body)"))]);
    let ascii_para = Paragraph::new(ascii_lines).alignment(Alignment::Left);
    f.render_widget(ascii_para, ascii_inner);
    f.render_widget(ascii_block, chunks[0]);

    // Right: textual list
    let list_para = Paragraph::new(list_lines)
        .alignment(Alignment::Left)
        .block(Block::default().borders(Borders::ALL).title("Parts"));
    f.render_widget(list_para, chunks[1]);
}

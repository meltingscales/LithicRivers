//! The gamepad context wheel (`gamepad-input` spec): while LB is held, a box of eight
//! actions around a centre, the highlighted one reversed.

use crate::gamepad::Dir8;
use crate::App;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// One slot's width in cells.
const CELL: usize = 14;

/// Grid positions of the eight directions, row by row (`None` is the centre).
const GRID: [[Option<Dir8>; 3]; 3] = [
    [Some(Dir8::NW), Some(Dir8::N), Some(Dir8::NE)],
    [Some(Dir8::W), None, Some(Dir8::E)],
    [Some(Dir8::SW), Some(Dir8::S), Some(Dir8::SE)],
];

fn centred(text: &str) -> String {
    let text: String = text.chars().take(CELL - 2).collect();
    format!("{text:^CELL$}")
}

pub(crate) fn render_gamepad_wheel(f: &mut Frame, app: &App) {
    let Some((kind, highlight)) = app.ui.gamepad.open_wheel() else {
        return;
    };
    let Some(slots) = app.ui.gamepad.layout.wheel(kind) else {
        return;
    };
    let label = |d: Dir8| {
        let i = Dir8::ALL.iter().position(|&x| x == d).unwrap_or(0);
        slots[i].as_ref().map(|a| a.label())
    };
    let centre = match highlight.and_then(label) {
        Some(name) => format!("> {name}"),
        None => "push a stick".to_string(),
    };
    let mut lines = Vec::new();
    for (r, row) in GRID.iter().enumerate() {
        if r > 0 {
            lines.push(Line::from(""));
        }
        let spans: Vec<Span> = row
            .iter()
            .map(|&cell| match cell {
                None => Span::styled(centred(&centre), Style::default().fg(Color::Yellow)),
                Some(d) => {
                    let text = centred(&label(d).unwrap_or_else(|| "-".to_string()));
                    let style = if highlight == Some(d) {
                        Style::default().add_modifier(Modifier::REVERSED)
                    } else if label(d).is_none() {
                        Style::default().fg(Color::DarkGray)
                    } else {
                        Style::default()
                    };
                    Span::styled(text, style)
                }
            })
            .collect();
        lines.push(Line::from(spans));
    }
    let width = (CELL * 3 + 2) as u16;
    let height = 7;
    let screen = f.area();
    let area = Rect::new(
        screen.x + screen.width.saturating_sub(width) / 2,
        screen.y + screen.height.saturating_sub(height) / 2,
        width.min(screen.width),
        height.min(screen.height),
    );
    f.render_widget(Clear, area);
    f.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} - release LB ", kind.title()))
                .title_alignment(Alignment::Center),
        ),
        area,
    );
}

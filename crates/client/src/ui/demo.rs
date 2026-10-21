//! The demo's overlay: a scrolling red DEMO banner over the top row, and the title card
//! between scenarios. Bold, minimal and readable at a glance (see docs/ART-DIRECTION.md).

use crate::App;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph},
    Frame,
};

/// The banner's repeating text.
pub const BANNER_UNIT: &str = "DEMO   ";
/// The banner moves one column this often.
pub const BANNER_STEP_MS: u128 = 100;

/// The banner text for a row `width` wide, `elapsed_ms` into the demo: "DEMO" repeated,
/// shifted one column left every `BANNER_STEP_MS`.
pub fn banner_text(width: usize, elapsed_ms: u128) -> String {
    let unit: Vec<char> = BANNER_UNIT.chars().collect();
    let shift = (elapsed_ms / BANNER_STEP_MS) as usize % unit.len();
    (0..width).map(|i| unit[(i + shift) % unit.len()]).collect()
}

pub(crate) fn render_demo_overlay(f: &mut Frame, app: &App) {
    let Some(demo) = &app.demo else {
        return;
    };
    let area = f.size();
    let now = app.clock.now();
    let elapsed = now.saturating_duration_since(demo.started).as_millis();
    let red = Style::default()
        .fg(Color::Black)
        .bg(Color::Red)
        .add_modifier(Modifier::BOLD);

    if demo.cut_until.is_some_and(|until| now < until) {
        // The title card: black, the scenario's name in the middle
        f.render_widget(Clear, area);
        f.render_widget(
            Paragraph::new("").style(Style::default().bg(Color::Black)),
            area,
        );
        let middle = Rect::new(area.x, area.y + area.height / 2, area.width, 1);
        f.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(" DEMO ", red),
                Span::styled(
                    format!("  {}", demo.current_name().to_uppercase()),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
            ]))
            .alignment(Alignment::Center),
            middle,
        );
    }

    let top = Rect::new(area.x, area.y, area.width, 1);
    let name = format!(" {} ", demo.current_name().to_uppercase());
    let name_width = name.chars().count() as u16;
    let banner_width = area.width.saturating_sub(name_width) as usize;
    f.render_widget(Clear, top);
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(banner_text(banner_width, elapsed), red),
            Span::styled(
                name,
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ),
        ])),
        top,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_banner_shifts_one_column_every_100_ms() {
        let at = |ms| banner_text(14, ms);
        assert_eq!(at(0), "DEMO   DEMO   ");
        assert_eq!(at(99), at(0));
        assert_eq!(at(100), "EMO   DEMO   D");
        assert_eq!(at(700), at(0), "wraps after one unit");
    }
}

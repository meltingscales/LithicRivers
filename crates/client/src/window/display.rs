//! The Display window: a small separate window with buttons for the game window's text
//! size and fullscreen (`game-window` spec).

use super::settings::WindowSettings;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};
use ratatui::Frame;

/// The Display window's size in cells.
pub const COLS: u16 = 24;
pub const ROWS: u16 = 6;
/// Its text height in pixels, fixed so it stays small whatever the game's size is.
pub const FONT_PX: u32 = 18;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Button {
    Smaller,
    Bigger,
    Fullscreen,
}

const SMALLER: &str = "[ A- ]";
const BIGGER: &str = "[ A+ ]";
const SIZE_ROW: u16 = 2;
const FULLSCREEN_ROW: u16 = 4;
const LEFT: u16 = 1;

fn size_label(settings: &WindowSettings) -> String {
    format!(" {:>2}px ", settings.font_px)
}

fn fullscreen_label(settings: &WindowSettings) -> String {
    format!(
        "[ Fullscreen: {} ]",
        if settings.fullscreen { "on " } else { "off" }
    )
}

/// The button at a cell, if any.
pub fn button_at(col: u16, row: u16, settings: &WindowSettings) -> Option<Button> {
    let width = |s: &str| s.chars().count() as u16;
    let within = |start: u16, label: &str| col >= start && col < start + width(label);
    match row {
        SIZE_ROW if within(LEFT, SMALLER) => Some(Button::Smaller),
        SIZE_ROW if within(LEFT + width(SMALLER) + width(&size_label(settings)), BIGGER) => {
            Some(Button::Bigger)
        }
        FULLSCREEN_ROW if within(LEFT, &fullscreen_label(settings)) => Some(Button::Fullscreen),
        _ => None,
    }
}

/// What a button does to the settings.
pub fn press(button: Button, settings: &mut WindowSettings) {
    match button {
        Button::Smaller => settings.smaller(),
        Button::Bigger => settings.bigger(),
        Button::Fullscreen => settings.fullscreen = !settings.fullscreen,
    }
}

/// Draw the panel filling a frame (the floating window).
pub fn draw(f: &mut Frame, settings: &WindowSettings) {
    let area = f.area();
    render(area, f.buffer_mut(), settings);
}

/// Draw the panel into an area; `button_at` takes cells relative to its top left.
pub fn render(area: Rect, buf: &mut Buffer, settings: &WindowSettings) {
    let button = Style::default()
        .fg(Color::Rgb(255, 210, 120))
        .add_modifier(Modifier::BOLD);
    let pad = " ".repeat(LEFT as usize);
    let lines = vec![
        Line::from(format!("{pad}Display")),
        Line::from(""),
        Line::from(vec![
            Span::raw(pad.clone()),
            Span::styled(SMALLER, button),
            Span::raw(size_label(settings)),
            Span::styled(BIGGER, button),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw(pad),
            Span::styled(fullscreen_label(settings), button),
        ]),
    ];
    Paragraph::new(lines).render(area, buf);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn screen(settings: &WindowSettings) -> Vec<String> {
        let mut terminal = Terminal::new(TestBackend::new(COLS, ROWS)).unwrap();
        terminal.draw(|f| draw(f, settings)).unwrap();
        let buf = terminal.backend().buffer().clone();
        (0..ROWS)
            .map(|y| {
                (0..COLS)
                    .map(|x| buf[(x, y)].symbol().to_string())
                    .collect()
            })
            .collect()
    }

    /// Every cell that shows a button's label hits that button, and only those cells.
    #[test]
    fn clicks_hit_the_buttons_they_show() {
        let settings = WindowSettings::default();
        let rows = screen(&settings);
        assert!(
            rows[2].contains("[ A- ]") && rows[2].contains("[ A+ ]"),
            "{rows:?}"
        );
        let find = |row: usize, label: &str| rows[row].find(label).unwrap() as u16;
        let (smaller, bigger) = (find(2, "[ A- ]"), find(2, "[ A+ ]"));
        let fullscreen = find(4, "[ Fullscreen");
        for col in 0..COLS {
            let expect = if (smaller..smaller + 6).contains(&col) {
                Some(Button::Smaller)
            } else if (bigger..bigger + 6).contains(&col) {
                Some(Button::Bigger)
            } else {
                None
            };
            assert_eq!(
                button_at(col, SIZE_ROW, &settings),
                expect,
                "size row col {col}"
            );
        }
        assert_eq!(
            button_at(fullscreen, FULLSCREEN_ROW, &settings),
            Some(Button::Fullscreen)
        );
        assert_eq!(button_at(fullscreen, 0, &settings), None);
    }

    #[test]
    fn buttons_change_the_settings() {
        let mut s = WindowSettings::default();
        press(Button::Bigger, &mut s);
        assert_eq!(s.font_px, super::super::settings::DEFAULT_FONT_PX + 2);
        press(Button::Smaller, &mut s);
        press(Button::Smaller, &mut s);
        assert_eq!(s.font_px, super::super::settings::DEFAULT_FONT_PX - 2);
        press(Button::Fullscreen, &mut s);
        assert!(s.fullscreen);
        press(Button::Fullscreen, &mut s);
        assert!(!s.fullscreen);
        assert!(screen(&s)[2].contains("16px"));
    }
}

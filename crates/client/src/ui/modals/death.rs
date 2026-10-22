//! The system-failure screen shown while the robot is dead.

use crate::app::input::death::{showing, CHOICES};
use crate::ui::centered_rect;
use crate::ui::click_targets::{inset, ClickTarget};
use crate::App;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub(crate) fn render_death_screen(f: &mut Frame, app: &App) {
    if !showing(app) {
        return;
    }
    let cause = app
        .core
        .game
        .res
        .player_state
        .death_cause
        .clone()
        .unwrap_or_else(|| "shut down".to_string());
    let mut lines = vec![
        Line::styled(
            "SYSTEM FAILURE",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
        Line::from(""),
        Line::from(format!("Your body was destroyed: {cause}.")),
        Line::from("It lies where it fell, with everything you carried."),
        Line::from(""),
    ];
    for (i, choice) in CHOICES.iter().enumerate() {
        let style = if i == app.panels.death_choice {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };
        lines.push(Line::styled(format!("  {choice}  "), style));
    }
    let area = centered_rect(56, 40, f.size());
    let area = Rect {
        height: area.height.min(lines.len() as u16 + 4),
        ..area
    };
    f.render_widget(Clear, area);
    app.ui.click_targets.push(area, ClickTarget::ModalArea);
    let key = |a| {
        app.core
            .config_manager
            .get_printable_key_for_keybind("ui", a)
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Backup systems ")
        .title_alignment(Alignment::Center)
        .title_bottom(format!(" {}: choose ", key("MENU_ACTIVATE")))
        .style(Style::default().fg(Color::White));
    f.render_widget(
        Paragraph::new(lines)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .block(block),
        area,
    );
    // The choices' rows, after the four lines above them and the title
    let inner = inset(area);
    let first_row = inner.y + 5 + extra_wrapped_rows(&cause, inner.width);
    let rows = Rect {
        y: first_row,
        height: CHOICES.len() as u16,
        ..inner
    };
    app.ui
        .click_targets
        .rows(rows, 0, CHOICES.len(), ClickTarget::DeathChoice);
}

/// How many extra rows the cause line wraps onto.
fn extra_wrapped_rows(cause: &str, width: u16) -> u16 {
    let text = format!("Your body was destroyed: {cause}.");
    let lines = Paragraph::new(text)
        .wrap(Wrap { trim: true })
        .line_count(width) as u16;
    lines.saturating_sub(1)
}

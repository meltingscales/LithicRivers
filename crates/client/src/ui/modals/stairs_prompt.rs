//! The "build stairs here?" prompt, shown when the player moves up or down with nothing
//! to climb.

use crate::app::input::stairs_prompt::CHOICES;
use crate::ui::centered_rect;
use crate::ui::click_targets::{inset, ClickTarget};
use crate::App;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub(crate) fn render_stairs_prompt(f: &mut Frame, app: &App) {
    let Some(prompt) = app.panels.stairs_prompt else {
        return;
    };
    let way = if prompt.dz > 0 { "up" } else { "down" };
    let mut lines = vec![
        Line::from(format!("Nothing to climb {way} here.")),
        Line::from(format!(
            "Build a stairway {way}? It uses {} Stairs.",
            prompt.cost
        )),
        Line::from(""),
    ];
    for (i, choice) in CHOICES.iter().enumerate() {
        let style = if i == prompt.selected {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };
        lines.push(Line::styled(format!("  {choice}  "), style));
    }
    let area = centered_rect(50, 30, f.size());
    let area = Rect {
        height: area.height.min(lines.len() as u16 + 2),
        ..area
    };
    f.render_widget(Clear, area);
    app.ui.click_targets.push(area, ClickTarget::ModalArea);
    let key = |c, a| app.core.config_manager.get_printable_key_for_keybind(c, a);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Build stairs? ")
        .title_alignment(Alignment::Center)
        .title_bottom(format!(
            " {}: choose  {}: cancel ",
            key("ui", "MENU_ACTIVATE"),
            key("ui", "CLOSE_HELP_MENU")
        ))
        .style(Style::default().fg(Color::White));
    f.render_widget(Paragraph::new(lines).block(block), area);
    let rows = Rect {
        y: area.y + 1 + 3,
        ..inset(area)
    };
    app.ui
        .click_targets
        .rows(rows, 0, CHOICES.len(), ClickTarget::StairsChoice);
}

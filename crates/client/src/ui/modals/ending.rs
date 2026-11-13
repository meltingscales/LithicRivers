//! The closing sequence's pages over the open Aperture's art, then the scrolling credits.

use crate::app::input::ending::{credits_scroll, on_credits, showing, PAGES};
use crate::App;
use ratatui::{
    layout::Alignment,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub(crate) fn render_ending(f: &mut Frame, app: &mut App) {
    if !showing(app) {
        return;
    }
    let area = f.area();
    f.render_widget(Clear, area);
    app.ui
        .click_targets
        .push(area, crate::ui::click_targets::ClickTarget::ModalArea);
    let hint = Line::styled("Press any key", Style::default().fg(Color::DarkGray));
    if on_credits(app) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Credits ")
            .title_alignment(Alignment::Center)
            .title_bottom(hint.alignment(Alignment::Center));
        let para = Paragraph::new(app.panels.credits.text.clone())
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .scroll((credits_scroll(app), 0))
            .block(block);
        f.render_widget(para, area);
        return;
    }
    let sprite = app
        .core
        .sprite_loader
        .load_sprite("aperture_core_open", "factory")
        .clone();
    let color = crate::rendering_helpers::parse_hex_color(&sprite.color).unwrap_or(Color::White);
    let mut lines: Vec<Line> = vec![Line::from(""); (area.height as usize).saturating_sub(16) / 2];
    for row in sprite
        .art12x8_sprites
        .first()
        .map_or(vec![], |a| a.lines().map(String::from).collect())
    {
        lines.push(Line::styled(row, Style::default().fg(color)));
    }
    lines.push(Line::from(""));
    for text in PAGES[app.panels.ending_page] {
        lines.push(Line::from(*text));
    }
    lines.push(Line::from(""));
    lines.push(hint);
    f.render_widget(Paragraph::new(lines).alignment(Alignment::Center), area);
}

use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_menu_panel(f: &mut Frame, app: &mut crate::App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(Span::styled(
        "Game Menu",
        Style::default().fg(Color::Cyan),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::raw(
        "Use configured keybinds to Save/Load/Quit (see config).",
    )));
    lines.push(Line::from(""));
    // Config source
    lines.push(Line::from(Span::styled(
        "Configuration:",
        Style::default().fg(Color::Cyan),
    )));
    lines.push(Line::from(Span::raw(format!(
        "  {}",
        app.game.res.config.source_label()
    ))));
    lines.push(Line::from(Span::raw(format!(
        "  Player: {}",
        app.game.res.player_name
    ))));
    lines.push(Line::from(Span::raw(format!(
        "  Developer mode: {}",
        if app.game.res.developer_mode {
            "ON"
        } else {
            "OFF"
        }
    ))));
    lines.push(Line::from(""));
    // Logging location information
    lines.push(Line::from(Span::raw("Logging:")));
    lines.push(Line::from(Span::raw(format!(
        "  Path: {}",
        app.log_full_path
    ))));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::raw(format!(
        "Seed: {}",
        app.game.res.seed
    ))));
    lines.push(Line::from(Span::raw(format!(
        "Tick: {}",
        app.game.res.gametick
    ))));
    lines.push(Line::from(Span::raw(format!(
        "View Z: {}",
        app.game.res.view_z
    ))));
    if let Some(e) = app.game.res.player_entity {
        if let Ok(pos) = app
            .game
            .world
            .get::<&lithicrivers_core::components::Position>(e)
        {
            lines.push(Line::from(Span::raw(format!(
                "Player: ({}, {}, {})",
                pos.x, pos.y, pos.z
            ))));
        }
    }
    let block = Block::default().borders(Borders::ALL).title("Menu");
    let inner = block.inner(area);
    let p = Paragraph::new(lines).alignment(Alignment::Left);
    f.render_widget(p, inner);
    f.render_widget(block, area);
}

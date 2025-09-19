use crate::app_state::BuildMode;
use crate::App;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_modes_panel(f: &mut Frame, app: &mut App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();

    // Battle mode status
    let battle_status = if app.combat.is_active() {
        Span::styled("BATTLE", Style::default().fg(Color::Red))
    } else {
        Span::styled("------", Style::default().fg(Color::DarkGray))
    };

    // Look mode status
    let look_status = if app.panels.look.mode {
        Span::styled("L", Style::default().fg(Color::Yellow))
    } else {
        Span::styled("-", Style::default().fg(Color::DarkGray))
    };

    // Build mode status
    let build_status = match app.panels.build.mode {
        BuildMode::Movement => Span::styled("-", Style::default().fg(Color::DarkGray)),
        BuildMode::Break => Span::styled("B", Style::default().fg(Color::Cyan)),
        BuildMode::Place => Span::styled("P", Style::default().fg(Color::Green)),
    };

    // Noclip mode status
    let noclip_status = if app.core.game.res.player_state.noclip_enabled {
        Span::styled("NC", Style::default().fg(Color::Magenta))
    } else {
        Span::styled("--", Style::default().fg(Color::DarkGray))
    };

    // Create a single line with all modes
    lines.push(Line::from(vec![
        battle_status,
        Span::raw(" "),
        look_status,
        Span::raw(" "),
        build_status,
        Span::raw(" "),
        noclip_status,
    ]));

    let content = Paragraph::new(lines)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("MODES")
                .style(Style::default().fg(Color::White)),
        );
    f.render_widget(content, area);
}

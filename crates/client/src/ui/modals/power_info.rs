//! The power pole info box: its network's supply, demand and size.

use crate::ui::centered_rect;
use crate::App;
use ratatui::{
    layout::Alignment,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub(crate) fn render_power_info_modal(f: &mut Frame, app: &App) {
    let Some(pole) = app.panels.power_info else {
        return;
    };
    let mut lines = vec![Line::from("")];
    match app.core.game.res.power.network_of(pole) {
        Some(network) => {
            lines.push(Line::from(format!(
                "Supply: {} kW   Demand: {} kW",
                network.supply_w / 1000,
                network.demand_w / 1000
            )));
            lines.push(Line::from(format!(
                "Poles: {}   Generators: {}   Machines: {}",
                network.poles.len(),
                network.generators.len(),
                network.consumers.len()
            )));
        }
        None => lines.push(Line::from("Not part of a network yet")),
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!(
            "{}: close",
            app.core
                .config_manager
                .get_printable_key_for_keybind("ui", "CLOSE_HELP_MENU")
        ),
        Style::default().fg(Color::DarkGray),
    )));
    let area = centered_rect(50, 30, f.size());
    f.render_widget(Clear, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Power Network ")
        .title_alignment(Alignment::Center);
    f.render_widget(
        Paragraph::new(lines)
            .alignment(Alignment::Center)
            .block(block),
        area,
    );
}

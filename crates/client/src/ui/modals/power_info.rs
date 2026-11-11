//! The power pole info box: its network's supply, demand and size, and a graph of its demand
//! over the last 60 game seconds or 10 game minutes, red where it went short.

use crate::ui::centered_rect;
use crate::App;
use ratatui::{
    layout::Alignment,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// A graph's rows as lines, each column green or, where `warn`, red.
fn graph_lines(graph: &crate::ui::graph::Graph) -> Vec<Line<'static>> {
    graph
        .rows
        .iter()
        .map(|row| {
            let mut spans: Vec<Span> = Vec::new();
            let mut run = String::new();
            let mut run_warn = false;
            for (i, c) in row.chars().enumerate() {
                let warn = graph.warn.get(i).copied().unwrap_or(false);
                if warn != run_warn && !run.is_empty() {
                    spans.push(Span::styled(std::mem::take(&mut run), color(run_warn)));
                }
                run_warn = warn;
                run.push(c);
            }
            spans.push(Span::styled(run, color(run_warn)));
            Line::from(spans)
        })
        .collect()
}

fn color(warn: bool) -> Style {
    Style::default().fg(if warn { Color::Red } else { Color::Green })
}

pub(crate) fn render_power_info_modal(f: &mut Frame, app: &App) {
    let Some(pole) = app.panels.power_info else {
        return;
    };
    let area = centered_rect(60, 60, f.area());
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
            if !network.batteries.is_empty() {
                let flow: i64 = network.battery_flows.iter().map(|(_, w)| w).sum();
                let doing = match flow {
                    w if w > 0 => format!("charging at {} kW", w / 1000),
                    w if w < 0 => format!("discharging at {} kW", -w / 1000),
                    _ => "idle".to_string(),
                };
                lines.push(Line::from(format!(
                    "Stored: {}, {doing}",
                    lithicrivers_core::teleporter::format_mj(network.stored_j),
                )));
            }
            lines.push(Line::from(""));
            let long = app.panels.power_range_long;
            let samples: Vec<_> =
                lithicrivers_core::factory_stats::history(&app.core.game.res, network.id)
                    .map(|h| {
                        let keep = if long {
                            lithicrivers_core::factory_stats::POWER_SAMPLES
                        } else {
                            lithicrivers_core::factory_stats::SHORT_SAMPLES
                        };
                        h.samples
                            .iter()
                            .skip(h.samples.len().saturating_sub(keep))
                            .copied()
                            .collect()
                    })
                    .unwrap_or_default();
            let peak = samples
                .iter()
                .map(|s| s.supply_w.max(s.demand_w))
                .max()
                .unwrap_or(0);
            let short = samples.iter().filter(|s| s.short()).count();
            lines.push(Line::from(format!(
                "Demand, {} (peak {} kW{})",
                if long {
                    "last 10 minutes"
                } else {
                    "last 60 seconds"
                },
                peak / 1000,
                if short > 0 { ", red: short" } else { "" }
            )));
            // One column a sample in the short view; the long one is bucketed to fit
            let width = if long {
                (area.width as usize).saturating_sub(6).min(60)
            } else {
                lithicrivers_core::factory_stats::SHORT_SAMPLES
            };
            let graph = crate::ui::graph::power(&samples, width, 4);
            lines.extend(graph_lines(&graph));
        }
        None => lines.push(Line::from("Not part of a network yet")),
    }
    lines.push(Line::from(""));
    let key = |c, a| app.key_label(c, a);
    lines.push(Line::from(Span::styled(
        format!(
            "{}: {}  {}: close",
            key("stats", "CYCLE_RANGE"),
            if app.panels.power_range_long {
                "last 60 seconds"
            } else {
                "last 10 minutes"
            },
            key("ui", "CLOSE_HELP_MENU")
        ),
        Style::default().fg(Color::DarkGray),
    )));
    f.render_widget(Clear, area);
    app.ui
        .click_targets
        .push(area, crate::ui::click_targets::ClickTarget::ModalArea);
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

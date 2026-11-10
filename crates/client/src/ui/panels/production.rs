//! The Production tab: every item made or used in the chosen time range with its rates per
//! game minute, and the selected item's sources and a graph of what was made each minute.

use crate::App;
use lithicrivers_core::components::itemkind_name;
use lithicrivers_core::factory_stats::{self, ItemRate};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

/// The time ranges, in game minutes, and how the tab names them.
pub const RANGES: [(u64, &str); 3] = [
    (1, "last minute"),
    (10, "last 10 minutes"),
    (60, "last hour"),
];

/// The item rows shown for the current range.
pub fn rows(app: &App) -> Vec<ItemRate> {
    let minutes = RANGES[app.panels.production.range].0;
    factory_stats::rates(&app.core.game.res, minutes)
}

fn rate(per_min: f64) -> String {
    if per_min == 0.0 {
        "-".to_string()
    } else if per_min >= 10.0 {
        format!("{per_min:.0}")
    } else {
        format!("{per_min:.1}")
    }
}

fn row_text(item: &str, made: &str, used: &str) -> String {
    format!("{item:<22}{made:>7}{used:>7}")
}

pub fn render_production_panel(f: &mut Frame, app: &App, area: Rect) {
    let (minutes, range_name) = RANGES[app.panels.production.range];
    let rows = rows(app);
    let key = |c, a| app.core.config_manager.get_printable_key_for_keybind(c, a);
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // The item list, under a header row, in one box
    let block = Block::default().borders(Borders::ALL).title(format!(
        " Per minute, {range_name} ({}: range) ",
        key("stats", "CYCLE_RANGE")
    ));
    let inner = block.inner(columns[0]);
    f.render_widget(block, columns[0]);
    let list_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(inner);
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            row_text("Item", "Made", "Used"),
            Style::default().fg(Color::DarkGray),
        ))),
        list_area[0],
    );
    let items: Vec<ListItem> = rows
        .iter()
        .map(|r| {
            ListItem::new(row_text(
                itemkind_name(r.item),
                &rate(r.produced_per_min),
                &rate(r.consumed_per_min),
            ))
        })
        .collect();
    let selected = app
        .panels
        .production
        .selected
        .min(rows.len().saturating_sub(1));
    let mut state = ListState::default();
    state.select((!rows.is_empty()).then_some(selected));
    f.render_stateful_widget(
        List::new(items).highlight_style(Style::default().add_modifier(Modifier::REVERSED)),
        list_area[1],
        &mut state,
    );

    let mut lines: Vec<Line> = Vec::new();
    match rows.get(selected) {
        None => {
            lines.push(Line::from("Nothing has been made or used yet."));
            lines.push(Line::from(""));
            lines.push(Line::from(
                "Mining, crafting, machines, fuel and planting are counted here.",
            ));
        }
        Some(row) => {
            let res = &app.core.game.res;
            lines.push(Line::from(Span::styled(
                itemkind_name(row.item).to_string(),
                Style::default().add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(format!(
                "Made {} / min, used {} / min",
                rate(row.produced_per_min),
                rate(row.consumed_per_min)
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!("Sources, {range_name}:"),
                Style::default().fg(Color::Yellow),
            )));
            for (source, made, used) in factory_stats::sources(res, row.item, minutes) {
                let mut parts = Vec::new();
                if made > 0 {
                    parts.push(format!("made {made}"));
                }
                if used > 0 {
                    parts.push(format!("used {used}"));
                }
                lines.push(Line::from(format!(
                    "  {}: {}",
                    source.name(),
                    parts.join(", ")
                )));
            }
            lines.push(Line::from(""));
            // At least 10 minutes, so a short range still shows a trend
            let per_minute = factory_stats::produced_per_minute(res, row.item, minutes.max(10));
            let peak = per_minute.iter().copied().max().unwrap_or(0);
            if peak == 0 {
                lines.push(Line::from("None made lately."));
            } else {
                let values: Vec<(u64, bool)> = per_minute.iter().map(|&v| (v, false)).collect();
                let width = (columns[1].width as usize)
                    .saturating_sub(4)
                    .min(values.len());
                let graph = crate::ui::graph::bars(&values, peak, width, 4);
                lines.push(Line::from(Span::styled(
                    format!("Made each minute, oldest first (peak {peak}):"),
                    Style::default().fg(Color::Yellow),
                )));
                for row in graph.rows {
                    lines.push(Line::from(Span::styled(
                        format!("  {row}"),
                        Style::default().fg(Color::Green),
                    )));
                }
            }
        }
    }
    f.render_widget(
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(" Details ")),
        columns[1],
    );
}

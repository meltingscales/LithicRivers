//! The teleporter window: the battery bank's charge and charge rate (or a station's reserve
//! power), then every destination with its distance and the jump's cost.

use crate::ui::centered_rect;
use crate::App;
use lithicrivers_core::power::PowerConsumer;
use lithicrivers_core::teleporter::{self, format_mj, Teleporter, TransitStation};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

pub(crate) fn render_teleporter_window(f: &mut Frame, app: &App) {
    let Some(window) = app.panels.teleporter_window.as_ref() else {
        return;
    };
    let world = &app.core.game.world;
    let key = |c: &str, a: &str| app.key_label(c, a);
    let teleporter = world
        .get::<&Teleporter>(window.entity)
        .ok()
        .map(|t| (*t).clone());
    let station = world
        .get::<&TransitStation>(window.entity)
        .ok()
        .map(|s| (*s).clone());
    // A teleporter's or station's bank: its charge and how fast it is charging
    let bank_line = |charge_j: u64| {
        let rate = world
            .get::<&PowerConsumer>(window.entity)
            .map_or(0, |c| teleporter::charge_rate_w(&c));
        let charging = if charge_j >= teleporter::CAPACITY_J {
            "full".to_string()
        } else {
            format!("charging at {} kW", rate / 1000)
        };
        format!(
            "Battery: {} / {}   ({charging})",
            format_mj(charge_j),
            format_mj(teleporter::CAPACITY_J)
        )
    };
    let (title, status) = match (&teleporter, &station) {
        (Some(t), _) => (t.name.clone(), bank_line(t.charge_j)),
        (None, Some(s)) => (s.name.clone(), bank_line(s.charge_j)),
        (None, None) if teleporter::carries_device(world, window.entity) => (
            "Mobile Tele-Access Device".to_string(),
            format!(
                "Fullest power cell: {}",
                format_mj(lithicrivers_core::power_cells::fullest_charge(
                    world,
                    window.entity
                ))
            ),
        ),
        (None, None) => return,
    };

    let destinations = teleporter::destinations(world, window.entity);
    let items: Vec<ListItem> = if destinations.is_empty() {
        vec![ListItem::new(match station {
            Some(_) => "  (no other station is awake yet)",
            None => "  (no other teleporter yet)",
        })]
    } else {
        destinations
            .iter()
            .map(|d| {
                let cost = if d.cost_j == 0 {
                    "free".to_string()
                } else {
                    format_mj(d.cost_j)
                };
                let short = match teleporter::bank_charge(world, window.entity) {
                    Some(charge) => charge < d.cost_j,
                    None => {
                        lithicrivers_core::power_cells::fullest_charge(world, window.entity)
                            < d.cost_j
                    }
                };
                let style = if short {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default()
                };
                ListItem::new(format!(
                    "  {:<24} {:>6} tiles  {:>8}",
                    d.name, d.distance, cost
                ))
                .style(style)
            })
            .collect()
    };

    let footer = match &window.renaming {
        Some(_) => format!(
            " {}: keep name  {}: cancel ",
            key("teleporter", "END_RENAME"),
            key("ui", "CLOSE_HELP_MENU")
        ),
        None if teleporter.is_some() => format!(
            " {}: jump  {}: rename  {}: close ",
            key("ui", "MENU_ACTIVATE"),
            key("teleporter", "RENAME"),
            key("ui", "CLOSE_HELP_MENU")
        ),
        None => format!(
            " {}: jump  {}: close ",
            key("ui", "MENU_ACTIVATE"),
            key("ui", "CLOSE_HELP_MENU")
        ),
    };
    let header = match &window.renaming {
        Some(name) => vec![Line::from(format!("Name: {name}_")), Line::from(status)],
        None => vec![Line::from(status), Line::from("")],
    };

    let area = centered_rect(60, 60, f.area());
    f.render_widget(Clear, area);
    app.ui
        .click_targets
        .push(area, crate::ui::click_targets::ClickTarget::ModalArea);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {title} "))
        .title_alignment(Alignment::Center)
        .title_bottom(footer);
    let inner = block.inner(area);
    f.render_widget(block, area);
    let [top, list_area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(1)])
        .areas(inner);
    f.render_widget(Paragraph::new(header).alignment(Alignment::Center), top);
    let list = List::new(items)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    let mut state = ListState::default();
    if !destinations.is_empty() {
        state.select(Some(window.selected.min(destinations.len() - 1)));
    }
    f.render_stateful_widget(list, list_area, &mut state);
}

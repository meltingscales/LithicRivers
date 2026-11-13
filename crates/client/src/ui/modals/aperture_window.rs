//! The Aperture's flow window: each input's required and actual rates, colored by how it
//! keeps up, the charge and the time left; or, while the ring is incomplete, where it
//! needs a block.

use crate::ui::centered_rect;
use crate::App;
use lithicrivers_core::aperture::{self, Aperture, Flow, INPUTS, OPEN_CHARGE};
use lithicrivers_core::components::itemkind_name;
use lithicrivers_core::factory::FactoryKind;
use ratatui::{
    layout::Alignment,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Milli-items as a rate with one decimal, like "4.0".
fn rate(milli: u64) -> String {
    format!("{}.{}", milli / 1000, milli % 1000 / 100)
}

fn points(milli: u64) -> String {
    format!("{}.{}", milli / 1000, milli % 1000 / 100)
}

fn flow_color(flow: Flow) -> Color {
    match flow {
        Flow::Green => Color::Green,
        Flow::Yellow => Color::Yellow,
        Flow::Red => Color::Red,
    }
}

/// The window's lines for one Aperture.
pub(crate) fn lines(
    a: &Aperture,
    core: lithicrivers_core::components::Position,
) -> Vec<Line<'static>> {
    let mut lines = vec![Line::from("")];
    if let Some(missing) = a.missing {
        let (kind, input) = aperture::needed((missing.x - core.x, missing.y - core.y));
        let what = match (kind, input) {
            (FactoryKind::AperturePort, Some(item)) => {
                format!("an Aperture port (for {})", itemkind_name(item))
            }
            _ => "an Aperture frame".to_string(),
        };
        lines.push(Line::from("The ring is incomplete."));
        lines.push(Line::from(format!(
            "It needs {what} at ({}, {}, {}).",
            missing.x, missing.y, missing.z
        )));
        return lines;
    }
    lines.push(Line::from(format!(
        "{:<22}{:>8}{:>10}{:>10}",
        "Input (per minute)", "needs", "last min", "10 min"
    )));
    for (i, input) in INPUTS.iter().enumerate() {
        let last = a.history.back().map_or(0, |m| m[i]) as u64 * 1000;
        let (percent, flow) = aperture::flow(a, i);
        let ten = input.milli_per_minute * percent / 100;
        lines.push(Line::from(Span::styled(
            format!(
                "{:<22}{:>8}{:>10}{:>10}  {percent}%",
                itemkind_name(input.item),
                rate(input.milli_per_minute),
                rate(last),
                rate(ten),
            ),
            Style::default().fg(flow_color(flow)),
        )));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(format!(
        "Charge: {} / {} points",
        points(a.charge_milli),
        OPEN_CHARGE / 1000
    )));
    lines.push(Line::from(match aperture::minutes_left(a) {
        Some(m) => format!(
            "Time left at these rates: about {} h {} min",
            m / 60,
            m % 60
        ),
        None => "Not charging: every input must arrive".to_string(),
    }));
    lines
}

pub(crate) fn render_aperture_window(f: &mut Frame, app: &App) {
    let Some(core) = app.panels.aperture_window else {
        return;
    };
    let world = &app.core.game.world;
    let (Ok(a), Ok(pos)) = (
        world.get::<&Aperture>(core),
        world.get::<&lithicrivers_core::components::Position>(core),
    ) else {
        return;
    };
    let area = centered_rect(70, 60, f.area());
    let mut lines = lines(&a, *pos);
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!("{}: close", app.key_label("ui", "CLOSE_HELP_MENU")),
        Style::default().fg(Color::DarkGray),
    )));
    f.render_widget(Clear, area);
    app.ui
        .click_targets
        .push(area, crate::ui::click_targets::ClickTarget::ModalArea);
    let title = if a.complete {
        " The Aperture - charging "
    } else {
        " The Aperture - incomplete "
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .title_alignment(Alignment::Center);
    f.render_widget(Paragraph::new(lines).block(block), area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use lithicrivers_core::components::Position;

    /// Ten steady minutes at these percentages of each input's rate.
    fn at_percent(percent: [u64; 5]) -> Aperture {
        let mut a = Aperture {
            complete: true,
            ..Default::default()
        };
        // Whole items a minute can't hit every percentage, so count in tenths of the rate
        // over ten minutes: total = rate * percent / 100 * 10 minutes
        let mut minutes = vec![[0u32; 5]; 10];
        for (i, input) in INPUTS.iter().enumerate() {
            let total = input.milli_per_minute * percent[i] / 100 * 10 / 1000;
            for (m, minute) in minutes.iter_mut().enumerate() {
                minute[i] = (total / 10 + u64::from((m as u64) < total % 10)) as u32;
            }
        }
        // Steady, not falling: put the larger minutes first
        a.history = minutes.into_iter().collect();
        a
    }

    fn row_color(lines: &[Line], item: lithicrivers_core::components::ItemKind) -> Color {
        lines
            .iter()
            .find(|l| {
                l.spans
                    .iter()
                    .any(|s| s.content.contains(itemkind_name(item)))
            })
            .and_then(|l| l.spans[0].style.fg)
            .unwrap()
    }

    #[test]
    fn inputs_are_green_yellow_or_red_by_how_they_keep_up() {
        // Meat slurry at 120%, ethanol at 100%, mash at 80%
        let a = at_percent([100, 120, 100, 80, 100]);
        let lines = lines(&a, Position { x: 0, y: 0, z: 0 });
        use lithicrivers_core::components::ItemKind;
        assert_eq!(row_color(&lines, ItemKind::MeatSlurry), Color::Green);
        assert_eq!(row_color(&lines, ItemKind::Ethanol), Color::Yellow);
        assert_eq!(row_color(&lines, ItemKind::Mash), Color::Red);
    }

    #[test]
    fn an_incomplete_ring_names_the_missing_block() {
        let a = Aperture {
            missing: Some(Position { x: 0, y: -5, z: 0 }),
            ..Default::default()
        };
        let text: String = lines(&a, Position { x: 0, y: 0, z: 0 })
            .iter()
            .map(|l| l.to_string() + "\n")
            .collect();
        assert!(
            text.contains("It needs an Aperture port (for Meat Slurry) at (0, -5, 0)."),
            "{text}"
        );
    }
}

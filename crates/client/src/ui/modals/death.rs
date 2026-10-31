//! The system-failure screen shown while the robot is dead.

use crate::app::input::death::{ready_vats, showing, wreck_position, CHOICES};
use crate::ui::centered_rect;
use crate::ui::click_targets::{inset, ClickTarget};
use crate::App;
use lithicrivers_core::model::body::{Body, BodyPartState, PART_ORDER};
use lithicrivers_core::vat::distance;
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
    if let Some(selected) = app.panels.resurrect {
        render_resurrect_screen(f, app, selected);
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
        Line::from("The Global Map marks where."),
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
    let area = centered_rect(56, 40, f.area());
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
    // The choices' rows, after the six lines above them
    let inner = inset(area);
    let first_row = inner.y + 6 + extra_wrapped_rows(&cause, inner.width);
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

/// "Damaged: torso, fusion core, legs. Missing: head, arms": the best state first, and a
/// pair named once when both of it share a state.
fn parts_summary(body: &Body) -> String {
    use lithicrivers_core::model::body::BodyPartType::*;
    let state = |part| {
        body.parts.get(&part).map_or(
            BodyPartState::Missing,
            |p: &lithicrivers_core::model::body::BodyPart| p.state,
        )
    };
    [
        (BodyPartState::Enhanced, "Enhanced"),
        (BodyPartState::Functional, "Functional"),
        (BodyPartState::Damaged, "Damaged"),
        (BodyPartState::Missing, "Missing"),
    ]
    .into_iter()
    .filter_map(|(wanted, word)| {
        let mut names: Vec<String> = Vec::new();
        for part in PART_ORDER {
            if state(part) != wanted {
                continue;
            }
            let pair = match part {
                LeftArm | RightArm => Some(((LeftArm, RightArm), "arms")),
                LeftLeg | RightLeg => Some(((LeftLeg, RightLeg), "legs")),
                _ => None,
            };
            match pair {
                Some(((left, right), both)) if state(left) == state(right) => {
                    if part == left {
                        names.push(both.to_string());
                    }
                }
                _ => names.push(part.label().to_lowercase()),
            }
        }
        (!names.is_empty()).then(|| format!("{word}: {}", names.join(", ")))
    })
    .collect::<Vec<_>>()
    .join(". ")
}

/// "crude resurrection vat" -> "Crude resurrection vat".
fn capitalized(name: &str) -> String {
    let mut chars = name.chars();
    chars
        .next()
        .map_or_else(String::new, |c| c.to_uppercase().chain(chars).collect())
}

/// The Resurrect screen: every ready vat, nearest first, then the junker body, two lines
/// each.
fn render_resurrect_screen(f: &mut Frame, app: &App, selected: usize) {
    let wreck = wreck_position(app);
    let mut entries: Vec<(String, String)> = ready_vats(app)
        .iter()
        .map(|v| {
            let p = v.position;
            (
                format!(
                    "{} at ({}, {}, {}), {} tiles",
                    capitalized(v.kind.name()),
                    p.x,
                    p.y,
                    p.z,
                    distance(wreck, p)
                ),
                parts_summary(&v.body),
            )
        })
        .collect();
    let (junker, at_hq) = app.core.game.junker_place();
    let place = if at_hq {
        "the SapienCorp HQ"
    } else {
        "the bunker"
    };
    entries.push((
        format!("Junker body at {place}, {} tiles", distance(wreck, junker)),
        "Every part damaged".to_string(),
    ));

    let mut lines = vec![
        Line::styled(
            "RESURRECT",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
        Line::from("Choose a body to wake in."),
        Line::from(""),
    ];
    let header = lines.len() as u16;
    for (i, (title, parts)) in entries.iter().enumerate() {
        let style = if i == selected {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };
        lines.push(Line::styled(format!(" {title} "), style));
        lines.push(Line::styled(format!("   {parts} "), style));
    }
    let area = centered_rect(80, 70, f.area());
    let area = Rect {
        height: area.height.min(lines.len() as u16 + 2),
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
        .title_bottom(format!(
            " {}: wake here  {}: back ",
            key("MENU_ACTIVATE"),
            key("CLOSE_HELP_MENU")
        ))
        .style(Style::default().fg(Color::White));
    f.render_widget(Paragraph::new(lines).block(block), area);
    let inner = inset(area);
    for i in 0..entries.len() {
        let rows = Rect {
            y: inner.y + header + 2 * i as u16,
            height: 2,
            ..inner
        };
        if rows.y + 1 < inner.y + inner.height {
            app.ui
                .click_targets
                .push(rows, ClickTarget::ResurrectChoice(i));
        }
    }
}

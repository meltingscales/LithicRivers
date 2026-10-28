use crate::rendering_helpers::build_body_ascii;
use lithicrivers_core::components::itemkind_name;
use lithicrivers_core::model::body::{Body, BodyPartState, PART_ORDER};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_body_panel(f: &mut Frame, app: &mut crate::App, area: Rect) {
    // Split area: left ASCII overview, right list, bottom controls
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    let main_area = vertical_chunks[0];
    let controls_area = vertical_chunks[1];

    // Split main area: left ASCII overview, right list
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(28), Constraint::Min(20)])
        .split(main_area);

    // Build from ECS: the selectable parts, what fits the selected one, and the effects
    let mut list_lines: Vec<Line> = Vec::new();
    let mut ascii_lines_opt: Option<Vec<Line<'static>>> = None;
    let selected = app.panels.body.selected_part.min(PART_ORDER.len() - 1);
    let selected_part = PART_ORDER[selected];
    let fitting = app.core.game.fitting_items(selected_part);
    let mut part_rows = 0;
    let mut item_row = None;
    if let Some(e) = app.core.game.get_player_entity() {
        if let Ok(body) = app.core.game.world.get::<&Body>(e) {
            for (i, part_type) in PART_ORDER.iter().enumerate() {
                let Some(p) = body.parts.get(part_type) else {
                    continue;
                };
                let (label, color) = match p.state {
                    BodyPartState::Missing => ("Missing", Color::DarkGray),
                    BodyPartState::Damaged => ("Damaged", Color::Yellow),
                    BodyPartState::Functional => ("Functional", Color::Green),
                    BodyPartState::Enhanced => ("Enhanced", Color::Cyan),
                };
                let is_selected = i == selected;
                let mut style = Style::default().fg(color);
                if is_selected && !app.panels.body.choosing_item {
                    style = style.add_modifier(Modifier::REVERSED);
                }
                let marker = if is_selected { ">" } else { " " };
                list_lines.push(Line::from(Span::styled(
                    format!("{marker}{:>14}: {} ({})", p.name, label, p.integrity),
                    style,
                )));
                part_rows += 1;
            }

            // What fits the selected part
            let name = body
                .parts
                .get(&selected_part)
                .map_or("part", |p| p.name.as_str());
            list_lines.push(Line::from(Span::styled(
                format!("Fits the {name}:"),
                Style::default().fg(Color::Cyan),
            )));
            item_row = Some(list_lines.len());
            if fitting.is_empty() {
                list_lines.push(Line::from(Span::styled(
                    "  (nothing in the inventory)",
                    Style::default().fg(Color::DarkGray),
                )));
            }
            for (i, (kind, qty)) in fitting.iter().enumerate() {
                let chosen = app.panels.body.choosing_item && i == app.panels.body.selected_item;
                let style = if chosen {
                    Style::default().add_modifier(Modifier::REVERSED)
                } else {
                    Style::default()
                };
                list_lines.push(Line::from(Span::styled(
                    format!("  {} x{qty}", itemkind_name(*kind)),
                    style,
                )));
            }

            // What the damage does, in plain words
            let effects = body.describe_effects();
            if !effects.is_empty() {
                list_lines.push(Line::from(""));
                list_lines.push(Line::from("Effects:"));
                for effect in effects {
                    list_lines.push(Line::from(Span::styled(
                        format!("  {effect}"),
                        Style::default().fg(Color::Yellow),
                    )));
                }
            }
            // Build ASCII from borrowed body
            ascii_lines_opt = Some(build_body_ascii(&body));
        }
    }
    if list_lines.is_empty() {
        list_lines.push(Line::from(Span::raw("(No Body data)")));
    }

    // Left: ASCII overview
    let ascii_block = Block::default()
        .borders(Borders::ALL)
        .title(Line::from("Body"));
    let ascii_inner = ascii_block.inner(chunks[0]);
    let ascii_lines = ascii_lines_opt.unwrap_or_else(|| vec![Line::from(Span::raw("(No Body)"))]);
    let ascii_para = Paragraph::new(ascii_lines).alignment(Alignment::Left);
    f.render_widget(ascii_para, ascii_inner);
    f.render_widget(ascii_block, chunks[0]);

    // Right: textual list
    let list_para = Paragraph::new(list_lines).alignment(Alignment::Left).block(
        Block::default()
            .borders(Borders::ALL)
            .title(Line::from("Parts")),
    );
    f.render_widget(list_para, chunks[1]);
    let rows = crate::ui::click_targets::inset(chunks[1]);
    app.ui.click_targets.rows(
        rows,
        0,
        part_rows,
        crate::ui::click_targets::ClickTarget::BodyPart,
    );
    if let Some(first) = item_row {
        let item_area = Rect::new(
            rows.x,
            rows.y + first as u16,
            rows.width,
            rows.height.saturating_sub(first as u16),
        );
        app.ui.click_targets.rows(
            item_area,
            0,
            fitting.len(),
            crate::ui::click_targets::ClickTarget::BodyItem,
        );
    }

    // Bottom: controls, naming what Install would do to the selected part
    let key = |a| {
        app.core
            .config_manager
            .get_printable_key_for_keybind("body", a)
    };
    let mut controls_text = "[R] Repair".to_string();
    match fitting.as_slice() {
        [] => {}
        [(item, _)] => controls_text.push_str(&format!(
            "   [{}] Install {}",
            key("INSTALL_PART"),
            itemkind_name(*item)
        )),
        _ => controls_text.push_str(&format!("   [{}] Install...", key("INSTALL_PART"))),
    }
    if !lithicrivers_core::model::body::is_locked(selected_part) {
        controls_text.push_str(&format!("   [{}] Remove", key("REMOVE_PART")));
    }
    let controls_para = Paragraph::new(controls_text)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Line::from("Controls")),
        );
    f.render_widget(controls_para, controls_area);
}

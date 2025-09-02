use crate::rendering_helpers::{
    block_art_12x8_lines_for_position, empty_art_12x8_lines_for_position,
    entity_art_12x8_lines_for_position,
};
use lithicrivers_core::world::CHUNK_SIZE;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_look_panel(f: &mut Frame, app: &mut crate::App, area: Rect) {
    let pos = app.look_cursor;

    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(Span::raw(format!(
        "Pos: ({}, {}, {})",
        pos.x, pos.y, pos.z
    ))));

    lines.push(Line::from(Span::raw(format!(
        "Chunk: ({}, {}, {})",
        pos.x / CHUNK_SIZE,
        pos.y / CHUNK_SIZE,
        pos.z
    ))));

    lines.push(Line::from(Span::raw(format!(""))));

    // Entity and item info
    let mut any_entity = false;
    for (_e, (e_pos, maybe_player, maybe_sr, maybe_drop)) in app
        .game
        .world
        .query::<(
            &lithicrivers_core::components::Position,
            Option<&lithicrivers_core::components::Player>,
            Option<&lithicrivers_core::components::SpriteRef>,
            Option<&lithicrivers_core::components::DroppedItem>,
        )>()
        .iter()
    {
        if *e_pos == pos {
            any_entity = true;
            if maybe_player.is_some() {
                lines.push(Line::from(Span::styled(
                    "Entity: Player",
                    Style::default().fg(Color::Green),
                )));
            } else if let Some(sr) = maybe_sr {
                lines.push(Line::from(Span::raw(format!(
                    "Entity: {}::{}",
                    sr.category, sr.name
                ))));
            } else if let Some(di) = maybe_drop {
                lines.push(Line::from(Span::raw(format!(
                    "Dropped: {:?} x{}",
                    di.kind, di.qty
                ))));
            } else {
                lines.push(Line::from(Span::raw("Entity: (unknown)")));
            }
        }
    }
    if !any_entity {
        lines.push(Line::from(Span::raw("Entities: (none)")));
    }

    lines.push(Line::from("Entity art:"));
    // Gather 12x8 art for entity under cursor (if any)
    if entity_art_12x8_lines_for_position(app, pos, &mut lines) {
        lines.push(Line::from(""));
    } else {
        lines.extend(empty_art_12x8_lines_for_position());
        lines.push(Line::from(""));
    }

    // Tile info
    let tile_kind = app.game.res.world.get_tile(pos.x, pos.y, pos.z);
    lines.push(Line::from(Span::raw(format!("Tile: {:?}", tile_kind))));

    lines.push(Line::from("Tile art:"));
    if block_art_12x8_lines_for_position(app, pos, &mut lines) {
        lines.push(Line::from(""));
    } else {
        lines.extend(empty_art_12x8_lines_for_position());
        lines.push(Line::from(""));
    }

    let content = Paragraph::new(lines)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Look at")
                .style(Style::default().fg(Color::White)),
        );
    f.render_widget(content, area);
}

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
    let pos = app.panels.look.cursor;

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
    for (_e, (e_pos, maybe_player, maybe_sr, maybe_drop, maybe_inventory)) in app
        .core
        .game
        .world
        .query::<(
            &lithicrivers_core::components::Position,
            Option<&lithicrivers_core::components::Player>,
            Option<&lithicrivers_core::components::SpriteRef>,
            Option<&lithicrivers_core::components::DroppedItem>,
            Option<&lithicrivers_core::components::Inventory>,
        )>()
        .iter()
    {
        //are we at the right position?
        if *e_pos == pos {
            any_entity = true;

            // determine if we have any entity.
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

            //display inventory
            if let Some(inventory) = maybe_inventory {
                if inventory.slots.is_empty() {
                    lines.push(Line::from(Span::raw("Entity Inventory: (empty)")));
                } else {
                    lines.push(Line::from(Span::raw("Entity Inventory:")));
                    for item_stack in &inventory.slots {
                        let item_name =
                            lithicrivers_core::components::itemkind_name(item_stack.kind);

                        lines.push(Line::from(Span::raw(format!(
                            "  {} x{}",
                            item_name, item_stack.qty
                        ))));
                    }
                }
            }
        }
    }
    if !any_entity {
        lines.push(Line::from(Span::raw("Entities: (none)")));
    }

    // print the entity's inventory in the Look panel if it exists, with a special case for empty
    //TODO: Claude: Please educate me on how to add this by showing me, not by editing the file yourself.
    //  I'm a great Python developer but don't really know Rust well.

    lines.push(Line::from("Entity art:"));
    // Gather 12x8 art for entity under cursor (if any)
    if entity_art_12x8_lines_for_position(app, pos, &mut lines) {
        lines.push(Line::from(""));
    } else {
        lines.extend(empty_art_12x8_lines_for_position());
        lines.push(Line::from(""));
    }

    // Tile info
    let tile_kind = app
        .core
        .game
        .res
        .world_state
        .world
        .get_tile_at_z(pos.x, pos.y, pos.z);
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
                .title(Line::from("Look at"))
                .style(Style::default().fg(Color::White)),
        );
    f.render_widget(content, area);
}

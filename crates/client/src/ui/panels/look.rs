use crate::rendering_helpers::{
    block_art_12x8_lines_for_position, empty_art_12x8_lines_for_position,
    entity_art_12x8_lines_for_position,
};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn render_look_panel(f: &mut Frame, app: &mut crate::App, area: Rect) {
    let pos = app.panels.look.cursor;

    // What is here first, then the layers around it, then the art (cut off first when
    // the panel is short)
    let mut lines: Vec<Line> = vec![Line::from(format!(
        "Pos: ({}, {}, {})",
        pos.x, pos.y, pos.z
    ))];

    // Entity and item info
    let mut any_entity = false;
    for (_e, (e_pos, maybe_player, maybe_sr, maybe_drop, maybe_inventory, maybe_block)) in app
        .core
        .game
        .world
        .query::<(
            &lithicrivers_core::components::Position,
            Option<&lithicrivers_core::components::Player>,
            Option<&lithicrivers_core::components::SpriteRef>,
            Option<&lithicrivers_core::components::DroppedItem>,
            Option<&lithicrivers_core::components::Inventory>,
            Option<&lithicrivers_core::factory::FactoryBlock>,
        )>()
        .iter()
    {
        //are we at the right position?
        if *e_pos == pos {
            any_entity = true;

            // determine if we have any entity.
            if maybe_player.is_some() {
                lines.push(Line::from(Span::styled(
                    "You",
                    Style::default().fg(Color::Green),
                )));
            } else if let Some(block) = maybe_block {
                lines.push(Line::from(capitalize(block.kind.name())));
            } else if let Some(di) = maybe_drop {
                lines.push(Line::from(format!(
                    "{} x{} (lying here)",
                    lithicrivers_core::components::itemkind_name(di.kind),
                    di.qty
                )));
            } else if let Some(sr) = maybe_sr {
                lines.push(Line::from(capitalize(&sr.name.replace('_', " "))));
            } else {
                lines.push(Line::from("Something"));
            }

            //display inventory
            if let Some(inventory) = maybe_inventory {
                if inventory.slots.is_empty() {
                    lines.push(Line::from("  (empty)"));
                } else {
                    for item_stack in &inventory.slots {
                        let item_name =
                            lithicrivers_core::components::itemkind_name(item_stack.kind);

                        lines.push(Line::from(format!("  {} x{}", item_name, item_stack.qty)));
                    }
                }
            }
        }
    }
    if !any_entity {
        lines.push(Line::from("Nothing here"));
    }

    // print the entity's inventory in the Look panel if it exists, with a special case for empty
    //TODO: Claude: Please educate me on how to add this by showing me, not by editing the file yourself.
    //  I'm a great Python developer but don't really know Rust well.

    // Tile info
    let tile_kind = app
        .core
        .game
        .res
        .world_state
        .world
        .get_tile_at_z(pos.x, pos.y, pos.z);
    // A factory block's tile is only its marker
    if tile_kind != lithicrivers_core::tiles::TileKind::FactoryBlock {
        lines.push(Line::from(format!("Tile: {:?}", tile_kind)));
    }
    let lift = lift_target(app, pos);
    let lift_dz = if lift.first().is_some_and(|l| l.starts_with("Lifts")) {
        Some(1)
    } else if lift.is_empty() {
        None
    } else {
        Some(-1)
    };
    for line in lift {
        lines.push(Line::from(line));
    }
    // The layers above and below, so ghost stairs can be read (a vertical conveyor's own
    // direction is already named above)
    for (label, dz) in [("Above", 1), ("Below", -1)] {
        if lift_dz == Some(dz) {
            continue;
        }
        let around = lithicrivers_core::components::Position {
            z: pos.z + dz,
            ..pos
        };
        lines.push(Line::from(Span::styled(
            format!("{label}: {}", describe(app, around)),
            Style::default().fg(Color::Gray),
        )));
    }

    lines.push(Line::from(""));
    // The entity's art if something is here, else the tile's
    let mut art = Vec::new();
    if !entity_art_12x8_lines_for_position(app, pos, &mut art)
        && !block_art_12x8_lines_for_position(app, pos, &mut art)
    {
        art = empty_art_12x8_lines_for_position();
    }
    lines.extend(art);

    let content = Paragraph::new(lines)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Line::from("Look at"))
                .style(Style::default().fg(Color::White)),
        );
    f.render_widget(content, area);
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    chars
        .next()
        .map_or(String::new(), |c| c.to_uppercase().chain(chars).collect())
}

/// A factory block's name, or the tile, at `pos`.
fn describe(app: &mut crate::App, pos: lithicrivers_core::components::Position) -> String {
    let game = &mut app.core.game;
    if let Some(name) = game.res.factory_index.get(&pos).and_then(|&e| {
        game.world
            .get::<&lithicrivers_core::factory::FactoryBlock>(e)
            .ok()
            .map(|b| capitalize(b.kind.name()))
    }) {
        return name;
    }
    let tile = game
        .res
        .world_state
        .world
        .get_tile_at_z(pos.x, pos.y, pos.z);
    format!("{tile:?}")
}

/// For a vertical conveyor: the layer it feeds and what is there, on two short lines (the
/// panel is narrow). Nothing for other tiles.
fn lift_target(app: &mut crate::App, pos: lithicrivers_core::components::Position) -> Vec<String> {
    use lithicrivers_core::factory::FactoryBlock;
    let game = &mut app.core.game;
    let Some(&entity) = game.res.factory_index.get(&pos) else {
        return Vec::new();
    };
    let Some(dz) = game
        .world
        .get::<&FactoryBlock>(entity)
        .ok()
        .and_then(|b| b.kind.lift())
    else {
        return Vec::new();
    };
    let target = lithicrivers_core::components::Position {
        z: pos.z + dz,
        ..pos
    };
    let verb = if dz > 0 { "Lifts" } else { "Lowers" };
    let what = match game.res.factory_index.get(&target) {
        Some(&e) => game
            .world
            .get::<&FactoryBlock>(e)
            .map_or("a block".to_string(), |b| b.kind.name().to_string()),
        None => {
            let tile = game
                .res
                .world_state
                .world
                .get_tile_at_z(target.x, target.y, target.z);
            if tile.is_passable() {
                format!("{tile:?}, empty")
            } else {
                format!("{tile:?}, blocked")
            }
        }
    };
    vec![format!("{verb} to z = {}:", target.z), format!("  {what}")]
}

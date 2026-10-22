use crate::{
    sprite_constants::{sprite_for_view_reticle, sprite_for_view_reticle_color},
    sprite_loader::{sprite_block_for_spriteref, sprite_block_for_tile},
};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Wrap},
    Frame,
};
use std::collections::HashMap;

/// Convert a color to grayscale for fog of war rendering
fn to_grayscale(color: Color) -> Color {
    match color {
        Color::Rgb(r, g, b) => {
            // Standard grayscale conversion formula
            let gray = ((r as f32 * 0.299) + (g as f32 * 0.587) + (b as f32 * 0.114)) as u8;
            Color::Rgb(gray, gray, gray)
        }
        Color::Indexed(i) => {
            // For indexed colors, map to grayscale approximations
            match i {
                0 => Color::Black,         // Black -> Black
                1..=15 => Color::DarkGray, // Standard colors -> Dark Gray
                16..=231 => Color::Gray,   // 216-color cube -> Gray
                232..=255 => Color::White, // Grayscale ramp -> White
            }
        }
        // Named colors to grayscale
        Color::Black => Color::Black,
        Color::Red | Color::Green | Color::Yellow | Color::Blue | Color::Magenta | Color::Cyan => {
            Color::DarkGray
        }
        Color::Gray => Color::Gray,
        Color::DarkGray => Color::DarkGray,
        Color::LightRed
        | Color::LightGreen
        | Color::LightYellow
        | Color::LightBlue
        | Color::LightMagenta
        | Color::LightCyan => Color::Gray,
        Color::White => Color::White,
        Color::Reset => Color::Reset,
    }
}

/// The world tile under the view cell (`col`, `row`) of a `cols` x `rows` view centered on
/// (`center_x`, `center_y`) at `scale` cells per tile, and the cell's position inside that
/// tile's sprite. Rendering and mouse clicks both use this, so they always agree.
pub fn cell_to_tile(
    (center_x, center_y): (i64, i64),
    (cols, rows): (i64, i64),
    scale: i64,
    (col, row): (i64, i64),
) -> (i64, i64, i64, i64) {
    let offset_x = col - cols / 2;
    let offset_y = row - rows / 2;
    (
        center_x + offset_x.div_euclid(scale),
        center_y + offset_y.div_euclid(scale),
        offset_x.rem_euclid(scale),
        offset_y.rem_euclid(scale),
    )
}

/// The world tile under a screen cell, if the cell is inside the world view last drawn.
pub fn screen_to_tile(
    app: &crate::App,
    col: u16,
    row: u16,
) -> Option<lithicrivers_core::components::Position> {
    let area = app.ui.world_view_rect?;
    let inside = (area.x..area.x + area.width).contains(&col)
        && (area.y..area.y + area.height).contains(&row);
    if !inside {
        return None;
    }
    let (x, y, _, _) = cell_to_tile(
        (app.ui.view_x, app.ui.view_y),
        (area.width as i64, area.height as i64),
        app.ui.scale.as_i64(),
        ((col - area.x) as i64, (row - area.y) as i64),
    );
    Some(lithicrivers_core::components::Position {
        x,
        y,
        z: app.ui.view_z,
    })
}

pub fn render_game_view(f: &mut Frame, app: &mut crate::App, area: Rect) {
    // Remember where the world is drawn, for mouse clicks
    app.ui.world_view_rect = Some(area);
    // Get the game view from the core
    let _view = app.core.game.build_view();

    // Create the game display text
    let mut lines = Vec::new();

    // Render the map to exactly the panel's area
    let target_cols = area.width as usize;
    let target_rows = area.height as usize;

    // Note: No longer need to sync generation Z - using viewport-safe methods

    let scale = app.ui.scale.as_i64();

    // Compute world-space bounds for current UI viewport and prefetch chunks
    let center_x = app.ui.view_x;
    let center_y = app.ui.view_y;

    let world_cols = target_cols as i64 / scale;
    let world_rows = target_rows as i64 / scale;

    let left = center_x - world_cols / 2;
    let top = center_y - world_rows / 2;
    let right = left + world_cols;
    let bottom = top + world_rows;

    app.core
        .game
        .res
        .world_state
        .world
        .prefetch_rect_at_z(left, top, right, bottom, app.ui.view_z);

    // Build an entity overlay map for current bounds and Z slice using SpriteRef
    let mut ent_overlay: HashMap<(i64, i64), (String, String)> = HashMap::new();
    let z = app.ui.view_z;
    for (_e, (pos, sr_opt, belt)) in app
        .core
        .game
        .world
        .query::<(
            &lithicrivers_core::components::Position,
            Option<&lithicrivers_core::components::SpriteRef>,
            Option<&lithicrivers_core::factory::BeltSlot>,
        )>()
        .iter()
    {
        if pos.z != z {
            continue;
        }
        if pos.x >= left && pos.x <= right && pos.y >= top && pos.y <= bottom {
            // A belt carrying something shows what it carries
            if let Some(item) = belt.and_then(|b| b.item) {
                let name = lithicrivers_core::components::itemkind_sprite_name(item);
                ent_overlay.insert((pos.x, pos.y), ("items".to_string(), name.to_string()));
            } else if let Some(sr) = sr_opt {
                ent_overlay.insert((pos.x, pos.y), (sr.category.clone(), sr.name.clone()));
            }
        }
    }

    // Power wires between linked poles, shown on even seconds so the terrain shows through
    let wires = if (app.clock.animation_millis() / 1000) % 2 == 0 {
        wire_cells(&app.core.game.res.power.links, z)
    } else {
        HashMap::new()
    };

    for row in 0..target_rows {
        let mut spans = Vec::with_capacity(target_cols);
        for col in 0..target_cols {
            let (world_x, world_y, sprite_x, sprite_y) = cell_to_tile(
                (center_x, center_y),
                (target_cols as i64, target_rows as i64),
                scale,
                (col as i64, row as i64),
            );
            let world_z = app.ui.view_z;

            // Base tile color/glyph
            let tile_kind = app
                .core
                .game
                .res
                .world_state
                .world
                .get_tile_at_z(world_x, world_y, world_z);

            // Check fog of war state for this tile
            use lithicrivers_core::systems::fog_of_war::{get_fog_state, FogState};
            let fog_state = get_fog_state(
                &app.core.game.world,
                &app.core.game.res,
                world_x,
                world_y,
                world_z,
            );

            // If unvisited and fog of war is enabled, render as very dark grey '?'
            if fog_state == FogState::Unvisited && app.core.game.res.player_state.fog_of_war_enabled
            {
                spans.push(Span::styled(
                    "?".to_string(),
                    Style::default().fg(ratatui::style::Color::Rgb(16, 16, 16)), // Very dark grey instead of pure black
                ));
                continue;
            }

            // render look mode cursor first
            if app.panels.look.mode
                && world_x == app.panels.look.cursor.x
                && world_y == app.panels.look.cursor.y
                && app.ui.view_z == app.panels.look.cursor.z
            {
                let reticle_sprites = sprite_for_view_reticle();
                let scale_index = (app.ui.scale.as_i64() - 1) as usize;
                let reticle_block = reticle_sprites
                    .get(scale_index)
                    .unwrap_or(&reticle_sprites[0]);

                let reticle_char = reticle_block
                    .lines()
                    .nth(sprite_y as usize)
                    .and_then(|line| line.chars().nth(sprite_x as usize))
                    .unwrap_or(' ');

                spans.push(Span::styled(
                    reticle_char.to_string(),
                    Style::default().fg(sprite_for_view_reticle_color()),
                ));
                continue;
            }

            // Entities next
            if let Some((cat, name)) = ent_overlay.get(&(world_x, world_y)) {
                let sr = lithicrivers_core::components::SpriteRef {
                    category: cat.clone(),
                    name: name.clone(),
                };
                let (block, mut color) =
                    sprite_block_for_spriteref(&mut app.core.sprite_loader, &sr, app.ui.scale);
                let sprite_char = block
                    .lines()
                    .nth(sprite_y as usize)
                    .and_then(|line| line.chars().nth(sprite_x as usize))
                    .unwrap_or(' ');

                // Apply grayscale for visited but not illuminated tiles
                if fog_state == FogState::Visited {
                    color = to_grayscale(color);
                }

                spans.push(Span::styled(
                    sprite_char.to_string(),
                    Style::default().fg(color),
                ));
            } else if let Some(wire) = wires.get(&(world_x, world_y)) {
                spans.push(Span::styled(
                    wire.to_string(),
                    Style::default().fg(ratatui::style::Color::Yellow),
                ));
            } else {
                let (block, mut color) =
                    sprite_block_for_tile(&mut app.core.sprite_loader, tile_kind, app.ui.scale)
                        .unwrap_or_else(|| {
                            panic!("Could not find sprite for tile kind: {:?}", tile_kind)
                        });
                let sprite_char = block
                    .lines()
                    .nth(sprite_y as usize)
                    .and_then(|line| line.chars().nth(sprite_x as usize))
                    .unwrap_or(' ');

                // Apply grayscale for visited but not illuminated tiles
                if fog_state == FogState::Visited {
                    color = to_grayscale(color);
                }

                spans.push(Span::styled(
                    sprite_char.to_string(),
                    Style::default().fg(color),
                ));
            }
        }
        lines.push(Line::from(spans));
    }

    hover_overlay(app, area, &mut lines);

    let paragraph = Paragraph::new(lines)
        .block(Block::default())
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

/// Highlight the tile under the mouse; in Place mode, show a dim ghost of the selected
/// hotbar block there, with its facing. Drawing only: nothing in the game changes.
fn hover_overlay(app: &mut crate::App, area: Rect, lines: &mut [Line]) {
    let Some((mouse_col, mouse_row)) = app.ui.mouse_cell else {
        return;
    };
    let Some(tile) = screen_to_tile(app, mouse_col, mouse_row) else {
        return;
    };
    let scale = app.ui.scale.as_i64();
    // The ghost: the selected hotbar block's sprite at this zoom
    let ghost = (app.panels.build.mode == crate::app_state::BuildMode::Place)
        .then(|| app.panels.build.hotbar_assignments[app.panels.build.selected_hotbar_slot])
        .flatten()
        .map(|item| {
            let sr = match lithicrivers_core::factory::FactoryKind::from_item(item) {
                Some(kind) => kind.sprite(app.panels.build.placement_facing),
                None => lithicrivers_core::components::SpriteRef::new(
                    "items",
                    lithicrivers_core::components::itemkind_sprite_name(item),
                ),
            };
            let (block, _) = crate::sprite_loader::sprite_block_for_spriteref(
                &mut app.core.sprite_loader,
                &sr,
                app.ui.scale,
            );
            block
                .lines()
                .map(|l| l.chars().collect::<Vec<char>>())
                .collect::<Vec<_>>()
        });
    for (row, line) in lines.iter_mut().enumerate() {
        for (col, span) in line.spans.iter_mut().enumerate() {
            let (x, y, sx, sy) = cell_to_tile(
                (app.ui.view_x, app.ui.view_y),
                (area.width as i64, area.height as i64),
                scale,
                (col as i64, row as i64),
            );
            if (x, y) != (tile.x, tile.y) {
                continue;
            }
            match &ghost {
                Some(rows) => {
                    let c = rows
                        .get(sy as usize)
                        .and_then(|r| r.get(sx as usize))
                        .copied()
                        .unwrap_or(' ');
                    *span = Span::styled(
                        c.to_string(),
                        Style::default()
                            .fg(Color::Gray)
                            .add_modifier(Modifier::REVERSED),
                    );
                }
                None => {
                    span.style = span.style.add_modifier(Modifier::REVERSED);
                }
            }
        }
    }
}

/// The tiles between linked poles on level `z`, with the character that draws the wire there:
/// a Bresenham line between the two poles, not including the poles themselves. Each tile's
/// character follows the step that reached it (`─`, `│`, `╲` or `╱`).
fn wire_cells(
    links: &[(
        lithicrivers_core::components::Position,
        lithicrivers_core::components::Position,
    )],
    z: i64,
) -> HashMap<(i64, i64), char> {
    let mut cells = HashMap::new();
    for (a, b) in links.iter().filter(|(a, _)| a.z == z) {
        let (dx, dy) = (b.x - a.x, b.y - a.y);
        let (step_x, step_y) = (dx.signum(), dy.signum());
        let (adx, ady) = (dx.abs(), -dy.abs());
        let (mut x, mut y, mut err) = (a.x, a.y, adx + ady);
        loop {
            let e2 = 2 * err;
            let (mut moved_x, mut moved_y) = (false, false);
            if e2 >= ady {
                err += ady;
                x += step_x;
                moved_x = true;
            }
            if e2 <= adx {
                err += adx;
                y += step_y;
                moved_y = true;
            }
            if (x, y) == (b.x, b.y) {
                break;
            }
            let glyph = match (moved_x, moved_y) {
                (true, false) => '─',
                (false, true) => '│',
                _ if step_x == step_y => '╲',
                _ => '╱',
            };
            cells.insert((x, y), glyph);
        }
    }
    cells
}

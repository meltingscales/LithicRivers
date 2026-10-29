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
/// How bright remembered surface tiles are at night, in thousandths.
const NIGHT_MEMORY_LIGHT: u16 = 350;

/// Darken an RGB color to `light` thousandths of its brightness.
fn dim(color: Color, light: u16) -> Color {
    match color {
        Color::Rgb(r, g, b) if light < 1000 => {
            let scale = |c: u8| (c as u32 * light as u32 / 1000) as u8;
            Color::Rgb(scale(r), scale(g), scale(b))
        }
        other => other,
    }
}

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

    // Dawn and dusk dim the surface between full light and 45%. At night the surface uses fog
    // of war, and what the robot remembers there is drawn dark, so night looks like night
    let (surface_light, remembered_light): (u16, u16) = {
        use lithicrivers_core::game_time::DayPhase;
        let time = &app.core.game.res.time;
        match time.phase() {
            DayPhase::Dawn | DayPhase::Dusk if app.ui.view_z >= 0 => {
                let light = (450 + 550 * time.daylight_permille() / 1000) as u16;
                (light, light)
            }
            DayPhase::Night if app.ui.view_z >= 0 => (1000, NIGHT_MEMORY_LIGHT),
            _ => (1000, 1000),
        }
    };
    use lithicrivers_core::systems::fog_of_war::{FogLookup, FogState};
    // The world's fog memory and the player's light, read once for the whole frame
    let fog = FogLookup::new(&app.core.game.world);
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
            let fog_state = fog.state(&app.core.game.res, world_x, world_y, world_z);

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
                    color = dim(to_grayscale(color), remembered_light);
                }
                color = dim(color, surface_light);

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
                    color = dim(to_grayscale(color), remembered_light);
                }
                color = dim(color, surface_light);

                spans.push(Span::styled(
                    sprite_char.to_string(),
                    Style::default().fg(color),
                ));
            }
        }
        lines.push(Line::from(spans));
    }
    drop(fog);

    climb_overlay(app, area, &mut lines);
    hover_overlay(app, area, &mut lines);
    blueprint_overlay(app, area, &mut lines);

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

type SpriteRows = Vec<Vec<char>>;

/// Ghost stairs, on odd seconds (the opposite beat to wires), on tiles with something to
/// climb directly above (gray) or below (dark gray): where a stairway would lead, or where
/// one ends. Tiles that are climbable themselves, or have anything on them, draw as they are.
fn climb_overlay(app: &mut crate::App, area: Rect, lines: &mut [Line]) {
    if (app.clock.animation_millis() / 1000) % 2 == 0 {
        return;
    }
    let z = app.ui.view_z;
    let game = &app.core.game;
    let climbable = |x: i64, y: i64, z: i64| {
        let pos = lithicrivers_core::components::Position { x, y, z };
        game.res
            .world_state
            .world
            .get_tile_cached_no_worldgen(x, y, z)
            == lithicrivers_core::tiles::TileKind::Stairs
            || game.res.factory_index.get(&pos).is_some_and(|&e| {
                game.world
                    .get::<&lithicrivers_core::factory::FactoryBlock>(e)
                    .is_ok_and(|b| b.kind.lift().is_some())
            })
    };
    // Only bare terrain: never over the player, creatures, items or blocks
    let occupied: std::collections::HashSet<(i64, i64)> = game
        .world
        .query::<&lithicrivers_core::components::Position>()
        .iter()
        .filter(|(_, p)| p.z == z)
        .map(|(_, p)| (p.x, p.y))
        .collect();
    let scale = app.ui.scale.as_i64();
    let mut ghosts: HashMap<(i64, i64), Option<Color>> = HashMap::new();
    for (row, line) in lines.iter().enumerate() {
        for col in 0..line.spans.len() {
            let (x, y, _, _) = cell_to_tile(
                (app.ui.view_x, app.ui.view_y),
                (area.width as i64, area.height as i64),
                scale,
                (col as i64, row as i64),
            );
            ghosts.entry((x, y)).or_insert_with(|| {
                if occupied.contains(&(x, y)) || climbable(x, y, z) {
                    None
                } else if climbable(x, y, z + 1) {
                    Some(Color::Gray)
                } else if climbable(x, y, z - 1) {
                    Some(Color::DarkGray)
                } else {
                    None
                }
            });
        }
    }
    if ghosts.values().all(Option::is_none) {
        return;
    }
    let (sprite, _) = crate::sprite_loader::sprite_block_for_spriteref(
        &mut app.core.sprite_loader,
        &lithicrivers_core::components::SpriteRef::new("tiles", "stairs"),
        app.ui.scale,
    );
    let rows: SpriteRows = sprite.lines().map(|l| l.chars().collect()).collect();
    for (row, line) in lines.iter_mut().enumerate() {
        for (col, span) in line.spans.iter_mut().enumerate() {
            let (x, y, sx, sy) = cell_to_tile(
                (app.ui.view_x, app.ui.view_y),
                (area.width as i64, area.height as i64),
                scale,
                (col as i64, row as i64),
            );
            if let Some(Some(color)) = ghosts.get(&(x, y)) {
                let c = rows
                    .get(sy as usize)
                    .and_then(|r| r.get(sx as usize))
                    .copied()
                    .unwrap_or(' ');
                *span = Span::styled(c.to_string(), Style::default().fg(*color));
            }
        }
    }
}

/// Blueprint mode: the cursor, the area being copied, and the paste preview, which shows on
/// even seconds like wires. Free tiles draw cyan, tiles it would replace yellow, and tiles
/// that block it red.
fn blueprint_overlay(app: &mut crate::App, area: Rect, lines: &mut [Line]) {
    use crate::app::input::blueprint;
    use lithicrivers_core::blueprints::{self as bp, Piece, Target};
    if app.panels.build.mode != crate::app_state::BuildMode::Blueprint {
        return;
    }
    let Some(cursor) = blueprint::cursor_tile(app) else {
        return;
    };
    // Each marked tile: the sprite to draw there (or none, to tint what is there), and a color
    let mut cells: HashMap<(i64, i64), (Option<SpriteRows>, Color)> = HashMap::new();
    let tool = app.panels.build.blueprint.clone();
    if tool.copying {
        let first = tool.first_corner.unwrap_or(cursor);
        for x in first.x.min(cursor.x)..=first.x.max(cursor.x) {
            for y in first.y.min(cursor.y)..=first.y.max(cursor.y) {
                cells.insert((x, y), (None, Color::LightMagenta));
            }
        }
    } else if let Some(blueprint) = blueprint::paste_preview(app) {
        let game = &mut app.core.game;
        let targets = bp::targets(&game.world, &mut game.res, &blueprint, cursor);
        let count = |t| targets.iter().filter(|(_, target)| *target == t).count();
        app.panels.build.blueprint.preview = (count(Target::Replace), count(Target::Blocked));
        if (app.clock.animation_millis() / 1000) % 2 == 0 {
            for (block, (pos, target)) in blueprint.blocks.iter().zip(targets) {
                let sr = match block.piece {
                    Piece::Factory(kind) => kind.sprite(block.facing),
                    Piece::Tile(tile) => {
                        lithicrivers_core::components::SpriteRef::new("tiles", tile.sprite_key())
                    }
                };
                let (sprite, _) = crate::sprite_loader::sprite_block_for_spriteref(
                    &mut app.core.sprite_loader,
                    &sr,
                    app.ui.scale,
                );
                let rows = sprite.lines().map(|l| l.chars().collect()).collect();
                let color = match target {
                    Target::Free => Color::Cyan,
                    Target::Replace => Color::Yellow,
                    Target::Blocked => Color::Red,
                };
                cells.insert((pos.x, pos.y), (Some(rows), color));
            }
        }
    }
    cells
        .entry((cursor.x, cursor.y))
        .or_insert((None, Color::LightMagenta));
    let scale = app.ui.scale.as_i64();
    for (row, line) in lines.iter_mut().enumerate() {
        for (col, span) in line.spans.iter_mut().enumerate() {
            let (x, y, sx, sy) = cell_to_tile(
                (app.ui.view_x, app.ui.view_y),
                (area.width as i64, area.height as i64),
                scale,
                (col as i64, row as i64),
            );
            match cells.get(&(x, y)) {
                Some((Some(rows), color)) => {
                    let c = rows
                        .get(sy as usize)
                        .and_then(|r| r.get(sx as usize))
                        .copied()
                        .unwrap_or(' ');
                    *span = Span::styled(c.to_string(), Style::default().fg(*color));
                }
                Some((None, color)) => {
                    span.style = span.style.fg(*color).add_modifier(Modifier::REVERSED);
                }
                None => {}
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

#[cfg(test)]
mod night_tests {
    use crate::headless::chain_commands;
    use ratatui::style::Color;

    /// The brightest channel of the cell `dx` columns from the player at 100x30.
    fn brightness(runner: &mut crate::headless::Runner, dx: i32) -> u8 {
        let (col, row) = ((38 + dx) as u16, 10);
        match runner.buffer()[(col, row)].fg {
            Color::Rgb(r, g, b) => r.max(g).max(b),
            other => panic!("expected an RGB tile color, got {other:?}"),
        }
    }

    #[test]
    fn remembered_surface_tiles_are_dark_at_night() {
        // Walk three tiles east, so the start is remembered but out of the light
        let walk = ["key movement.MOVE_EAST 3"];
        let mut day = chain_commands(&["skip-splash", "key ui.MENU_PREV", "cheat tp 0 0 0"]);
        day.chain(&walk);
        let mut night = chain_commands(&[
            "skip-splash",
            "key ui.MENU_PREV",
            "cheat tp 0 0 0",
            "cheat set_time 23:00",
            "key movement.WAIT",
        ]);
        night.chain(&walk);
        let (day_tile, night_tile) = (brightness(&mut day, -3), brightness(&mut night, -3));
        assert!(
            night_tile * 2 < day_tile,
            "remembered at night {night_tile}, by day {day_tile}"
        );
        // Within the light, night looks like day
        assert_eq!(brightness(&mut night, 1), brightness(&mut day, 1));
    }

    /// Performance guard: drawing the world builds one fog lookup for the whole frame,
    /// not one per tile.
    #[test]
    fn a_frame_reads_fog_of_war_once() {
        use lithicrivers_core::systems::fog_of_war::fog_lookups_built;
        // Underground, where every tile needs its fog state
        let mut runner = chain_commands(&["skip-splash", "key ui.MENU_PREV"]);
        let before = fog_lookups_built();
        runner.screen_text();
        assert_eq!(fog_lookups_built() - before, 1);
    }
}

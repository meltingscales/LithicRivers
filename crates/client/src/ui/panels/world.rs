use crate::{
    sprite_constants::{sprite_for_view_reticle, sprite_for_view_reticle_color},
    sprite_loader::{sprite_block_for_spriteref, sprite_block_for_tile},
};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
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

pub fn render_game_view(f: &mut Frame, app: &mut crate::App, area: Rect) {
    // Get the game view from the core
    let _view = app.core.game.build_view();

    // Create the game display text
    let mut lines = Vec::new();

    // Render the map to exactly the panel's area
    let target_cols = area.width as usize;
    let target_rows = area.height as usize;

    // Note: No longer need to sync generation Z - using viewport-safe methods

    let scale = app.ui.scale.as_i64() as i64;

    // Compute world-space bounds for current UI viewport and prefetch chunks
    let center_x = app.ui.view_x;
    let center_y = app.ui.view_y;

    let world_cols = (target_cols as i64 / scale) as i64;
    let world_rows = (target_rows as i64 / scale) as i64;

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
    for (_e, (pos, sr_opt)) in app
        .core
        .game
        .world
        .query::<(
            &lithicrivers_core::components::Position,
            Option<&lithicrivers_core::components::SpriteRef>,
        )>()
        .iter()
    {
        if pos.z != z {
            continue;
        }
        if pos.x >= left && pos.x <= right && pos.y >= top && pos.y <= bottom {
            if let Some(sr) = sr_opt {
                ent_overlay.insert((pos.x, pos.y), (sr.category.clone(), sr.name.clone()));
            }
        }
    }

    for row in 0..target_rows {
        let mut spans = Vec::with_capacity(target_cols);
        for col in 0..target_cols {
            let offset_x = col as i64 - target_cols as i64 / 2;
            let offset_y = row as i64 - target_rows as i64 / 2;
            let world_x = center_x + offset_x.div_euclid(scale);
            let world_y = center_y + offset_y.div_euclid(scale);
            let world_z = app.ui.view_z;

            let sprite_x = (col as i64 - target_cols as i64 / 2).rem_euclid(scale);
            let sprite_y = (row as i64 - target_rows as i64 / 2).rem_euclid(scale);

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
            let fog_state = get_fog_state(&app.core.game.world, world_x, world_y, world_z);

            // If unvisited, render as black '?'
            if fog_state == FogState::Unvisited {
                spans.push(Span::styled(
                    "?".to_string(),
                    Style::default().fg(ratatui::style::Color::Black),
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

    let paragraph = Paragraph::new(lines)
        .block(Block::default())
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

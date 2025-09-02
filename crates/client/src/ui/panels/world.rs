use crate::{
    sprite_constants::{sprite_for_view_reticle, sprite_for_view_reticle_color},
    sprite_loader::{sprite_block_for_spriteref, sprite_block_for_tile},
};
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Paragraph, Wrap},
    Frame,
};
use std::collections::HashMap;

pub fn render_game_view(f: &mut Frame, app: &mut crate::App, area: Rect) {
    // Get the game view from the core
    let _view = app.game.build_view();

    // Create the game display text
    let mut lines = Vec::new();

    // Render the map to exactly the panel's area
    let target_cols = area.width as usize;
    let target_rows = area.height as usize;

    // Sync world generation Z with current view slice
    app.game.res.world.set_generation_z(app.game.res.view_z);

    let scale = app.scale.as_u32() as i32;

    // Compute world-space bounds for current viewport and prefetch chunks
    let center_x = app.game.res.view_x;
    let center_y = app.game.res.view_y;

    let world_cols = (target_cols as f32 / scale as f32).ceil() as i32;
    let world_rows = (target_rows as f32 / scale as f32).ceil() as i32;

    let left = center_x - world_cols / 2;
    let top = center_y - world_rows / 2;
    let right = left + world_cols;
    let bottom = top + world_rows;

    app.game
        .res
        .world
        .prefetch_rect(left, top, right, bottom, app.game.res.view_z);

    // Build an entity overlay map for current bounds and Z slice using SpriteRef
    let mut ent_overlay: HashMap<(i32, i32), (String, String)> = HashMap::new();
    let z = app.game.res.view_z;
    for (_e, (pos, sr_opt)) in app
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
            let offset_x = col as i32 - target_cols as i32 / 2;
            let offset_y = row as i32 - target_rows as i32 / 2;
            let world_x = center_x + offset_x.div_euclid(scale);
            let world_y = center_y + offset_y.div_euclid(scale);
            let world_z = app.game.res.view_z;

            let sprite_x = (col as i32 - target_cols as i32 / 2).rem_euclid(scale);
            let sprite_y = (row as i32 - target_rows as i32 / 2).rem_euclid(scale);

            // Base tile color/glyph
            let tile_kind = app
                .game
                .res
                .world
                .get_tile_cached(world_x, world_y, world_z);

            // render look mode cursor first
            if app.look_mode
                && world_x == app.look_cursor.x
                && world_y == app.look_cursor.y
                && app.game.res.view_z == app.look_cursor.z
            {
                let reticle_sprites = sprite_for_view_reticle();
                let scale_index = (app.scale.as_u32() - 1) as usize;
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
                let (block, color) =
                    sprite_block_for_spriteref(&mut app.sprite_loader, &sr, app.scale);
                let sprite_char = block
                    .lines()
                    .nth(sprite_y as usize)
                    .and_then(|line| line.chars().nth(sprite_x as usize))
                    .unwrap_or(' ');
                spans.push(Span::styled(
                    sprite_char.to_string(),
                    Style::default().fg(color),
                ));
            } else {
                let (block, color) =
                    sprite_block_for_tile(&mut app.sprite_loader, tile_kind, app.scale)
                        .unwrap_or_else(|| {
                            panic!("Could not find sprite for tile kind: {:?}", tile_kind)
                        });
                let sprite_char = block
                    .lines()
                    .nth(sprite_y as usize)
                    .and_then(|line| line.chars().nth(sprite_x as usize))
                    .unwrap_or(' ');
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

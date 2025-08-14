use crate::components::{Glyph, Position};
use crate::resources::Resources;
use hecs::{World};

#[derive(Debug, Clone)]
pub struct RenderView {
    pub gametick: u64,
    pub player_pos: Position,
    pub map_lines: Vec<String>,
}

pub fn build_render_view(world: &World, res: &Resources) -> RenderView {
    let mut player_pos = Position { x: 0, y: 0, z: 0 };
    if let Some(e) = res.player_entity {
        if let Ok(p) = world.get::<&Position>(e) {
            player_pos = *p;
        }
    }
    // Build a window around the player from the infinite world at z=0.
    // Choose a generous default window; TUI will crop/scale it.
    let win_w: i32 = 50;
    let win_h: i32 = 50;
    let half_w = win_w / 2;
    let half_h = win_h / 2;
    let center_x = player_pos.x;
    let center_y = player_pos.y;

    let top = center_y - half_h;
    let left = center_x - half_w;

    // Build background from tiles
    let mut buffer: Vec<Vec<char>> = vec![vec![' '; win_w as usize]; win_h as usize];
    for y in 0..win_h {
        let wy = top + y;
        for x in 0..win_w {
            let wx = left + x;
            // Rendering is now handled in the client crate (sprite_loader)
buffer[y as usize][x as usize] = ' ';
            // Overlay fluid glyph if fluid exists at this position (z=0)
            if let Some(fluid) = res.fluids.get_fluid(crate::components::Position { x: wx, y: wy, z: 0 }) {
                // For now, always '~' for water
                buffer[y as usize][x as usize] = match fluid.fluid_type {
                    crate::resources::fluids::FluidType::Water => '~',
                    // Add more fluid types here
                };
            }
        }
    }
    // Overlay entities with Glyph in this window at the player's z
    let z = player_pos.z;
    for (_e, (pos, glyph)) in world.query::<(&Position, &Glyph)>().iter() {
        if pos.z != z { continue; }
        let vx = pos.x - left; let vy = pos.y - top;
        if vx >= 0 && vx < win_w && vy >= 0 && vy < win_h {
            buffer[vy as usize][vx as usize] = glyph.0;
        }
    }
    // Convert to lines
    let mut lines: Vec<String> = Vec::with_capacity(win_h as usize);
    for y in 0..win_h { lines.push(buffer[y as usize].iter().collect()); }
    RenderView { gametick: res.gametick, player_pos, map_lines: lines }
}

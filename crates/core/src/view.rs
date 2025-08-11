use crate::components::Position;
use crate::resources::Resources;
use crate::resources::world::Tile;
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
    // Build simple ASCII map from res.world at z=0
    let mut lines = Vec::with_capacity(res.world.height);
    for y in 0..res.world.height {
        let mut row = String::with_capacity(res.world.width);
        for x in 0..res.world.width {
            let ch = match res.world.get(x, y) {
                Tile::Wall => '#',
                Tile::Floor => '.',
            };
            row.push(ch);
        }
        lines.push(row);
    }
    // Overlay player '@' if within bounds (z ignored)
    if player_pos.z == 0 {
        let (px, py) = (player_pos.x, player_pos.y);
        if px >= 0 && py >= 0 {
            let (px, py) = (px as usize, py as usize);
            if py < lines.len() && px < lines[py].len() {
                let bytes = unsafe { lines[py].as_bytes_mut() };
                bytes[px] = b'@';
            }
        }
    }
    RenderView { gametick: res.gametick, player_pos, map_lines: lines }
}

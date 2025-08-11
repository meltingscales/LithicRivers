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

pub fn build_render_view(world: &World, res: &mut Resources) -> RenderView {
    let mut player_pos = Position { x: 0, y: 0, z: 0 };
    if let Some(e) = res.player_entity {
        if let Ok(p) = world.get::<&Position>(e) {
            player_pos = *p;
        }
    }
    // Build a window around the player from the infinite world at z=0.
    // Choose a generous default window; TUI will crop/scale it.
    let win_w: i32 = 160;
    let win_h: i32 = 80;
    let half_w = win_w / 2;
    let half_h = win_h / 2;
    let center_x = player_pos.x;
    let center_y = player_pos.y;

    let top = center_y - half_h;
    let left = center_x - half_w;

    let mut lines: Vec<String> = Vec::with_capacity(win_h as usize);
    for y in 0..win_h {
        let wy = top + y;
        let mut row = String::with_capacity(win_w as usize);
        for x in 0..win_w {
            let wx = left + x;
            let ch = if player_pos.z == 0 && wx == center_x && wy == center_y {
                '@'
            } else {
                match res.world.get_tile(wx, wy) {
                    Tile::Wall => '#',
                    Tile::Floor => '.',
                }
            };
            row.push(ch);
        }
        lines.push(row);
    }
    RenderView { gametick: res.gametick, player_pos, map_lines: lines }
}

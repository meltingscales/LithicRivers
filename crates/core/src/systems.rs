use crate::components::Position;
use crate::resources::Resources;
use crate::tiles::TileKind;
use hecs::World;
use tracing::info;

pub fn move_player_system(world: &mut World, res: &mut Resources) {
    if let Some((dx, dy)) = res.player_move_intent.take() {
        if let Some(player_e) = res.player_entity {
            if let Ok(mut pos) = world.get::<&mut Position>(player_e) {
                let nx = pos.x + dx;
                let ny = pos.y + dy;
                let t = res.world.get_tile(nx, ny);
                if t.is_passable() {
                    pos.x = nx;
                    pos.y = ny;
                } else {
                    info!("Blocked by {:?} at ({}, {})", t, nx, ny);
                    // Set resource for last blocked tile
                    res.last_blocked_tile = Some((nx, ny));
                }
            }
        }
    }
}

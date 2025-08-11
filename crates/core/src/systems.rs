use crate::components::Position;
use crate::resources::Resources;
use hecs::World;

pub fn move_player_system(world: &mut World, res: &mut Resources) {
    if let Some((dx, dy)) = res.player_move_intent.take() {
        if let Some(player_e) = res.player_entity {
            if let Ok(mut pos) = world.get::<&mut Position>(player_e) {
                let nx = pos.x + dx;
                let ny = pos.y + dy;
                // Collision check against walls in infinite, lazy-generated world
                use crate::resources::world::Tile;
                let t = res.world.get_tile(nx, ny);
                if t != Tile::Wall {
                    pos.x = nx;
                    pos.y = ny;
                }
            }
        }
    }
}

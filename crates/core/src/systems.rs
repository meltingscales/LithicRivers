use crate::components::Position;
use crate::resources::Resources;
use hecs::World;

pub fn move_player_system(world: &mut World, res: &mut Resources) {
    if let Some((dx, dy)) = res.player_move_intent.take() {
        if let Some(player_e) = res.player_entity {
            if let Ok(mut pos) = world.get::<&mut Position>(player_e) {
                let nx = pos.x + dx;
                let ny = pos.y + dy;
                // Bounds and collision check
                if nx >= 0 && ny >= 0 {
                    let (ux, uy) = (nx as usize, ny as usize);
                    if ux < res.world.width && uy < res.world.height {
                        use crate::resources::world::Tile;
                        let t = res.world.get(ux, uy);
                        if t != Tile::Wall {
                            pos.x = nx;
                            pos.y = ny;
                        }
                    }
                }
            }
        }
    }
}

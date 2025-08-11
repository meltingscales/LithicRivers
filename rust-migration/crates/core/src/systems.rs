use crate::components::Position;
use crate::resources::Resources;
use hecs::World;

pub fn move_player_system(world: &mut World, res: &mut Resources) {
    if let Some(player_e) = res.player_entity {
        if let Ok(mut pos) = world.get::<&mut Position>(player_e) {
            // Simple demo: move +1 x every 10 ticks deterministically
            if res.gametick % 10 == 0 {
                pos.x += 1;
            }
        }
    }
}

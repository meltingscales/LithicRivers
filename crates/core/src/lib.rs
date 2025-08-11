pub mod components;
pub mod resources;
pub mod systems;
pub mod view;

use components::*;
use resources::*;
use systems::*;
use view::*;

use hecs::World;

pub struct Game {
    pub world: World,
    pub res: Resources,
}

impl Game {
    pub fn new(seed: u64) -> Self {
        let mut world = World::new();
        let mut res = Resources::new(seed);
        // Spawn a player entity with a Position
        let player = world.spawn((Position { x: 1, y: 1, z: 0 }, Player));
        res.player_entity = Some(player);
        Self { world, res }
    }

    pub fn queue_player_move(&mut self, dx: i32, dy: i32) {
        self.res.player_move_intent = Some((dx, dy));
    }

    pub fn tick(&mut self) {
        // In the future, run an ordered system schedule.
        // For now, just increment tick and maybe move the player slowly.
        self.res.gametick += 1;
        move_player_system(&mut self.world, &mut self.res);
    }

    pub fn build_view(&self) -> RenderView {
        build_render_view(&self.world, &self.res)
    }
}

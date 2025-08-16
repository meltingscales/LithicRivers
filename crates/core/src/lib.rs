pub mod components;
pub mod palette;
pub mod resources;
mod structure;
pub mod systems;
pub mod tiles;
pub mod view;
pub mod model;

use components::*;
use resources::*;
use systems::*;
use view::*;
use model::body::Body;

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
        let player = world.spawn((
            Position { x: 1, y: 1, z: 0 },
            GameEntity,
            Player,
            Body::default(),
            Glyph('@'),
            BlocksMovement,
        ));
        res.player_entity = Some(player);
        // Spawn debug fluid pools around player
        crate::resources::fluids::spawn_debug_pools(&mut res.fluids, Position { x: 1, y: 1, z: 0 });
        // Spawn a simple StumblingSheep near the player (closer for visibility)
        world.spawn((
            Position { x: 2, y: 2, z: 0 },
            GameEntity,
            Sheep,
            Glyph('s'),
            BlocksMovement,
        ));
        Self { world, res }
    }

    pub fn queue_player_move(&mut self, dx: i32, dy: i32) {
        self.res.player_move_intent = Some((dx, dy));
        // Set the move cost so the next tick advances by this many ticks,
        // using the player's Body.walk_speed_mult if available.
        let mult: f32 = if let Some(e) = self.res.player_entity {
            if let Ok(body) = self.world.get::<&Body>(e) {
                body.walk_speed_modifier()
            } else {
                1.0
            }
        } else {
            1.0
        };
        let base: f32 = 200.0;
        let cost = (base / mult.max(0.01)).round().max(1.0) as u64;
        self.res.pending_tick_increase = Some(cost);
    }

    pub fn tick(&mut self) {
        // In the future, run an ordered system schedule.
        // For now, just increment tick and maybe move the player slowly.
        let inc = self.res.pending_tick_increase.take().unwrap_or(1);
        self.res.gametick = self.res.gametick.saturating_add(inc);
        move_player_system(&mut self.world, &mut self.res);
        stumbling_sheep_system(&mut self.world, &mut self.res);
        // Process fluids
        self.res
            .fluids
            .process_fluids(&self.res.world, self.res.gametick);
    }

    pub fn build_view(&self) -> RenderView {
        build_render_view(&self.world, &self.res)
    }
}

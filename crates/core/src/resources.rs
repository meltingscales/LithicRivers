use hecs::Entity;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;

pub mod fluids;
pub mod world;

pub struct Resources {
    pub seed: u64,
    pub rng: ChaCha20Rng,
    pub gametick: u64,
    pub player_entity: Option<Entity>,
    pub world: world::World,
    pub fluids: fluids::FluidManager, // Fluid system
    // Input intents (single-step for now)
    pub player_move_intent: Option<(i32, i32)>,
    pub last_blocked_tile: Option<(i32, i32)>,
    // If set, the next tick will advance by this many ticks (e.g., move/action cost)
    pub pending_tick_increase: Option<u64>,
}

impl Resources {
    pub fn new(seed: u64) -> Self {
        let rng = ChaCha20Rng::seed_from_u64(seed);
        Self {
            seed,
            rng,
            gametick: 0,
            player_entity: None,
            world: world::World::new(80, 24, seed),
            fluids: fluids::FluidManager::default(),
            player_move_intent: None,
            last_blocked_tile: None,
            pending_tick_increase: None,
        }
    }

    /// Placeholder for a future body-based speed system.
    /// In the Python version this comes from player.get_walk_speed_modifier().
    /// For now, return 1.0 (no modification).
    pub fn walk_speed_modifier(&self) -> f32 {
        1.0
    }

    /// Compute the tick cost for a single player move, applying speed modifiers.
    /// Base is 200 ticks; higher speed reduces cost.
    pub fn move_cost_ticks(&self) -> u64 {
        let base: f32 = 200.0;
        let modifier = self.walk_speed_modifier().max(0.01);
        let cost = (base / modifier).round();
        cost.max(1.0) as u64
    }
}

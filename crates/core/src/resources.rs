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
    // Mining intent: when set, player will mine the current tile on next tick
    pub mining_intent: bool,
    // Simple message log for UI
    pub messages: Vec<String>,
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
            mining_intent: false,
            messages: Vec::new(),
        }
    }

    pub fn log<S: Into<String>>(&mut self, msg: S) {
        let m = msg.into();
        self.messages.push(format!("[{}] {}", self.gametick, m));
        if self.messages.len() > 200 {
            let overflow = self.messages.len() - 200;
            self.messages.drain(0..overflow);
        }
    }
}

use crate::config::ConfigManager;
use hecs::Entity;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use tracing::info;

pub mod world;

// Re-export World and Chunk for easier access
pub use world::{Chunk, World};

pub struct Resources {
    pub seed: u64,
    pub rng: ChaCha20Rng,
    pub gametick: u64,
    pub player_entity: Option<Entity>,
    pub world: world::World,
    pub config: ConfigManager,
    pub developer_mode: bool,
    pub player_name: String,
    // Input intents (single-step for now)
    pub player_move_intent: Option<(i32, i32)>,
    pub last_blocked_tile: Option<(i32, i32)>,
    // If set, the next tick will advance by this many ticks (e.g., move/action cost)
    pub pending_tick_increase: Option<u64>,
    // Mining intent: when set, player will mine the current tile on next tick
    pub mining_intent: bool,
    // Simple message log for UI
    pub messages: Vec<String>,
    // Current viewed viewport center in world-space
    pub view_x: i32,
    pub view_y: i32,
    // Current viewed Z level (slice) for ASCII rendering
    pub view_z: i32,
    // Vertical movement intent (dz)
    pub player_move_intent_z: Option<i32>,
}

impl Resources {
    pub fn new(seed: u64) -> Self {
        let rng = ChaCha20Rng::seed_from_u64(seed);
        let cfg = ConfigManager::new();
        let developer_mode = cfg
            .get_setting("game", "DEVELOPER_MODE")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let player_name = cfg
            .get_setting("game", "DEFAULT_PLAYER_NAME")
            .and_then(|v| v.as_str())
            .unwrap_or("Player")
            .to_string();
        Self {
            seed,
            rng,
            gametick: 0,
            player_entity: None,
            world: world::World::new(80, 24, seed),
            config: cfg,
            developer_mode,
            player_name,
            player_move_intent: None,
            last_blocked_tile: None,
            pending_tick_increase: None,
            mining_intent: false,
            messages: Vec::new(),
            view_x: 0,
            view_y: 0,
            view_z: 0,
            player_move_intent_z: None,
        }
    }

    pub fn log<S: Into<String>>(&mut self, msg: S) {
        let m = msg.into();
        self.messages.push(format!("[{}] {}", self.gametick, m));
        // Also forward to tracing for persistent logs
        info!(target: "game", "msg tick={} {}", self.gametick, m);
        if self.messages.len() > 200 {
            let overflow = self.messages.len() - 200;
            self.messages.drain(0..overflow);
        }
    }
}

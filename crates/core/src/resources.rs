use crate::config::ConfigManager;
use crate::intent::PlayerIntent;
use crate::world::World;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use tracing::info;

pub struct Resources {
    pub seed: u64,
    pub rng: ChaCha20Rng,
    pub gametick: u64,
    // REMOVED: player_entity - use ECS queries instead
    pub world: World,
    pub config: ConfigManager,
    pub developer_mode: bool,
    pub player_name: String,
    // Consolidated player action intent
    pub player_intent: PlayerIntent,
    pub last_blocked_tile: Option<(i32, i32)>,
    // Simple message log for UI
    pub messages: Vec<String>,
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
            world: World::new(80, 24, seed),
            config: cfg,
            developer_mode,
            player_name,
            player_intent: PlayerIntent::default(),
            last_blocked_tile: None,
            messages: Vec::new(),
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

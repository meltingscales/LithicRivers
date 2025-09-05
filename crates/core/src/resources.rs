use crate::game_config::GameConfig;
use crate::game_events::GameEvents;
use crate::game_time::GameTime;
use crate::player_state::PlayerState;
use crate::world_state::WorldState;

pub struct Resources {
    pub time: GameTime,
    pub world_state: WorldState,
    pub config: GameConfig,
    pub player_state: PlayerState,
    pub events: GameEvents,
}

impl Resources {
    pub fn new(seed: u64) -> Self {
        Self {
            time: GameTime::new(),
            world_state: WorldState::new(seed),
            config: GameConfig::new(),
            player_state: PlayerState::new(),
            events: GameEvents::new(),
        }
    }

    /// Log a game event (backwards compatibility)
    pub fn log<S: Into<String>>(&mut self, msg: S) {
        self.events.game_event(msg, self.time.tick);
    }
}

use crate::game_config::GameConfig;
use crate::game_time::GameTime;
use crate::message_log::MessageLog;
use crate::player_state::PlayerState;
use crate::world_state::WorldState;

pub struct Resources {
    pub time: GameTime,
    pub world_state: WorldState,
    pub config: GameConfig,
    pub player_state: PlayerState,
    pub message_log: MessageLog,
}

impl Resources {
    pub fn new(seed: u64) -> Self {
        Self {
            time: GameTime::new(),
            world_state: WorldState::new(seed),
            config: GameConfig::new(),
            player_state: PlayerState::new(),
            message_log: MessageLog::new(),
        }
    }

    pub fn log<S: Into<String>>(&mut self, msg: S) {
        self.message_log.log(msg, self.time.tick);
    }
}

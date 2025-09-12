use crate::game_config::GameConfig;
use crate::game_events::GameEvents;
use crate::game_time::GameTime;
use crate::player_state::PlayerState;
use crate::target_tracker::TargetTracker;
use crate::world_state::WorldState;

pub struct Resources {
    pub time: GameTime,
    pub world_state: WorldState,
    pub config: GameConfig,
    pub player_state: PlayerState,
    pub events: GameEvents,
    pub target_tracker: TargetTracker,
}

impl Resources {
    pub fn new(seed: u64) -> Self {
        Self {
            time: GameTime::new(),
            world_state: WorldState::new(seed),
            config: GameConfig::new(),
            player_state: PlayerState::new(),
            events: GameEvents::new(),
            target_tracker: TargetTracker::new(),
        }
    }

    /// Log a game event (backwards compatibility)
    pub fn log<S: Into<String>>(&mut self, msg: S) {
        self.events.game_event(msg, self.time.tick);
    }

    /// Log a game event with color
    pub fn log_colored<S: Into<String>>(
        &mut self,
        msg: S,
        color: crate::message_log::MessageColor,
    ) {
        self.events.game_event_colored(msg, self.time.tick, color);
    }

    /// Log a red message (for warnings and errors)
    pub fn log_red<S: Into<String>>(&mut self, msg: S) {
        self.log_colored(msg, crate::message_log::MessageColor::Red);
    }

    /// Log a yellow message (for warnings)
    pub fn log_yellow<S: Into<String>>(&mut self, msg: S) {
        self.log_colored(msg, crate::message_log::MessageColor::Yellow);
    }

    /// Log a green message (for success/positive events)
    pub fn log_green<S: Into<String>>(&mut self, msg: S) {
        self.log_colored(msg, crate::message_log::MessageColor::Green);
    }
}

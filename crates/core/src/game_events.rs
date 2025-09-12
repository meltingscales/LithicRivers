use crate::message_log::MessageLog;
use tracing::{debug, info, warn};

/// Unified logging facade for game events and debug information
/// Provides structured logging with proper targeting and consistent formatting
pub struct GameEvents {
    message_log: MessageLog,
}

impl GameEvents {
    pub fn new() -> Self {
        Self {
            message_log: MessageLog::new(),
        }
    }

    /// Log a player-visible game event (shows in UI message log)
    pub fn game_event<S: Into<String>>(&mut self, message: S, tick: u64) {
        let msg = message.into();
        self.message_log.log(&msg, tick);
        info!(target: "game_events", "tick={} {}", tick, msg);
    }

    /// Log a player-visible game event with color (shows in UI message log)
    pub fn game_event_colored<S: Into<String>>(
        &mut self,
        message: S,
        tick: u64,
        color: crate::message_log::MessageColor,
    ) {
        let msg = message.into();
        self.message_log.log_colored(&msg, tick, color);
        info!(target: "game_events", "tick={} {}", tick, msg);
    }

    /// Log debug information (development/troubleshooting only)
    pub fn debug<S: Into<String>>(&self, message: S, tick: u64) {
        let msg = message.into();
        debug!(target: "game_debug", "tick={} {}", tick, msg);
    }

    /// Log important system information
    pub fn system_info<S: Into<String>>(&self, message: S, tick: u64) {
        let msg = message.into();
        info!(target: "game_systems", "tick={} {}", tick, msg);
    }

    /// Log combat events with specific targeting
    pub fn combat_event<S: Into<String>>(&mut self, message: S, tick: u64) {
        let msg = message.into();
        self.message_log.log(&msg, tick);
        info!(target: "combat", "tick={} {}", tick, msg);
    }

    /// Log movement/interaction events
    pub fn interaction_event<S: Into<String>>(&mut self, message: S, tick: u64) {
        let msg = message.into();
        self.message_log.log(&msg, tick);
        info!(target: "interaction", "tick={} {}", tick, msg);
    }

    /// Log warnings (potential issues)
    pub fn warn<S: Into<String>>(&self, message: S, tick: u64) {
        let msg = message.into();
        warn!(target: "game_warnings", "tick={} {}", tick, msg);
    }

    /// Get access to message log for UI display
    pub fn get_message_log(&self) -> &MessageLog {
        &self.message_log
    }

    /// Get mutable access to message log
    pub fn get_message_log_mut(&mut self) -> &mut MessageLog {
        &mut self.message_log
    }
}

impl Default for GameEvents {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience macros for structured logging
#[macro_export]
macro_rules! game_event {
    ($events:expr, $tick:expr, $($arg:tt)*) => {
        $events.game_event(format!($($arg)*), $tick)
    };
}

#[macro_export]
macro_rules! combat_event {
    ($events:expr, $tick:expr, $($arg:tt)*) => {
        $events.combat_event(format!($($arg)*), $tick)
    };
}

#[macro_export]
macro_rules! interaction_event {
    ($events:expr, $tick:expr, $($arg:tt)*) => {
        $events.interaction_event(format!($($arg)*), $tick)
    };
}

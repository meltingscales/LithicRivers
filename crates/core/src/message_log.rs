use tracing::info;

#[derive(Debug, Clone, PartialEq)]
pub enum MessageColor {
    Default,
    Red,
    Yellow,
    Green,
    Blue,
    Cyan,
    Magenta,
}

#[derive(Debug, Clone)]
pub struct GameMessage {
    pub text: String,
    pub color: MessageColor,
}

#[derive(Debug, Clone)]
pub struct MessageLog {
    pub messages: Vec<GameMessage>,
}

impl MessageLog {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    pub fn log<S: Into<String>>(&mut self, msg: S, tick: u64) {
        self.log_colored(msg, tick, MessageColor::Default);
    }

    pub fn log_colored<S: Into<String>>(&mut self, msg: S, tick: u64, color: MessageColor) {
        let m = msg.into();
        let formatted_text = format!("[{}] {}", tick, m);
        self.messages.push(GameMessage {
            text: formatted_text,
            color,
        });
        // Also forward to tracing for persistent logs
        info!(target: "game", "msg tick={} {}", tick, m);
        if self.messages.len() > 200 {
            let overflow = self.messages.len() - 200;
            self.messages.drain(0..overflow);
        }
    }
}

impl Default for MessageLog {
    fn default() -> Self {
        Self::new()
    }
}

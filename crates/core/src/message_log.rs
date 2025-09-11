use tracing::info;

#[derive(Debug, Clone)]
pub struct MessageLog {
    pub messages: Vec<String>,
}

impl MessageLog {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    pub fn log<S: Into<String>>(&mut self, msg: S, tick: u64) {
        let m = msg.into();
        self.messages.push(format!("[{}] {}", tick, m));
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

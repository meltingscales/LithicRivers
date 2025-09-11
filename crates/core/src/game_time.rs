#[derive(Debug, Clone)]
pub struct GameTime {
    pub tick: u64,
}

impl GameTime {
    pub fn new() -> Self {
        Self { tick: 0 }
    }

    pub fn advance(&mut self) {
        self.tick += 1;
    }
}

impl Default for GameTime {
    fn default() -> Self {
        Self::new()
    }
}

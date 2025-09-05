use crate::intent::PlayerIntent;

#[derive(Debug, Clone)]
pub struct PlayerState {
    pub intent: PlayerIntent,
    pub last_blocked_tile: Option<(i32, i32)>,
}

impl PlayerState {
    pub fn new() -> Self {
        Self {
            intent: PlayerIntent::default(),
            last_blocked_tile: None,
        }
    }
}

impl Default for PlayerState {
    fn default() -> Self {
        Self::new()
    }
}

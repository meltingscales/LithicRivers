use crate::intent::PlayerIntent;

#[derive(Debug, Clone)]
pub struct PlayerState {
    pub intent: PlayerIntent,
    pub last_blocked_tile: Option<(i32, i32)>,
    pub combat_ended_this_tick: bool,
    pub last_combat_end_tick: u64,
    pub combat_active: bool,
}

impl PlayerState {
    pub fn new() -> Self {
        Self {
            intent: PlayerIntent::default(),
            last_blocked_tile: None,
            combat_ended_this_tick: false,
            last_combat_end_tick: 0,
            combat_active: false,
        }
    }
}

impl Default for PlayerState {
    fn default() -> Self {
        Self::new()
    }
}

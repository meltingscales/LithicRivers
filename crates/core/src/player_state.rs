use crate::intent::PlayerIntent;

#[derive(Debug, Clone)]
pub struct PlayerState {
    pub intent: PlayerIntent,
    pub last_blocked_tile: Option<(i64, i64)>,
    pub combat_ended_this_tick: bool,
    pub last_combat_end_tick: u64,
    pub combat_active: bool,
    pub noclip_enabled: bool,
}

impl PlayerState {
    pub fn new() -> Self {
        Self {
            intent: PlayerIntent::default(),
            last_blocked_tile: None,
            combat_ended_this_tick: false,
            last_combat_end_tick: 0,
            combat_active: false,
            noclip_enabled: false,
        }
    }
}

impl Default for PlayerState {
    fn default() -> Self {
        Self::new()
    }
}

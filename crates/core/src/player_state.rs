use crate::intent::PlayerIntent;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlayerState {
    pub intent: PlayerIntent,
    pub last_blocked_tile: Option<(i64, i64)>,
    pub combat_ended_this_tick: bool,
    pub last_combat_end_tick: u64,
    pub combat_active: bool,
    pub noclip_enabled: bool,
    pub fog_of_war_enabled: bool,
    /// Where the player comes back after dying: where a new game placed them
    #[serde(default)]
    pub respawn_point: Option<crate::components::Position>,
    /// What destroyed the player, while they are dead
    #[serde(default)]
    pub death_cause: Option<String>,
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
            fog_of_war_enabled: true,
            respawn_point: None,
            death_cause: None,
        }
    }
}

impl Default for PlayerState {
    fn default() -> Self {
        Self::new()
    }
}

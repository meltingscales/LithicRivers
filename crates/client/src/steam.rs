//! Everything Steam: connecting to the Steam client, running its callbacks, and unlocking
//! achievements. The real connection exists only with the `steam` feature (release
//! builds); without it the same type is a stub that never connects, so callers need no
//! `cfg`. Unlocks are always recorded in `unlocked_this_session`, so the achievement logic
//! is testable without Steam.

/// The game's Steam app id.
#[cfg_attr(not(feature = "steam"), allow(dead_code))]
pub const APP_ID: u32 = 3940720;

/// An achievement, as the game unlocks it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Achievement {
    /// Lose your body for the first time ("System Failure")
    FirstDeath,
    /// Reach the surface ("Daylight")
    Surface,
    /// Destroy an enemy ("Self-Defense")
    FirstKill,
    /// Complete Repair the Broken Android ("Spare Parts")
    RepairAndroid,
    /// Complete the arm quest ("Two Hands")
    ReplaceArm,
    /// Teleport for the first time ("Transit")
    FirstJump,
}

impl Achievement {
    /// The achievement's API name in Steamworks.
    #[cfg_attr(not(feature = "steam"), allow(dead_code))]
    pub fn api_name(self) -> &'static str {
        match self {
            Achievement::FirstDeath => "ACH_FIRST_DEATH",
            Achievement::Surface => "ACH_SURFACE",
            Achievement::FirstKill => "ACH_FIRST_KILL",
            Achievement::RepairAndroid => "ACH_REPAIR_ANDROID",
            Achievement::ReplaceArm => "ACH_REPLACE_ARM",
            Achievement::FirstJump => "ACH_FIRST_JUMP",
        }
    }
}

/// The connection to Steam, if any.
#[derive(Default)]
pub struct Steam {
    #[cfg(feature = "steam")]
    client: Option<steamworks::Client>,
    /// Achievements unlocked since the game started, connected or not
    pub unlocked_this_session: Vec<Achievement>,
}

impl Steam {
    /// Connect to the running Steam client. Logs "Steam unavailable" and returns a
    /// disconnected value when Steam isn't running or the build has no Steam support.
    pub fn connect() -> Self {
        #[cfg(feature = "steam")]
        {
            match steamworks::Client::init_app(APP_ID) {
                Ok(client) => {
                    tracing::info!(target: "game", "Connected to Steam");
                    return Self {
                        client: Some(client),
                        ..Self::default()
                    };
                }
                Err(e) => tracing::warn!(target: "game", "Steam unavailable: {e:?}"),
            }
        }
        #[cfg(not(feature = "steam"))]
        tracing::info!(target: "game", "Steam unavailable: built without Steam support");
        Self::default()
    }

    /// Let Steam deliver its callbacks; called once a frame.
    pub fn run_callbacks(&self) {
        #[cfg(feature = "steam")]
        if let Some(client) = &self.client {
            client.run_callbacks();
        }
    }

    /// Unlock an achievement: recorded for this session always, and set and stored in
    /// Steam when connected. Does nothing for one already unlocked this session, so a
    /// condition that stays true doesn't call Steam every frame.
    pub fn unlock(&mut self, achievement: Achievement) {
        if self.unlocked_this_session.contains(&achievement) {
            return;
        }
        self.unlocked_this_session.push(achievement);
        #[cfg(feature = "steam")]
        if let Some(client) = &self.client {
            let stats = client.user_stats();
            if stats.achievement(achievement.api_name()).set().is_err()
                || stats.store_stats().is_err()
            {
                tracing::warn!(target: "game", "Steam didn't take {}", achievement.api_name());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Achievement;
    use crate::headless::chain_commands;

    /// A world with cheats off (or on), checked once alive.
    fn alive(cheats: bool) -> crate::headless::Runner {
        let mut runner = chain_commands(&["skip-splash"]);
        runner.app.core.game.res.world_settings.cheats_allowed = cheats;
        runner.app.check_achievements();
        runner
    }

    fn die(runner: &mut crate::headless::Runner) {
        let game = &mut runner.app.core.game;
        let player = game.get_player_entity().unwrap();
        lithicrivers_core::death::player_died(
            &mut game.world,
            &mut game.res,
            player,
            "test".to_string(),
        );
        runner.app.check_achievements();
    }

    #[test]
    fn dying_without_cheats_unlocks_first_death_once() {
        let mut runner = alive(false);
        die(&mut runner);
        runner.app.check_achievements();
        assert_eq!(
            runner.app.steam.unlocked_this_session,
            vec![Achievement::FirstDeath]
        );
        assert_eq!(Achievement::FirstDeath.api_name(), "ACH_FIRST_DEATH");
    }

    #[test]
    fn dying_with_cheats_unlocks_nothing() {
        let mut runner = alive(true);
        die(&mut runner);
        assert!(runner.app.steam.unlocked_this_session.is_empty());
    }

    #[test]
    fn reaching_the_surface_unlocks_daylight_once() {
        let mut runner = alive(false);
        assert!(
            runner.app.steam.unlocked_this_session.is_empty(),
            "in the bunker"
        );
        let game = &mut runner.app.core.game;
        let player = game.get_player_entity().unwrap();
        *game
            .world
            .get::<&mut lithicrivers_core::components::Position>(player)
            .unwrap() = lithicrivers_core::components::Position { x: 0, y: 0, z: 0 };
        for _ in 0..10 {
            runner.app.check_achievements();
        }
        assert_eq!(
            runner.app.steam.unlocked_this_session,
            vec![Achievement::Surface]
        );
    }

    #[test]
    fn kills_quests_and_jumps_unlock_theirs() {
        use lithicrivers_core::dialogue::{ActiveQuest, QuestType};
        let mut runner = alive(false);
        let res = &mut runner.app.core.game.res;
        res.stats.enemies_killed = 1;
        res.stats.teleports = 1;
        for quest in [QuestType::RepairBrokenAndroid, QuestType::ReplaceArm] {
            res.start_quest(ActiveQuest::new(
                quest,
                format!("{quest:?}"),
                String::new(),
                vec![],
                None,
            ));
            res.complete_quest(quest);
        }
        runner.app.check_achievements();
        let mut unlocked = runner.app.steam.unlocked_this_session.clone();
        unlocked.sort_by_key(|a| a.api_name());
        assert_eq!(
            unlocked,
            vec![
                Achievement::FirstJump,
                Achievement::FirstKill,
                Achievement::RepairAndroid,
                Achievement::ReplaceArm,
            ]
        );
    }

    #[test]
    fn worlds_with_cheats_unlock_nothing() {
        let mut runner = alive(true);
        let res = &mut runner.app.core.game.res;
        res.stats.enemies_killed = 3;
        res.stats.teleports = 2;
        runner.app.check_achievements();
        assert!(runner.app.steam.unlocked_this_session.is_empty());
    }

    #[test]
    fn a_world_loaded_while_dead_is_not_a_new_death() {
        let mut runner = alive(false);
        let game = &mut runner.app.core.game;
        let player = game.get_player_entity().unwrap();
        lithicrivers_core::death::player_died(
            &mut game.world,
            &mut game.res,
            player,
            "test".to_string(),
        );
        // As after loading a world: the last state is unknown
        runner.app.reset_for_game();
        runner.app.check_achievements();
        assert!(runner.app.steam.unlocked_this_session.is_empty());
    }
}

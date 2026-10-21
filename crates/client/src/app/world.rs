//! Starting, loading, saving and leaving worlds.

use crate::worlds::{self, WorldMeta};
use crate::{App, SplashState};
use lithicrivers_core::components::Position;
use lithicrivers_core::game::Game;
use lithicrivers_core::resources::WorldSettings;
use std::path::PathBuf;

/// The world being played.
pub struct CurrentWorld {
    /// Its folder; `None` until first saved (worlds started by `skip-splash`)
    pub folder: Option<PathBuf>,
    pub meta: WorldMeta,
}

impl App {
    /// Start a new world and reset all in-game UI for it. Does not save or change screens.
    pub(crate) fn start_world(&mut self, name: &str, seed: u64, settings: WorldSettings) {
        self.core.game = Game::new_with_settings(seed, settings);
        self.reset_for_game();
        let now = self.clock.unix_seconds();
        self.current_world = Some(CurrentWorld {
            folder: None,
            meta: WorldMeta::new(name, seed, settings, now),
        });
    }

    /// Start a world the way `skip-splash` does: the default seed, cheats on, other settings
    /// from config, then straight to the main UI.
    pub(crate) fn start_default_world(&mut self) {
        let settings = WorldSettings::defaults(&self.core.game.res.config);
        self.start_world("New World", self.default_seed, settings);
        self.splash.state = SplashState::MainUI;
    }

    /// Load a saved world from its folder and show the main UI.
    pub(crate) fn load_world(&mut self, folder: PathBuf) -> anyhow::Result<()> {
        let meta = worlds::read_meta(&folder)?;
        self.core.game = Game::new_empty(meta.seed, Position { x: 0, y: 0, z: 0 });
        self.core.game.res.world_settings = meta.settings;
        self.reset_for_game();
        let save = folder.join(worlds::SAVE_FILE);
        self.load_from_file(&save.to_string_lossy())?;
        self.core.game.res.world_settings = meta.settings;
        self.current_world = Some(CurrentWorld {
            folder: Some(folder),
            meta,
        });
        self.splash.state = SplashState::MainUI;
        Ok(())
    }

    /// Save the current world into its folder (creating the folder on first save).
    pub(crate) fn save_world(&mut self) -> anyhow::Result<()> {
        let now = self.clock.unix_seconds();
        let settings = self.core.game.res.world_settings;
        let saves_dir = self.saves_dir.clone();
        let Some(world) = self.current_world.as_mut() else {
            anyhow::bail!("no world is being played");
        };
        world.meta.last_played_unix = now;
        world.meta.settings = settings;
        let folder = match &world.folder {
            Some(folder) => folder.clone(),
            None => worlds::create_world_folder(&saves_dir, &world.meta)?,
        };
        world.folder = Some(folder.clone());
        worlds::write_meta(&folder, &world.meta)?;
        self.save_to_file(&folder.join(worlds::SAVE_FILE).to_string_lossy())
    }

    /// Save, then leave the world for the title screen.
    pub(crate) fn save_and_quit_to_title(&mut self) -> anyhow::Result<()> {
        self.save_world()?;
        self.current_world = None;
        self.core.game = Game::new_empty(self.default_seed, Position { x: 0, y: 0, z: 0 });
        self.reset_for_game();
        self.show_title();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::headless::Runner;
    use lithicrivers_core::resources::WorldSettings;

    #[test]
    fn saved_world_loads_back_with_position_inventory_tick_and_settings() {
        let mut runner = Runner::new(12345, (100, 30));
        let settings = WorldSettings {
            cheats_allowed: false,
            developer_mode: false,
            show_tutorial: false,
            world_type: lithicrivers_core::resources::WorldType::Normal,
        };
        runner.app.start_world("Round Trip", 777, settings);
        runner.app.splash.state = crate::SplashState::MainUI;
        runner.chain(&["key movement.MOVE_SOUTH 2", "tick 3"]);
        let game = &runner.app.core.game;
        let before = (
            game.get_player_position(),
            format!(
                "{:?}",
                game.get_player_component::<lithicrivers_core::components::Inventory>()
            ),
            game.res.time.tick,
        );

        runner.app.save_and_quit_to_title().unwrap();
        let folder = crate::worlds::list_worlds(&runner.app.saves_dir)[0]
            .folder
            .clone();
        runner.app.load_world(folder).unwrap();

        let game = &runner.app.core.game;
        let after = (
            game.get_player_position(),
            format!(
                "{:?}",
                game.get_player_component::<lithicrivers_core::components::Inventory>()
            ),
            game.res.time.tick,
        );
        assert_eq!(after, before);
        assert_eq!(game.res.world_settings, settings);
        assert_eq!(game.res.world_state.seed, 777);
        assert!(!runner.app.ui.tutorial_visible);
    }
}

//! The title menu (title screen, world list, create world, options) and its input.
//! Screens are `SplashState` variants; this module holds their selections and form state.

use crate::worlds::{self, WorldEntry};
use crate::{App, EmbeddedAssets, SplashState};
use crossterm::event::KeyCode;
use lithicrivers_core::resources::WorldSettings;

pub const TITLE_CHOICES: [&str; 3] = ["Play Game", "Options", "Quit"];
pub const WORLD_ACTIONS: [&str; 4] = ["Play", "Rename", "Delete", "Back"];
pub const DELETE_CHOICES: [&str; 2] = ["Cancel", "Delete"];
/// Create-world rows, in order.
pub const CREATE_FIELDS: [&str; 7] = [
    "World name",
    "Seed",
    "Allow cheats",
    "Developer mode",
    "Show tutorial",
    "Create",
    "Cancel",
];
pub const OPTIONS_ROWS: [&str; 3] = ["Music volume", "Sound volume", "Back"];
const VOLUME_STEP: f64 = 0.05;
const MAX_TEXT_LEN: usize = 32;

pub struct CreateWorldForm {
    pub name: String,
    pub seed: String,
    pub settings: WorldSettings,
    pub field: usize,
}

pub struct TitleMenuState {
    /// Selected title-screen choice
    pub selected: usize,
    pub splash_texts: Vec<String>,
    pub splash_index: usize,
    pub worlds: Vec<WorldEntry>,
    /// Row in the world list: a world, then Create New World, then Back
    pub world_selected: usize,
    pub world_action: usize,
    pub delete_choice: usize,
    pub create: CreateWorldForm,
    pub options_selected: usize,
    /// The new name being typed on the rename screen
    pub rename: String,
    /// A problem to show on the world list (failed load, rename or delete)
    pub message: Option<String>,
}

impl TitleMenuState {
    pub fn new(developer_mode: bool) -> Self {
        let splash_texts = EmbeddedAssets::get("config/splashes.txt")
            .map(|d| String::from_utf8_lossy(&d.data).to_string())
            .unwrap_or_else(|| panic!("splashes.txt not found"))
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(String::from)
            .collect();
        Self {
            selected: 0,
            splash_texts,
            splash_index: 0,
            worlds: Vec::new(),
            world_selected: 0,
            world_action: 0,
            delete_choice: 0,
            create: CreateWorldForm::new(developer_mode),
            options_selected: 0,
            rename: String::new(),
            message: None,
        }
    }

    pub fn splash_text(&self) -> &str {
        self.splash_texts
            .get(self.splash_index)
            .map_or("", String::as_str)
    }

    /// The world the world-list cursor is on, if it is on a world.
    pub fn selected_world(&self) -> Option<&WorldEntry> {
        self.worlds.get(self.world_selected)
    }
}

impl CreateWorldForm {
    fn new(developer_mode: bool) -> Self {
        Self {
            name: "New World".to_string(),
            seed: String::new(),
            settings: WorldSettings {
                cheats_allowed: false,
                developer_mode,
                show_tutorial: true,
            },
            field: 0,
        }
    }
}

fn step(index: &mut usize, len: usize, down: bool) {
    *index = if down {
        (*index + 1) % len
    } else {
        (*index + len - 1) % len
    };
}

impl App {
    /// Show the title screen, with Play Game selected and a splash text picked by the clock.
    pub(crate) fn show_title(&mut self) {
        self.splash.state = SplashState::Title;
        self.title.selected = 0;
        let count = self.title.splash_texts.len().max(1) as u64;
        self.title.splash_index = (self.clock.unix_seconds() % count) as usize;
    }

    fn show_world_list(&mut self) {
        self.title.worlds = worlds::list_worlds(&self.saves_dir);
        let rows = self.title.worlds.len() + 2;
        self.title.world_selected = self.title.world_selected.min(rows - 1);
        self.splash.state = SplashState::WorldList;
    }

    fn matches(&self, action: &str, key: &KeyCode) -> bool {
        self.ui.keybinds.matches("title", action, key)
    }

    /// Up/down for a menu of `len` rows; returns true if the key moved the cursor.
    fn navigate(&self, key: &KeyCode, index: &mut usize, len: usize) -> bool {
        if self.matches("UP", key) {
            step(index, len, false);
            true
        } else if self.matches("DOWN", key) {
            step(index, len, true);
            true
        } else {
            false
        }
    }

    /// Input for the title-menu screens. Returns false if the current screen isn't one.
    pub(crate) fn handle_title_input(&mut self, key: KeyCode) -> bool {
        match self.splash.state {
            SplashState::Title => {
                let mut selected = self.title.selected;
                if self.navigate(&key, &mut selected, TITLE_CHOICES.len()) {
                    self.title.selected = selected;
                } else if self.matches("SELECT", &key) {
                    match self.title.selected {
                        0 => {
                            self.title.world_selected = 0;
                            self.title.message = None;
                            self.show_world_list();
                        }
                        1 => {
                            self.title.options_selected = 0;
                            self.splash.state = SplashState::Options;
                        }
                        _ => self.core.should_quit = true,
                    }
                }
            }
            SplashState::WorldList => {
                let rows = self.title.worlds.len() + 2;
                let mut selected = self.title.world_selected;
                if self.navigate(&key, &mut selected, rows) {
                    self.title.world_selected = selected;
                } else if self.matches("BACK", &key) {
                    self.show_title();
                } else if self.matches("SELECT", &key) {
                    let worlds = self.title.worlds.len();
                    if selected < worlds {
                        self.title.world_action = 0;
                        self.splash.state = SplashState::WorldActions;
                    } else if selected == worlds {
                        self.title.create =
                            CreateWorldForm::new(self.core.game.res.config.developer_mode);
                        self.splash.state = SplashState::CreateWorld;
                    } else {
                        self.show_title();
                    }
                }
            }
            SplashState::WorldActions => {
                let mut action = self.title.world_action;
                if self.navigate(&key, &mut action, WORLD_ACTIONS.len()) {
                    self.title.world_action = action;
                } else if self.matches("BACK", &key) {
                    self.splash.state = SplashState::WorldList;
                } else if self.matches("SELECT", &key) {
                    let Some(world) = self.title.selected_world().cloned() else {
                        self.show_world_list();
                        return true;
                    };
                    match action {
                        0 if world.meta.is_none() => {
                            self.title.message =
                                Some("This world can't be read, so it can't be played.".into());
                            self.splash.state = SplashState::WorldList;
                        }
                        0 => {
                            if let Err(e) = self.load_world(world.folder) {
                                self.title.message = Some(format!("Could not load world: {e:#}"));
                                self.show_world_list();
                            }
                        }
                        1 if world.meta.is_none() => {
                            self.title.message =
                                Some("This world can't be read, so it can't be renamed.".into());
                            self.splash.state = SplashState::WorldList;
                        }
                        1 => {
                            self.title.rename = world.meta.map(|m| m.name).unwrap_or_default();
                            self.splash.state = SplashState::RenameWorld;
                        }
                        2 => {
                            self.title.delete_choice = 0;
                            self.splash.state = SplashState::ConfirmDelete;
                        }
                        _ => self.splash.state = SplashState::WorldList,
                    }
                }
            }
            SplashState::ConfirmDelete => {
                let mut choice = self.title.delete_choice;
                if self.navigate(&key, &mut choice, DELETE_CHOICES.len()) {
                    self.title.delete_choice = choice;
                } else if self.matches("BACK", &key) {
                    self.splash.state = SplashState::WorldList;
                } else if self.matches("SELECT", &key) {
                    if choice == 1 {
                        if let Some(world) = self.title.selected_world() {
                            if let Err(e) = worlds::delete_world(&self.saves_dir, &world.folder) {
                                self.title.message = Some(format!("Could not delete: {e:#}"));
                            }
                        }
                    }
                    self.show_world_list();
                }
            }
            SplashState::RenameWorld => {
                if self.matches("BACK", &key) {
                    self.splash.state = SplashState::WorldList;
                } else if self.matches("SELECT", &key) {
                    let name = self.title.rename.trim().to_string();
                    if let (Some(world), false) = (self.title.selected_world(), name.is_empty()) {
                        if let Err(e) = worlds::rename_world(&world.folder, &name) {
                            self.title.message = Some(format!("Could not rename: {e:#}"));
                        }
                    }
                    self.show_world_list();
                } else if self.matches("DELETE_CHAR", &key) {
                    self.title.rename.pop();
                } else if let KeyCode::Char(c) = key {
                    if self.title.rename.chars().count() < MAX_TEXT_LEN {
                        self.title.rename.push(c);
                    }
                }
            }
            SplashState::CreateWorld => self.handle_create_input(key),
            SplashState::Options => self.handle_options_input(key),
            _ => return false,
        }
        true
    }

    fn handle_create_input(&mut self, key: KeyCode) {
        let mut field = self.title.create.field;
        if self.navigate(&key, &mut field, CREATE_FIELDS.len()) {
            self.title.create.field = field;
            return;
        }
        if self.matches("BACK", &key) {
            self.show_world_list();
            return;
        }
        let select = self.matches("SELECT", &key);
        let toggle = select || self.matches("LEFT", &key) || self.matches("RIGHT", &key);
        let erase = self.matches("DELETE_CHAR", &key);
        let form = &mut self.title.create;
        match field {
            0 | 1 => {
                let text = if field == 0 {
                    &mut form.name
                } else {
                    &mut form.seed
                };
                if erase {
                    text.pop();
                } else if let KeyCode::Char(c) = key {
                    if text.chars().count() < MAX_TEXT_LEN {
                        text.push(c);
                    }
                }
            }
            2 if toggle => form.settings.cheats_allowed ^= true,
            3 if toggle => form.settings.developer_mode ^= true,
            4 if toggle => form.settings.show_tutorial ^= true,
            5 if select => self.create_world_from_form(),
            6 if select => self.show_world_list(),
            _ => {}
        }
    }

    /// Create the world described by the form, save it, and play the boot intro.
    fn create_world_from_form(&mut self) {
        let form = &self.title.create;
        let name = match form.name.trim() {
            "" => "New World".to_string(),
            name => name.to_string(),
        };
        let seed = worlds::parse_seed(&form.seed, self.clock.unix_seconds());
        let settings = form.settings;
        self.start_world(&name, seed, settings);
        if let Err(e) = self.save_world() {
            self.title.message = Some(format!("Could not save new world: {e:#}"));
            self.show_world_list();
            return;
        }
        self.start_boot_message();
    }

    /// Show the boot-message intro from the start.
    pub(crate) fn start_boot_message(&mut self) {
        self.splash.state = SplashState::BootMessage;
        self.splash.start_time = Some(self.clock.now());
        self.splash.boot_display_text.clear();
        self.splash.boot_line_index = 0;
        self.splash.last_line_time = self.clock.now();
        self.splash.boot_complete = false;
    }

    fn handle_options_input(&mut self, key: KeyCode) {
        let mut row = self.title.options_selected;
        if self.navigate(&key, &mut row, OPTIONS_ROWS.len()) {
            self.title.options_selected = row;
            return;
        }
        let back = self.matches("BACK", &key) || (row == 2 && self.matches("SELECT", &key));
        if back {
            self.show_title();
            return;
        }
        let delta = if self.matches("LEFT", &key) {
            -VOLUME_STEP
        } else if self.matches("RIGHT", &key) {
            VOLUME_STEP
        } else {
            return;
        };
        let setting = if row == 0 {
            "MUSIC_VOLUME"
        } else {
            "SFX_VOLUME"
        };
        let volume = (self.volume_setting(setting) + delta).clamp(0.0, 1.0);
        // Round so repeated steps land on exact percentages
        let volume = (volume * 100.0).round() / 100.0;
        self.set_volume_setting(setting, volume);
    }

    /// A volume from the config's `audio` settings, 0.0 to 1.0.
    pub(crate) fn volume_setting(&self, key: &str) -> f64 {
        self.core
            .config_manager
            .get_setting("audio", key)
            .and_then(|v| v.as_f64())
            .unwrap_or(0.5)
    }

    /// Set a volume, apply it to the audio manager, and save it to the config file
    /// (headless runs never write the config file).
    fn set_volume_setting(&mut self, key: &str, volume: f64) {
        let settings = &mut self.core.config_manager.data.settings;
        settings["audio"][key] = serde_json::json!(volume);
        self.apply_volumes();
        if self.persist_config {
            if let Err(e) = self.core.config_manager.save() {
                tracing::warn!(target: "game", "could not save config: {e:#}");
            }
        }
    }

    /// Apply the configured music and sound volumes to the audio manager.
    pub(crate) fn apply_volumes(&mut self) {
        let music = self.volume_setting("MUSIC_VOLUME") as f32;
        let sfx = self.volume_setting("SFX_VOLUME") as f32;
        self.audio.audio.set_music_volume(music);
        self.audio.audio.set_sfx_volume(sfx);
    }
}

#[cfg(test)]
mod tests {
    use crate::headless::{parse_script, state_json, Runner, EXIT_OK};
    use crate::{MenuTab, SplashState};

    const TO_TITLE: &str = "rawkey ENTER\nrawkey ENTER\n";
    /// From the title screen: Play Game, Create New World.
    const TO_CREATE: &str = "key title.SELECT\nkey title.SELECT\n";
    /// From the create form's name field: set the seed, press Create, skip the boot intro.
    const CREATE_12345: &str =
        "key title.DOWN\ntype \"12345\"\nkey title.DOWN 4\nkey title.SELECT\nrawkey ENTER\nrawkey ENTER\n";
    const MENU_TAB: &str = "key ui.MENU_NEXT 6\n";

    fn run(runner: &mut Runner, text: &str) -> String {
        let report = runner.run_steps(&parse_script(text).unwrap().steps);
        assert_eq!(report.status, EXIT_OK, "{:?}", report.error);
        report.output
    }

    fn after(text: &str) -> Runner {
        let mut runner = Runner::new(12345, (100, 30));
        run(&mut runner, text);
        runner
    }

    fn last_message(runner: &Runner) -> String {
        let log = &runner.app.core.game.res.events.get_message_log().messages;
        log.last().map(|m| m.text.clone()).unwrap_or_default()
    }

    #[test]
    fn title_screen_shows_choices_and_back_returns_to_it() {
        let mut runner = after(TO_TITLE);
        assert_eq!(runner.app.splash.state, SplashState::Title);
        let screen = runner.screen_text();
        for choice in [
            "Play Game",
            "Options",
            "Quit",
            runner.app.title.splash_text(),
        ] {
            assert!(screen.contains(choice), "{choice}: {screen}");
        }
        run(&mut runner, "key title.SELECT\nkey title.BACK");
        assert_eq!(runner.app.splash.state, SplashState::Title);
        run(&mut runner, "key title.BACK");
        assert_eq!(
            runner.app.splash.state,
            SplashState::Title,
            "back does nothing"
        );
        run(&mut runner, "key title.UP\nkey title.SELECT");
        assert!(
            runner.app.core.should_quit,
            "Quit is above Play Game when wrapping"
        );
    }

    #[test]
    fn creating_seed_12345_matches_skip_splash() {
        let mut created = after(&format!("{TO_TITLE}{TO_CREATE}{CREATE_12345}"));
        let mut skipped = after("skip-splash");
        assert_eq!(created.app.splash.state, SplashState::MainUI);
        assert_eq!(state_json(&created.app), state_json(&skipped.app));
        assert_eq!(created.screen_text(), skipped.screen_text());
        assert!(!created.app.core.game.res.world_settings.cheats_allowed);
    }

    #[test]
    fn save_quit_and_play_restores_position_and_inventory() {
        let mut runner = after(&format!(
            "{TO_TITLE}{TO_CREATE}{CREATE_12345}key movement.MOVE_SOUTH 2\ncheat giveitem log 4\n"
        ));
        let game = &runner.app.core.game;
        let before = (
            game.get_player_position(),
            format!(
                "{:?}",
                game.get_player_component::<lithicrivers_core::components::Inventory>()
            ),
        );
        run(&mut runner, &format!("{MENU_TAB}rawkey Q"));
        assert_eq!(runner.app.splash.state, SplashState::Title);
        run(
            &mut runner,
            "key title.SELECT\nexpect-text \"seed 12345\"\nkey title.SELECT\nkey title.SELECT",
        );
        assert_eq!(runner.app.splash.state, SplashState::MainUI);
        let game = &runner.app.core.game;
        let after = (
            game.get_player_position(),
            format!(
                "{:?}",
                game.get_player_component::<lithicrivers_core::components::Inventory>()
            ),
        );
        assert_eq!(after, before);
    }

    #[test]
    fn quit_tab_saves_and_returns_to_the_title() {
        let mut runner = after("skip-splash\nkey movement.MOVE_SOUTH\nkey ui.MENU_NEXT 9");
        assert_eq!(runner.app.ui.current_tab, MenuTab::Quit);
        let pos = runner.app.core.game.get_player_position();
        run(&mut runner, "key ui.MENU_ACTIVATE");
        assert_eq!(runner.app.splash.state, SplashState::Title);
        assert!(
            !runner.app.core.should_quit,
            "the game keeps running at the title"
        );
        run(
            &mut runner,
            "key title.SELECT\nkey title.SELECT\nkey title.SELECT",
        );
        assert_eq!(runner.app.splash.state, SplashState::MainUI);
        assert_eq!(runner.app.core.game.get_player_position(), pos);
    }

    #[test]
    fn cheat_console_follows_the_world_setting() {
        // Allow cheats is off by default
        let mut off = after(&format!(
            "{TO_TITLE}{TO_CREATE}{CREATE_12345}key ui.OPEN_COMMAND_MENU"
        ));
        assert!(!off.app.panels.cheat_console.is_open());
        assert!(last_message(&off).contains("Cheats are disabled for this world"));
        // Headless cheat steps still work
        run(&mut off, "cheat tp 0 0 0");

        let on = after(&format!(
            "{TO_TITLE}{TO_CREATE}key title.DOWN 2\nkey title.SELECT\nkey title.UP 2\n{CREATE_12345}key ui.OPEN_COMMAND_MENU"
        ));
        assert!(on.app.core.game.res.world_settings.cheats_allowed);
        assert!(on.app.panels.cheat_console.is_open());
    }

    #[test]
    fn world_without_tutorial_has_no_tutorial_tab() {
        let runner = after(&format!(
            "{TO_TITLE}{TO_CREATE}key title.DOWN 4\nkey title.SELECT\nkey title.UP 4\n{CREATE_12345}"
        ));
        assert!(!runner.app.core.game.res.world_settings.show_tutorial);
        assert!(!runner.app.ui.tutorial_visible);
        assert_eq!(runner.app.ui.current_tab, MenuTab::World);
        let mut runner = runner;
        assert!(!runner.screen_text().contains("Tutorial │"));
    }

    #[test]
    fn duplicate_names_make_two_worlds_and_delete_needs_confirmation() {
        let then = format!("{CREATE_12345}{MENU_TAB}rawkey Q\n");
        // The second time, the list's cursor starts on the first world
        let mut runner = after(&format!(
            "{TO_TITLE}{TO_CREATE}{then}key title.SELECT\nkey title.DOWN\nkey title.SELECT\n{then}"
        ));
        run(&mut runner, "key title.SELECT");
        assert_eq!(runner.app.title.worlds.len(), 2);
        let folders: Vec<_> = runner
            .app
            .title
            .worlds
            .iter()
            .map(|w| w.folder.clone())
            .collect();
        assert_ne!(folders[0], folders[1]);

        // Delete, then cancel: nothing changes
        run(
            &mut runner,
            "key title.SELECT\nkey title.DOWN 2\nkey title.SELECT\nkey title.SELECT",
        );
        assert_eq!(runner.app.title.worlds.len(), 2);
        // Delete, then confirm
        run(
            &mut runner,
            "key title.SELECT\nkey title.DOWN 2\nkey title.SELECT\nkey title.DOWN\nkey title.SELECT",
        );
        assert_eq!(runner.app.splash.state, SplashState::WorldList);
        assert_eq!(runner.app.title.worlds.len(), 1);
        assert!(!folders[0].exists());
    }

    #[test]
    fn rename_changes_the_listed_name_and_back_cancels() {
        let mut runner = after(&format!(
            "{TO_TITLE}{TO_CREATE}{CREATE_12345}{MENU_TAB}rawkey Q\nkey title.SELECT"
        ));
        let folder = runner.app.title.worlds[0].folder.clone();
        // Rename, erase "World", type "Place", confirm
        run(
            &mut runner,
            "key title.SELECT\nkey title.DOWN\nkey title.SELECT\nkey title.DELETE_CHAR 5\n\
             type \"Place\"\nexpect-text \"New name: [New Place_]\"\nkey title.SELECT",
        );
        assert_eq!(runner.app.splash.state, SplashState::WorldList);
        let world = &runner.app.title.worlds[0];
        assert_eq!(world.meta.as_ref().unwrap().name, "New Place");
        assert_eq!(world.folder, folder);
        assert!(runner.screen_text().contains("New Place  (seed 12345"));

        // Back leaves the name alone
        run(
            &mut runner,
            "key title.SELECT\nkey title.DOWN\nkey title.SELECT\ntype \"xyz\"\nkey title.BACK",
        );
        assert_eq!(
            runner.app.title.worlds[0].meta.as_ref().unwrap().name,
            "New Place"
        );
    }

    #[test]
    fn options_change_volume_in_config_and_audio() {
        let mut runner = after(&format!("{TO_TITLE}key title.DOWN\nkey title.SELECT"));
        assert_eq!(runner.app.splash.state, SplashState::Options);
        let music = runner.app.volume_setting("MUSIC_VOLUME");
        run(&mut runner, "key title.LEFT 2");
        let lowered = runner.app.volume_setting("MUSIC_VOLUME");
        assert!(
            (lowered - (music - 0.10)).abs() < 1e-9,
            "{music} -> {lowered}"
        );
        assert!((runner.app.audio.audio.music_volume() as f64 - lowered).abs() < 1e-6);
        assert!(runner
            .screen_text()
            .contains(&format!("{}%", (lowered * 100.0).round())));
        run(&mut runner, "key title.DOWN\nkey title.RIGHT");
        assert!(runner.app.volume_setting("SFX_VOLUME") > 0.35);
        run(&mut runner, "key title.BACK");
        assert_eq!(runner.app.splash.state, SplashState::Title);
    }
}

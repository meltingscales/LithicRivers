//! The title menu (title screen, world list, create world, options) and its input.
//! Screens are `SplashState` variants; this module holds their selections and form state.

use crate::worlds::{self, WorldEntry};
use crate::{App, EmbeddedAssets, SplashState};
use crossterm::event::KeyCode;
use lithicrivers_core::resources::{WorldSettings, WorldType};

pub const TITLE_CHOICES: [&str; 3] = ["Play Game", "Options", "Quit"];
pub const WORLD_ACTIONS: [&str; 4] = ["Play", "Rename", "Delete", "Back"];
pub const DELETE_CHOICES: [&str; 2] = ["Cancel", "Delete"];
/// A row of the create-world form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreateField {
    Name,
    Seed,
    Cheats,
    DeveloperMode,
    /// Only shown with Developer mode on
    WorldType,
    Tutorial,
    Create,
    Cancel,
}

impl CreateField {
    pub fn label(self) -> &'static str {
        match self {
            CreateField::Name => "World name",
            CreateField::Seed => "Seed",
            CreateField::Cheats => "Allow cheats",
            CreateField::DeveloperMode => "Developer mode",
            CreateField::WorldType => "World type",
            CreateField::Tutorial => "Show tutorial",
            CreateField::Create => "Create",
            CreateField::Cancel => "Cancel",
        }
    }
}
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
                world_type: WorldType::Normal,
            },
            field: 0,
        }
    }

    /// The rows shown, in order: World type only with Developer mode on.
    pub fn fields(&self) -> Vec<CreateField> {
        use CreateField::*;
        [
            Name,
            Seed,
            Cheats,
            DeveloperMode,
            WorldType,
            Tutorial,
            Create,
            Cancel,
        ]
        .into_iter()
        .filter(|&f| f != WorldType || self.settings.developer_mode)
        .collect()
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
        let fields = self.title.create.fields();
        let mut field = self.title.create.field;
        if self.navigate(&key, &mut field, fields.len()) {
            self.title.create.field = field;
            return;
        }
        if self.matches("BACK", &key) {
            self.show_world_list();
            return;
        }
        let select = self.matches("SELECT", &key);
        let left = self.matches("LEFT", &key);
        let toggle = select || left || self.matches("RIGHT", &key);
        let erase = self.matches("DELETE_CHAR", &key);
        let form = &mut self.title.create;
        match fields[field] {
            CreateField::Name | CreateField::Seed => {
                let text = if fields[field] == CreateField::Name {
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
            CreateField::Cheats if toggle => form.settings.cheats_allowed ^= true,
            CreateField::DeveloperMode if toggle => {
                form.settings.developer_mode ^= true;
                // Debug worlds need developer mode
                if !form.settings.developer_mode {
                    form.settings.world_type = WorldType::Normal;
                }
            }
            CreateField::WorldType if toggle => {
                let t = form.settings.world_type;
                form.settings.world_type = if left { t.prev() } else { t.next() };
            }
            CreateField::Tutorial if toggle => form.settings.show_tutorial ^= true,
            CreateField::Create if select => self.create_world_from_form(),
            CreateField::Cancel if select => self.show_world_list(),
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
    use crate::headless::{chain_commands, state_json};
    use crate::{MenuTab, SplashState};

    const TO_TITLE: &[&str] = &["rawkey ENTER", "rawkey ENTER"];
    /// From the title screen: Play Game, Create New World.
    const TO_CREATE: &[&str] = &["key title.SELECT", "key title.SELECT"];
    /// From the create form's name field: set the seed, press Create, skip the boot intro.
    const CREATE_12345: &[&str] = &[
        "key title.DOWN",
        "type \"12345\"",
        "key title.DOWN 5",
        "key title.SELECT",
        "rawkey ENTER",
        "rawkey ENTER",
    ];
    const MENU_TAB: &[&str] = &["key ui.MENU_NEXT 6"];
    /// From the title screen: create seed 12345, then save and quit back to the title.
    fn create_and_quit() -> Vec<&'static str> {
        [TO_CREATE, CREATE_12345, MENU_TAB, &["rawkey Q"]].concat()
    }

    fn last_message(runner: &crate::headless::Runner) -> String {
        let log = &runner.app.core.game.res.events.get_message_log().messages;
        log.last().map(|m| m.text.clone()).unwrap_or_default()
    }

    #[test]
    fn title_screen_shows_choices_and_back_returns_to_it() {
        let mut runner = chain_commands(TO_TITLE);
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
        runner.chain(&["key title.SELECT", "key title.BACK"]);
        assert_eq!(runner.app.splash.state, SplashState::Title);
        runner.chain(&["key title.BACK"]);
        assert_eq!(
            runner.app.splash.state,
            SplashState::Title,
            "back does nothing"
        );
        runner.chain(&["key title.UP", "key title.SELECT"]);
        assert!(
            runner.app.core.should_quit,
            "Quit is above Play Game when wrapping"
        );
    }

    #[test]
    fn creating_seed_12345_matches_skip_splash() {
        let mut created = chain_commands(&[TO_TITLE, TO_CREATE, CREATE_12345].concat());
        let mut skipped = chain_commands(&["skip-splash"]);
        assert_eq!(created.app.splash.state, SplashState::MainUI);
        assert_eq!(state_json(&created.app), state_json(&skipped.app));
        assert_eq!(created.screen_text(), skipped.screen_text());
        assert!(!created.app.core.game.res.world_settings.cheats_allowed);
    }

    #[test]
    fn save_quit_and_play_restores_position_and_inventory() {
        let mut runner = chain_commands(
            &[
                TO_TITLE,
                TO_CREATE,
                CREATE_12345,
                &["key movement.MOVE_SOUTH 2", "cheat giveitem log 4"],
            ]
            .concat(),
        );
        let snapshot = |runner: &crate::headless::Runner| {
            let game = &runner.app.core.game;
            (
                game.get_player_position(),
                format!(
                    "{:?}",
                    game.get_player_component::<lithicrivers_core::components::Inventory>()
                ),
            )
        };
        let before = snapshot(&runner);
        runner.chain(&[MENU_TAB, &["rawkey Q"]].concat());
        assert_eq!(runner.app.splash.state, SplashState::Title);
        runner.chain(&[
            "key title.SELECT",
            "expect-text \"seed 12345\"",
            "key title.SELECT",
            "key title.SELECT",
        ]);
        assert_eq!(runner.app.splash.state, SplashState::MainUI);
        assert_eq!(snapshot(&runner), before);
    }

    #[test]
    fn quit_tab_saves_and_returns_to_the_title() {
        let mut runner = chain_commands(&[
            "skip-splash",
            "key movement.MOVE_SOUTH",
            "key ui.MENU_NEXT 9",
        ]);
        assert_eq!(runner.app.ui.current_tab, MenuTab::Quit);
        let pos = runner.app.core.game.get_player_position();
        runner.chain(&["key ui.MENU_ACTIVATE"]);
        assert_eq!(runner.app.splash.state, SplashState::Title);
        assert!(
            !runner.app.core.should_quit,
            "the game keeps running at the title"
        );
        runner.chain(&["key title.SELECT", "key title.SELECT", "key title.SELECT"]);
        assert_eq!(runner.app.splash.state, SplashState::MainUI);
        assert_eq!(runner.app.core.game.get_player_position(), pos);
    }

    #[test]
    fn cheat_console_follows_the_world_setting() {
        // Allow cheats is off by default
        let mut off = chain_commands(
            &[
                TO_TITLE,
                TO_CREATE,
                CREATE_12345,
                &["key ui.OPEN_COMMAND_MENU"],
            ]
            .concat(),
        );
        assert!(!off.app.panels.cheat_console.is_open());
        assert!(last_message(&off).contains("Cheats are disabled for this world"));
        // Headless cheat steps still work
        off.chain(&["cheat tp 0 0 0"]);

        let allow_cheats: &[&str] = &["key title.DOWN 2", "key title.SELECT", "key title.UP 2"];
        let on = chain_commands(
            &[
                TO_TITLE,
                TO_CREATE,
                allow_cheats,
                CREATE_12345,
                &["key ui.OPEN_COMMAND_MENU"],
            ]
            .concat(),
        );
        assert!(on.app.core.game.res.world_settings.cheats_allowed);
        assert!(on.app.panels.cheat_console.is_open());
    }

    #[test]
    fn world_type_row_needs_developer_mode() {
        // Developer mode is on by default: pick Void, then turn developer mode off
        let mut runner = chain_commands(
            &[
                TO_TITLE,
                TO_CREATE,
                &[
                    "key title.DOWN 4",
                    "key title.RIGHT",
                    "expect-text \"World type: < Void >\"",
                    "key title.UP",
                    "key title.SELECT",
                ],
            ]
            .concat(),
        );
        assert!(!runner.screen_text().contains("World type"));
        assert_eq!(
            runner.app.title.create.settings.world_type,
            lithicrivers_core::resources::WorldType::Normal,
            "turning developer mode off resets the type"
        );
        // Without the row, Create is one row closer
        runner.chain(&[
            "key title.UP 2",
            "type \"7\"",
            "key title.DOWN 4",
            "key title.SELECT",
            "rawkey ENTER",
            "rawkey ENTER",
        ]);
        assert_eq!(runner.app.splash.state, SplashState::MainUI);
        assert_eq!(
            runner.app.core.game.res.world_settings.world_type,
            lithicrivers_core::resources::WorldType::Normal
        );
    }

    #[test]
    fn a_void_world_is_listed_with_its_type() {
        let create_lab: &[&str] = &[
            "key title.DELETE_CHAR 9",
            "type \"Lab\"",
            "key title.DOWN 4",
            "key title.RIGHT",
            "key title.DOWN 2",
            "key title.SELECT",
            "rawkey ENTER",
            "rawkey ENTER",
        ];
        let mut runner = chain_commands(&[TO_TITLE, TO_CREATE, create_lab].concat());
        assert_eq!(
            runner.app.core.game.res.world_settings.world_type,
            lithicrivers_core::resources::WorldType::Void
        );
        assert_eq!(
            runner.app.core.game.get_player_position(),
            Some(lithicrivers_core::debug_worlds::DEBUG_SPAWN)
        );
        runner.chain(&[MENU_TAB, &["rawkey Q", "key title.SELECT"]].concat());
        assert!(runner.screen_text().contains("Lab (Void)"));
    }

    #[test]
    fn world_without_tutorial_has_no_tutorial_tab() {
        let hide_tutorial: &[&str] = &["key title.DOWN 5", "key title.SELECT", "key title.UP 5"];
        let mut runner =
            chain_commands(&[TO_TITLE, TO_CREATE, hide_tutorial, CREATE_12345].concat());
        assert!(!runner.app.core.game.res.world_settings.show_tutorial);
        assert!(!runner.app.ui.tutorial_visible);
        assert_eq!(runner.app.ui.current_tab, MenuTab::World);
        assert!(!runner.screen_text().contains("Tutorial │"));
    }

    #[test]
    fn duplicate_names_make_two_worlds_and_delete_needs_confirmation() {
        // The second time, the list's cursor starts on the first world
        let second: &[&str] = &["key title.SELECT", "key title.DOWN", "key title.SELECT"];
        let mut runner = chain_commands(
            &[
                TO_TITLE,
                &create_and_quit(),
                second,
                CREATE_12345,
                MENU_TAB,
                &["rawkey Q"],
            ]
            .concat(),
        );
        runner.chain(&["key title.SELECT"]);
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
        runner.chain(&[
            "key title.SELECT",
            "key title.DOWN 2",
            "key title.SELECT",
            "key title.SELECT",
        ]);
        assert_eq!(runner.app.title.worlds.len(), 2);
        // Delete, then confirm
        runner.chain(&[
            "key title.SELECT",
            "key title.DOWN 2",
            "key title.SELECT",
            "key title.DOWN",
            "key title.SELECT",
        ]);
        assert_eq!(runner.app.splash.state, SplashState::WorldList);
        assert_eq!(runner.app.title.worlds.len(), 1);
        assert!(!folders[0].exists());
    }

    #[test]
    fn rename_changes_the_listed_name_and_back_cancels() {
        let mut runner =
            chain_commands(&[TO_TITLE, &create_and_quit(), &["key title.SELECT"]].concat());
        let folder = runner.app.title.worlds[0].folder.clone();
        // Rename, erase "World", type "Place", confirm
        runner.chain(&[
            "key title.SELECT",
            "key title.DOWN",
            "key title.SELECT",
            "key title.DELETE_CHAR 5",
            "type \"Place\"",
            "expect-text \"New name: [New Place_]\"",
            "key title.SELECT",
        ]);
        assert_eq!(runner.app.splash.state, SplashState::WorldList);
        let world = &runner.app.title.worlds[0];
        assert_eq!(world.meta.as_ref().unwrap().name, "New Place");
        assert_eq!(world.folder, folder);
        assert!(runner.screen_text().contains("New Place  (seed 12345"));

        // Back leaves the name alone
        runner.chain(&[
            "key title.SELECT",
            "key title.DOWN",
            "key title.SELECT",
            "type \"xyz\"",
            "key title.BACK",
        ]);
        assert_eq!(
            runner.app.title.worlds[0].meta.as_ref().unwrap().name,
            "New Place"
        );
    }

    #[test]
    fn options_change_volume_in_config_and_audio() {
        let mut runner =
            chain_commands(&[TO_TITLE, &["key title.DOWN", "key title.SELECT"]].concat());
        assert_eq!(runner.app.splash.state, SplashState::Options);
        let music = runner.app.volume_setting("MUSIC_VOLUME");
        runner.chain(&["key title.LEFT 2"]);
        let lowered = runner.app.volume_setting("MUSIC_VOLUME");
        assert!(
            (lowered - (music - 0.10)).abs() < 1e-9,
            "{music} -> {lowered}"
        );
        assert!((runner.app.audio.audio.music_volume() as f64 - lowered).abs() < 1e-6);
        assert!(runner
            .screen_text()
            .contains(&format!("{}%", (lowered * 100.0).round())));
        runner.chain(&["key title.DOWN", "key title.RIGHT"]);
        assert!(runner.app.volume_setting("SFX_VOLUME") > 0.35);
        runner.chain(&["key title.BACK"]);
        assert_eq!(runner.app.splash.state, SplashState::Title);
    }
}

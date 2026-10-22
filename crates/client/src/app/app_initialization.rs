use crate::app_state::*;
use crate::tutorialsystem::TutorialDatabase;
use crate::{
    audio, boot_message, clock::Clock, App, EmbeddedAssets, MenuTab, Scale, SplashState,
    SpriteLoader,
};
use chrono::prelude::Local;
use lithicrivers_core::components::Position;
use lithicrivers_core::config::ConfigManager;
use lithicrivers_core::dialogue_database::DialogueDatabase;
use lithicrivers_core::game::Game;
use lithicrivers_core::recipe_handler::{RecipeHandler, RepairRecipeHandler};

impl App {
    pub fn new_with_seed(seed: u64) -> App {
        let audio = audio::AudioManager::new().unwrap_or_else(|e| {
            panic!("Failed to initialize audio: {}", e);
        });
        Self::build(seed, audio, Clock::Real)
    }

    /// App for scripted runs: no audio device and a fake clock that only moves when advanced.
    pub fn new_headless(seed: u64) -> App {
        let mut app = Self::build(seed, audio::AudioManager::disabled(), Clock::fake());
        // The real path contains the working directory and today's date; keep output reproducible
        app.logging.log_full_path = "(hidden in headless runs)".to_string();
        // Never write the player's config file from a scripted run
        app.persist_config = false;
        app
    }

    fn build(seed: u64, mut audio: audio::AudioManager, clock: Clock) -> App {
        // The title menu shows before any world exists; until one starts, a cheap empty game
        // stands in so the rest of the client can assume a game is there
        let game = Game::new_empty(seed, Position { x: 0, y: 0, z: 0 });
        let mut sprite_loader = SpriteLoader::new(None);
        // Preload all assets to eliminate runtime I/O during rendering
        sprite_loader.preload_all();

        // Register and play the opening music
        let music_track = audio::AudioTrack::new("crates/client/assets/sound/music/opening.mp3");
        audio.register_music("opening", music_track);
        if let Err(e) = audio.play_music("opening") {
            panic!("Failed to play music: {}", e);
        }

        let wood_crack =
            audio::AudioTrack::new("crates/client/assets/sound/effects/wood_crack.mp3");
        audio.register_sound_effect("wood_crack", wood_crack);

        // Build credits text from embedded config files
        let version = EmbeddedAssets::get("config/VERSION")
            .map(|d| String::from_utf8_lossy(&d.data).to_string())
            .unwrap_or_else(|| "(missing VERSION)".to_string());
        let app_id = EmbeddedAssets::get("config/STEAM_APP_ID")
            .map(|d| String::from_utf8_lossy(&d.data).to_string())
            .unwrap_or_else(|| "(missing STEAM_APP_ID)".to_string());
        let git_branch = EmbeddedAssets::get("config/GIT_BRANCH")
            .map(|d| String::from_utf8_lossy(&d.data).to_string())
            .unwrap_or_else(|| "(missing GIT_BRANCH)".to_string());
        let git_commit = EmbeddedAssets::get("config/GIT_SHA")
            .map(|d| String::from_utf8_lossy(&d.data).to_string())
            .unwrap_or_else(|| "(missing GIT_SHA)".to_string());
        let credits_body = EmbeddedAssets::get("config/credits.txt")
            .map(|d| String::from_utf8_lossy(&d.data).to_string())
            .unwrap_or_else(|| panic!("credits.txt not found"));

        // Load logo text
        let logo_text = EmbeddedAssets::get("config/logo.txt")
            .map(|d| String::from_utf8_lossy(&d.data).to_string())
            .unwrap_or_else(|| panic!("logo.txt not found"));

        // Load game title text
        let game_title_text = EmbeddedAssets::get("config/gametitle.txt")
            .map(|d| String::from_utf8_lossy(&d.data).to_string())
            .unwrap_or_else(|| panic!("gametitle.txt not found"));

        // Load and process boot message
        let boot_message = boot_message::get_boot_message();
        let boot_message_lines = boot_message.lines().map(String::from).collect::<Vec<_>>();

        let credits_text = format!(
            "Version: {}\nSTEAM_APP_ID: {}\nGit Branch: {}\nGit Commit: {}\n\n{}",
            version.trim(),
            app_id.trim(),
            git_branch.trim(),
            git_commit.trim(),
            credits_body
        );

        // Determine absolute logging directory and concrete file name for today
        let log_dir_abs = std::env::current_dir()
            .ok()
            .and_then(|p| p.canonicalize().ok())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string());
        let today = Local::now().format("%Y-%m-%d").to_string();
        let log_full_path = format!("{}/LithicRivers-{}.log", log_dir_abs, today);

        // Build keybinds from game config
        let config_manager = ConfigManager::new();
        let keybinds = Keybinds::from_config(&config_manager);
        let show_context_help = config_manager
            .get_setting("ui", "SHOW_CONTEXT_HELP")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let repair_handler = RepairRecipeHandler::new();
        let saves_dir = config_manager
            .get_setting("game", "SAVES_FOLDER")
            .and_then(|v| v.as_str())
            .unwrap_or("lithicrivers-saves/saves")
            .into();

        let developer_mode = game.res.config.developer_mode;
        let mut app = App {
            core: CoreState {
                game,
                config_manager,
                sprite_loader,
                repair_handler,
                should_quit: false,
            },
            ui: UiState {
                current_tab: MenuTab::Tutorial,
                scale: Scale::Small,
                bottom_menu_rect: None,
                world_view_rect: None,
                mouse_cell: None,
                click_targets: Default::default(),
                keybinds,
                tutorial_system: TutorialDatabase::create_tutorial_system_with_basics(),
                tutorial_visible: true, // Tutorial tab is visible by default
                show_context_help,
                view_x: 0,
                view_y: 0,
                view_z: 0,
            },
            audio: AudioState { audio },
            logging: LoggingState { log_full_path },
            combat: CombatUiState::default(),
            panels: Self::fresh_panels(credits_text),
            splash: SplashScreenState {
                state: SplashState::Logo,
                start_time: Some(clock.now()),
                logo_text,
                game_title_text,
                boot_message,
                boot_message_lines,
                boot_display_text: String::new(),
                boot_line_index: 0,
                boot_scroll: 0,
                last_line_time: clock.now(),
                boot_complete: false,
            },
            last_combat_tick: None,
            clock,
            default_seed: seed,
            saves_dir,
            current_world: None,
            title: crate::app::title_menu::TitleMenuState::new(developer_mode),
            persist_config: true,
            demo: None,
        };
        app.apply_volumes();
        app.reset_for_game();
        app
    }

    /// Panel state as a newly started world has it.
    fn fresh_panels(credits_text: String) -> PanelStates {
        PanelStates {
            power_info: None,
            furnace_window: None,
            item_dictionary: Default::default(),
            filter_picker: None,
            inventory: InventoryPanelState { selected: 0 },
            crafting: CraftingPanelState {
                recipe_handler: RecipeHandler::new(),
                selected: 0,
                message: None,
            },
            credits: CreditsPanelState {
                text: credits_text,
                scroll: 0,
            },
            help: HelpPanelState { scroll: 0 },
            look: LookPanelState {
                mode: false,
                cursor: Position { x: 0, y: 0, z: 0 },
            },
            build: BuildPanelState::default(),
            corpse_looting: CorpseLootingState::default(),
            hotbar_assignment: HotbarAssignmentState::default(),
            npc_interaction: NPCInteractionState::default(),
            multi_action_select: MultiActionSelectState::default(),
            dialogue_tree: {
                let database = DialogueDatabase::new();
                database
                    .get_tree("broken_android_tutorial")
                    .expect("broken_android_tutorial dialogue tree should exist")
                    .clone()
            },
            body_repair: BodyRepairState::default(),
            cheat_console: CheatConsoleState::default(),
            global_map: GlobalMapPanelState::default(),
            tutorial_selection: TutorialSelectionState::default(),
            quests: QuestPanelState::default(),
        }
    }

    /// Put all in-game UI state back to how a new world starts, for the current `core.game`:
    /// Tutorial tab, view and look cursor on the player, fresh panels, tutorial and combat.
    pub(crate) fn reset_for_game(&mut self) {
        let credits_text = std::mem::take(&mut self.panels.credits.text);
        self.panels = Self::fresh_panels(credits_text);
        self.ui.current_tab = MenuTab::Tutorial;
        self.ui.tutorial_system = TutorialDatabase::create_tutorial_system_with_basics();
        self.ui.tutorial_visible = self.core.game.res.world_settings.show_tutorial;
        if !self.ui.tutorial_visible {
            self.ui.current_tab = MenuTab::World;
        }
        let player = self
            .core
            .game
            .get_player_position()
            .unwrap_or(Position { x: 0, y: 0, z: 0 });
        self.panels.look.cursor = player;
        (self.ui.view_x, self.ui.view_y, self.ui.view_z) = (player.x, player.y, player.z);
        self.combat = CombatUiState::default();
        self.last_combat_tick = None;
    }
}

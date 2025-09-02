use crate::app_state::*;
use crate::{audio, boot_message, App, EmbeddedAssets, MenuTab, Scale, SplashState, SpriteLoader};
use chrono::prelude::Local;
use lithicrivers_core::components::Position;
use lithicrivers_core::config::ConfigManager;
use lithicrivers_core::game::Game;
use lithicrivers_core::recipe_handler::RecipeHandler;
use std::time::Instant;

impl App {
    pub fn new_with_seed(seed: u64) -> App {
        // Initialize game and sprite loader
        let mut game = Game::new(seed);
        let mut sprite_loader = SpriteLoader::new(None);
        // Preload all assets to eliminate runtime I/O during rendering
        sprite_loader.preload_all();

        // Initialize audio manager
        let mut audio = audio::AudioManager::new().unwrap_or_else(|e| {
            panic!("Failed to initialize audio: {}", e);
        });

        // Set volumes
        audio.set_music_volume(0.60);
        audio.set_sfx_volume(0.35);

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

        // Initialize look cursor to player's position (or origin fallback)
        let mut look_cursor = Position { x: 0, y: 0, z: 0 };
        if let Some(e) = game.res.player_entity {
            if let Ok(pos) = game.world.get::<&Position>(e) {
                look_cursor = *pos;
                // Center initial viewport on player
                game.res.view_x = look_cursor.x;
                game.res.view_y = look_cursor.y;
                game.res.view_z = look_cursor.z;
            }
        }

        let recipe_handler = RecipeHandler::new();

        App {
            core: CoreState {
                game,
                config_manager,
                sprite_loader,
                should_quit: false,
            },
            ui: UiState {
                current_tab: MenuTab::World,
                scale: Scale::Small,
                bottom_menu_rect: None,
                keybinds,
            },
            audio: AudioState { audio },
            logging: LoggingState { log_full_path },
            combat: CombatState {
                combat_happening: false,
            },
            panels: PanelStates {
                inventory: InventoryPanelState { selected: 0 },
                crafting: CraftingPanelState {
                    recipe_handler,
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
                    cursor: look_cursor,
                },
            },
            splash: SplashScreenState {
                state: SplashState::Logo,
                start_time: Some(Instant::now()),
                logo_text,
                game_title_text,
                boot_message,
                boot_message_lines,
                boot_display_text: String::new(),
                boot_line_index: 0,
                boot_scroll: 0,
                last_line_time: Instant::now(),
                boot_complete: false,
            },
        }
    }

    #[allow(dead_code)]
    pub fn new() -> App {
        Self::new_with_seed(12345) //TODO use seed from config...
    }
}

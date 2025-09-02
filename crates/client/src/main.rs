use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
mod boot_message;
mod input;
mod ui;

use boot_message::get_boot_message;
use lithicrivers_core::game::GameTickResult;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style, Stylize as _},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs, Wrap},
    Frame, Terminal,
};
use rust_embed::RustEmbed;
use std::{cmp::max, collections::VecDeque, error::Error, io, time::Duration, time::Instant};
use tracing_subscriber::EnvFilter;

// Tracing file appender for log file output
use chrono::Local;
use tracing_appender as _tracing_appender_hidden; // avoid "unused extern crate" lint

#[derive(RustEmbed)]
#[folder = "assets/"]
struct EmbeddedAssets;

// Removed unused import
use lithicrivers_core::components::{
    itemkind_name, itemkind_sprite_name, DroppedItem, Inventory as InvComp, ItemKind, ItemStack,
    Position, SpriteRef,
};
use lithicrivers_core::config::ConfigManager;
use lithicrivers_core::{recipe_handler::RecipeHandler, Game};
use std::collections::HashMap;
mod audio;
mod rendering_helpers;
mod sprite_constants;
mod sprite_loader;
use crate::{
    input::format_keycode,
    rendering_helpers::{
        block_art_12x8_lines_for_position, build_body_ascii, empty_art_12x8_lines_for_position,
        entity_art_12x8_lines_for_position, parse_hex_color,
    },
    sprite_constants::{sprite_for_view_reticle, sprite_for_view_reticle_color},
    sprite_loader::{sprite_block_for_spriteref, sprite_block_for_tile, Scale, SpriteLoader},
    ui::{
        centered_rect,
        panels::{
            get_player_inventory, render_body_panel, render_crafting_panel, render_credits_panel,
            render_help_panel, render_inventory_list_only, render_inventory_panel,
            render_quit_panel,
        },
    },
};
use lithicrivers_core::model::body::{Body, BodyPart, BodyPartState};
use lithicrivers_core::world::CHUNK_SIZE;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SplashState {
    Logo,
    GameTitle,
    BootMessage,
    MainUI,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MenuTab {
    World,
    Body,
    Inventory,
    Crafting,
    Menu,
    Help,
    Credits,
    Quit,
}

const BOOT_MESSAGE_TYPEWRITER_MS: u64 = 100;

impl MenuTab {
    const COUNT: usize = 8;

    fn next(self) -> Self {
        match self {
            MenuTab::World => MenuTab::Body,
            MenuTab::Body => MenuTab::Inventory,
            MenuTab::Inventory => MenuTab::Crafting,
            MenuTab::Crafting => MenuTab::Menu,
            MenuTab::Menu => MenuTab::Help,
            MenuTab::Help => MenuTab::Credits,
            MenuTab::Credits => MenuTab::Quit,
            MenuTab::Quit => MenuTab::World,
        }
    }

    fn prev(self) -> Self {
        match self {
            MenuTab::World => MenuTab::Quit,
            MenuTab::Body => MenuTab::World,
            MenuTab::Inventory => MenuTab::Body,
            MenuTab::Crafting => MenuTab::Inventory,
            MenuTab::Menu => MenuTab::Crafting,
            MenuTab::Help => MenuTab::Menu,
            MenuTab::Credits => MenuTab::Help,
            MenuTab::Quit => MenuTab::Credits,
        }
    }

    fn as_index(&self) -> usize {
        *self as usize
    }
}

struct App {
    game: Game,
    config_manager: ConfigManager,
    sprite_loader: SpriteLoader,
    should_quit: bool,
    // UI state: remember bottom menu rect for click handling
    bottom_menu_rect: Option<Rect>,
    current_tab: MenuTab,
    scale: Scale,
    audio: audio::AudioManager,
    // Credits panel state
    credits_text: String,
    credits_scroll: u16,
    // Logging info
    log_full_path: String,
    keybinds: Keybinds,
    // Look mode state
    look_mode: bool,
    look_cursor: lithicrivers_core::components::Position,
    // Inventory panel state
    inv_selected: usize,
    // combat state below
    combat_happening: bool,
    // Crafting system
    recipe_handler: RecipeHandler,
    craft_selected: usize,
    craft_message: Option<(String, u8)>, // (message, timer)
    // Splash screen state
    splash_state: SplashState,
    splash_start_time: Option<std::time::Instant>,
    logo_text: String,
    game_title_text: String,
    boot_message: String,
    boot_message_lines: Vec<String>,
    boot_display_text: String,
    boot_line_index: usize,
    boot_scroll: u16, // Tracks scroll position for boot message
    last_line_time: Instant,
    boot_complete: bool,
    // Help panel state
    help_scroll: u16,
}

#[derive(Debug, Clone)]
struct Keybinds {
    // key: "category:ACTION" => list of KeyCodes
    map: HashMap<String, Vec<KeyCode>>,
}

impl Keybinds {
    fn from_config(cfg: &ConfigManager) -> Self {
        let mut map: HashMap<String, Vec<KeyCode>> = HashMap::new();
        if let Some(obj) = cfg.data.keybinds.as_object() {
            for (category, actions) in obj.iter() {
                if let Some(act_obj) = actions.as_object() {
                    for (action, arr) in act_obj.iter() {
                        let mut codes: Vec<KeyCode> = Vec::new();
                        if let Some(list) = arr.as_array() {
                            for v in list {
                                if let Some(s) = v.as_str() {
                                    if let Some(code) = Self::parse_keycode(s) {
                                        codes.push(code);
                                    }
                                }
                            }
                        }
                        if !codes.is_empty() {
                            map.insert(format!("{}:{}", category, action), codes);
                        }
                    }
                }
            }
        }

        tracing::info!(target: "game", "built keybind map: {:?}", map);
        Self { map }
    }

    fn matches(&self, category: &str, action: &str, key: &KeyCode) -> bool {
        let k = format!("{}:{}", category, action);

        // tracing::info!(target: "game", "self.map: {:?}", self.map);
        // tracing::info!(target: "game", "k: {:?}", k);
        // tracing::info!(target: "game", "key: {:?}", key);

        if let Some(list) = self.map.get(&k) {
            for c in list {
                if c == key {
                    // tracing::info!(target: "game", "match!");
                    return true;
                }
            }
        }
        false
    }

    fn matches_movement(&self, key: &KeyCode) -> bool {
        self.matches("movement", "MOVE_NORTH", key)
            || self.matches("movement", "MOVE_SOUTH", key)
            || self.matches("movement", "MOVE_WEST", key)
            || self.matches("movement", "MOVE_EAST", key)
            || self.matches("movement", "MOVE_NORTHWEST", key)
            || self.matches("movement", "MOVE_NORTHEAST", key)
            || self.matches("movement", "MOVE_SOUTHWEST", key)
            || self.matches("movement", "MOVE_SOUTHEAST", key)
            || self.matches("movement", "WAIT", key)
            || self.matches("movement", "MOVE_UP", key)
            || self.matches("movement", "MOVE_DOWN", key)
    }

    fn parse_keycode(s: &str) -> Option<KeyCode> {
        lithicrivers_core::keycode_mapping::parse_keycode(s)
    }
}

impl App {
    fn new_with_seed(seed: u64) -> App {
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
        let mut boot_message_lines = boot_message.lines().map(String::from).collect::<Vec<_>>();

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
        let mut look_cursor = lithicrivers_core::components::Position { x: 0, y: 0, z: 0 };
        if let Some(e) = game.res.player_entity {
            if let Ok(pos) = game
                .world
                .get::<&lithicrivers_core::components::Position>(e)
            {
                look_cursor = *pos;
                // Center initial viewport on player
                game.res.view_x = look_cursor.x;
                game.res.view_y = look_cursor.y;
                game.res.view_z = look_cursor.z;
            }
        }

        let recipe_handler = RecipeHandler::new();

        App {
            game,
            config_manager,
            sprite_loader,
            should_quit: false,
            bottom_menu_rect: None,
            audio,
            current_tab: MenuTab::World,
            scale: Scale::Small,
            credits_text,
            credits_scroll: 0,
            log_full_path,
            keybinds,
            look_mode: false,
            look_cursor,
            inv_selected: 0,
            recipe_handler,
            craft_selected: 0,
            craft_message: None,
            // Help panel state
            help_scroll: 0,
            // Combat state
            combat_happening: false,
            // Initialize splash screen state
            splash_state: SplashState::Logo,
            splash_start_time: Some(Instant::now()),
            logo_text,
            game_title_text,
            boot_message,
            boot_message_lines,
            boot_display_text: String::new(),
            boot_line_index: 0,
            boot_scroll: 0,
            last_line_time: Instant::now(),
            boot_complete: false,
        }
    }

    fn new() -> App {
        Self::new_with_seed(12345) //TODO use seed from config...
    }

    fn on_tick(&mut self) {
        // Handle splash screen timing
        if let Some(start_time) = self.splash_start_time {
            match self.splash_state {
                SplashState::Logo => {
                    if start_time.elapsed() >= Duration::from_secs(1) {
                        self.splash_state = SplashState::GameTitle;
                        self.splash_start_time = Some(Instant::now());
                    }
                }
                SplashState::GameTitle => {
                    if start_time.elapsed() >= Duration::from_secs(1) {
                        self.splash_state = SplashState::BootMessage;
                        self.splash_start_time = Some(Instant::now());
                        self.boot_display_text.clear();
                        self.boot_line_index = 0;
                        self.last_line_time = Instant::now();
                        self.boot_complete = false;
                    }
                }
                SplashState::BootMessage => {
                    // Handle typewriter effect
                    if !self.boot_complete {
                        let elapsed = self.last_line_time.elapsed();
                        if elapsed >= Duration::from_millis(BOOT_MESSAGE_TYPEWRITER_MS) {
                            // ~(1000/x) characters per second
                            self.last_line_time = Instant::now();

                            // Get all text as a single string with newlines
                            let full_text = self.boot_message_lines.join("\n");

                            if self.boot_line_index < full_text.len() {
                                // Move to next line
                                if let Some(next_newline) =
                                    full_text[self.boot_line_index..].find('\n')
                                {
                                    self.boot_line_index += next_newline + 1;
                                } else {
                                    self.boot_line_index = full_text.len();
                                }
                                self.boot_display_text =
                                    full_text[..self.boot_line_index].to_string();
                            } else if !self.boot_complete {
                                self.boot_complete = true;
                                // Set a minimum display time after completion
                                self.splash_start_time = Some(Instant::now());
                            }
                        }
                    } else if start_time.elapsed() >= Duration::from_secs(2) {
                        // 2 seconds after completion, move to main UI
                        self.splash_state = SplashState::MainUI;
                    }
                }
                _ => {}
            }
        }
    }

    fn snap_view_to_player_z(&mut self) {
        if let Some(e) = self.game.res.player_entity {
            if let Ok(pos) = self
                .game
                .world
                .get::<&lithicrivers_core::components::Position>(e)
            {
                // Snap entire viewport center and Z slice to player
                self.game.res.view_x = pos.x;
                self.game.res.view_y = pos.y;
                self.game.res.view_z = pos.z;
            }
        }
    }

    fn handle_input(&mut self, key: KeyCode) -> Result<(), Box<dyn Error>> {
        // Handle splash screen skipping first
        if let Some(start_time) = self.splash_start_time {
            match self.splash_state {
                SplashState::Logo => {
                    // Any key skips to next screen
                    self.splash_state = SplashState::GameTitle;
                    self.splash_start_time = Some(Instant::now());
                    return Ok(());
                }
                SplashState::GameTitle => {
                    // Any key skips to boot message
                    self.splash_state = SplashState::BootMessage;
                    self.splash_start_time = Some(Instant::now());
                    self.boot_display_text.clear();
                    self.boot_line_index = 0;
                    self.last_line_time = Instant::now();
                    self.boot_complete = false;
                    return Ok(());
                }
                SplashState::BootMessage => {
                    if !self.boot_complete {
                        // Skip to end of text
                        let full_text = self.boot_message_lines.join("\n");
                        self.boot_display_text = full_text.clone();
                        self.boot_line_index = full_text.len();
                        self.boot_complete = true;
                        self.splash_start_time = Some(Instant::now());
                    } else {
                        // Move to main UI if already complete
                        self.splash_state = SplashState::MainUI;
                    }
                    return Ok(());
                }
                _ => {}
            }
        }

        // log key to log
        // tracing::info!(target: "game", "key pressed: {:?}", key);

        // First, handle configurable keybind actions
        // Toggle Look mode
        if self.keybinds.matches("action", "LOOK_TOGGLE", &key) {
            self.look_mode = !self.look_mode;
            // Reset cursor to player on toggle on
            if self.look_mode {
                if let Some(e) = self.game.res.player_entity {
                    if let Ok(pos) = self
                        .game
                        .world
                        .get::<&lithicrivers_core::components::Position>(e)
                    {
                        self.look_cursor = *pos;
                        // Align all view coords to cursor
                        self.game.res.view_x = self.look_cursor.x;
                        self.game.res.view_y = self.look_cursor.y;
                        self.game.res.view_z = self.look_cursor.z;
                    }
                }
                self.game.res.log("Look mode: ON");
            } else {
                self.game.res.log("Look mode: OFF");
            }
            return Ok(());
        }

        // In Look mode, remap movement keys to move the look cursor without ticking
        if self.look_mode {
            let mut moved = false;
            if self.keybinds.matches("movement", "MOVE_NORTH", &key) {
                self.look_cursor.y -= 1;
                self.game.res.view_y = self.look_cursor.y;
                moved = true;
            } else if self.keybinds.matches("movement", "MOVE_SOUTH", &key) {
                self.look_cursor.y += 1;
                self.game.res.view_y = self.look_cursor.y;
                moved = true;
            } else if self.keybinds.matches("movement", "MOVE_WEST", &key) {
                self.look_cursor.x -= 1;
                self.game.res.view_x = self.look_cursor.x;
                moved = true;
            } else if self.keybinds.matches("movement", "MOVE_EAST", &key) {
                self.look_cursor.x += 1;
                self.game.res.view_x = self.look_cursor.x;
                moved = true;
            } else if self.keybinds.matches("movement", "MOVE_NORTHWEST", &key) {
                self.look_cursor.x -= 1;
                self.look_cursor.y -= 1;
                self.game.res.view_x = self.look_cursor.x;
                self.game.res.view_y = self.look_cursor.y;
                moved = true;
            } else if self.keybinds.matches("movement", "MOVE_NORTHEAST", &key) {
                self.look_cursor.x += 1;
                self.look_cursor.y -= 1;
                self.game.res.view_x = self.look_cursor.x;
                self.game.res.view_y = self.look_cursor.y;
                moved = true;
            } else if self.keybinds.matches("movement", "MOVE_SOUTHWEST", &key) {
                self.look_cursor.x -= 1;
                self.look_cursor.y += 1;
                self.game.res.view_x = self.look_cursor.x;
                self.game.res.view_y = self.look_cursor.y;
                moved = true;
            } else if self.keybinds.matches("movement", "MOVE_SOUTHEAST", &key) {
                self.look_cursor.x += 1;
                self.look_cursor.y += 1;
                self.game.res.view_x = self.look_cursor.x;
                self.game.res.view_y = self.look_cursor.y;
                moved = true;
            } else if self.keybinds.matches("movement", "WAIT", &key) {
                // no-op, but treat as handled to avoid player waiting
                moved = true;
            } else if self.keybinds.matches("movement", "MOVE_UP", &key) {
                self.look_cursor.z += 1;
                self.game.res.view_z = self.look_cursor.z;
                moved = true;
            } else if self.keybinds.matches("movement", "MOVE_DOWN", &key) {
                self.look_cursor.z -= 1;
                self.game.res.view_z = self.look_cursor.z;
                moved = true;
            } else if self.keybinds.matches("viewport", "VIEW_Z_UP", &key) {
                self.look_cursor.z = self.look_cursor.z.saturating_add(1);
                self.game.res.view_z = self.look_cursor.z;
                moved = true;
            } else if self.keybinds.matches("viewport", "VIEW_Z_DOWN", &key) {
                self.look_cursor.z = self.look_cursor.z.saturating_sub(1);
                self.game.res.view_z = self.look_cursor.z;
                moved = true;
            }

            if moved {
                // Prefetch around the new cursor position for smoother draw
                let radius = 20i32;
                let left = self.look_cursor.x - radius;
                let top = self.look_cursor.y - radius;
                let right = self.look_cursor.x + radius;
                let bottom = self.look_cursor.y + radius;
                self.game
                    .res
                    .world
                    .prefetch_rect(left, top, right, bottom, self.look_cursor.z);
                return Ok(());
            }
        }
        // Inventory: toggle item auto-pickup
        if self.current_tab == MenuTab::Inventory
            && self
                .keybinds
                .matches("inventory", "TOGGLE_ITEM_AUTO_PICKUP_KEY", &key)
        {
            if let Some(e) = self.game.res.player_entity {
                if let Ok(mut inv) = self.game.world.get::<&mut InvComp>(e) {
                    inv.auto_pickup = !inv.auto_pickup;
                    let state = if inv.auto_pickup { "ON" } else { "OFF" };
                    self.game.res.log(format!("Item auto-pickup: {}", state));
                }
            }
            return Ok(());
        }

        // Crafting panel-specific navigation and actions
        if self.current_tab == MenuTab::Crafting {
            // Get player inventory for crafting checks
            let inventory = get_player_inventory(self);
            let recipe_count = self.recipe_handler.get_recipes().len();

            // Navigation: Up/Down or North/South to move selection
            if self.keybinds.matches("ui", "CREDITS_SCROLL_UP", &key)
                || self.keybinds.matches("movement", "MOVE_NORTH", &key)
            {
                if recipe_count > 0 {
                    if self.craft_selected == 0 {
                        self.craft_selected = recipe_count - 1;
                    } else {
                        self.craft_selected = self.craft_selected.saturating_sub(1);
                    }
                }
                return Ok(());
            }
            if self.keybinds.matches("ui", "CREDITS_SCROLL_DOWN", &key)
                || self.keybinds.matches("movement", "MOVE_SOUTH", &key)
            {
                if recipe_count > 0 {
                    self.craft_selected = (self.craft_selected + 1) % recipe_count;
                }
                return Ok(());
            }

            // Craft item on Enter or Activate key
            if self.keybinds.matches("ui", "ACTIVATE", &key) || key == KeyCode::Enter {
                if let Some(recipe) = self.recipe_handler.get_recipes().get(self.craft_selected) {
                    if self
                        .recipe_handler
                        .can_craft(self.craft_selected, &inventory)
                    {
                        if let Some(e) = self.game.res.player_entity {
                            if let Ok(mut inv) = self.game.world.get::<&mut InvComp>(e) {
                                // Consume ingredients
                                for &(item, qty) in recipe.ingredients {
                                    let mut remaining = qty;
                                    for slot in inv.slots.iter_mut() {
                                        if slot.kind == item && remaining > 0 {
                                            let consume = slot.qty.min(remaining);
                                            slot.qty -= consume;
                                            remaining -= consume;

                                            if slot.qty == 0 {
                                                // Remove empty slots - the inventory will be compacted later
                                                // by the game's inventory management system
                                            }
                                        }
                                    }
                                }

                                // Add crafted item to inventory
                                let mut added = false;
                                for slot in inv.slots.iter_mut() {
                                    if slot.kind == recipe.result && slot.qty < 1000 {
                                        // Arbitrary max stack size
                                        slot.qty = slot.qty.saturating_add(recipe.quantity);
                                        added = true;
                                        break;
                                    }
                                }

                                if !added {
                                    inv.slots.push(ItemStack {
                                        kind: recipe.result,
                                        qty: recipe.quantity,
                                    });
                                }

                                self.craft_message = Some((
                                    format!(
                                        "Crafted {}x {}",
                                        recipe.quantity,
                                        itemkind_name(recipe.result)
                                    ),
                                    30, // Display for 30 frames (~0.5 seconds at 60 FPS)
                                ));

                                self.game.res.log(format!(
                                    "Crafted {}x {}",
                                    recipe.quantity,
                                    itemkind_name(recipe.result)
                                ));
                            }
                        }
                    } else {
                        self.craft_message = Some((
                            "Not enough resources to craft this item".to_string(),
                            60, // Display for 1 second at 60 FPS
                        ));
                    }
                }
                return Ok(());
            }

            // Don't process movement keys in crafting panel
            if self.keybinds.matches("movement", "MOVE_NORTH", &key)
                || self.keybinds.matches("movement", "MOVE_SOUTH", &key)
                || self.keybinds.matches("movement", "MOVE_WEST", &key)
                || self.keybinds.matches("movement", "MOVE_EAST", &key)
                || self.keybinds.matches("movement", "MOVE_NORTHWEST", &key)
                || self.keybinds.matches("movement", "MOVE_NORTHEAST", &key)
                || self.keybinds.matches("movement", "MOVE_SOUTHWEST", &key)
                || self.keybinds.matches("movement", "MOVE_SOUTHEAST", &key)
                || self.keybinds.matches("movement", "WAIT", &key)
                || self.keybinds.matches("movement", "MOVE_UP", &key)
                || self.keybinds.matches("movement", "MOVE_DOWN", &key)
            {
                return Ok(());
            }
        }

        // Inventory panel-specific navigation and actions
        if self.current_tab == MenuTab::Inventory {
            // Move selection: support Up/Down keys and numpad 8/2 (MOVE_NORTH/SOUTH)
            if self.keybinds.matches("ui", "CREDITS_SCROLL_UP", &key)
                || self.keybinds.matches("movement", "MOVE_NORTH", &key)
            {
                if let Some(e) = self.game.res.player_entity {
                    if let Ok(inv) = self.game.world.get::<&InvComp>(e) {
                        if !inv.slots.is_empty() {
                            if self.inv_selected == 0 {
                                self.inv_selected = inv.slots.len() - 1;
                            } else {
                                self.inv_selected -= 1;
                            }
                        }
                    }
                }
                return Ok(());
            }
            if self.keybinds.matches("ui", "CREDITS_SCROLL_DOWN", &key)
                || self.keybinds.matches("movement", "MOVE_SOUTH", &key)
            {
                if let Some(e) = self.game.res.player_entity {
                    if let Ok(inv) = self.game.world.get::<&InvComp>(e) {
                        if !inv.slots.is_empty() {
                            self.inv_selected = (self.inv_selected + 1) % inv.slots.len();
                        }
                    }
                }
                return Ok(());
            }

            // Helper to get current selection
            let mut selected: Option<(ItemKind, u32)> = None;
            if let Some(e) = self.game.res.player_entity {
                if let Ok(inv) = self.game.world.get::<&InvComp>(e) {
                    if !inv.slots.is_empty() {
                        let idx = self.inv_selected.min(inv.slots.len() - 1);
                        selected = Some((inv.slots[idx].kind, inv.slots[idx].qty));
                    }
                }
            }

            // Drop selected item (quantity 1 for now)
            if self.keybinds.matches("inventory", "DROP_ITEM", &key) {
                if let Some((kind, qty)) = selected {
                    if qty == 0 {
                        return Ok(());
                    }
                    if let Some(e) = self.game.res.player_entity {
                        // Copy player position, then drop immutable borrow before mutating world
                        let (px, py, pz) = {
                            let Ok(ppos) = self
                                .game
                                .world
                                .get::<&lithicrivers_core::components::Position>(e)
                            else {
                                return Ok(());
                            };
                            (ppos.x, ppos.y, ppos.z)
                        };

                        let drop_qty = 1u32;
                        // Decrement inventory (mutable borrow scope ends before spawn)
                        if let Ok(mut inv) = self.game.world.get::<&mut InvComp>(e) {
                            if self.inv_selected < inv.slots.len() {
                                let slot = &mut inv.slots[self.inv_selected];
                                if slot.qty >= drop_qty {
                                    slot.qty -= drop_qty;
                                    if slot.qty == 0 {
                                        inv.slots.remove(self.inv_selected);
                                        if self.inv_selected > 0 {
                                            self.inv_selected -= 1;
                                        }
                                    }
                                }
                            }
                        }
                        // Spawn DroppedItem entity with SpriteRef
                        let _ = self.game.world.spawn((
                            Position {
                                x: px,
                                y: py,
                                z: pz,
                            },
                            DroppedItem {
                                kind,
                                qty: drop_qty,
                            },
                            SpriteRef::new("items", itemkind_sprite_name(kind)),
                        ));
                        self.game
                            .res
                            .log(format!("Dropped 1 {}", itemkind_name(kind)));
                    }
                }
                return Ok(());
            }

            // Duplicate selected item (cheat)
            if self
                .keybinds
                .matches("inventory", "CHEAT_DUPLICATE_ITEM", &key)
            {
                if let Some(e) = self.game.res.player_entity {
                    if let Ok(mut inv) = self.game.world.get::<&mut InvComp>(e) {
                        if !inv.slots.is_empty() {
                            let idx = self.inv_selected.min(inv.slots.len() - 1);
                            let kind = inv.slots[idx].kind;
                            inv.slots[idx].qty = inv.slots[idx].qty.saturating_add(1);
                            self.game
                                .res
                                .log(format!("Duplicated 1 {}", itemkind_name(kind)));
                        }
                    }
                }
                return Ok(());
            }

            // Destroy selected item (remove 1)
            if self.keybinds.matches("inventory", "DESTROY_ITEM", &key) {
                if let Some(e) = self.game.res.player_entity {
                    if let Ok(mut inv) = self.game.world.get::<&mut InvComp>(e) {
                        if !inv.slots.is_empty() {
                            let idx = self.inv_selected.min(inv.slots.len() - 1);
                            let kind = inv.slots[idx].kind;
                            if inv.slots[idx].qty > 0 {
                                inv.slots[idx].qty -= 1;
                                if inv.slots[idx].qty == 0 {
                                    inv.slots.remove(idx);
                                    if self.inv_selected > 0 {
                                        self.inv_selected -= 1;
                                    }
                                }
                                self.game
                                    .res
                                    .log(format!("Destroyed 1 {}", itemkind_name(kind)));
                            }
                        }
                    }
                }
                return Ok(());
            }
        }

        if self.keybinds.matches_movement(&key) {
            if self.keybinds.matches("movement", "MOVE_NORTH", &key) {
                self.game.queue_player_move(0, -1);
            }
            if self.keybinds.matches("movement", "MOVE_SOUTH", &key) {
                self.game.queue_player_move(0, 1);
            }
            if self.keybinds.matches("movement", "MOVE_WEST", &key) {
                self.game.queue_player_move(-1, 0);
            }
            if self.keybinds.matches("movement", "MOVE_EAST", &key) {
                self.game.queue_player_move(1, 0);
            }
            if self.keybinds.matches("movement", "MOVE_NORTHWEST", &key) {
                self.game.queue_player_move(-1, -1);
            }
            if self.keybinds.matches("movement", "MOVE_NORTHEAST", &key) {
                self.game.queue_player_move(1, -1);
            }
            if self.keybinds.matches("movement", "MOVE_SOUTHWEST", &key) {
                self.game.queue_player_move(-1, 1);
            }
            if self.keybinds.matches("movement", "MOVE_SOUTHEAST", &key) {
                self.game.queue_player_move(1, 1);
            }
            if self.keybinds.matches("movement", "WAIT", &key) {
                self.game.queue_player_move(0, 0);
            }
            if self.keybinds.matches("movement", "MOVE_UP", &key) {
                self.game.queue_player_move_z(1);
            }
            if self.keybinds.matches("movement", "MOVE_DOWN", &key) {
                self.game.queue_player_move_z(-1);
            }

            let tick_result = self.game.tick();
            if tick_result.contains(GameTickResult::CombatTriggered) {
                self.combat_happening = true;
                panic!("todo show combat panel...");
            }
            self.snap_view_to_player_z();
            return Ok(());
        }

        if self.keybinds.matches("action", "MINE", &key) {
            self.game.queue_mine();
            let tick_result = self.game.tick();
            if tick_result.contains(GameTickResult::MiningSuccess) {
                self.snap_view_to_player_z();
            }
            return Ok(());
        }

        if self.keybinds.matches("scale", "SCALE_UP", &key) {
            self.scale = match self.scale {
                Scale::Small => Scale::Medium,
                Scale::Medium => Scale::Large,
                Scale::Large => Scale::Large,
            };
            return Ok(());
        }
        if self.keybinds.matches("scale", "SCALE_DOWN", &key) {
            self.scale = match self.scale {
                Scale::Large => Scale::Medium,
                Scale::Medium => Scale::Small,
                Scale::Small => Scale::Small,
            };
            return Ok(());
        }
        if self.keybinds.matches("scale", "SCALE_RESET", &key) {
            self.scale = Scale::Small;
            return Ok(());
        }
        // UI: Quit
        if self.keybinds.matches("ui", "QUIT", &key) {
            self.game.res.log("Quit requested (keybind)");
            tracing::info!(target: "game", "quit_requested tick={}", self.game.res.gametick);
            self.should_quit = true;
            return Ok(());
        }
        // UI: Menu activation and paging
        if self.keybinds.matches("ui", "MENU_ACTIVATE", &key) {
            self.activate_menu();
            return Ok(());
        }
        if self.keybinds.matches("ui", "MENU_PREV", &key) {
            self.current_tab = self.current_tab.prev();
            return Ok(());
        }
        if self.keybinds.matches("ui", "MENU_NEXT", &key) {
            self.current_tab = self.current_tab.next();
            return Ok(());
        }
        // Credits scroll
        if self.current_tab == MenuTab::Credits {
            if self.keybinds.matches("ui", "CREDITS_SCROLL_UP", &key) {
                self.credits_scroll = self.credits_scroll.saturating_sub(1);
                return Ok(());
            }
            if self.keybinds.matches("ui", "CREDITS_SCROLL_DOWN", &key) {
                self.credits_scroll = self.credits_scroll.saturating_add(1);
                return Ok(());
            }
        }
        // Help scroll
        if self.current_tab == MenuTab::Help {
            match key {
                KeyCode::Up => {
                    self.help_scroll = self.help_scroll.saturating_sub(1);
                    return Ok(());
                }
                KeyCode::Down => {
                    self.help_scroll = self.help_scroll.saturating_add(1);
                    return Ok(());
                }
                _ => {}
            }
        }
        // View Z slice up/down
        if self.keybinds.matches("viewport", "VIEW_Z_UP", &key) {
            if self.current_tab == MenuTab::Credits {
                self.credits_scroll = self.credits_scroll.saturating_sub(10);
            } else {
                self.game.res.view_z = self.game.res.view_z.saturating_add(1);
            }
            return Ok(());
        }
        if self.keybinds.matches("viewport", "VIEW_Z_DOWN", &key) {
            if self.current_tab == MenuTab::Credits {
                self.credits_scroll = self.credits_scroll.saturating_add(10);
            } else {
                self.game.res.view_z = self.game.res.view_z.saturating_sub(1);
            }
            return Ok(());
        }
        // Save/Load via config
        if self.keybinds.matches("ui", "SAVE_JSON", &key) {
            tracing::info!(target: "game", "save_begin path=save.json tick={}", self.game.res.gametick);
            self.game
                .save_json("save.json")
                .map_err(|e| format!("save_json error: {:?}", e))?;
            self.game.res.log("Saved to save.json");
            tracing::info!(target: "game", "save_end path=save.json tick={}", self.game.res.gametick);
            return Ok(());
        }
        if self.keybinds.matches("ui", "LOAD_JSON", &key) {
            tracing::info!(target: "game", "load_begin path=save.json tick={}", self.game.res.gametick);
            self.game
                .load_json("save.json")
                .map_err(|e| format!("load_json error: {:?}", e))?;
            self.game.res.log("Loaded from save.json");
            tracing::info!(target: "game", "load_end path=save.json tick={}", self.game.res.gametick);
            return Ok(());
        }

        Ok(())
    }

    fn activate_menu(&mut self) {
        match self.current_tab {
            MenuTab::World => {
                // World (already active view)
                self.game.res.log("World map active");
            }
            MenuTab::Body => {
                // Body
                self.game.res.log("Body panel active");
            }
            MenuTab::Inventory => {
                // Inventory (placeholder)
                self.game.res.log("Inventory panel (WIP)");
            }
            MenuTab::Menu => {
                // Menu
                self.game
                    .res
                    .log("Menu panel active (press S to Save, L to Load)");
            }
            MenuTab::Help => {
                // Help
                self.game.res.log("Help panel active");
            }
            MenuTab::Credits => {
                // Credits
                self.game
                    .res
                    .log("Credits panel active (Up/Down to scroll)");
            }
            MenuTab::Quit => {
                // Quit
                self.game.res.log("Quit requested (menu)");
                tracing::info!(target: "game", "quit_requested input=menu tick={}", self.game.res.gametick);
                self.should_quit = true;
            }
            MenuTab::Crafting => {
                // Crafting
                self.game.res.log("Crafting panel active");
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Load configuration to determine logging and seed before starting app
    let cm = ConfigManager::new();
    let log_level = cm
        .get_setting("game", "LOGGINGLEVEL")
        .and_then(|v| v.as_str())
        .unwrap_or("INFO");
    let seed_val: u64 = cm
        .get_setting("game", "DEFAULT_SEED")
        .and_then(|v| v.as_u64())
        .unwrap_or(12345);

    // Initialize tracing to write logs to LithicRivers.log (rotated daily)
    {
        let file_appender = _tracing_appender_hidden::rolling::daily(".", "LithicRivers.log");
        let (non_blocking, _guard) = _tracing_appender_hidden::non_blocking(file_appender);
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(log_level.to_lowercase()));
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_writer(non_blocking)
            .with_ansi(false)
            .init();
        // Keep _guard alive for program lifetime to ensure logs flush properly
        let _ = Box::leak(Box::new(_guard));
    }

    // Ensure we restore the terminal if a panic occurs
    std::panic::set_hook(Box::new(|info| {
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen, DisableMouseCapture);
        eprintln!("\n\nPanic: {info}");
    }));

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let mut app = App::new_with_seed(seed_val);
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<(), Box<dyn Error>> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if crossterm::event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Press {
                        app.handle_input(key.code)?;
                    }
                }
                Event::Mouse(me) => {
                    if let crossterm::event::MouseEventKind::ScrollDown = me.kind {
                        // Scroll down (move view down, which means increase scroll position)
                        if let SplashState::BootMessage = app.splash_state {
                            let total_lines = app.boot_display_text.lines().count() as u16;
                            let visible_lines = 20; // Approximate visible lines
                            if app.boot_scroll + visible_lines < total_lines {
                                app.boot_scroll = app.boot_scroll.saturating_add(3);
                                // Scroll 3 lines at a time
                            }
                        }
                    } else if let crossterm::event::MouseEventKind::ScrollUp = me.kind {
                        // Scroll up (move view up, which means decrease scroll position)
                        if let SplashState::BootMessage = app.splash_state {
                            app.boot_scroll = app.boot_scroll.saturating_sub(3);
                            // Scroll 3 lines at a time
                        }
                    } else {
                        // Log other mouse events
                        tracing::debug!("UNHANDLED Mouse event: {:?}", me);
                    }
                }
                _ => {}
            }
        }

        app.on_tick();

        if app.should_quit {
            app.game.res.log("Shutting down...");
            tracing::info!(
                target: "game",
                "shutdown tick={} view_z={} seed={}",
                app.game.res.gametick,
                app.game.res.view_z,
                app.game.res.seed
            );
            return Ok(());
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    // Show splash screens if needed
    match app.splash_state {
        SplashState::Logo => {
            // Show centered logo
            let logo_paragraph = Paragraph::new(app.logo_text.as_str())
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::NONE));

            // Center the logo in the middle of the screen
            let area = centered_rect(50, 50, f.size());
            f.render_widget(logo_paragraph, area);
            return;
        }
        SplashState::GameTitle => {
            // Show centered game title
            let title_paragraph = Paragraph::new(app.game_title_text.as_str())
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::NONE));

            // Center the title in the middle of the screen
            let area = centered_rect(70, 70, f.size());
            f.render_widget(title_paragraph, area);
            return;
        }
        SplashState::BootMessage => {
            // Show boot message with glitch effect
            let block = Block::default()
                .borders(Borders::ALL)
                .border_set(border::THICK)
                .title(" System Boot ")
                .title_alignment(Alignment::Center)
                .border_style(Style::default().fg(Color::LightBlue));

            // Inner and centered content area with padding
            let inner = block.inner(Rect::new(0, 0, f.size().width, f.size().height));
            let area = centered_rect(80, 60, inner);

            // Clear content area before drawing
            f.render_widget(Clear, area);

            // Calculate available height for text (accounting for borders and padding)
            let inner_area = area.inner(&Margin {
                horizontal: 2,
                vertical: 2,
            });
            let text_area = inner_area.inner(&Margin {
                horizontal: 1,
                vertical: 1,
            });

            // Calculate visible lines and update scroll position if needed
            let visible_lines = text_area.height.saturating_sub(2); // Leave room for border
            let total_lines = app.boot_display_text.lines().count() as u16;

            // Auto-scroll if we're at the bottom
            if app.boot_scroll + visible_lines >= total_lines.saturating_sub(1) {
                app.boot_scroll = total_lines.saturating_sub(visible_lines);
            }

            // Create a scrollable paragraph
            let paragraph = Paragraph::new(app.boot_display_text.as_str())
                .block(Block::default().borders(Borders::NONE))
                .wrap(Wrap { trim: false })
                .scroll((app.boot_scroll, 0));

            // Render content with padding
            f.render_widget(paragraph, inner_area);

            // Add scrollbar if needed
            if total_lines > visible_lines {
                // Create a simple scrollbar on the right
                let scrollbar_area = Rect {
                    x: inner_area.right() - 1,
                    y: inner_area.y,
                    width: 1,
                    height: inner_area.height,
                };

                // Calculate scrollbar thumb position and height
                let thumb_height = (visible_lines as f32 / total_lines as f32
                    * visible_lines as f32)
                    .max(1.0) as u16;
                let thumb_position = (app.boot_scroll as f32 / (total_lines - visible_lines) as f32
                    * (visible_lines - thumb_height) as f32)
                    .round() as u16;

                // Draw scrollbar track
                let track_span = Span::styled("│", Style::default().fg(Color::DarkGray));
                for y in inner_area.top()..inner_area.bottom() {
                    f.render_widget(
                        Paragraph::new(track_span.clone()),
                        Rect::new(scrollbar_area.x, y, 1, 1),
                    );
                }

                // Draw scrollbar thumb
                let thumb_span = Span::styled("▐", Style::default().fg(Color::LightBlue));
                for y in 0..thumb_height {
                    let y_pos = inner_area.y + thumb_position + y;
                    if y_pos < inner_area.bottom() {
                        f.render_widget(
                            Paragraph::new(thumb_span.clone()),
                            Rect::new(scrollbar_area.x, y_pos, 1, 1),
                        );
                    }
                }
            }

            // Render border last to ensure it's on top
            f.render_widget(block, area);
            return;
        }
        SplashState::MainUI => {
            // Normal UI rendering continues below
        }
    }

    // Only render the main UI if we're past all splash screens
    if app.splash_state != SplashState::MainUI {
        return;
    }

    let root_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Length(1), // Title line
            Constraint::Min(0),    // Main area (map + inventory)
            Constraint::Length(5), // Message log
            Constraint::Length(3), // Bottom menu bar (needs 3 for borders + content)
        ])
        .split(f.size());

    // Title
    let title = Paragraph::new("LithicRivers (Ratatui Client)")
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center);
    f.render_widget(title, root_chunks[0]);

    // Main area depends on selected tab
    match app.current_tab {
        MenuTab::World => {
            if app.look_mode {
                // With Look mode: show look panel on the right
                let main_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Min(20), Constraint::Length(24)])
                    .split(root_chunks[1]);
                render_game_view(f, app, main_chunks[0]);
                render_look_panel(f, app, main_chunks[1]);
            } else {
                // Show inventory list as sidebar, but hide Item detail panel
                let main_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Min(20), Constraint::Length(24)])
                    .split(root_chunks[1]);
                render_game_view(f, app, main_chunks[0]);
                render_inventory_list_only(f, app, main_chunks[1]);
            }
        }
        MenuTab::Crafting => {
            // Crafting panel
            render_crafting_panel(f, app, root_chunks[1]);
        }
        MenuTab::Body => {
            // Body panel
            render_body_panel(f, app, root_chunks[1]);
        }
        MenuTab::Inventory => {
            // Inventory panel
            render_inventory_panel(f, app, root_chunks[1]);
        }
        MenuTab::Menu => {
            // Menu panel
            render_menu_panel(f, app, root_chunks[1]);
        }
        MenuTab::Help => {
            // Help: controls
            render_help_panel(f, app, root_chunks[1]);
        }
        MenuTab::Credits => {
            // Credits: scrollable
            render_credits_panel(f, app, root_chunks[1]);
        }
        MenuTab::Quit => {
            // Quit: show diamond pattern panel
            render_quit_panel(f, app, root_chunks[1]);
        }
    }

    // Message log
    render_message_log(f, app, root_chunks[2]);

    // Bottom menu bar
    render_bottom_menu(f, app, root_chunks[3]);
}

fn render_game_view(f: &mut Frame, app: &mut App, area: Rect) {
    // Get the game view from the core
    let _view = app.game.build_view();

    // Create the game display text
    let mut lines = Vec::new();

    // Render the map to exactly the panel's area
    let target_cols = area.width as usize;
    let target_rows = area.height as usize;

    // Sync world generation Z with current view slice
    app.game.res.world.set_generation_z(app.game.res.view_z);

    let scale = app.scale.as_u32() as i32;

    // Compute world-space bounds for current viewport and prefetch chunks
    let center_x = app.game.res.view_x;
    let center_y = app.game.res.view_y;

    let world_cols = (target_cols as f32 / scale as f32).ceil() as i32;
    let world_rows = (target_rows as f32 / scale as f32).ceil() as i32;

    let left = center_x - world_cols / 2;
    let top = center_y - world_rows / 2;
    let right = left + world_cols;
    let bottom = top + world_rows;

    app.game
        .res
        .world
        .prefetch_rect(left, top, right, bottom, app.game.res.view_z);

    // Build an entity overlay map for current bounds and Z slice using SpriteRef
    let mut ent_overlay: std::collections::HashMap<(i32, i32), (String, String)> =
        std::collections::HashMap::new();
    let z = app.game.res.view_z;
    for (_e, (pos, sr_opt)) in app
        .game
        .world
        .query::<(
            &lithicrivers_core::components::Position,
            Option<&lithicrivers_core::components::SpriteRef>,
        )>()
        .iter()
    {
        if pos.z != z {
            continue;
        }
        if pos.x >= left && pos.x <= right && pos.y >= top && pos.y <= bottom {
            if let Some(sr) = sr_opt {
                ent_overlay.insert((pos.x, pos.y), (sr.category.clone(), sr.name.clone()));
            }
        }
    }

    for row in 0..target_rows {
        let mut spans = Vec::with_capacity(target_cols);
        for col in 0..target_cols {
            let offset_x = col as i32 - target_cols as i32 / 2;
            let offset_y = row as i32 - target_rows as i32 / 2;
            let world_x = center_x + offset_x.div_euclid(scale);
            let world_y = center_y + offset_y.div_euclid(scale);
            let world_z = app.game.res.view_z;

            let sprite_x = (col as i32 - target_cols as i32 / 2).rem_euclid(scale);
            let sprite_y = (row as i32 - target_rows as i32 / 2).rem_euclid(scale);

            // Base tile color/glyph
            let tile_kind = app
                .game
                .res
                .world
                .get_tile_cached(world_x, world_y, world_z);

            // render look mode cursor first
            if app.look_mode
                && world_x == app.look_cursor.x
                && world_y == app.look_cursor.y
                && app.game.res.view_z == app.look_cursor.z
            {
                let reticle_sprites = sprite_for_view_reticle();
                let scale_index = (app.scale.as_u32() - 1) as usize;
                let reticle_block = reticle_sprites
                    .get(scale_index)
                    .unwrap_or(&reticle_sprites[0]);

                let reticle_char = reticle_block
                    .lines()
                    .nth(sprite_y as usize)
                    .and_then(|line| line.chars().nth(sprite_x as usize))
                    .unwrap_or(' ');

                spans.push(Span::styled(
                    reticle_char.to_string(),
                    Style::default().fg(sprite_for_view_reticle_color()),
                ));
                continue;
            }

            // Entities next
            if let Some((cat, name)) = ent_overlay.get(&(world_x, world_y)) {
                let sr = lithicrivers_core::components::SpriteRef {
                    category: cat.clone(),
                    name: name.clone(),
                };
                let (block, color) =
                    sprite_block_for_spriteref(&mut app.sprite_loader, &sr, app.scale);
                let sprite_char = block
                    .lines()
                    .nth(sprite_y as usize)
                    .and_then(|line| line.chars().nth(sprite_x as usize))
                    .unwrap_or(' ');
                spans.push(Span::styled(
                    sprite_char.to_string(),
                    Style::default().fg(color),
                ));
            } else {
                let (block, color) =
                    sprite_block_for_tile(&mut app.sprite_loader, tile_kind, app.scale)
                        .unwrap_or_else(|| {
                            panic!("Could not find sprite for tile kind: {:?}", tile_kind)
                        });
                let sprite_char = block
                    .lines()
                    .nth(sprite_y as usize)
                    .and_then(|line| line.chars().nth(sprite_x as usize))
                    .unwrap_or(' ');
                spans.push(Span::styled(
                    sprite_char.to_string(),
                    Style::default().fg(color),
                ));
            }
        }
        lines.push(Line::from(spans));
    }

    let paragraph = Paragraph::new(lines)
        .block(Block::default())
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

fn render_message_log(f: &mut Frame, app: &mut App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();
    // Account for the border (top+bottom) since Paragraph has a Block
    let visible_rows = area.height.saturating_sub(2) as usize;
    let start = if app.game.res.messages.len() > visible_rows {
        app.game.res.messages.len() - visible_rows
    } else {
        0
    };
    for msg in app.game.res.messages.iter().skip(start) {
        lines.push(Line::from(Span::raw(msg.clone())));
    }
    let paragraph = Paragraph::new(lines)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Messages")
                .style(Style::default().fg(Color::White)),
        );
    f.render_widget(paragraph, area);
}

fn render_bottom_menu(f: &mut Frame, app: &mut App, area: Rect) {
    // Remember for click handling
    app.bottom_menu_rect = Some(area);
    // All tabs white; selected tab green
    let titles = vec![
        Span::raw("World"),
        Span::raw("Body"),
        Span::raw("Inventory"),
        Span::raw("Crafting"),
        Span::raw("Menu"),
        Span::raw("Help"),
        Span::raw("Credits"),
        Span::raw("Quit"),
    ];
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Menu"))
        .select(app.current_tab.as_index())
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Green));
    f.render_widget(tabs, area);
}

fn render_menu_panel(f: &mut Frame, app: &mut App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(Span::styled(
        "Game Menu",
        Style::default().fg(Color::Cyan),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::raw(
        "Use configured keybinds to Save/Load/Quit (see config).",
    )));
    lines.push(Line::from(""));
    // Config source
    lines.push(Line::from(Span::styled(
        "Configuration:",
        Style::default().fg(Color::Cyan),
    )));
    lines.push(Line::from(Span::raw(format!(
        "  {}",
        app.game.res.config.source_label()
    ))));
    lines.push(Line::from(Span::raw(format!(
        "  Player: {}",
        app.game.res.player_name
    ))));
    lines.push(Line::from(Span::raw(format!(
        "  Developer mode: {}",
        if app.game.res.developer_mode {
            "ON"
        } else {
            "OFF"
        }
    ))));
    lines.push(Line::from(""));
    // Logging location information
    lines.push(Line::from(Span::raw("Logging:")));
    lines.push(Line::from(Span::raw(format!(
        "  Path: {}",
        app.log_full_path
    ))));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::raw(format!(
        "Seed: {}",
        app.game.res.seed
    ))));
    lines.push(Line::from(Span::raw(format!(
        "Tick: {}",
        app.game.res.gametick
    ))));
    lines.push(Line::from(Span::raw(format!(
        "View Z: {}",
        app.game.res.view_z
    ))));
    if let Some(e) = app.game.res.player_entity {
        if let Ok(pos) = app
            .game
            .world
            .get::<&lithicrivers_core::components::Position>(e)
        {
            lines.push(Line::from(Span::raw(format!(
                "Player: ({}, {}, {})",
                pos.x, pos.y, pos.z
            ))));
        }
    }
    let block = Block::default().borders(Borders::ALL).title("Menu");
    let inner = block.inner(area);
    let p = Paragraph::new(lines).alignment(Alignment::Left);
    f.render_widget(p, inner);
    f.render_widget(block, area);
}

// Helper function to get player's inventory as a HashMap

fn render_look_panel(f: &mut Frame, app: &mut App, area: Rect) {
    let pos = app.look_cursor;

    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(Span::raw(format!(
        "Pos: ({}, {}, {})",
        pos.x, pos.y, pos.z
    ))));

    lines.push(Line::from(Span::raw(format!(
        "Chunk: ({}, {}, {})",
        pos.x / CHUNK_SIZE,
        pos.y / CHUNK_SIZE,
        pos.z
    ))));

    lines.push(Line::from(Span::raw(format!(""))));

    // Entity and item info
    let mut any_entity = false;
    for (_e, (e_pos, maybe_player, maybe_sr, maybe_drop)) in app
        .game
        .world
        .query::<(
            &lithicrivers_core::components::Position,
            Option<&lithicrivers_core::components::Player>,
            Option<&lithicrivers_core::components::SpriteRef>,
            Option<&lithicrivers_core::components::DroppedItem>,
        )>()
        .iter()
    {
        if *e_pos == pos {
            any_entity = true;
            if maybe_player.is_some() {
                lines.push(Line::from(Span::styled(
                    "Entity: Player",
                    Style::default().fg(Color::Green),
                )));
            } else if let Some(sr) = maybe_sr {
                lines.push(Line::from(Span::raw(format!(
                    "Entity: {}::{}",
                    sr.category, sr.name
                ))));
            } else if let Some(di) = maybe_drop {
                lines.push(Line::from(Span::raw(format!(
                    "Dropped: {:?} x{}",
                    di.kind, di.qty
                ))));
            } else {
                lines.push(Line::from(Span::raw("Entity: (unknown)")));
            }
        }
    }
    if !any_entity {
        lines.push(Line::from(Span::raw("Entities: (none)")));
    }

    lines.push(Line::from("Entity art:"));
    // Gather 12x8 art for entity under cursor (if any)
    if entity_art_12x8_lines_for_position(app, pos, &mut lines) {
        lines.push(Line::from(""));
    } else {
        lines.extend(empty_art_12x8_lines_for_position());
        lines.push(Line::from(""));
    }

    // Tile info
    let tile_kind = app.game.res.world.get_tile(pos.x, pos.y, pos.z);
    lines.push(Line::from(Span::raw(format!("Tile: {:?}", tile_kind))));

    lines.push(Line::from("Tile art:"));
    if block_art_12x8_lines_for_position(app, pos, &mut lines) {
        lines.push(Line::from(""));
    } else {
        lines.extend(empty_art_12x8_lines_for_position());
        lines.push(Line::from(""));
    }

    let content = Paragraph::new(lines)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Look at")
                .style(Style::default().fg(Color::White)),
        );
    f.render_widget(content, area);
}

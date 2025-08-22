use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs, Wrap},
    Frame, Terminal,
};
use rust_embed::RustEmbed;
use std::{error::Error, io, time::Duration};
use tracing_subscriber::EnvFilter;

// Tracing file appender for log file output
use chrono::Local;
use tracing_appender as _tracing_appender_hidden; // avoid "unused extern crate" lint

#[derive(RustEmbed)]
#[folder = "assets/"]
struct EmbeddedAssets;

use lithicrivers_core::config::ConfigManager;
use lithicrivers_core::Game;
use std::collections::HashMap;
mod audio;
mod rendering_helpers;
mod sprite_constants;
mod sprite_loader;
use crate::rendering_helpers::{
    block_art_12x8_lines_for_position, build_body_ascii, empty_art_12x8_lines_for_position,
    entity_art_12x8_lines_for_position, parse_hex_color,
};
use crate::sprite_constants::{sprite_for_view_reticle, sprite_for_view_reticle_color};
use crate::sprite_loader::{
    sprite_block_for_spriteref, sprite_block_for_tile, Scale, SpriteLoader,
};
use lithicrivers_core::components::{
    DroppedItem, Inventory as InvComp, ItemKind, Position, SpriteRef,
};
use lithicrivers_core::model::body::{Body, BodyPart, BodyPartState, BodyPartType};
use lithicrivers_core::resources::world::CHUNK_SIZE;

struct App {
    game: Game,
    sprite_loader: SpriteLoader,
    should_quit: bool,
    // UI state: remember bottom menu rect for click handling
    bottom_menu_rect: Option<Rect>,
    menu_index: usize,
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

    fn parse_keycode(s: &str) -> Option<KeyCode> {
        // Single character mappings
        if s.len() == 1 {
            let ch = s.chars().next().unwrap();
            return Some(KeyCode::Char(ch));
        }
        match s {
            // Common named keys
            "ESCAPE" => Some(KeyCode::Esc),
            "ENTER" => Some(KeyCode::Enter),
            "SPACE" => Some(KeyCode::Char(' ')),
            "TAB" => Some(KeyCode::Tab),
            "BACKSPACE" => Some(KeyCode::Backspace),

            // Arrows and paging
            "UP" => Some(KeyCode::Up),
            "DOWN" => Some(KeyCode::Down),
            "LEFT" => Some(KeyCode::Left),
            "RIGHT" => Some(KeyCode::Right),
            "PAGEUP" | "PAGE_UP" => Some(KeyCode::PageUp),
            "PAGEDOWN" | "PAGE_DOWN" => Some(KeyCode::PageDown),

            // Numpad (map to equivalent characters)
            "NUMPAD_1" => Some(KeyCode::Char('1')),
            "NUMPAD_2" => Some(KeyCode::Char('2')),
            "NUMPAD_3" => Some(KeyCode::Char('3')),
            "NUMPAD_4" => Some(KeyCode::Char('4')),
            "NUMPAD_5" => Some(KeyCode::Char('5')),
            "NUMPAD_6" => Some(KeyCode::Char('6')),
            "NUMPAD_7" => Some(KeyCode::Char('7')),
            "NUMPAD_8" => Some(KeyCode::Char('8')),
            "NUMPAD_9" => Some(KeyCode::Char('9')),

            _ => None,
        }
    }
}

fn render_help_panel(f: &mut Frame, _app: &mut App, area: Rect) {
    let mut lines: Vec<Line<'static>> = Vec::new();
    lines.push(Line::from(Span::styled(
        "Controls",
        Style::default().fg(Color::Cyan),
    )));
    lines.push(Line::from(""));
    // Movement
    lines.push(Line::from(Span::raw("Movement (Numpad):")));
    lines.push(Line::from(Span::raw("  7 8 9  - diagonals/cardinals")));
    lines.push(Line::from(Span::raw("  4 5 6  - 5 to wait")));
    lines.push(Line::from(Span::raw("  1 2 3")));
    lines.push(Line::from(""));
    // Vertical
    lines.push(Line::from(Span::raw("Vertical movement:")));
    lines.push(Line::from(Span::raw("  <  - move up a Z-level")));
    lines.push(Line::from(Span::raw("  >  - move down a Z-level")));
    lines.push(Line::from(Span::raw(
        "  PageUp/PageDown - change viewed Z slice",
    )));
    lines.push(Line::from(""));
    // Actions
    lines.push(Line::from(Span::raw("Actions:")));
    lines.push(Line::from(Span::raw("  m  - mine (plays SFX on success)")));
    lines.push(Line::from(Span::raw("  l  - toggle Look mode")));
    lines.push(Line::from(""));
    // Zoom
    lines.push(Line::from(Span::raw("Zoom:")));
    lines.push(Line::from(Span::raw("  =/+ - zoom in")));
    lines.push(Line::from(Span::raw("  -   - zoom out")));
    lines.push(Line::from(Span::raw("  0   - reset zoom")));
    lines.push(Line::from(""));
    // Save/Load
    lines.push(Line::from(Span::raw("Save/Load:")));
    lines.push(Line::from(Span::raw("  S - save to save.json")));
    lines.push(Line::from(Span::raw("  L - load from save.json")));
    lines.push(Line::from(""));
    // Menu
    lines.push(Line::from(Span::raw("Menu navigation:")));
    lines.push(Line::from(Span::raw(
        "  Left/Right - switch tabs (World, Body, Inventory, Menu, Help, Quit)",
    )));
    lines.push(Line::from(Span::raw(
        "  Enter/Space - activate selected tab",
    )));
    lines.push(Line::from(Span::raw("  Mouse - click tab labels")));
    lines.push(Line::from(""));
    // Quit
    lines.push(Line::from(Span::raw("General:")));
    lines.push(Line::from(Span::raw("  q - quit")));

    let block = Block::default().borders(Borders::ALL).title("Help");
    let inner = block.inner(area);
    let p = Paragraph::new(lines).alignment(Alignment::Left);
    f.render_widget(p, inner);
    f.render_widget(block, area);
}

fn render_credits_panel(f: &mut Frame, app: &mut App, area: Rect) {
    // Build a scrollable paragraph from preloaded embedded text
    let block = Block::default().borders(Borders::ALL).title("Credits");
    let inner = block.inner(area);
    let para = Paragraph::new(app.credits_text.clone())
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false })
        .scroll((app.credits_scroll, 0));
    f.render_widget(para, inner);
    f.render_widget(block, area);
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
            .unwrap_or_else(|| "(missing credits.txt)".to_string());
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
        let log_full_path = format!("{}/LithicRivers.log.{}", log_dir_abs, today);

        // Build keybinds from game config
        let keybinds = Keybinds::from_config(&game.res.config);

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

        App {
            game,
            sprite_loader,
            should_quit: false,
            bottom_menu_rect: None,
            audio,
            menu_index: 0,
            scale: Scale::Small,
            credits_text,
            credits_scroll: 0,
            log_full_path,
            keybinds,
            look_mode: false,
            look_cursor,
            inv_selected: 0,
        }
    }
    fn new() -> App {
        Self::new_with_seed(12345) //TODO use seed from config...
    }

    fn on_tick(&mut self) {
        // Turn-based: do not auto-tick. Ticks only occur on player actions in handle_input().
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
        // log key to log
        // tracing::info!(target: "game", "key pressed: {:?}", key);

        // First, handle configurable keybind actions
        // Toggle Look mode
        if self.keybinds.matches("ui", "LOOK_TOGGLE", &key) {
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
        if self.menu_index == 2
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

        // Inventory panel-specific navigation and actions
        if self.menu_index == 2 {
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
                            SpriteRef::new("items", item_sprite_name(kind)),
                        ));
                        self.game.res.log(format!("Dropped 1 {}", kind_name(kind)));
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
                                .log(format!("Duplicated 1 {}", kind_name(kind)));
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
                                    .log(format!("Destroyed 1 {}", kind_name(kind)));
                            }
                        }
                    }
                }
                return Ok(());
            }
        }

        if self.keybinds.matches("movement", "MOVE_NORTH", &key) {
            self.game.queue_player_move(0, -1);
            self.game.tick();
            self.snap_view_to_player_z();
            return Ok(());
        }
        if self.keybinds.matches("movement", "MOVE_SOUTH", &key) {
            self.game.queue_player_move(0, 1);
            self.game.tick();
            self.snap_view_to_player_z();
            return Ok(());
        }
        if self.keybinds.matches("movement", "MOVE_WEST", &key) {
            self.game.queue_player_move(-1, 0);
            self.game.tick();
            self.snap_view_to_player_z();
            return Ok(());
        }
        if self.keybinds.matches("movement", "MOVE_EAST", &key) {
            self.game.queue_player_move(1, 0);
            self.game.tick();
            self.snap_view_to_player_z();
            return Ok(());
        }
        if self.keybinds.matches("movement", "MOVE_NORTHWEST", &key) {
            self.game.queue_player_move(-1, -1);
            self.game.tick();
            self.snap_view_to_player_z();
            return Ok(());
        }
        if self.keybinds.matches("movement", "MOVE_NORTHEAST", &key) {
            self.game.queue_player_move(1, -1);
            self.game.tick();
            self.snap_view_to_player_z();
            return Ok(());
        }
        if self.keybinds.matches("movement", "MOVE_SOUTHWEST", &key) {
            self.game.queue_player_move(-1, 1);
            self.game.tick();
            self.snap_view_to_player_z();
            return Ok(());
        }
        if self.keybinds.matches("movement", "MOVE_SOUTHEAST", &key) {
            self.game.queue_player_move(1, 1);
            self.game.tick();
            self.snap_view_to_player_z();
            return Ok(());
        }
        if self.keybinds.matches("movement", "WAIT", &key) {
            self.game.queue_player_move(0, 0);
            self.game.tick();
            return Ok(());
        }
        if self.keybinds.matches("movement", "MOVE_UP", &key) {
            self.game.queue_player_move_z(1);
            self.game.tick();
            self.snap_view_to_player_z();
            return Ok(());
        }
        if self.keybinds.matches("movement", "MOVE_DOWN", &key) {
            self.game.queue_player_move_z(-1);
            self.game.tick();
            self.snap_view_to_player_z();
            return Ok(());
        }
        if self.keybinds.matches("action", "MINE", &key) {
            self.game.queue_mine();
            let mining_success = self.game.tick();
            if mining_success {
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
            if self.menu_index == 0 {
                self.menu_index = 6;
            } else {
                self.menu_index -= 1;
            }
            return Ok(());
        }
        if self.keybinds.matches("ui", "MENU_NEXT", &key) {
            self.menu_index = (self.menu_index + 1) % 7;
            return Ok(());
        }
        // Credits scroll
        if self.menu_index == 5 {
            if self.keybinds.matches("ui", "CREDITS_SCROLL_UP", &key) {
                self.credits_scroll = self.credits_scroll.saturating_sub(1);
                return Ok(());
            }
            if self.keybinds.matches("ui", "CREDITS_SCROLL_DOWN", &key) {
                self.credits_scroll = self.credits_scroll.saturating_add(1);
                return Ok(());
            }
        }
        // View Z slice up/down
        if self.keybinds.matches("viewport", "VIEW_Z_UP", &key) {
            if self.menu_index == 5 {
                self.credits_scroll = self.credits_scroll.saturating_sub(10);
            } else {
                self.game.res.view_z = self.game.res.view_z.saturating_add(1);
            }
            return Ok(());
        }
        if self.keybinds.matches("viewport", "VIEW_Z_DOWN", &key) {
            if self.menu_index == 5 {
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
        match self.menu_index {
            0 => {
                // World (already active view)
                self.game.res.log("World map active");
            }
            1 => {
                // Body
                self.game.res.log("Body panel active");
            }
            2 => {
                // Inventory (placeholder)
                self.game.res.log("Inventory panel (WIP)");
            }
            3 => {
                // Menu
                self.game
                    .res
                    .log("Menu panel active (press S to Save, L to Load)");
            }
            4 => {
                // Help
                self.game.res.log("Help panel active");
            }
            5 => {
                // Credits
                self.game
                    .res
                    .log("Credits panel active (Up/Down to scroll)");
            }
            _ => {
                // Quit
                self.game.res.log("Quit requested (menu)");
                tracing::info!(target: "game", "quit_requested input=menu tick={}", self.game.res.gametick);
                self.should_quit = true;
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
                    // debug log that we don't handle mouse events.
                    // TODO we can handle these later
                    tracing::debug!("UNHANDLED Mouse event: {:?}", me);
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
    if app.menu_index == 0 {
        // World: map with inventory sidebar
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(20), Constraint::Length(24)])
            .split(root_chunks[1]);
        render_game_view(f, app, main_chunks[0]);
        if app.look_mode {
            render_look_panel(f, app, main_chunks[1]);
        } else {
            render_inventory_panel(f, app, main_chunks[1]);
        }
    } else if app.menu_index == 1 {
        // Body: fullscreen body panel
        render_body_panel(f, app, root_chunks[1]);
    } else if app.menu_index == 2 {
        // Inventory: fullscreen inventory panel
        render_inventory_panel(f, app, root_chunks[1]);
    } else if app.menu_index == 3 {
        // Menu: save/load panel
        render_menu_panel(f, app, root_chunks[1]);
    } else if app.menu_index == 4 {
        // Help: controls
        render_help_panel(f, app, root_chunks[1]);
    } else if app.menu_index == 5 {
        // Credits: scrollable
        render_credits_panel(f, app, root_chunks[1]);
    } else {
        // Quit selected: do nothing special here; run loop will exit
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

    // Compute world-space bounds for current viewport and prefetch chunks
    // Center the viewport on the tracked viewport center, not always the player
    let center_x = app.game.res.view_x;
    let center_y = app.game.res.view_y;
    let left = center_x - (target_cols as i32 / 2);
    let top = center_y - (target_rows as i32 / 2);
    let right = left + target_cols as i32 - 1;
    let bottom = top + target_rows as i32 - 1;
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
            let world_x = left + col as i32;
            let world_y = top + row as i32;
            let world_z = app.game.res.view_z;

            // Base tile color/glyph
            let tile_kind = app
                .game
                .res
                .world
                .get_tile_cached(world_x, world_y, world_z);

            // render look mode cursor first
            //TODO make this blink and cycle through overlapping tiles/entities
            if app.look_mode
                && world_x == app.look_cursor.x
                && world_y == app.look_cursor.y
                && app.game.res.view_z == app.look_cursor.z
            {
                spans.push(Span::styled(
                    sprite_for_view_reticle(),
                    Style::default().fg(sprite_for_view_reticle_color()),
                ));
                continue;
            }

            // Fluids removed: fall through to entities/tiles
            // Entities next
            if let Some((cat, name)) = ent_overlay.get(&(world_x, world_y)) {
                let sr = lithicrivers_core::components::SpriteRef {
                    category: cat.clone(),
                    name: name.clone(),
                };
                let (block, color) =
                    sprite_block_for_spriteref(&mut app.sprite_loader, &sr, app.scale);
                let ech = block.chars().next().unwrap_or(' ');
                spans.push(Span::styled(ech.to_string(), Style::default().fg(color)));
            } else {
                let (block, color) =
                    sprite_block_for_tile(&mut app.sprite_loader, tile_kind, app.scale)
                        .unwrap_or_else(|| {
                            panic!("Could not find sprite for tile kind: {:?}", tile_kind)
                        });
                let ch = block.chars().next().unwrap_or(' ');
                spans.push(Span::styled(ch.to_string(), Style::default().fg(color)));
            }
        }
        lines.push(Line::from(spans));
    }

    let paragraph = Paragraph::new(lines)
        .block(Block::default())
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

fn render_inventory_panel(f: &mut Frame, app: &mut App, area: Rect) {
    // Split: left art/details, right list
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(28), Constraint::Min(20)])
        .split(area);

    // Build list with selection highlight
    let mut list_lines: Vec<Line<'static>> = Vec::new();
    let mut selected_kind: Option<ItemKind> = None;
    let mut selected_qty: u32 = 0;
    if let Some(e) = app.game.res.player_entity {
        if let Ok(inv) = app.game.world.get::<&InvComp>(e) {
            if inv.slots.is_empty() {
                list_lines.push(Line::from(Span::raw("(Empty)")));
                app.inv_selected = 0;
            } else {
                if app.inv_selected >= inv.slots.len() {
                    app.inv_selected = inv.slots.len() - 1;
                }
                for (i, s) in inv.slots.iter().enumerate() {
                    let label = format!("{} x{}", kind_name(s.kind), s.qty);
                    if i == app.inv_selected {
                        selected_kind = Some(s.kind);
                        selected_qty = s.qty;
                        list_lines.push(Line::from(Span::styled(
                            label,
                            Style::default().fg(Color::Yellow),
                        )));
                    } else {
                        list_lines.push(Line::from(Span::raw(label)));
                    }
                }
            }
        } else {
            list_lines.push(Line::from(Span::raw("(No Inventory component)")));
        }
    } else {
        list_lines.push(Line::from(Span::raw("(No player)")));
    }

    // Left: art + description if any selection
    let left_block = Block::default().borders(Borders::ALL).title("Item");
    let left_inner = left_block.inner(chunks[0]);
    let mut left_lines: Vec<Line<'static>> = Vec::new();
    if let Some(kind) = selected_kind {
        let (category, sprite_name) = ("items", item_sprite_name(kind));
        let sd = app.sprite_loader.load_sprite(sprite_name, category);
        // Art
        if let Some(block) = sd.art12x8_sprites.first() {
            let color = parse_hex_color(&sd.color);
            for row in block.split('\n') {
                let sp = match color {
                    Some(c) => Span::styled(row.to_string(), Style::default().fg(c)),
                    None => Span::raw(row.to_string()),
                };
                left_lines.push(Line::from(sp));
            }
        }
        // Spacer and description
        left_lines.push(Line::from(""));
        left_lines.push(Line::from(Span::styled(
            format!("{} (x{})", kind_name(kind), selected_qty),
            Style::default().fg(Color::Cyan),
        )));
        left_lines.push(Line::from(""));
        left_lines.push(Line::from(Span::raw(sd.description.clone())));
        left_lines.push(Line::from(""));
        left_lines.push(Line::from(Span::raw(
            "Keys: d=drop, .=duplicate, x=destroy",
        )));
    } else {
        left_lines.push(Line::from(Span::raw("Select an item")));
    }
    let left_para = Paragraph::new(left_lines).alignment(Alignment::Left);
    f.render_widget(left_para, left_inner);
    f.render_widget(left_block, chunks[0]);

    // Right: list panel
    let list_para = Paragraph::new(list_lines)
        .alignment(Alignment::Left)
        .block(Block::default().borders(Borders::ALL).title("Inventory"));
    f.render_widget(list_para, chunks[1]);
}

fn render_message_log(f: &mut Frame, app: &mut App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();
    let start = if app.game.res.messages.len() > area.height as usize {
        app.game.res.messages.len() - area.height as usize
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
        Span::raw("Menu"),
        Span::raw("Help"),
        Span::raw("Credits"),
        Span::raw("Quit"),
    ];
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Menu"))
        .select(app.menu_index)
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

fn render_body_panel(f: &mut Frame, app: &mut App, area: Rect) {
    // Split area: left ASCII overview, right list
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(28), Constraint::Min(20)])
        .split(area);

    // Build from ECS
    let mut list_lines: Vec<Line> = Vec::new();
    let mut ascii_lines_opt: Option<Vec<Line<'static>>> = None;
    if let Some(e) = app.game.res.player_entity {
        if let Ok(body) = app.game.world.get::<&Body>(e) {
            // Build list
            let mut parts: Vec<&BodyPart> = body.parts.values().collect();
            parts.sort_by_key(|p| p.part_type as i32);
            for p in parts {
                let (label, color) = match p.state {
                    BodyPartState::Missing => ("Missing", Color::DarkGray),
                    BodyPartState::Damaged => ("Damaged", Color::Yellow),
                    BodyPartState::Functional => ("Functional", Color::Green),
                    BodyPartState::Enhanced => ("Enhanced", Color::Cyan),
                };
                list_lines.push(Line::from(Span::styled(
                    format!("{:>9}: {}", p.name, label),
                    Style::default().fg(color),
                )));
            }
            // Build ASCII from borrowed body
            ascii_lines_opt = Some(build_body_ascii(&*body));
        }
    }
    if list_lines.is_empty() {
        list_lines.push(Line::from(Span::raw("(No Body data)")));
    }

    // Left: ASCII overview
    let ascii_block = Block::default().borders(Borders::ALL).title("Body");
    let ascii_inner = ascii_block.inner(chunks[0]);
    let ascii_lines = ascii_lines_opt.unwrap_or_else(|| vec![Line::from(Span::raw("(No Body)"))]);
    let ascii_para = Paragraph::new(ascii_lines).alignment(Alignment::Left);
    f.render_widget(ascii_para, ascii_inner);
    f.render_widget(ascii_block, chunks[0]);

    // Right: textual list
    let list_para = Paragraph::new(list_lines)
        .alignment(Alignment::Left)
        .block(Block::default().borders(Borders::ALL).title("Parts"));
    f.render_widget(list_para, chunks[1]);
}

fn kind_name(kind: ItemKind) -> &'static str {
    match kind {
        ItemKind::Wood => "Wood",
        ItemKind::Acorn => "Acorn",
        ItemKind::Stick => "Stick",
    }
}

fn item_sprite_name(kind: ItemKind) -> &'static str {
    match kind {
        ItemKind::Wood => "log",
        ItemKind::Acorn => "acorn",
        ItemKind::Stick => "stick",
    }
}

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

use crate::{audio, MenuTab, Scale, SplashState, SpriteLoader};
use crossterm::event::KeyCode;
use lithicrivers_core::components::Position;
use lithicrivers_core::config::ConfigManager;
use lithicrivers_core::recipe_handler::RecipeHandler;
use lithicrivers_core::Game;
use ratatui::layout::Rect;
use std::collections::HashMap;
use std::time::Instant;

/// Core game engine state - the fundamental game systems
pub struct CoreState {
    pub game: Game,
    pub config_manager: ConfigManager,
    pub sprite_loader: SpriteLoader,
    pub should_quit: bool,
}

/// UI framework state - controls overall interface behavior
pub struct UiState {
    pub current_tab: MenuTab,
    pub scale: Scale,
    pub bottom_menu_rect: Option<Rect>, // Remember for click handling
    pub keybinds: Keybinds,
}

/// Audio system state
pub struct AudioState {
    #[allow(dead_code)]
    pub audio: audio::AudioManager,
}

/// Logging configuration
pub struct LoggingState {
    pub log_full_path: String,
}

/// Client-side combat UI state
#[derive(Debug, Clone, PartialEq)]
pub enum CombatUiState {
    None,
    Active {
        current_move: usize,
        current_enemy: usize,
        enemy_timers: Vec<u32>, // Deprecated - will be removed once UI fully migrated to ActionQueue
    },
}

impl CombatUiState {
    pub fn is_active(&self) -> bool {
        matches!(self, CombatUiState::Active { .. })
    }
}

impl Default for CombatUiState {
    fn default() -> Self {
        CombatUiState::None
    }
}

/// All UI panel-specific states grouped by functionality
pub struct PanelStates {
    pub inventory: InventoryPanelState,
    pub crafting: CraftingPanelState,
    pub credits: CreditsPanelState,
    pub help: HelpPanelState,
    pub look: LookPanelState,
}

/// Inventory panel state
pub struct InventoryPanelState {
    pub selected: usize,
}

/// Crafting panel state
pub struct CraftingPanelState {
    pub recipe_handler: RecipeHandler,
    pub selected: usize,
    pub message: Option<(String, u8)>, // (message, timer)
}

/// Credits panel state
pub struct CreditsPanelState {
    pub text: String,
    pub scroll: u16,
}

/// Help panel state
pub struct HelpPanelState {
    pub scroll: u16,
}

/// Look mode panel state
pub struct LookPanelState {
    pub mode: bool,
    pub cursor: Position,
}

/// Splash screen system state - handles startup sequence
pub struct SplashScreenState {
    pub state: SplashState,
    pub start_time: Option<Instant>,
    pub logo_text: String,
    pub game_title_text: String,
    #[allow(dead_code)]
    pub boot_message: String,
    pub boot_message_lines: Vec<String>,
    pub boot_display_text: String,
    pub boot_line_index: usize,
    pub boot_scroll: u16,
    pub last_line_time: Instant,
    pub boot_complete: bool,
}

/// Keybind configuration and matching
#[derive(Debug, Clone)]
pub struct Keybinds {
    // key: "category:ACTION" => list of KeyCodes
    map: HashMap<String, Vec<KeyCode>>,
}

impl Keybinds {
    pub fn from_config(cfg: &ConfigManager) -> Self {
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

    pub fn matches(&self, category: &str, action: &str, key: &KeyCode) -> bool {
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

    pub fn matches_movement(&self, key: &KeyCode) -> bool {
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

    pub fn get_map(&self) -> &HashMap<String, Vec<KeyCode>> {
        &self.map
    }

    fn parse_keycode(s: &str) -> Option<KeyCode> {
        lithicrivers_core::keycode_mapping::parse_keycode(s)
    }
}

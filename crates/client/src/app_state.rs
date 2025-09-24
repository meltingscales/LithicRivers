use crate::dialogue_engine::{ConversationState, DialogueEngine};
use crate::{audio, MenuTab, Scale, SplashState, SpriteLoader};
use crossterm::event::KeyCode;
use lithicrivers_core::components::{ItemKind, Position};
use lithicrivers_core::config::ConfigManager;
use lithicrivers_core::recipe_handler::{RecipeHandler, RepairRecipeHandler};
use lithicrivers_core::Game;
use ratatui::layout::Rect;
use std::collections::HashMap;
use std::time::Instant;

/// Comprehensive dialogue system types from demo integration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum DialogueType {
    Linear,    // Simple linear conversation
    Branching, // Player choices affect dialogue
    Shop,      // Trading interface with dialogue
    Quest,     // Quest giving with conditions
    Battle,    // Pre/post battle dialogue
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NPCMood {
    Friendly,
    Neutral,
    Hostile,
    Sad,
    Excited,
    Mysterious,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DialogueChoice {
    pub text: String,
    pub leads_to: Option<usize>, // Index of next dialogue node, None = end conversation
    pub requires_item: Option<String>,
    pub mood_change: Option<NPCMood>,
    pub unlocks_quest: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DialogueNode {
    pub id: usize,
    pub speaker: String,
    pub text: String,
    pub mood: NPCMood,
    pub choices: Vec<DialogueChoice>,
    pub auto_continue: bool, // If true, automatically continues without player input
    pub shop_item: Option<String>, // If set, this node offers to sell/trade this item
}

#[derive(Debug, Clone, PartialEq)]
pub struct NPCData {
    pub name: String,
    pub portrait: String, // ASCII art portrait
    pub dialogue_type: DialogueType,
    pub current_mood: NPCMood,
    pub initial_dialogue: usize, // Starting dialogue node ID
    pub met_before: bool,
    pub has_quest: bool,
    pub shop_inventory: Vec<String>,
}

/// Core game engine state - the fundamental game systems
pub struct CoreState {
    pub game: Game,
    pub config_manager: ConfigManager,
    pub sprite_loader: SpriteLoader,
    #[allow(dead_code)]
    pub repair_handler: RepairRecipeHandler,
    pub should_quit: bool,
}

/// UI framework state - controls overall interface behavior
pub struct UiState {
    pub current_tab: MenuTab,
    pub scale: Scale,
    pub bottom_menu_rect: Option<Rect>, // Remember for click handling
    pub keybinds: Keybinds,
    // UI-owned viewport state - what the player is currently viewing
    pub view_x: i64,
    pub view_y: i64,
    pub view_z: i64,
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
        move_scroll_offset: usize, // For scrolling through many moves
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
    pub build: BuildPanelState,
    pub corpse_looting: CorpseLootingState,
    pub hotbar_assignment: HotbarAssignmentState,
    pub npc_interaction: NPCInteractionState,
    pub multi_action_select: MultiActionSelectState,
    pub dialogue_engine: DialogueEngine,
    #[allow(dead_code)]
    pub body_repair: BodyRepairState,
    pub cheat_console: CheatConsoleState,
    pub global_map: GlobalMapPanelState,
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

/// Build mode states: Movement -> Break -> Place
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildMode {
    Movement,
    Break,
    Place,
}

impl BuildMode {
    pub fn next(self) -> Self {
        match self {
            BuildMode::Movement => BuildMode::Break,
            BuildMode::Break => BuildMode::Place,
            BuildMode::Place => BuildMode::Movement,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            BuildMode::Movement => "Movement",
            BuildMode::Break => "Break",
            BuildMode::Place => "Place",
        }
    }
}

/// Build mode panel state
pub struct BuildPanelState {
    pub mode: BuildMode,
    pub selected_hotbar_slot: usize,
    /// Assigned blocks for each hotbar slot (None = empty slot)
    pub hotbar_assignments: [Option<ItemKind>; 12],
    /// Track last F-key press for double-tap detection
    pub last_fkey_press: Option<(usize, Instant)>,
}

impl Default for BuildPanelState {
    fn default() -> Self {
        Self {
            mode: BuildMode::Movement,
            selected_hotbar_slot: 0,
            hotbar_assignments: [None; 12],
            last_fkey_press: None,
        }
    }
}

/// Corpse looting interaction state
#[derive(Debug, Clone, PartialEq)]
pub enum CorpseLootingState {
    None,
    SelectingCorpse {
        adjacent_entities: Vec<hecs::Entity>, // List of adjacent corpse entities
        selected_corpse: usize,
    },
    LootingCorpse {
        entity: hecs::Entity, // Entity being looted
        selected_loot_item: usize,
        selected_player_item: usize,
        loot_panel_focus: bool, // true = corpse inventory, false = player inventory
    },
}

impl Default for CorpseLootingState {
    fn default() -> Self {
        CorpseLootingState::None
    }
}

/// Hotbar assignment interaction state
#[derive(Debug, Clone, PartialEq)]
pub enum HotbarAssignmentState {
    None,
    ChoosingBlock {
        hotbar_slot: usize,              // Which F-key slot we're assigning to
        available_blocks: Vec<ItemKind>, // Blocks available in inventory
        selected_block: usize,           // Currently selected block in the list
    },
}

impl Default for HotbarAssignmentState {
    fn default() -> Self {
        HotbarAssignmentState::None
    }
}

/// Body repair modal state
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum BodyRepairState {
    None,
    SelectingRepairAndPart {
        available_repairs: Vec<usize>, // Indices into repair recipe handler
        selected_repair: usize,
        selected_body_part: Option<lithicrivers_core::model::body::BodyPartType>,
    },
}

impl Default for BodyRepairState {
    fn default() -> Self {
        BodyRepairState::None
    }
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

/// Different types of interactions available
#[derive(Debug, Clone, PartialEq)]
pub enum InteractionType {
    PickupItem {
        entity: hecs::Entity,
        item_name: String,
        position: Position,
    },
    LootCorpse {
        entity: hecs::Entity,
        position: Position,
    },
    TalkToNPC {
        entity: hecs::Entity,
        npc_name: String,
        position: Position,
    },
    OpenCloseDoor {
        position: Position,
        is_open: bool,
    },
}

impl InteractionType {
    pub fn display_name_with_context(
        &self,
        world: &hecs::World,
        player_pos: Option<Position>,
    ) -> String {
        match self {
            InteractionType::PickupItem { item_name, .. } => format!("Pick up {}", item_name),
            InteractionType::LootCorpse { entity, .. } => {
                // Try to get inventory to count items
                if let Ok(inventory) =
                    world.get::<&lithicrivers_core::components::Inventory>(*entity)
                {
                    let item_count = inventory.slots.len();
                    format!("Loot corpse ({} items)", item_count)
                } else {
                    "Loot corpse".to_string()
                }
            }
            InteractionType::TalkToNPC { npc_name, .. } => format!("Talk to {}", npc_name),
            InteractionType::OpenCloseDoor { position, is_open } => {
                let direction_text = if let Some(player_position) = player_pos {
                    let dx = position.x - player_position.x;
                    let dy = position.y - player_position.y;

                    let direction = match (dx.signum(), dy.signum()) {
                        (0, -1) => "N",   // North
                        (1, -1) => "NE",  // Northeast
                        (1, 0) => "E",    // East
                        (1, 1) => "SE",   // Southeast
                        (0, 1) => "S",    // South
                        (-1, 1) => "SW",  // Southwest
                        (-1, 0) => "W",   // West
                        (-1, -1) => "NW", // Northwest
                        _ => "",          // Same position (shouldn't happen)
                    };

                    if direction.is_empty() {
                        String::new()
                    } else {
                        format!(" ({})", direction)
                    }
                } else {
                    String::new()
                };

                if *is_open {
                    format!("Close door{}", direction_text)
                } else {
                    format!("Open door{}", direction_text)
                }
            }
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            InteractionType::PickupItem { .. } => "^", // Up arrow for pickup
            InteractionType::LootCorpse { .. } => "x", // x for corpse looting
            InteractionType::TalkToNPC { .. } => "t",  // t for talking
            InteractionType::OpenCloseDoor { is_open, .. } => {
                if *is_open {
                    "o"
                } else {
                    "#"
                }
            }
        }
    }
}

/// Multi-action selection state - handles choosing between multiple available interactions
#[derive(Debug, Clone, PartialEq)]
pub enum MultiActionSelectState {
    None,
    SelectingAction {
        available_actions: Vec<InteractionType>,
        selected_action: usize,
    },
}

impl Default for MultiActionSelectState {
    fn default() -> Self {
        MultiActionSelectState::None
    }
}

/// NPC interaction state - handles NPC selection and dialogue with new architecture
#[derive(Debug, Clone, PartialEq)]
pub enum NPCInteractionState {
    None,
    SelectingNPC {
        adjacent_npcs: Vec<(hecs::Entity, String)>, // (entity, name) pairs
        selected_npc: usize,
    },
    InDialogue {
        npc_entity: hecs::Entity,
        conversation: ConversationState,
        selected_choice: usize,
    },
}

impl Default for NPCInteractionState {
    fn default() -> Self {
        NPCInteractionState::None
    }
}

/// Cheat console state - handles cheat command input
#[derive(Debug, Clone, PartialEq)]
pub enum CheatConsoleState {
    None,
    Open {
        input: String,
        cursor_position: usize,
    },
}

impl Default for CheatConsoleState {
    fn default() -> Self {
        CheatConsoleState::None
    }
}

/// Global Map panel state - handles marker selection and arrow display
pub struct GlobalMapPanelState {
    pub selected_marker_index: usize, // Index of currently selected marker (for arrow display)
}

impl Default for GlobalMapPanelState {
    fn default() -> Self {
        GlobalMapPanelState {
            selected_marker_index: 0,
        }
    }
}

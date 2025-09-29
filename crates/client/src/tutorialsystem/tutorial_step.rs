use crossterm::event::KeyCode;
use lithicrivers_core::TileKind;

/// Represents a single tutorial step with instruction and required action
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TutorialStep {
    pub id: String,
    pub title: String,
    pub instruction: String,
    pub action: TutorialAction,
    pub highlight_area: Option<HighlightArea>,
    pub completed: bool,
}

/// Actions that the tutorial system can wait for
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum TutorialAction {
    /// Wait for a specific key press
    KeyPress(KeyCode),
    /// Wait for any key press
    AnyKey,
    /// Wait for a specific keybind to be pressed
    Keybind { category: String, action: String },
    /// Wait for a specific keybind to be pressed while the player is adjacent to a specific block
    KeybindAdjacentToBlock {
        category: String,
        action: String,
        block: TileKind,
    },
    /// Wait for multiple movement keys to be pressed (tracks which ones have been pressed)
    MovementKeys {
        required_keys: Vec<KeyCode>,
        pressed_keys: Vec<KeyCode>,
    },
    /// Wait for multiple keybinds to be pressed (tracks which ones have been pressed)
    MultipleKeybinds {
        required_keybinds: Vec<(String, String)>, // (category, action) pairs
        pressed_keybinds: Vec<(String, String)>,
    },
    /// Wait for menu to be opened
    OpenMenu,
    /// Wait for inventory to be opened  
    OpenInventory,
    /// Wait for a specific UI state change
    StateChange(String),
    /// Just display text (auto-advance after delay)
    Display { auto_advance_ms: Option<u64> },
}

/// Area of the screen to highlight for tutorial
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct HighlightArea {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub description: String,
}

#[allow(dead_code)]
impl TutorialStep {
    pub fn new(id: &str, title: &str, instruction: &str, action: TutorialAction) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            instruction: instruction.to_string(),
            action,
            highlight_area: None,
            completed: false,
        }
    }

    pub fn with_highlight(mut self, area: HighlightArea) -> Self {
        self.highlight_area = Some(area);
        self
    }

    pub fn is_action_satisfied(
        &mut self,
        key: &KeyCode,
        keybinds: &crate::app_state::Keybinds,
        world: Option<&lithicrivers_core::world_state::WorldState>,
        player_pos: Option<lithicrivers_core::components::Position>,
    ) -> bool {
        match &mut self.action {
            TutorialAction::KeyPress(expected_key) => key == expected_key,
            TutorialAction::AnyKey => true,
            TutorialAction::Keybind { category, action } => keybinds.matches(category, action, key),
            TutorialAction::KeybindAdjacentToBlock {
                category,
                action,
                block,
            } => {
                // First check if the keybind matches
                if !keybinds.matches(category, action, key) {
                    return false;
                }

                // Then check if player is adjacent to the specified block type
                if let (Some(world_state), Some(pos)) = (world, player_pos) {
                    // Check all 8 adjacent positions (N, S, E, W, NE, NW, SE, SW)
                    let adjacent_positions = [
                        (pos.x, pos.y - 1, pos.z),     // North
                        (pos.x, pos.y + 1, pos.z),     // South
                        (pos.x + 1, pos.y, pos.z),     // East
                        (pos.x - 1, pos.y, pos.z),     // West
                        (pos.x + 1, pos.y - 1, pos.z), // Northeast
                        (pos.x - 1, pos.y - 1, pos.z), // Northwest
                        (pos.x + 1, pos.y + 1, pos.z), // Southeast
                        (pos.x - 1, pos.y + 1, pos.z), // Southwest
                    ];

                    for (x, y, z) in adjacent_positions {
                        let tile = world_state.world.get_tile(x, y, z);
                        if tile == *block {
                            return true;
                        }
                    }
                    false
                } else {
                    // If we don't have world data, can't check adjacency
                    false
                }
            }
            TutorialAction::MovementKeys {
                required_keys,
                pressed_keys,
            } => {
                // Check if this key is one of the required movement keys
                if required_keys.contains(key) && !pressed_keys.contains(key) {
                    pressed_keys.push(*key);
                }
                // Return true if all required keys have been pressed
                pressed_keys.len() >= required_keys.len()
            }
            TutorialAction::MultipleKeybinds {
                required_keybinds,
                pressed_keybinds,
            } => {
                // Check if this key matches any of the required keybinds
                for (category, action) in required_keybinds.iter() {
                    if keybinds.matches(category, action, key) {
                        let keybind_pair = (category.clone(), action.clone());
                        if !pressed_keybinds.contains(&keybind_pair) {
                            pressed_keybinds.push(keybind_pair);
                        }
                        break;
                    }
                }
                // Return true if all required keybinds have been pressed
                pressed_keybinds.len() >= required_keybinds.len()
            }
            _ => false,
        }
    }
}

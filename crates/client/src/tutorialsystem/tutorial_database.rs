use lithicrivers_core::TileKind;

use super::{TutorialAction, TutorialStep, TutorialSystem};

/// Metadata about an available tutorial
#[derive(Debug, Clone)]
pub struct TutorialInfo {
    pub id: String,
    pub title: String,
    pub description: String,
    pub category: String,
}

/// Tutorial database containing all tutorial sequences and steps
pub struct TutorialDatabase;

impl TutorialDatabase {
    /// Get tutorials grouped by category in display order
    pub fn get_tutorials_by_category() -> Vec<(String, Vec<TutorialInfo>)> {
        let tutorials = Self::get_available_tutorials();
        let mut categories: std::collections::HashMap<String, Vec<TutorialInfo>> =
            std::collections::HashMap::new();

        for tutorial in tutorials {
            categories
                .entry(tutorial.category.clone())
                .or_default()
                .push(tutorial);
        }

        let category_order = vec!["Essential", "Gameplay", "Advanced"];
        let mut result = Vec::new();

        for category in category_order {
            if let Some(tutorials) = categories.remove(category) {
                if !tutorials.is_empty() {
                    result.push((category.to_string(), tutorials));
                }
            }
        }

        result
    }

    /// Get a list of all available tutorials
    pub fn get_available_tutorials() -> Vec<TutorialInfo> {
        vec![
            TutorialInfo {
                id: "basics".to_string(),
                title: "Movement and Navigation".to_string(),
                description: "Learn basic movement controls and UI navigation".to_string(),
                category: "Essential".to_string(),
            },
            TutorialInfo {
                id: "inventory".to_string(),
                title: "Inventory Management".to_string(),
                description: "Learn how to manage your items and equipment".to_string(),
                category: "Gameplay".to_string(),
            },
            TutorialInfo {
                id: "crafting".to_string(),
                title: "Crafting System".to_string(),
                description: "Learn how to craft items and tools".to_string(),
                category: "Gameplay".to_string(),
            },
            TutorialInfo {
                id: "combat".to_string(),
                title: "Combat Basics".to_string(),
                description: "Learn combat mechanics and strategies".to_string(),
                category: "Advanced".to_string(),
            },
            TutorialInfo {
                id: "building".to_string(),
                title: "Building and Mining".to_string(),
                description: "Learn how to build structures and mine resources".to_string(),
                category: "Advanced".to_string(),
            },
        ]
    }

    /// Create and initialize a tutorial system with the basics tutorial sequence
    pub fn create_tutorial_system_with_basics() -> TutorialSystem {
        let mut tutorial_system = TutorialSystem::new();

        let basics_steps = Self::get_basics_tutorial();
        tutorial_system.start_tutorial("basics", basics_steps);

        tutorial_system
    }

    /// Get the basics tutorial sequence
    pub fn get_basics_tutorial() -> Vec<TutorialStep> {
        vec![
            Self::create_tutorial_tutorial(),
            Self::create_movement_tutorial(),
            Self::create_zoom_tutorial(),
            Self::create_torch_tutorial(),
            Self::create_door_tutorial(),
            Self::create_look_tutorial(),
        ]
    }

    fn create_door_tutorial() -> TutorialStep {
        TutorialStep::new(
            "open_door",
            "Open the door!",
            "Press the key below to open/close doors. This key also is used to loot corpses and interact with other objects. You can also use it to talk to NPCs. By default, it starts an interaction, and only opens a menu if there are multiple options.",
            TutorialAction::KeybindAdjacentToBlock {
                category: "action".to_string(),
                action: "INTERACT".to_string(),
                block: TileKind::Door,
            },
        )
    }

    /// Create the torch tutorial step
    fn create_torch_tutorial() -> TutorialStep {
        TutorialStep::new(
            "light_it_up",
            "Light it up!",
            "Toggle your torch on and off. This will help you see in dark areas!",
            TutorialAction::Keybind {
                category: "action".to_string(),
                action: "TOGGLE_TORCH".to_string(),
            },
        )
    }

    /// Create the movement tutorial step
    fn create_tutorial_tutorial() -> TutorialStep {
        TutorialStep::new(
            "tutorial_tutorial",
            "Tutorial Tutorial",
            "Welcome to LithicRivers! This is an entirely ASCII-based and keyboard-focused game. This is the tutorial system. Below are some keybinds that control it. If you really don't like it, you can disable it (see below). For this first tutorial, just press the next/previous buttons that control the menu pages.",
            TutorialAction::MultipleKeybinds {
                required_keybinds: vec![
                    ("ui".to_string(), "MENU_PREV".to_string()),
                    ("ui".to_string(), "MENU_NEXT".to_string()),
                ],
                pressed_keybinds: vec![],
            },
        )
    }

    /// Create the movement tutorial step
    fn create_movement_tutorial() -> TutorialStep {
        TutorialStep::new(
            "learn_to_move",
            "Learn to Move",
            "Make sure NUMLOCK is on! Try moving in all 9 directions using the NUMPAD keys. Press each movement key at least once. Then press LEFT ARROW to complete this tutorial. NUMPAD controls both UI navigation and player movement. ARROW KEYS control page movement - LEFT ARROW closes this tutorial screen, and RIGHT ARROW revisits it later.",
            TutorialAction::MultipleKeybinds {
                required_keybinds: vec![
                    ("movement".to_string(), "MOVE_NORTHWEST".to_string()),
                    ("movement".to_string(), "MOVE_NORTH".to_string()),
                    ("movement".to_string(), "MOVE_NORTHEAST".to_string()),
                    ("movement".to_string(), "MOVE_WEST".to_string()),
                    ("movement".to_string(), "WAIT".to_string()),
                    ("movement".to_string(), "MOVE_EAST".to_string()),
                    ("movement".to_string(), "MOVE_SOUTHWEST".to_string()),
                    ("movement".to_string(), "MOVE_SOUTH".to_string()),
                    ("movement".to_string(), "MOVE_SOUTHEAST".to_string()),
                    ("ui".to_string(), "MENU_PREV".to_string()),
                ],
                pressed_keybinds: vec![],
            },
        )
    }

    /// create a zoom tutorial step
    fn create_zoom_tutorial() -> TutorialStep {
        TutorialStep::new(
            "zoom_in_out",
            "Zoom In/Out",
            "Zoom in and out to get a smaller or larger view of the world! This game has 3 scale modes: 1x1, 2x2, and 3x3.",
            TutorialAction::MultipleKeybinds {
                required_keybinds: vec![
                    ("scale".to_string(), "SCALE_UP".to_string()),
                    ("scale".to_string(), "SCALE_DOWN".to_string()),
                ],
                pressed_keybinds: vec![],
            },
        )
    }

    /// Create the look mode tutorial step
    fn create_look_tutorial() -> TutorialStep {
        TutorialStep::new(
            "look_at_this",
            "Look at this!",
            "Toggle LOOK mode. This lets you examine the world around you without moving! You can press it ONCE to turn it on, and a SECOND time to turn it back off. Note that you must close the tutorial panel to view the LOOK mode interface.",
            TutorialAction::Keybind {
                category: "action".to_string(),
                action: "LOOK_TOGGLE".to_string(),
            },
        )
    }
}

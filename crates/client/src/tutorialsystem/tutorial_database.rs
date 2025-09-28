use super::{TutorialAction, TutorialStep, TutorialSystem};

/// Tutorial database containing all tutorial sequences and steps
pub struct TutorialDatabase;

impl TutorialDatabase {
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
            Self::create_movement_tutorial(),
            Self::create_torch_tutorial(),
            Self::create_look_tutorial(),
        ]
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
    fn create_movement_tutorial() -> TutorialStep {
        use crossterm::event::KeyCode;

        TutorialStep::new(
            "learn_to_move",
            "Learn to Move",
            "Try moving in all 9 directions using the NUMPAD keys (7,8,9,4,5,6,1,2,3). Press each key at least once. NUMPAD controls both UI navigation and player movement. ARROW KEYS control page movement - press LEFT ARROW to close this tutorial screen, and RIGHT ARROW to revisit it later.",
            TutorialAction::MovementKeys {
                required_keys: vec![
                    KeyCode::Char('7'), KeyCode::Char('8'), KeyCode::Char('9'),
                    KeyCode::Char('4'), KeyCode::Char('5'), KeyCode::Char('6'),
                    KeyCode::Char('1'), KeyCode::Char('2'), KeyCode::Char('3'),
                ],
                pressed_keys: vec![],
            },
        )
    }

    /// Create the look mode tutorial step
    fn create_look_tutorial() -> TutorialStep {
        TutorialStep::new(
            "look_at_this",
            "Look at this!",
            "Toggle look mode. This lets you examine the world around you without moving!",
            TutorialAction::Keybind {
                category: "action".to_string(),
                action: "LOOK_TOGGLE".to_string(),
            },
        )
    }
}

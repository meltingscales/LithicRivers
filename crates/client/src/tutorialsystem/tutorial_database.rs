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
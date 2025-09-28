use crossterm::event::KeyCode;

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
        &self,
        key: &KeyCode,
        keybinds: &crate::app_state::Keybinds,
    ) -> bool {
        match &self.action {
            TutorialAction::KeyPress(expected_key) => key == expected_key,
            TutorialAction::AnyKey => true,
            TutorialAction::Keybind { category, action } => keybinds.matches(category, action, key),
            _ => false,
        }
    }
}

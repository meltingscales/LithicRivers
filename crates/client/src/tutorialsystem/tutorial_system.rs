use super::TutorialStep;
use crossterm::event::KeyCode;

/// Main tutorial system that manages tutorial progression and state
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TutorialSystem {
    pub enabled: bool,
    pub current_step_index: usize,
    pub steps: Vec<TutorialStep>,
    pub completed_tutorials: Vec<String>,
    pub show_panel: bool,
}

impl Default for TutorialSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl TutorialSystem {
    pub fn new() -> Self {
        Self {
            enabled: true, // Default enabled as requested
            current_step_index: 0,
            steps: Vec::new(),
            completed_tutorials: Vec::new(),
            show_panel: true, // Default panel visible
        }
    }

    /// Toggle tutorial system on/off
    pub fn toggle_enabled(&mut self) {
        self.enabled = !self.enabled;
    }

    /// Toggle tutorial panel visibility
    pub fn toggle_panel(&mut self) {
        self.show_panel = !self.show_panel;
    }

    /// Hide tutorial panel when menu is open (as requested)
    pub fn set_panel_visible(&mut self, visible: bool) {
        self.show_panel = visible;
    }

    /// Start a specific tutorial sequence
    pub fn start_tutorial(&mut self, _tutorial_id: &str, steps: Vec<TutorialStep>) {
        if !self.enabled {
            return;
        }

        self.steps = steps;
        self.current_step_index = 0;

        // Mark all steps as incomplete
        for step in &mut self.steps {
            step.completed = false;
        }
    }

    /// Get the current tutorial step
    pub fn current_step(&self) -> Option<&TutorialStep> {
        if !self.enabled || self.steps.is_empty() {
            return None;
        }
        self.steps.get(self.current_step_index)
    }

    /// Process a key input for tutorial progression
    pub fn handle_key_input(&mut self, key: KeyCode) -> bool {
        if !self.enabled || self.steps.is_empty() {
            return false;
        }

        if let Some(current_step) = self.steps.get_mut(self.current_step_index) {
            if current_step.is_action_satisfied(&key) {
                current_step.completed = true;
                self.advance_step();
                return true;
            }
        }
        false
    }

    /// Advance to the next tutorial step
    pub fn advance_step(&mut self) {
        if self.current_step_index + 1 < self.steps.len() {
            self.current_step_index += 1;
        } else {
            // Tutorial completed
            self.complete_current_tutorial();
        }
    }

    /// Complete the current tutorial
    pub fn complete_current_tutorial(&mut self) {
        if let Some(step) = self.current_step() {
            self.completed_tutorials.push(step.id.clone());
        }
        self.steps.clear();
        self.current_step_index = 0;
    }

    /// Skip current tutorial
    pub fn skip_tutorial(&mut self) {
        self.steps.clear();
        self.current_step_index = 0;
    }

    /// Check if tutorial system should be visible
    pub fn should_show(&self) -> bool {
        self.enabled && self.show_panel && !self.steps.is_empty()
    }

    /// Check if tutorial system has active content
    pub fn has_active_tutorial(&self) -> bool {
        self.enabled && !self.steps.is_empty()
    }

    /// Reset all tutorial progress
    pub fn reset_progress(&mut self) {
        self.completed_tutorials.clear();
        self.steps.clear();
        self.current_step_index = 0;
    }
}

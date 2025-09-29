use super::TutorialStep;
use crossterm::event::KeyCode;
use std::time::{Duration, Instant};

/// Represents an active UI highlight
#[derive(Debug, Clone)]
pub struct ActiveHighlight {
    pub element_id: String,
    pub start_time: Instant,
    pub duration: Duration,
}

impl ActiveHighlight {
    pub fn new(element_id: String, duration_secs: u64) -> Self {
        Self {
            element_id,
            start_time: Instant::now(),
            duration: Duration::from_secs(duration_secs),
        }
    }

    pub fn is_active(&self) -> bool {
        self.start_time.elapsed() < self.duration
    }

    pub fn flash_intensity(&self) -> f32 {
        let elapsed = self.start_time.elapsed().as_secs_f32();
        let cycle_time = 0.5; // Flash every 0.5 seconds
        ((elapsed / cycle_time).sin().abs() * 0.7) + 0.3 // Oscillate between 0.3 and 1.0
    }
}

/// Main tutorial system that manages tutorial progression and state
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TutorialSystem {
    pub enabled: bool,
    pub current_step_index: usize,
    pub steps: Vec<TutorialStep>,
    pub completed_tutorials: Vec<String>,
    pub show_panel: bool,
    pub active_highlights: Vec<ActiveHighlight>,
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
            active_highlights: Vec::new(),
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
    /// Returns true if the tutorial was advanced, but doesn't consume the key
    pub fn handle_key_input(
        &mut self,
        key: KeyCode,
        keybinds: &crate::app_state::Keybinds,
        world: Option<&lithicrivers_core::world_state::WorldState>,
        player_pos: Option<lithicrivers_core::components::Position>,
    ) -> bool {
        if !self.enabled || self.steps.is_empty() {
            return false;
        }

        if let Some(current_step) = self.steps.get_mut(self.current_step_index) {
            if current_step.is_action_satisfied(&key, keybinds, world, player_pos) {
                current_step.completed = true;
                self.advance_step();
                return true;
            }
        }
        false
    }

    /// Check inventory changes for tutorial progression
    /// Returns true if the tutorial was advanced
    pub fn handle_inventory_change(
        &mut self,
        inventory: &lithicrivers_core::components::Inventory,
    ) -> bool {
        if !self.enabled || self.steps.is_empty() {
            return false;
        }

        if let Some(current_step) = self.steps.get_mut(self.current_step_index) {
            if current_step.is_action_satisfied_by_inventory(inventory) {
                current_step.completed = true;
                self.advance_step();
                return true;
            }
        }
        false
    }

    /// Advance to the next tutorial step
    pub fn advance_step(&mut self) {
        // Check if we just completed the torch tutorial
        if let Some(completed_step) = self.steps.get(self.current_step_index) {
            if completed_step.id == "light_it_up" {
                // Add highlight for the torch mode indicator
                self.add_highlight("mode_indicator_torch".to_string(), 10);
            }
        }

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
        self.active_highlights.clear();
    }

    /// Add a highlight for a UI element
    pub fn add_highlight(&mut self, element_id: String, duration_secs: u64) {
        self.active_highlights
            .push(ActiveHighlight::new(element_id, duration_secs));
    }

    /// Update highlights and remove expired ones
    pub fn update_highlights(&mut self) {
        self.active_highlights
            .retain(|highlight| highlight.is_active());
    }

    /// Check if a specific UI element should be highlighted
    pub fn is_element_highlighted(&self, element_id: &str) -> Option<f32> {
        for highlight in &self.active_highlights {
            if highlight.element_id == element_id && highlight.is_active() {
                return Some(highlight.flash_intensity());
            }
        }
        None
    }
}

//! Player intent system - consolidated action processing
//!
//! Replaces the scattered movement intent fields with a single, clean structure

#[derive(Debug, Clone, Default)]
pub struct PlayerIntent {
    pub action: Option<PlayerAction>,
    pub tick_cost: u64,
}

#[derive(Debug, Clone)]
pub enum PlayerAction {
    /// Move in world space (dx, dy, dz)
    Move { dx: i32, dy: i32, dz: i32 },
    /// Mine the current tile
    Mine,
    /// Mine at specific coordinates
    MineAt { x: i32, y: i32, z: i32 },
    /// Interact with nearby objects (items, corpses, NPCs)
    Interact,
    /// Interact with specific target
    InteractWith { target_id: Option<usize> },
}

impl PlayerIntent {
    /// Create a movement intent
    pub fn movement(dx: i32, dy: i32, dz: i32, cost: u64) -> Self {
        Self {
            action: Some(PlayerAction::Move { dx, dy, dz }),
            tick_cost: cost,
        }
    }

    /// Create a mining intent
    pub fn mine(cost: u64) -> Self {
        Self {
            action: Some(PlayerAction::Mine),
            tick_cost: cost,
        }
    }

    /// Create a mining intent at specific coordinates
    pub fn mine_at(x: i32, y: i32, z: i32, cost: u64) -> Self {
        Self {
            action: Some(PlayerAction::MineAt { x, y, z }),
            tick_cost: cost,
        }
    }

    /// Create an interact intent
    pub fn interact(cost: u64) -> Self {
        Self {
            action: Some(PlayerAction::Interact),
            tick_cost: cost,
        }
    }

    /// Create an interact intent with specific target
    pub fn interact_with(target_id: Option<usize>, cost: u64) -> Self {
        Self {
            action: Some(PlayerAction::InteractWith { target_id }),
            tick_cost: cost,
        }
    }

    /// Take the current intent, leaving None
    pub fn take(&mut self) -> Option<PlayerAction> {
        let action = self.action.take();
        if action.is_some() {
            // Keep tick cost for processing, but clear action
            self.action = None;
        }
        action
    }

    /// Clear the intent completely
    pub fn clear(&mut self) {
        self.action = None;
        self.tick_cost = 0;
    }

    /// Check if there's a pending action
    pub fn has_action(&self) -> bool {
        self.action.is_some()
    }

    /// Get the tick cost (returns 0 if no action)
    pub fn cost(&self) -> u64 {
        if self.action.is_some() {
            self.tick_cost
        } else {
            0
        }
    }
}

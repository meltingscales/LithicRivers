use hecs::Entity;
use std::collections::{HashMap, HashSet};

use crate::moves::{CombatAction, QueuedAction};

/// Efficient targeting relationship tracker to optimize queue cleanup performance.
///
/// Instead of O(n²) cleanup where we scan all entities and their action queues,
/// we maintain a reverse lookup: target -> set of entities targeting it.
/// This reduces cleanup to O(1) lookup + O(k) cleanup where k = entities targeting the dead entity.
#[derive(Debug, Default)]
pub struct TargetTracker {
    /// Maps: target_entity -> set of entities that have actions targeting it
    /// When target_entity dies, we can quickly find all entities that need queue cleanup
    targeting_relationships: HashMap<Entity, HashSet<Entity>>,
}

impl TargetTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a targeting relationship: entity has an action targeting target
    pub fn add_targeting(&mut self, entity: Entity, target: Entity) {
        self.targeting_relationships
            .entry(target)
            .or_insert_with(HashSet::new)
            .insert(entity);
    }

    /// Remove a targeting relationship: entity no longer has actions targeting target
    pub fn remove_targeting(&mut self, entity: Entity, target: Entity) {
        if let Some(targeting_set) = self.targeting_relationships.get_mut(&target) {
            targeting_set.remove(&entity);
            // Clean up empty entries to prevent memory leaks
            if targeting_set.is_empty() {
                self.targeting_relationships.remove(&target);
            }
        }
    }

    /// Get all entities that have actions targeting the given target
    /// Returns an empty slice if no entities are targeting it
    pub fn get_entities_targeting(&self, target: Entity) -> Vec<Entity> {
        self.targeting_relationships
            .get(&target)
            .map(|set| set.iter().copied().collect())
            .unwrap_or_default()
    }

    /// Remove all targeting relationships for an entity (when entity dies or queue is cleared)
    pub fn clear_entity_targeting(&mut self, entity: Entity) {
        // Remove this entity from all targeting relationships
        self.targeting_relationships
            .retain(|_target, targeting_set| {
                targeting_set.remove(&entity);
                !targeting_set.is_empty()
            });
    }

    /// Update targeting relationships based on an action queue
    /// This should be called whenever an action queue is modified
    pub fn update_from_action_queue(&mut self, entity: Entity, actions: &[QueuedAction]) {
        // First, clear existing relationships for this entity
        self.clear_entity_targeting(entity);

        // Add new relationships based on current actions
        for action in actions {
            if let Some(target) = self.extract_target_from_action(action) {
                self.add_targeting(entity, target);
            }
        }
    }

    /// Update targeting relationships for a single action (when current action changes)
    pub fn update_from_current_action(&mut self, entity: Entity, action: Option<&QueuedAction>) {
        // Note: This only handles the current action, not queued actions
        // For a complete update, use update_from_action_queue instead
        if let Some(action) = action {
            if let Some(target) = self.extract_target_from_action(action) {
                self.add_targeting(entity, target);
            }
        }
    }

    /// Extract the target entity from a combat action, if any
    fn extract_target_from_action(&self, action: &QueuedAction) -> Option<Entity> {
        match &action.action {
            CombatAction::PlayerMove { target_entity, .. } => *target_entity,
            CombatAction::EnemyAttack { target_entity, .. } => Some(*target_entity),
        }
    }

    /// Get statistics about the tracker (for debugging/monitoring)
    pub fn get_stats(&self) -> TargetTrackerStats {
        let total_targets = self.targeting_relationships.len();
        let total_relationships: usize = self
            .targeting_relationships
            .values()
            .map(|set| set.len())
            .sum();
        let max_targeters = self
            .targeting_relationships
            .values()
            .map(|set| set.len())
            .max()
            .unwrap_or(0);

        TargetTrackerStats {
            total_targets,
            total_relationships,
            max_targeters,
        }
    }
}

/// Statistics about the target tracker performance
#[derive(Debug, Clone)]
pub struct TargetTrackerStats {
    /// Number of entities being targeted by at least one other entity
    pub total_targets: usize,
    /// Total number of targeting relationships tracked
    pub total_relationships: usize,
    /// Maximum number of entities targeting a single target
    pub max_targeters: usize,
}

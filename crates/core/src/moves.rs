use crate::components::Position;
use crate::model::body::{Body, BodyPartState, BodyPartType};
use hecs::Entity;
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MoveType {
    Melee,
    Tackle,
    Fireball,
    Escape,
    DebugInstantKill,
    Heal,
    Shield,
    LightningBolt,
    PowerStrike,
}

impl MoveType {
    pub fn human_name(&self) -> &'static str {
        match self {
            MoveType::Melee => "Melee",
            MoveType::Fireball => "Fireball",
            MoveType::Tackle => "Tackle",
            MoveType::Escape => "Escape",
            MoveType::DebugInstantKill => "Debug Instant Kill",
            MoveType::Heal => "Heal",
            MoveType::Shield => "Shield",
            MoveType::LightningBolt => "Lightning Bolt",
            MoveType::PowerStrike => "Power Strike",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Move {
    pub name: String,
    pub move_type: MoveType,
    pub energy_cost: u32,
    pub execution_time_ticks: u64,
    pub splash_radius: Option<i64>,
    pub damage: u32,
    pub description: String,
}

impl Move {
    pub fn melee() -> Self {
        Self {
            name: "Melee".to_string(),
            move_type: MoveType::Melee,
            energy_cost: 0,
            execution_time_ticks: 5,
            splash_radius: None,
            damage: 10,
            description: "Basic melee attack".to_string(),
        }
    }

    pub fn tackle() -> Self {
        Self {
            name: "Tackle".to_string(),
            move_type: MoveType::Tackle,
            energy_cost: 20,
            execution_time_ticks: 10,
            splash_radius: None,
            damage: 15,
            description: "Pushes enemy back 2 spaces, 50% chance to stun for 600 ticks".to_string(),
        }
    }

    pub fn fireball() -> Self {
        Self {
            name: "Fireball (AoE)".to_string(),
            move_type: MoveType::Fireball,
            energy_cost: 40,
            execution_time_ticks: 12,
            splash_radius: Some(1),
            damage: 30,
            description: "Area of effect fire damage".to_string(),
        }
    }

    pub fn escape() -> Self {
        Self {
            name: "Escape".to_string(),
            move_type: MoveType::Escape,
            energy_cost: 20,
            execution_time_ticks: 15,
            splash_radius: None,
            damage: 0,
            description: "Flee from combat, adds BattleDelay".to_string(),
        }
    }

    pub fn debuginstantkill() -> Self {
        Self {
            name: "DebugInstantKill".to_string(),
            move_type: MoveType::DebugInstantKill,
            energy_cost: 0,
            execution_time_ticks: 5,
            splash_radius: None,
            damage: 9999,
            description: "Debug instant kill".to_string(),
        }
    }

    pub fn heal() -> Self {
        Self {
            name: "Heal".to_string(),
            move_type: MoveType::Heal,
            energy_cost: 30,
            execution_time_ticks: 10,
            splash_radius: None,
            damage: 0,
            description: "Restore health".to_string(),
        }
    }

    pub fn shield() -> Self {
        Self {
            name: "Shield".to_string(),
            move_type: MoveType::Shield,
            energy_cost: 25,
            execution_time_ticks: 5,
            splash_radius: None,
            damage: 0,
            description: "Block incoming attacks".to_string(),
        }
    }

    pub fn lightning_bolt() -> Self {
        Self {
            name: "Lightning Bolt".to_string(),
            move_type: MoveType::LightningBolt,
            energy_cost: 50,
            execution_time_ticks: 10,
            splash_radius: None,
            damage: 40,
            description: "Fast electric attack".to_string(),
        }
    }

    pub fn power_strike() -> Self {
        Self {
            name: "Power Strike".to_string(),
            move_type: MoveType::PowerStrike,
            energy_cost: 40,
            execution_time_ticks: 13,
            splash_radius: None,
            damage: 50,
            description: "Powerful melee attack".to_string(),
        }
    }
}

/// Result of executing a move
#[derive(Debug, Clone)]
pub struct MoveResult {
    pub success: bool,
    pub message: String,
    pub effects: Vec<MoveEffect>,
}

/// Effects that can result from moves
#[derive(Debug, Clone)]
pub enum MoveEffect {
    Damage {
        target: Entity,
        amount: u32,
    },
    BodyDamage {
        target: Entity,
        part: BodyPartType,
    },
    Push {
        target: Entity,
        from: Position,
        to: Position,
    },
    Stun {
        target: Entity,
        ticks: u64,
    },
    PlayerMove {
        from: Position,
        to: Position,
    },
    BattleDelay {
        target: Entity,
        ticks: u64,
    },
    EndCombat,
}

pub fn get_available_moves() -> Vec<Move> {
    vec![
        Move::melee(),
        Move::fireball(),
        Move::tackle(),
        Move::escape(),
        Move::heal(),
        Move::shield(),
        Move::lightning_bolt(),
        Move::power_strike(),
        Move::debuginstantkill(),
    ]
}

/// Calculate distance for tackle push mechanics
pub fn calculate_push_position(from: Position, target: Position, push_distance: i64) -> Position {
    let dx = target.x - from.x;
    let dy = target.y - from.y;

    // Normalize direction (basic implementation)
    let (push_x, push_y) = if dx == 0 && dy == 0 {
        (0, 0) // No movement if same position
    } else if dx.abs() >= dy.abs() {
        (
            if dx > 0 {
                push_distance
            } else {
                -push_distance
            },
            0,
        )
    } else {
        (
            0,
            if dy > 0 {
                push_distance
            } else {
                -push_distance
            },
        )
    };

    Position {
        x: target.x + push_x,
        y: target.y + push_y,
        z: target.z, // Keep same Z level
    }
}

/// Calculate overall body integrity percentage for a robot player
pub fn calculate_body_integrity(body: &Body) -> f32 {
    let total_parts = body.parts.len() as f32;
    if total_parts == 0.0 {
        return 0.0;
    }

    let mut integrity_sum = 0.0;
    for part in body.parts.values() {
        let part_integrity = match part.state {
            BodyPartState::Missing => 0.0,
            BodyPartState::Damaged => 0.5,
            BodyPartState::Functional => 1.0,
            BodyPartState::Enhanced => 1.2, // Better than baseline
        };
        integrity_sum += part_integrity;
    }

    (integrity_sum / total_parts).min(1.0) // Cap at 100%
}

/// Queued action in the combat system
#[derive(Debug, Clone)]
pub struct QueuedAction {
    pub entity: Entity,
    pub action: CombatAction,
    pub execution_time_ticks: u64,
    pub remaining_time_ticks: u64,
}

/// Types of combat actions that can be queued
#[derive(Debug, Clone)]
pub enum CombatAction {
    PlayerMove {
        move_data: Move,
        target_entity: Option<Entity>,
        target_position: Option<Position>,
    },
    EnemyAttack {
        target_entity: Entity,
        damage: u32,
    },
}

/// Component for the action queue system - attached to combat entities
#[derive(Debug, Clone, Default)]
pub struct ActionQueue {
    pub actions: VecDeque<QueuedAction>,
    pub current_action: Option<QueuedAction>,
}

impl ActionQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn queue_action(&mut self, action: QueuedAction) {
        self.actions.push_back(action);
    }

    pub fn start_next_action(&mut self) -> Option<QueuedAction> {
        if self.current_action.is_none() {
            self.current_action = self.actions.pop_front();
        }
        self.current_action.clone()
    }

    pub fn update_timers(&mut self, delta_ticks: u64) -> Option<QueuedAction> {
        if let Some(ref mut current) = self.current_action {
            current.remaining_time_ticks = current.remaining_time_ticks.saturating_sub(delta_ticks);

            if current.remaining_time_ticks == 0 {
                let completed = self.current_action.take();
                // Start the next action automatically
                self.start_next_action();
                return completed;
            }
        }
        None
    }

    /// Advance timers without executing actions (Phase 1 of two-phase processing)
    pub fn advance_timers(&mut self, delta_ticks: u64) {
        // Start next action if none is current
        if self.current_action.is_none() {
            self.start_next_action();
        }

        // Update timer for current action
        if let Some(ref mut current) = self.current_action {
            current.remaining_time_ticks = current.remaining_time_ticks.saturating_sub(delta_ticks);
        }
    }

    /// Pop a completed action if one is ready (Phase 2 of two-phase processing)
    pub fn pop_completed_action(&mut self) -> Option<QueuedAction> {
        if let Some(ref current) = self.current_action {
            if current.remaining_time_ticks == 0 {
                let completed = self.current_action.take();
                // Start the next action for future ticks
                self.start_next_action();
                return completed;
            }
        }
        None
    }

    pub fn get_queued_actions(&self) -> &VecDeque<QueuedAction> {
        &self.actions
    }

    pub fn get_current_action(&self) -> &Option<QueuedAction> {
        &self.current_action
    }

    pub fn clear(&mut self) {
        self.actions.clear();
        self.current_action = None;
    }

    /// Remove all actions targeting a specific dead entity
    pub fn remove_actions_targeting(&mut self, dead_entity: Entity) {
        // Check current action
        if let Some(ref current) = self.current_action {
            let should_remove_current = match &current.action {
                CombatAction::PlayerMove { target_entity, .. } => {
                    target_entity.map_or(false, |target| target == dead_entity)
                }
                CombatAction::EnemyAttack { target_entity, .. } => *target_entity == dead_entity,
            };

            if should_remove_current {
                self.current_action = None;
                // Don't immediately start next action - let the normal flow handle it
                // This prevents starting another action that might also target the dead entity
            }
        }

        // Remove queued actions targeting the dead entity
        let original_len = self.actions.len();
        self.actions.retain(|action| match &action.action {
            CombatAction::PlayerMove { target_entity, .. } => {
                target_entity.map_or(true, |target| target != dead_entity)
            }
            CombatAction::EnemyAttack { target_entity, .. } => *target_entity != dead_entity,
        });
        let _removed_count = original_len - self.actions.len();

        // After removing actions, if we have no current action and there are remaining
        // valid actions in the queue, start the next one
        if self.current_action.is_none() && !self.actions.is_empty() {
            self.start_next_action();
        }

        // Log was here but removed to avoid adding tracing dependency
    }

    /// Clear all actions and update target tracker
    pub fn clear_with_tracker(
        &mut self,
        entity: Entity,
        target_tracker: &mut crate::target_tracker::TargetTracker,
    ) {
        // Remove this entity from all target tracking relationships
        target_tracker.clear_entity_targeting(entity);

        // Clear the queue
        self.actions.clear();
        self.current_action = None;
    }
}

/// Get a description of body state for display
pub fn get_body_status_description(body: &Body) -> String {
    let integrity = calculate_body_integrity(body);
    match integrity {
        i if i >= 0.9 => "Systems Nominal".to_string(),
        i if i >= 0.7 => "Minor Damage Detected".to_string(),
        i if i >= 0.5 => "Moderate System Damage".to_string(),
        i if i >= 0.3 => "Critical Systems Damaged".to_string(),
        _ => "Severe System Failure".to_string(),
    }
}

/// Damage a random body part (for taking damage as a robot)
pub fn damage_random_body_part(
    body: &mut Body,
    world_seed: u64,
    tick: u64,
    damage: u32,
) -> Option<BodyPartType> {
    // Simple deterministic "random" selection based on seed and tick
    let available_parts: Vec<_> = body
        .parts
        .iter()
        .filter(|(_, part)| part.state != BodyPartState::Missing)
        .map(|(part_type, _)| *part_type)
        .collect();

    if available_parts.is_empty() {
        return None;
    }

    let index = ((world_seed + tick) % available_parts.len() as u64) as usize;
    let part_type = available_parts[index];

    if let Some(part) = body.parts.get_mut(&part_type) {
        // 20% chance for severing damage on strong attacks
        let can_sever = damage > 20 && ((world_seed + tick + 1) % 5) == 0;
        part.receive_damage(damage as i64, can_sever);
        Some(part_type)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_creation() {
        let melee = Move::melee();
        assert_eq!(melee.move_type, MoveType::Melee);
        assert_eq!(melee.damage, 10);
        assert_eq!(melee.energy_cost, 0);

        let tackle = Move::tackle();
        assert_eq!(tackle.move_type, MoveType::Tackle);
        assert_eq!(tackle.damage, 15);
        assert_eq!(tackle.energy_cost, 20);

        let fireball = Move::fireball();
        assert_eq!(fireball.move_type, MoveType::Fireball);
        assert_eq!(fireball.damage, 30);
        assert_eq!(fireball.splash_radius, Some(1));
    }

    #[test]
    fn test_move_human_names() {
        assert_eq!(MoveType::Melee.human_name(), "Melee");
        assert_eq!(MoveType::Tackle.human_name(), "Tackle");
        assert_eq!(MoveType::Fireball.human_name(), "Fireball");
        assert_eq!(MoveType::Escape.human_name(), "Escape");
        assert_eq!(
            MoveType::DebugInstantKill.human_name(),
            "Debug Instant Kill"
        );
    }

    #[test]
    fn test_get_available_moves() {
        let moves = get_available_moves();
        assert_eq!(moves.len(), 9);

        let move_types: Vec<MoveType> = moves.iter().map(|m| m.move_type).collect();
        assert!(move_types.contains(&MoveType::Melee));
        assert!(move_types.contains(&MoveType::Tackle));
        assert!(move_types.contains(&MoveType::Fireball));
        assert!(move_types.contains(&MoveType::Escape));
    }

    #[test]
    fn test_calculate_push_position() {
        let player_pos = Position { x: 0, y: 0, z: 0 };

        // Test pushing east
        let enemy_east = Position { x: 1, y: 0, z: 0 };
        let pushed = calculate_push_position(player_pos, enemy_east, 2);
        assert_eq!(pushed, Position { x: 3, y: 0, z: 0 });

        // Test pushing west
        let enemy_west = Position { x: -1, y: 0, z: 0 };
        let pushed = calculate_push_position(player_pos, enemy_west, 2);
        assert_eq!(pushed, Position { x: -3, y: 0, z: 0 });

        // Test pushing north
        let enemy_north = Position { x: 0, y: 1, z: 0 };
        let pushed = calculate_push_position(player_pos, enemy_north, 2);
        assert_eq!(pushed, Position { x: 0, y: 3, z: 0 });

        // Test pushing south
        let enemy_south = Position { x: 0, y: -1, z: 0 };
        let pushed = calculate_push_position(player_pos, enemy_south, 2);
        assert_eq!(pushed, Position { x: 0, y: -3, z: 0 });

        // Test same position (no push)
        let pushed = calculate_push_position(player_pos, player_pos, 2);
        assert_eq!(pushed, Position { x: 0, y: 0, z: 0 });
    }

    #[test]
    fn test_tackle_mechanics() {
        let tackle = Move::tackle();

        // Verify tackle properties match specification
        assert_eq!(tackle.damage, 15);
        assert_eq!(tackle.energy_cost, 20);
        assert_eq!(tackle.execution_time_ticks, 10);
        assert_eq!(tackle.move_type, MoveType::Tackle);
        assert!(tackle.description.contains("Pushes enemy back 2 spaces"));
        assert!(tackle.description.contains("50% chance to stun"));
        assert!(tackle.description.contains("600 ticks"));
    }

    #[test]
    fn test_action_queue_basic_operations() {
        let mut queue = ActionQueue::new();
        assert!(queue.current_action.is_none());
        assert_eq!(queue.actions.len(), 0);

        // Create a mock entity (Entity is opaque, so we'll use Entity::DANGLING for testing)
        let entity = Entity::DANGLING;

        let action = QueuedAction {
            entity,
            action: CombatAction::PlayerMove {
                move_data: Move::melee(),
                target_entity: None,
                target_position: None,
            },
            execution_time_ticks: 5,
            remaining_time_ticks: 5,
        };

        queue.queue_action(action);
        assert_eq!(queue.actions.len(), 1);

        // Start next action
        let started = queue.start_next_action();
        assert!(started.is_some());
        assert!(queue.current_action.is_some());
        assert_eq!(queue.actions.len(), 0);
    }

    #[test]
    fn test_action_queue_timer_updates() {
        let mut queue = ActionQueue::new();
        let entity = Entity::DANGLING;

        let action = QueuedAction {
            entity,
            action: CombatAction::PlayerMove {
                move_data: Move::melee(),
                target_entity: None,
                target_position: None,
            },
            execution_time_ticks: 10,
            remaining_time_ticks: 10,
        };

        queue.queue_action(action);
        queue.start_next_action();

        // Advance timers by 3 ticks
        queue.advance_timers(3);
        if let Some(ref current) = queue.current_action {
            assert_eq!(current.remaining_time_ticks, 7);
        }

        // Complete the action
        queue.advance_timers(7);
        let completed = queue.pop_completed_action();
        assert!(completed.is_some());
        assert!(queue.current_action.is_none());
    }

    #[test]
    fn test_calculate_body_integrity() {
        use crate::model::body::{Body, BodyPart, BodyPartState, BodyPartType};
        use std::collections::HashMap;

        let mut parts = HashMap::new();
        parts.insert(
            BodyPartType::Head,
            BodyPart {
                part_type: BodyPartType::Head,
                state: BodyPartState::Functional,
                integrity: 100,
                name: "Head".to_string(),
                description: "Test head".to_string(),
                walk_speed_modifier: 1.0,
                break_speed_modifier: 1.0,
                health_modifier: 0,
                stamina_modifier: 0,
            },
        );
        parts.insert(
            BodyPartType::LeftArm,
            BodyPart {
                part_type: BodyPartType::LeftArm,
                state: BodyPartState::Functional,
                integrity: 100,
                name: "Left Arm".to_string(),
                description: "Test arm".to_string(),
                walk_speed_modifier: 1.0,
                break_speed_modifier: 1.0,
                health_modifier: 0,
                stamina_modifier: 0,
            },
        );

        let body = Body { parts };
        let integrity = calculate_body_integrity(&body);
        assert_eq!(integrity, 1.0); // 100% functional

        // Test with damaged parts
        let mut parts = HashMap::new();
        parts.insert(
            BodyPartType::Head,
            BodyPart {
                part_type: BodyPartType::Head,
                state: BodyPartState::Functional,
                integrity: 100,
                name: "Head".to_string(),
                description: "Test head".to_string(),
                walk_speed_modifier: 1.0,
                break_speed_modifier: 1.0,
                health_modifier: 0,
                stamina_modifier: 0,
            },
        );
        parts.insert(
            BodyPartType::LeftArm,
            BodyPart {
                part_type: BodyPartType::LeftArm,
                state: BodyPartState::Damaged,
                integrity: 25,
                name: "Left Arm".to_string(),
                description: "Damaged arm".to_string(),
                walk_speed_modifier: 1.0,
                break_speed_modifier: 1.0,
                health_modifier: 0,
                stamina_modifier: 0,
            },
        );

        let body = Body { parts };
        let integrity = calculate_body_integrity(&body);
        assert_eq!(integrity, 0.75); // (1.0 + 0.5) / 2 = 0.75
    }

    #[test]
    fn test_get_body_status_description() {
        use crate::model::body::{Body, BodyPart, BodyPartState, BodyPartType};
        use std::collections::HashMap;

        // Test nominal status
        let mut parts = HashMap::new();
        parts.insert(
            BodyPartType::Head,
            BodyPart {
                part_type: BodyPartType::Head,
                state: BodyPartState::Functional,
                integrity: 100,
                name: "Head".to_string(),
                description: "Test head".to_string(),
                walk_speed_modifier: 1.0,
                break_speed_modifier: 1.0,
                health_modifier: 0,
                stamina_modifier: 0,
            },
        );
        let body = Body { parts };
        assert_eq!(get_body_status_description(&body), "Systems Nominal");

        // Test critical damage
        let mut parts = HashMap::new();
        parts.insert(
            BodyPartType::Head,
            BodyPart {
                part_type: BodyPartType::Head,
                state: BodyPartState::Missing,
                integrity: 0,
                name: "Head".to_string(),
                description: "Missing head".to_string(),
                walk_speed_modifier: 1.0,
                break_speed_modifier: 1.0,
                health_modifier: 0,
                stamina_modifier: 0,
            },
        );
        let body = Body { parts };
        assert_eq!(get_body_status_description(&body), "Severe System Failure");
    }
}

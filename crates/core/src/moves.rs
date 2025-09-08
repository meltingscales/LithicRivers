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
}

impl MoveType {
    pub fn execution_time_ticks(&self) -> u64 {
        match self {
            MoveType::Melee => 15,
            MoveType::Fireball => 40,
            MoveType::Tackle => 80,
            MoveType::Escape => 25,
            MoveType::DebugInstantKill => 5,
        }
    }

    pub fn human_name(&self) -> &'static str {
        match self {
            MoveType::Melee => "Melee",
            MoveType::Fireball => "Fireball",
            MoveType::Tackle => "Tackle",
            MoveType::Escape => "Escape",
            MoveType::DebugInstantKill => "Debug Instant Kill",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Move {
    pub name: String,
    pub move_type: MoveType,
    pub energy_cost: u32,
    pub execution_time_ticks: i32,
    pub damage: u32,
    pub description: String,
}

impl Move {
    pub fn melee() -> Self {
        Self {
            name: "Melee".to_string(),
            move_type: MoveType::Melee,
            energy_cost: 0,
            execution_time_ticks: 15,
            damage: 10,
            description: "Basic melee attack".to_string(),
        }
    }

    pub fn tackle() -> Self {
        Self {
            name: "Tackle".to_string(),
            move_type: MoveType::Tackle,
            energy_cost: 20,
            execution_time_ticks: 80,
            damage: 15,
            description: "Pushes enemy back 2 spaces, 50% chance to stun for 600 ticks".to_string(),
        }
    }

    pub fn fireball() -> Self {
        Self {
            name: "Fireball (AoE)".to_string(),
            move_type: MoveType::Fireball,
            energy_cost: 40,
            execution_time_ticks: 40,
            damage: 30,
            description: "Area of effect fire damage".to_string(),
        }
    }

    pub fn escape() -> Self {
        Self {
            name: "Escape".to_string(),
            move_type: MoveType::Escape,
            energy_cost: 20,
            execution_time_ticks: 25,
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
            damage: 9999,
            description: "Debug instant kill".to_string(),
        }
    }

    pub fn heal() -> Self {
        Self {
            name: "Heal".to_string(),
            move_type: MoveType::Melee, // Reusing MoveType for now
            energy_cost: 30,
            execution_time_ticks: 20,
            damage: 0,
            description: "Restore health".to_string(),
        }
    }

    pub fn shield() -> Self {
        Self {
            name: "Shield".to_string(),
            move_type: MoveType::Melee, // Reusing MoveType for now
            energy_cost: 25,
            execution_time_ticks: 10,
            damage: 0,
            description: "Block incoming attacks".to_string(),
        }
    }

    pub fn lightning_bolt() -> Self {
        Self {
            name: "Lightning Bolt".to_string(),
            move_type: MoveType::Fireball, // Reusing MoveType for now
            energy_cost: 50,
            execution_time_ticks: 30,
            damage: 40,
            description: "Fast electric attack".to_string(),
        }
    }

    pub fn power_strike() -> Self {
        Self {
            name: "Power Strike".to_string(),
            move_type: MoveType::Melee, // Reusing MoveType for now
            energy_cost: 40,
            execution_time_ticks: 40,
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
pub fn calculate_push_position(from: Position, target: Position, push_distance: i32) -> Position {
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
        part.receive_damage(damage as i32, can_sever);
        Some(part_type)
    } else {
        None
    }
}

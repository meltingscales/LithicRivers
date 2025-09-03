use crate::components::{Energy, Position};
use crate::model::body::{Body, BodyPartState, BodyPartType};
use hecs::Entity;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MoveType {
    Melee,
    Tackle,
    Fireball,
    Escape,
}

#[derive(Debug, Clone)]
pub struct Move {
    pub name: String,
    pub move_type: MoveType,
    pub energy_cost: u32,
    pub cooldown_ticks: u64,
    pub damage: u32,
    pub description: String,
}

impl Move {
    pub fn melee() -> Self {
        Self {
            name: "Melee".to_string(),
            move_type: MoveType::Melee,
            energy_cost: 0,
            cooldown_ticks: 0,
            damage: 10,
            description: "Basic melee attack".to_string(),
        }
    }

    pub fn tackle() -> Self {
        Self {
            name: "Tackle".to_string(),
            move_type: MoveType::Tackle,
            energy_cost: 20,
            cooldown_ticks: 800,
            damage: 15,
            description: "Pushes enemy back 2 spaces, 50% chance to stun for 600 ticks".to_string(),
        }
    }

    pub fn fireball() -> Self {
        Self {
            name: "Fireball (AoE)".to_string(),
            move_type: MoveType::Fireball,
            energy_cost: 40,
            cooldown_ticks: 0,
            damage: 30,
            description: "Area of effect fire damage".to_string(),
        }
    }

    pub fn escape() -> Self {
        Self {
            name: "Escape".to_string(),
            move_type: MoveType::Escape,
            energy_cost: 20,
            cooldown_ticks: 0,
            damage: 0,
            description: "Flee from combat, adds BattleDelay".to_string(),
        }
    }
}

/// Component to track move cooldowns for an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveCooldowns {
    pub melee: u64,
    pub tackle: u64,
    pub fireball: u64,
    pub escape: u64,
}

impl MoveCooldowns {
    pub fn new() -> Self {
        Self {
            melee: 0,
            tackle: 0,
            fireball: 0,
            escape: 0,
        }
    }

    pub fn get_cooldown(&self, move_type: MoveType) -> u64 {
        match move_type {
            MoveType::Melee => self.melee,
            MoveType::Tackle => self.tackle,
            MoveType::Fireball => self.fireball,
            MoveType::Escape => self.escape,
        }
    }

    pub fn set_cooldown(&mut self, move_type: MoveType, ticks: u64) {
        match move_type {
            MoveType::Melee => self.melee = ticks,
            MoveType::Tackle => self.tackle = ticks,
            MoveType::Fireball => self.fireball = ticks,
            MoveType::Escape => self.escape = ticks,
        }
    }

    pub fn tick(&mut self) {
        self.melee = self.melee.saturating_sub(1);
        self.tackle = self.tackle.saturating_sub(1);
        self.fireball = self.fireball.saturating_sub(1);
        self.escape = self.escape.saturating_sub(1);
    }

    pub fn can_use(&self, move_type: MoveType) -> bool {
        self.get_cooldown(move_type) == 0
    }
}

impl Default for MoveCooldowns {
    fn default() -> Self {
        Self::new()
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
    ]
}

pub fn can_use_move(energy: &Energy, cooldowns: &MoveCooldowns, mv: &Move) -> bool {
    energy.current >= mv.energy_cost && cooldowns.can_use(mv.move_type)
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
) -> Option<BodyPartType> {
    // Simple deterministic "random" selection based on seed and tick
    let available_parts: Vec<_> = body
        .parts
        .iter()
        .filter(|(_, part)| {
            part.state == BodyPartState::Functional || part.state == BodyPartState::Enhanced
        })
        .map(|(part_type, _)| *part_type)
        .collect();

    if available_parts.is_empty() {
        return None;
    }

    let index = ((world_seed + tick) % available_parts.len() as u64) as usize;
    let part_type = available_parts[index];

    if let Some(part) = body.parts.get_mut(&part_type) {
        part.state = match part.state {
            BodyPartState::Enhanced => BodyPartState::Functional,
            BodyPartState::Functional => BodyPartState::Damaged,
            _ => part.state, // Already damaged or missing
        };
        Some(part_type)
    } else {
        None
    }
}

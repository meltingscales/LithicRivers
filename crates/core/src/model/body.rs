use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum BodyPartType {
    Head,
    Torso,
    PowerSource,
    LeftArm,
    RightArm,
    LeftLeg,
    RightLeg,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
pub enum BodyPartState {
    Missing,
    Damaged,
    Functional,
    Enhanced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyPart {
    pub part_type: BodyPartType,
    pub state: BodyPartState,
    pub integrity: i64,
    pub name: String,
    pub description: String,
    // Stat modifiers when this part is in different states
    pub walk_speed_modifier: f64,
    pub break_speed_modifier: f64,
    pub health_modifier: i64,
    pub stamina_modifier: i64,
}

impl BodyPart {
    pub fn receive_damage(&mut self, damage: i64, can_sever: bool) {
        self.integrity -= damage;

        if can_sever && self.integrity <= 0 {
            self.integrity = 0;
            self.state = BodyPartState::Missing;
        } else if !can_sever && self.integrity < 1 {
            self.integrity = 1;
        }

        self.update_state_from_integrity();
    }

    pub fn update_state_from_integrity(&mut self) {
        self.state = match self.integrity {
            0 => BodyPartState::Missing,
            1..=50 => BodyPartState::Damaged,
            51..=100 => BodyPartState::Functional,
            _ => BodyPartState::Enhanced,
        };
    }

    pub fn get_walk_speed_modifier(&self) -> f64 {
        match self.state {
            BodyPartState::Missing => 0.0,
            BodyPartState::Damaged => self.walk_speed_modifier * 0.5,
            BodyPartState::Functional => self.walk_speed_modifier,
            BodyPartState::Enhanced => self.walk_speed_modifier * 1.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Body {
    pub parts: HashMap<BodyPartType, BodyPart>,
}

impl Body {
    pub fn walk_speed_modifier(&self) -> f64 {
        // Focus on legs primarily; take the minimum of left/right leg modifiers to represent weakest link.
        let left = self
            .parts
            .get(&BodyPartType::LeftLeg)
            .map(|p| p.get_walk_speed_modifier())
            .unwrap_or(0.0);
        let right = self
            .parts
            .get(&BodyPartType::RightLeg)
            .map(|p| p.get_walk_speed_modifier())
            .unwrap_or(0.0);
        let legs = left.min(right);
        if legs > 0.0 {
            legs
        } else {
            // If legs unusable, fall back to average of all positive modifiers (still may be 0)
            let mut sum = 0.0f64;
            let mut count = 0u64;
            for p in self.parts.values() {
                let m = p.get_walk_speed_modifier();
                if m > 0.0 {
                    sum += m;
                    count += 1;
                }
            }
            if count > 0 {
                sum / count as f64
            } else {
                0.0
            }
        }
    }
}

impl Default for Body {
    fn default() -> Self {
        // Seed defaults based on boot_message.dat diagnostics:
        // - Left arm: No signal detected (missing)
        // - Right leg: Torque mismatch (treat as damaged)
        let mut parts = HashMap::new();
        let mut insert = |part_type: BodyPartType,
                          state: BodyPartState,
                          integrity: i64,
                          name: &str,
                          desc: &str| {
            parts.insert(
                part_type,
                BodyPart {
                    part_type,
                    state,
                    integrity,
                    name: name.to_string(),
                    description: desc.to_string(),
                    walk_speed_modifier: 1.0,
                    break_speed_modifier: 1.0,
                    health_modifier: 0,
                    stamina_modifier: 0,
                },
            );
        };
        insert(
            BodyPartType::Head,
            BodyPartState::Functional,
            100,
            "Head",
            "Standard cranial unit",
        );
        insert(
            BodyPartType::Torso,
            BodyPartState::Functional,
            100,
            "Torso",
            "Reinforced chassis",
        );
        insert(
            BodyPartType::PowerSource,
            BodyPartState::Damaged,
            21,
            "Fusion Core",
            "Nearly limitless energy source, very degraded voltage output detected",
        );
        insert(
            BodyPartType::LeftArm,
            BodyPartState::Missing,
            0,
            "Left Arm",
            "Limb absent per diagnostics",
        );
        insert(
            BodyPartType::RightArm,
            BodyPartState::Functional,
            100,
            "Right Arm",
            "Nominal voltage detected",
        );
        insert(
            BodyPartType::LeftLeg,
            BodyPartState::Functional,
            100,
            "Left Leg",
            "Nominal voltage detected",
        );
        insert(
            BodyPartType::RightLeg,
            BodyPartState::Damaged,
            43,
            "Right Leg",
            "Torque feedback mismatch detected",
        );
        Body { parts }
    }
}

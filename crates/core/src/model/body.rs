use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum BodyPartType {
    Head,
    Torso,
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
    pub integrity: i32,
    pub name: String,
    pub description: String,
    // Stat modifiers when this part is in different states
    pub walk_speed_modifier: f32,
    pub break_speed_modifier: f32,
    pub health_modifier: i32,
    pub stamina_modifier: i32,
}

impl BodyPart {

    pub fn receive_damage(&self, damage:i32, can_sever: bool)->None{
        //TODO:
        // if can_sever is true, apply damage and destroy the body part if it goes to 0, setting it to "Missing" and "0"
        // if can_sever is false, apply damage and set to "Damaged" if it goes below 50, but not below 1
    }

    pub fn update_state_from_integrity(&self)->None {
        match self.integrity {
            //TODO:
            //0: absent
            //1-50: damaged
            //50-100: functional
            //100+: enhanced
        }
    }

    pub fn get_walk_speed_modifier(&self) -> f32 {
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
    pub fn walk_speed_modifier(&self) -> f32 {
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
            let mut sum = 0.0f32;
            let mut count = 0u32;
            for p in self.parts.values() {
                let m = p.get_walk_speed_modifier();
                if m > 0.0 {
                    sum += m;
                    count += 1;
                }
            }
            if count > 0 {
                sum / count as f32
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
        let mut insert = |part_type: BodyPartType, state: BodyPartState, integrity: i32, name: &str, desc: &str| {
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

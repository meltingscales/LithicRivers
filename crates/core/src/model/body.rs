use crate::components::ItemKind;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash, PartialOrd, Ord)]
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

    /// Bring a part that isn't missing back to full integrity; enhanced parts stay enhanced.
    pub fn repair_fully(&mut self) {
        if self.state != BodyPartState::Missing {
            self.integrity = self.integrity.max(100);
            self.update_state_from_integrity();
        }
    }

    pub fn update_state_from_integrity(&mut self) {
        self.state = match self.integrity {
            0 => BodyPartState::Missing,
            1..=50 => BodyPartState::Damaged,
            51..=100 => BodyPartState::Functional,
            _ => BodyPartState::Enhanced,
        };
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Body {
    pub parts: HashMap<BodyPartType, BodyPart>,
}

/// Integrity of a freshly installed replacement part: Functional, with room to repair.
pub const INSTALLED_PART_INTEGRITY: i64 = 70;

/// The body parts a replacement item can be installed into; empty for other items.
pub fn replacement_parts(item: ItemKind) -> &'static [BodyPartType] {
    match item {
        ItemKind::SalvagedArm => &[BodyPartType::LeftArm, BodyPartType::RightArm],
        _ => &[],
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallError {
    /// The part is there already; only a missing part takes a replacement.
    NotMissing,
    /// The item isn't a replacement for this part.
    DoesNotFit,
}

impl Body {
    /// Install a replacement item into a missing part, making it Functional at
    /// `INSTALLED_PART_INTEGRITY`. The caller uses up the item on success.
    pub fn install(&mut self, part: BodyPartType, item: ItemKind) -> Result<(), InstallError> {
        if !replacement_parts(item).contains(&part) {
            return Err(InstallError::DoesNotFit);
        }
        let Some(target) = self.parts.get_mut(&part) else {
            return Err(InstallError::DoesNotFit);
        };
        if target.state != BodyPartState::Missing {
            return Err(InstallError::NotMissing);
        }
        target.integrity = INSTALLED_PART_INTEGRITY;
        target.update_state_from_integrity();
        Ok(())
    }
}

/// Walking time with the weaker leg damaged, one leg missing, or both missing.
pub const LEG_DAMAGED_WALK: f64 = 2.0;
pub const LEG_MISSING_WALK: f64 = 4.0;
pub const LEGS_MISSING_WALK: f64 = 8.0;
/// Walking time with the weaker leg enhanced (1.5 times as fast).
pub const LEG_ENHANCED_WALK: f64 = 1.0 / 1.5;
/// Mining time and melee damage per damaged arm, and for one missing arm.
pub const ARM_DAMAGED_MINE: f64 = 1.5;
pub const ARM_DAMAGED_MELEE: f64 = 0.75;
pub const ARM_MISSING_MINE: f64 = 2.0;
pub const ARM_MISSING_MELEE: f64 = 0.6;
/// Light radius lost with a damaged head, and the radius left with no head.
pub const HEAD_DAMAGED_LIGHT_PENALTY: u32 = 1;
pub const HEAD_MISSING_LIGHT_RADIUS: u32 = 1;
/// Combat energy recovery with a damaged power source.
pub const POWER_DAMAGED_ENERGY_REGEN: f64 = 0.5;

/// What the body's part states do to the robot. Factors multiply the time an action
/// takes (walking, mining) or what it gives (melee damage, energy recovery).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BodyEffects {
    pub walk_factor: f64,
    pub mine_factor: f64,
    pub melee_factor: f64,
    /// False with both arms missing: no mining, breaking or melee.
    pub has_working_arm: bool,
    pub head: BodyPartState,
    pub energy_regen_factor: f64,
}

impl Default for BodyEffects {
    /// No effects: a whole body.
    fn default() -> Self {
        Self {
            walk_factor: 1.0,
            mine_factor: 1.0,
            melee_factor: 1.0,
            has_working_arm: true,
            head: BodyPartState::Functional,
            energy_regen_factor: 1.0,
        }
    }
}

impl BodyEffects {
    /// The light radius a source of `base` tiles gives with this head, never below 1.
    pub fn light_radius(&self, base: u32) -> u32 {
        match self.head {
            BodyPartState::Missing => HEAD_MISSING_LIGHT_RADIUS,
            BodyPartState::Damaged => base.saturating_sub(HEAD_DAMAGED_LIGHT_PENALTY).max(1),
            _ => base.max(1),
        }
    }

    /// `base` ticks of walking, at least 1.
    pub fn walk_cost(&self, base: f64) -> u64 {
        (base * self.walk_factor).round().max(1.0) as u64
    }

    /// `base` ticks of mining or breaking, at least 1.
    pub fn mine_cost(&self, base: f64) -> u64 {
        (base * self.mine_factor).round().max(1.0) as u64
    }

    /// Melee damage from a move's `base`, rounded down and at least 1, or 0 without arms.
    pub fn melee_damage(&self, base: u32) -> u32 {
        if !self.has_working_arm {
            return 0;
        }
        ((base as f64 * self.melee_factor).floor() as u32).max(1)
    }
}

/// "2x", "1.5x", "2.25x".
fn times(factor: f64) -> String {
    format!("{}x", (factor * 100.0).round() / 100.0)
}

/// "-40%".
fn less(factor: f64) -> String {
    format!("-{}%", ((1.0 - factor) * 100.0).round())
}

const LEGS: [BodyPartType; 2] = [BodyPartType::LeftLeg, BodyPartType::RightLeg];
const ARMS: [BodyPartType; 2] = [BodyPartType::LeftArm, BodyPartType::RightArm];

fn leg_walk(state: BodyPartState) -> f64 {
    match state {
        BodyPartState::Missing => LEG_MISSING_WALK,
        BodyPartState::Damaged => LEG_DAMAGED_WALK,
        BodyPartState::Functional => 1.0,
        BodyPartState::Enhanced => LEG_ENHANCED_WALK,
    }
}

/// An arm's mining time and melee damage factors.
fn arm_factors(state: BodyPartState) -> (f64, f64) {
    match state {
        BodyPartState::Missing => (ARM_MISSING_MINE, ARM_MISSING_MELEE),
        BodyPartState::Damaged => (ARM_DAMAGED_MINE, ARM_DAMAGED_MELEE),
        _ => (1.0, 1.0),
    }
}

fn state_word(state: BodyPartState) -> &'static str {
    match state {
        BodyPartState::Missing => "missing",
        BodyPartState::Damaged => "damaged",
        BodyPartState::Functional => "functional",
        BodyPartState::Enhanced => "enhanced",
    }
}

impl Body {
    fn state(&self, part: BodyPartType) -> BodyPartState {
        self.parts
            .get(&part)
            .map_or(BodyPartState::Missing, |p| p.state)
    }

    fn part_name(&self, part: BodyPartType) -> String {
        self.parts
            .get(&part)
            .map_or_else(|| format!("{part:?}"), |p| p.name.clone())
    }

    /// The leg that sets the walking pace.
    fn weaker_leg(&self) -> BodyPartType {
        let [left, right] = LEGS;
        if leg_walk(self.state(right)) > leg_walk(self.state(left)) {
            right
        } else {
            left
        }
    }

    /// The effects of every part's state, computed in one place.
    pub fn effects(&self) -> BodyEffects {
        // Legs: the weaker leg sets the pace; walking never becomes impossible.
        let walk_factor = if LEGS
            .iter()
            .all(|&l| self.state(l) == BodyPartState::Missing)
        {
            LEGS_MISSING_WALK
        } else {
            leg_walk(self.state(self.weaker_leg()))
        };

        // Arms: each damaged arm and one missing arm slow mining and soften melee.
        let has_working_arm = ARMS
            .iter()
            .any(|&a| self.state(a) != BodyPartState::Missing);
        let (mut mine_factor, mut melee_factor) = (1.0, 1.0);
        if has_working_arm {
            for arm in ARMS {
                let (mine, melee) = arm_factors(self.state(arm));
                mine_factor *= mine;
                melee_factor *= melee;
            }
        }

        let energy_regen_factor = if self.state(BodyPartType::PowerSource) == BodyPartState::Damaged
        {
            POWER_DAMAGED_ENERGY_REGEN
        } else {
            1.0
        };

        BodyEffects {
            walk_factor,
            mine_factor,
            melee_factor,
            has_working_arm,
            head: self.state(BodyPartType::Head),
            energy_regen_factor,
        }
    }

    /// One plain line per active damage effect, naming the part, for the Body tab.
    pub fn describe_effects(&self) -> Vec<String> {
        let effects = self.effects();
        let mut lines = Vec::new();

        if effects.walk_factor > 1.0 {
            let who = if LEGS
                .iter()
                .all(|&l| self.state(l) == BodyPartState::Missing)
            {
                "Both legs missing".to_string()
            } else {
                let leg = self.weaker_leg();
                format!("{} {}", self.part_name(leg), state_word(self.state(leg)))
            };
            lines.push(format!(
                "{who}: walking {} slower",
                times(effects.walk_factor)
            ));
        }

        if effects.has_working_arm {
            for arm in ARMS {
                let (mine, melee) = arm_factors(self.state(arm));
                if mine > 1.0 {
                    lines.push(format!(
                        "{} {}: mining {} slower, melee {}",
                        self.part_name(arm),
                        state_word(self.state(arm)),
                        times(mine),
                        less(melee)
                    ));
                }
            }
        } else {
            lines.push("Both arms missing: no mining or melee".to_string());
        }

        let head = self.part_name(BodyPartType::Head);
        match effects.head {
            BodyPartState::Damaged => lines.push(format!(
                "{head} damaged: light radius -{HEAD_DAMAGED_LIGHT_PENALTY}"
            )),
            BodyPartState::Missing => lines.push(format!(
                "{head} missing: light radius {HEAD_MISSING_LIGHT_RADIUS}"
            )),
            _ => {}
        }

        if effects.energy_regen_factor < 1.0 {
            lines.push(format!(
                "{} damaged: energy recovers {} slower",
                self.part_name(BodyPartType::PowerSource),
                times(1.0 / effects.energy_regen_factor)
            ));
        }
        lines
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

#[cfg(test)]
mod tests {
    use super::*;
    use BodyPartState::*;
    use BodyPartType::*;

    /// A whole body with the given parts set to other states.
    fn body(states: &[(BodyPartType, BodyPartState)]) -> Body {
        let mut body = Body::default();
        for part in body.parts.values_mut() {
            part.state = Functional;
        }
        for &(part, state) in states {
            body.parts.get_mut(&part).unwrap().state = state;
        }
        body
    }

    #[test]
    fn a_whole_body_has_no_effects() {
        assert_eq!(body(&[]).effects(), BodyEffects::default());
        assert!(body(&[]).describe_effects().is_empty());
    }

    #[test]
    fn legs_follow_the_weaker_leg() {
        let walk = |states: &[(BodyPartType, BodyPartState)]| body(states).effects().walk_factor;
        assert_eq!(walk(&[(RightLeg, Damaged)]), 2.0);
        assert_eq!(walk(&[(LeftLeg, Missing)]), 4.0);
        assert_eq!(walk(&[(LeftLeg, Missing), (RightLeg, Damaged)]), 4.0);
        assert_eq!(walk(&[(LeftLeg, Missing), (RightLeg, Missing)]), 8.0);
        assert_eq!(walk(&[(LeftLeg, Enhanced)]), 1.0);
        assert_eq!(
            walk(&[(LeftLeg, Enhanced), (RightLeg, Enhanced)]),
            1.0 / 1.5
        );
    }

    #[test]
    fn walking_costs_stay_as_before_for_damaged_and_enhanced_legs() {
        // Before: 200 / 0.5 for a damaged leg, 200 / 1.5 for two enhanced legs
        assert_eq!(body(&[(RightLeg, Damaged)]).effects().walk_cost(200.0), 400);
        let enhanced = body(&[(LeftLeg, Enhanced), (RightLeg, Enhanced)]);
        assert_eq!(enhanced.effects().walk_cost(200.0), 133);
    }

    #[test]
    fn a_damaged_leg_does_not_slow_mining() {
        let effects = body(&[(RightLeg, Damaged)]).effects();
        assert_eq!(effects.mine_factor, 1.0);
        assert_eq!(effects.mine_cost(300.0), 300);
    }

    #[test]
    fn arms_slow_mining_and_soften_melee() {
        let arms = |states: &[(BodyPartType, BodyPartState)]| {
            let e = body(states).effects();
            (e.mine_factor, e.melee_factor, e.has_working_arm)
        };
        assert_eq!(arms(&[(LeftArm, Damaged)]), (1.5, 0.75, true));
        assert_eq!(
            arms(&[(LeftArm, Damaged), (RightArm, Damaged)]),
            (2.25, 0.5625, true)
        );
        assert_eq!(arms(&[(LeftArm, Missing)]), (2.0, 0.6, true));
        assert_eq!(
            arms(&[(LeftArm, Missing), (RightArm, Damaged)]),
            (3.0, 0.6 * 0.75, true)
        );
        assert_eq!(arms(&[(RightArm, Enhanced)]), (1.0, 1.0, true));
        assert!(!arms(&[(LeftArm, Missing), (RightArm, Missing)]).2);
    }

    #[test]
    fn melee_damage_rounds_down_and_needs_an_arm() {
        assert_eq!(body(&[(LeftArm, Missing)]).effects().melee_damage(10), 6);
        assert_eq!(body(&[(LeftArm, Damaged)]).effects().melee_damage(10), 7);
        assert_eq!(body(&[(LeftArm, Missing)]).effects().melee_damage(1), 1);
        let armless = body(&[(LeftArm, Missing), (RightArm, Missing)]);
        assert_eq!(armless.effects().melee_damage(10), 0);
    }

    #[test]
    fn the_head_dims_the_light() {
        let damaged = body(&[(Head, Damaged)]).effects();
        assert_eq!(damaged.light_radius(8), 7);
        assert_eq!(damaged.light_radius(2), 1);
        assert_eq!(damaged.light_radius(1), 1);
        let missing = body(&[(Head, Missing)]).effects();
        assert_eq!(missing.light_radius(8), 1);
        assert_eq!(body(&[(Head, Enhanced)]).effects().light_radius(8), 8);
    }

    #[test]
    fn a_damaged_power_source_halves_energy_recovery() {
        let effects = body(&[(PowerSource, Damaged)]).effects();
        assert_eq!(effects.energy_regen_factor, 0.5);
        assert_eq!(body(&[(Torso, Damaged)]).effects(), BodyEffects::default());
    }

    #[test]
    fn the_starting_body_lists_its_arm_and_leg_effects() {
        let body = Body::default();
        let effects = body.effects();
        assert_eq!(effects.walk_factor, 2.0);
        assert_eq!(effects.mine_factor, 2.0);
        assert_eq!(effects.melee_factor, 0.6);
        assert_eq!(
            body.describe_effects(),
            vec![
                "Right Leg damaged: walking 2x slower",
                "Left Arm missing: mining 2x slower, melee -40%",
                "Fusion Core damaged: energy recovers 2x slower",
            ]
        );
    }

    #[test]
    fn a_salvaged_arm_installs_into_the_missing_arm_only() {
        let mut body = Body::default();
        assert_eq!(
            body.install(LeftLeg, ItemKind::SalvagedArm),
            Err(InstallError::DoesNotFit)
        );
        assert_eq!(
            body.install(LeftArm, ItemKind::Log),
            Err(InstallError::DoesNotFit)
        );
        assert_eq!(
            body.install(RightArm, ItemKind::SalvagedArm),
            Err(InstallError::NotMissing)
        );

        assert_eq!(body.install(LeftArm, ItemKind::SalvagedArm), Ok(()));
        let arm = &body.parts[&LeftArm];
        assert_eq!((arm.state, arm.integrity), (Functional, 70));
        assert_eq!(
            body.install(LeftArm, ItemKind::SalvagedArm),
            Err(InstallError::NotMissing)
        );
        assert_eq!(body.effects().mine_factor, 1.0);
    }

    #[test]
    fn an_installed_arm_can_be_repaired() {
        use crate::recipe_handler::RepairRecipeHandler;
        let mut body = Body::default();
        body.install(LeftArm, ItemKind::SalvagedArm).unwrap();
        let repairs = RepairRecipeHandler::new();
        let recipe = (0..repairs.get_recipes().len())
            .find(|&i| repairs.can_repair_part(i, LeftArm, body.parts[&LeftArm].state))
            .expect("a repair recipe for the arm");
        let mut inventory: HashMap<ItemKind, u32> = repairs.get_recipes()[recipe]
            .ingredients
            .iter()
            .copied()
            .collect();
        let integrity = repairs
            .apply_repair(recipe, &mut inventory, body.parts[&LeftArm].integrity)
            .unwrap();
        assert!(integrity > INSTALLED_PART_INTEGRITY);
    }

    #[test]
    fn missing_limbs_are_described() {
        let all = body(&[
            (LeftLeg, Missing),
            (RightLeg, Missing),
            (LeftArm, Missing),
            (RightArm, Missing),
            (Head, Missing),
        ]);
        assert_eq!(
            all.describe_effects(),
            vec![
                "Both legs missing: walking 8x slower",
                "Both arms missing: no mining or melee",
                "Head missing: light radius 1",
            ]
        );
    }
}

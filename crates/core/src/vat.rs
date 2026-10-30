//! Robot Resurrection Vats: factory blocks that hold a spare body, built from crafted body
//! parts, for the robot to wake in after it dies.
//!
//! A vat keeps its parts in an `Inventory`, so the player fills it through the transfer
//! window and extractors and belts can feed it. Which part goes in which slot is decided
//! when the body is assembled: the first arm item becomes the left arm, the second the
//! right arm, and the same for legs.

use crate::components::{Inventory, ItemKind, Position};
use crate::factory::{FactoryBlock, FactoryKind};
use crate::model::body::{
    crafted_part, installed_integrity, replacement_parts, Body, BodyPartType, PartGrade, PART_ORDER,
};
use crate::power::PowerConsumer;
use hecs::{Entity, World};

/// What a standard vat draws while it holds a ready body: a trickle.
pub const STANDARD_VAT_W: u64 = 15_000;
/// What an advanced vat draws while it holds a ready body: two burner generators' worth.
pub const ADVANCED_VAT_W: u64 = 180_000;

/// Distinct part items a vat can hold at once (one stack per kind).
pub const VAT_SLOTS: usize = PART_ORDER.len();

/// Whether `kind` is a resurrection vat.
pub fn is_vat(kind: FactoryKind) -> bool {
    matches!(
        kind,
        FactoryKind::CrudeVat | FactoryKind::StandardVat | FactoryKind::AdvancedVat
    )
}

/// The power a vat draws while ready, or `None` for the crude vat, which needs none.
pub fn demand_w(kind: FactoryKind) -> Option<u64> {
    match kind {
        FactoryKind::StandardVat => Some(STANDARD_VAT_W),
        FactoryKind::AdvancedVat => Some(ADVANCED_VAT_W),
        _ => None,
    }
}

/// The best part grade a vat takes: crude vats only crude parts, standard vats up to
/// standard, advanced vats any.
fn best_grade(kind: FactoryKind) -> PartGrade {
    match kind {
        FactoryKind::StandardVat => PartGrade::Standard,
        FactoryKind::AdvancedVat => PartGrade::Advanced,
        _ => PartGrade::Crude,
    }
}

/// Whether a vat of `kind` ever takes `item`: a body part of a grade it supports. A
/// salvaged arm counts as a standard part.
pub fn takes(kind: FactoryKind, item: ItemKind) -> bool {
    let grade = match item {
        ItemKind::SalvagedArm => PartGrade::Standard,
        _ => match crafted_part(item) {
            Some((grade, _)) => grade,
            None => return false,
        },
    };
    is_vat(kind) && grade <= best_grade(kind)
}

/// The part slots the items in `inv` fill, in inventory order.
fn assign(inv: &Inventory) -> Vec<(BodyPartType, ItemKind)> {
    let mut filled: Vec<(BodyPartType, ItemKind)> = Vec::new();
    for stack in &inv.slots {
        for _ in 0..stack.qty {
            let free = replacement_parts(stack.kind)
                .iter()
                .find(|part| !filled.iter().any(|(p, _)| p == *part));
            if let Some(&part) = free {
                filled.push((part, stack.kind));
            }
        }
    }
    filled
}

/// Whether the vat would take `item` right now: a part it supports, with a slot left for it.
pub fn accepts(world: &World, vat: Entity, item: ItemKind) -> bool {
    let Ok(block) = world.get::<&FactoryBlock>(vat) else {
        return false;
    };
    if !takes(block.kind, item) {
        return false;
    }
    let Ok(inv) = world.get::<&Inventory>(vat) else {
        return false;
    };
    let filled = assign(&inv);
    replacement_parts(item)
        .iter()
        .any(|part| !filled.iter().any(|(p, _)| p == part))
}

/// The spare body a vat's parts make: the parts it has at their grade's integrity, and
/// every other part missing.
pub fn spare_body(inv: &Inventory) -> Body {
    let filled = assign(inv);
    let mut body = Body::default();
    for part in body.parts.values_mut() {
        part.integrity = filled
            .iter()
            .find(|(p, _)| *p == part.part_type)
            .map_or(0, |&(_, item)| installed_integrity(item));
        part.update_state_from_integrity();
    }
    body
}

/// Whether a vat's parts make a body the robot can wake in: it has a torso and a fusion core.
pub fn has_core_parts(inv: &Inventory) -> bool {
    let filled = assign(inv);
    [BodyPartType::Torso, BodyPartType::PowerSource]
        .iter()
        .all(|part| filled.iter().any(|(p, _)| p == part))
}

/// The parts a vat is still missing, in Body tab order.
pub fn missing_parts(inv: &Inventory) -> Vec<BodyPartType> {
    let filled = assign(inv);
    PART_ORDER
        .into_iter()
        .filter(|part| !filled.iter().any(|(p, _)| p == part))
        .collect()
}

/// A vat's state, for its window and the Resurrect screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VatStatus {
    /// No torso or no fusion core yet
    Incomplete,
    /// A powered vat whose body is complete, but that got no power last step
    Dormant,
    Ready,
}

impl VatStatus {
    pub fn label(self) -> &'static str {
        match self {
            VatStatus::Incomplete => "needs a torso and a fusion core",
            VatStatus::Dormant => "dormant: no power",
            VatStatus::Ready => "ready",
        }
    }
}

pub fn status(world: &World, vat: Entity) -> VatStatus {
    let complete = world
        .get::<&Inventory>(vat)
        .is_ok_and(|inv| has_core_parts(&inv));
    if !complete {
        return VatStatus::Incomplete;
    }
    match world.get::<&PowerConsumer>(vat) {
        Ok(consumer) if consumer.satisfaction_permille == 0 => VatStatus::Dormant,
        _ => VatStatus::Ready,
    }
}

/// Powered vats want power only while they hold a complete body.
pub fn update_demand(world: &mut World) {
    for (_, (block, inv, consumer)) in
        world.query_mut::<(&FactoryBlock, &Inventory, &mut PowerConsumer)>()
    {
        if !is_vat(block.kind) {
            continue;
        }
        consumer.active = has_core_parts(inv);
        if !consumer.active {
            consumer.satisfaction_permille = 0;
        }
    }
}

/// A vat the robot could wake in.
#[derive(Debug, Clone)]
pub struct ReadyVat {
    pub entity: Entity,
    pub kind: FactoryKind,
    pub position: Position,
    pub body: Body,
}

/// Every ready vat, nearest to `from` first, then by position.
pub fn ready_vats(world: &World, from: Position) -> Vec<ReadyVat> {
    let mut vats: Vec<ReadyVat> = world
        .query::<(&Position, &FactoryBlock, &Inventory)>()
        .iter()
        .filter(|(_, (_, block, _))| is_vat(block.kind))
        .map(|(entity, (pos, block, inv))| ReadyVat {
            entity,
            kind: block.kind,
            position: *pos,
            body: spare_body(inv),
        })
        .collect();
    vats.retain(|v| status(world, v.entity) == VatStatus::Ready);
    vats.sort_by_key(|v| {
        (
            distance(from, v.position),
            v.position.z,
            v.position.y,
            v.position.x,
        )
    });
    vats
}

/// Tiles between two positions, counting a layer as a tile.
pub fn distance(a: Position, b: Position) -> u64 {
    a.x.abs_diff(b.x)
        .max(a.y.abs_diff(b.y))
        .max(a.z.abs_diff(b.z))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::{self, Facing};
    use crate::model::body::BodyPartState;
    use crate::test_support::TestWorld;
    use crate::tiles::TileKind;
    use crate::Game;

    fn game() -> Game {
        TestWorld::new(3)
            .fill((-8, -8, 0), (8, 8, 0), TileKind::Grass)
            .build()
            .game
    }

    /// Let `n` factory steps pass.
    fn steps(game: &mut Game, n: u64) {
        game.res.player_state.intent =
            crate::intent::PlayerIntent::movement(0, 0, 0, n * crate::factory::FACTORY_STEP_TICKS);
        game.tick();
    }

    fn vat(game: &mut Game, kind: FactoryKind, x: i64) -> Entity {
        factory::place(
            &mut game.world,
            &mut game.res,
            kind,
            Position { x, y: 3, z: 0 },
            Facing::North,
        )
        .unwrap()
    }

    fn fill(game: &mut Game, vat: Entity, items: &[ItemKind]) {
        for &item in items {
            assert!(factory::insert(&mut game.world, vat, item), "{item:?}");
        }
    }

    #[test]
    fn a_crude_vat_takes_only_crude_parts_and_one_per_slot() {
        let mut g = game();
        let v = vat(&mut g, FactoryKind::CrudeVat, 0);
        assert!(!factory::accepts(&g.world, v, ItemKind::StandardTorso));
        assert!(!factory::accepts(&g.world, v, ItemKind::Log));
        fill(
            &mut g,
            v,
            &[ItemKind::CrudeTorso, ItemKind::CrudeArm, ItemKind::CrudeArm],
        );
        assert!(!factory::accepts(&g.world, v, ItemKind::CrudeTorso));
        assert!(!factory::accepts(&g.world, v, ItemKind::CrudeArm));
        assert!(factory::accepts(&g.world, v, ItemKind::CrudeLeg));
    }

    #[test]
    fn a_standard_vat_takes_crude_and_standard_parts_and_salvaged_arms() {
        let mut g = game();
        let v = vat(&mut g, FactoryKind::StandardVat, 0);
        assert!(!factory::accepts(&g.world, v, ItemKind::AdvancedLeg));
        fill(&mut g, v, &[ItemKind::SalvagedArm, ItemKind::CrudeArm]);
        assert!(!factory::accepts(&g.world, v, ItemKind::StandardArm));
        assert!(factory::accepts(&g.world, v, ItemKind::StandardLeg));
    }

    #[test]
    fn a_crude_vat_is_ready_with_a_torso_and_core_and_needs_no_power() {
        let mut g = game();
        let v = vat(&mut g, FactoryKind::CrudeVat, 0);
        fill(&mut g, v, &[ItemKind::CrudeTorso]);
        assert_eq!(status(&g.world, v), VatStatus::Incomplete);
        fill(&mut g, v, &[ItemKind::CrudeFusionCore, ItemKind::CrudeLeg]);
        assert_eq!(status(&g.world, v), VatStatus::Ready);
        let inv = g.world.get::<&Inventory>(v).unwrap();
        let body = spare_body(&inv);
        assert_eq!(
            body.parts[&BodyPartType::Torso].state,
            BodyPartState::Damaged
        );
        assert_eq!(body.parts[&BodyPartType::Torso].integrity, 40);
        assert_eq!(
            body.parts[&BodyPartType::LeftLeg].state,
            BodyPartState::Damaged
        );
        assert_eq!(
            body.parts[&BodyPartType::RightLeg].state,
            BodyPartState::Missing
        );
        assert_eq!(
            body.parts[&BodyPartType::Head].state,
            BodyPartState::Missing
        );
        assert_eq!(
            missing_parts(&inv),
            vec![
                BodyPartType::Head,
                BodyPartType::LeftArm,
                BodyPartType::RightArm,
                BodyPartType::RightLeg
            ]
        );
    }

    #[test]
    fn a_standard_vat_is_dormant_without_power_and_ready_with_it() {
        let mut g = game();
        let v = vat(&mut g, FactoryKind::StandardVat, 0);
        fill(
            &mut g,
            v,
            &[
                ItemKind::StandardTorso,
                ItemKind::StandardFusionCore,
                ItemKind::StandardLeg,
                ItemKind::StandardLeg,
            ],
        );
        steps(&mut g, 2);
        assert_eq!(status(&g.world, v), VatStatus::Dormant);
        assert!(ready_vats(&g.world, Position { x: 0, y: 0, z: 0 }).is_empty());

        // A fuelled generator beside it, and a pole in reach of both
        let pole = Position { x: 1, y: 2, z: 0 };
        factory::place(
            &mut g.world,
            &mut g.res,
            FactoryKind::PowerPole,
            pole,
            Facing::North,
        )
        .unwrap();
        let gen = factory::place(
            &mut g.world,
            &mut g.res,
            FactoryKind::BurnerGenerator,
            Position { x: 2, y: 3, z: 0 },
            Facing::North,
        )
        .unwrap();
        g.world
            .get::<&mut Inventory>(gen)
            .unwrap()
            .add(ItemKind::Biochar, 4);
        steps(&mut g, 4);
        assert_eq!(status(&g.world, v), VatStatus::Ready);
        let ready = ready_vats(&g.world, Position { x: 0, y: 0, z: 0 });
        assert_eq!(ready.len(), 1);
        let legs = &ready[0].body.parts[&BodyPartType::RightLeg];
        assert_eq!(
            (legs.state, legs.integrity),
            (BodyPartState::Functional, 100)
        );
    }

    #[test]
    fn advanced_parts_wake_enhanced() {
        let mut g = game();
        let v = vat(&mut g, FactoryKind::AdvancedVat, 0);
        fill(&mut g, v, &[ItemKind::AdvancedArm, ItemKind::CrudeArm]);
        let inv = g.world.get::<&Inventory>(v).unwrap();
        let body = spare_body(&inv);
        assert_eq!(
            body.parts[&BodyPartType::LeftArm].state,
            BodyPartState::Enhanced
        );
        assert_eq!(
            body.parts[&BodyPartType::RightArm].state,
            BodyPartState::Damaged
        );
    }

    #[test]
    fn ready_vats_are_listed_nearest_first() {
        let mut g = game();
        let far = vat(&mut g, FactoryKind::CrudeVat, 6);
        let near = vat(&mut g, FactoryKind::CrudeVat, -2);
        for v in [far, near] {
            fill(
                &mut g,
                v,
                &[ItemKind::CrudeTorso, ItemKind::CrudeFusionCore],
            );
        }
        let order: Vec<Entity> = ready_vats(&g.world, Position { x: 0, y: 0, z: 0 })
            .iter()
            .map(|v| v.entity)
            .collect();
        assert_eq!(order, vec![near, far]);
    }
}

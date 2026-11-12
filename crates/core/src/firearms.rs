//! Guns: one weapon slot per working arm, and the Shoot move that fires every equipped,
//! loaded gun in one turn. Shots follow the same rules as enemy shots
//! (`systems::ranged::fire`): range, line of sight, and a hit chance that falls off with
//! distance, rolled from the world's seeded RNG.

use crate::components::{Inventory, ItemKind, Position};
use crate::model::body::{Body, BodyPartState, BodyPartType};
use crate::resources::Resources;
use crate::systems::ranged::{fire, hit_chance, tile_distance, Shot};
use hecs::{Entity, World};

/// What a gun fires.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ammo {
    /// One of this item per round
    Item(ItemKind),
    /// This many joules per round, from the fullest carried power cell
    Charge(u64),
}

/// One kind of gun and how it shoots. A Shoot fires `burst` rounds; each round fires
/// `pellets` pellets, and each pellet rolls its own hit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gun {
    pub item: ItemKind,
    pub ammo: Ammo,
    pub range: i64,
    /// Percent chance to hit from 1 tile away
    pub accuracy_at_1: u32,
    /// Percent lost per tile beyond the first
    pub falloff: u32,
    pub damage: (u32, u32),
    /// Combat ticks a shot takes
    pub ticks: u64,
    /// What firing one round leaves in the inventory
    pub casing: Option<ItemKind>,
    pub pellets: u32,
    pub burst: u32,
    /// Percent chance a hit stuns
    pub stun_percent: u32,
    /// The shot rolls against every enemy on its line out to its range
    pub pierce: bool,
    /// Damage a hit also deals to every other enemy beside the target
    pub splash: Option<(u32, u32)>,
}

/// How long a stunning hit stuns, in ticks (as long as Tackle's stun).
pub const STUN_TICKS: u64 = 600;

const BASE: Gun = Gun {
    item: ItemKind::PipeGun,
    ammo: Ammo::Item(ItemKind::PaperCartridge),
    range: 6,
    accuracy_at_1: 70,
    falloff: 8,
    damage: (10, 18),
    ticks: 12,
    casing: None,
    pellets: 1,
    burst: 1,
    stun_percent: 0,
    pierce: false,
    splash: None,
};

pub const PIPE_GUN: Gun = BASE;

pub const RIFLE: Gun = Gun {
    item: ItemKind::Rifle,
    ammo: Ammo::Item(ItemKind::BrassCartridge),
    range: 12,
    accuracy_at_1: 90,
    falloff: 4,
    damage: (16, 26),
    ticks: 10,
    casing: Some(ItemKind::ShellCasing),
    ..BASE
};

pub const NAIL_DRIVER: Gun = Gun {
    item: ItemKind::NailDriver,
    ammo: Ammo::Item(ItemKind::Nail),
    range: 4,
    accuracy_at_1: 65,
    falloff: 10,
    damage: (5, 9),
    ticks: 8,
    ..BASE
};

pub const SCATTERGUN: Gun = Gun {
    item: ItemKind::Scattergun,
    ammo: Ammo::Item(ItemKind::ScatterShell),
    range: 3,
    accuracy_at_1: 75,
    falloff: 20,
    damage: (4, 7),
    ticks: 12,
    pellets: 5,
    ..BASE
};

pub const BURST_CARBINE: Gun = Gun {
    item: ItemKind::BurstCarbine,
    ammo: Ammo::Item(ItemKind::BrassCartridge),
    range: 8,
    accuracy_at_1: 75,
    falloff: 6,
    damage: (10, 16),
    ticks: 12,
    casing: Some(ItemKind::ShellCasing),
    burst: 3,
    ..BASE
};

pub const SLUG_THROWER: Gun = Gun {
    item: ItemKind::SlugThrower,
    ammo: Ammo::Item(ItemKind::BrassSlug),
    range: 5,
    accuracy_at_1: 60,
    falloff: 10,
    damage: (24, 34),
    ticks: 16,
    casing: Some(ItemKind::ShellCasing),
    stun_percent: 30,
    ..BASE
};

pub const ARC_CASTER: Gun = Gun {
    item: ItemKind::ArcCaster,
    ammo: Ammo::Charge(1_000_000),
    range: 8,
    accuracy_at_1: 85,
    falloff: 5,
    damage: (18, 28),
    ticks: 10,
    ..BASE
};

pub const RIVERGLASS_LANCE: Gun = Gun {
    item: ItemKind::RiverglassLance,
    ammo: Ammo::Charge(4_000_000),
    range: 14,
    accuracy_at_1: 95,
    falloff: 2,
    damage: (30, 40),
    ticks: 20,
    pierce: true,
    ..BASE
};

pub const ISOTOPE_LAUNCHER: Gun = Gun {
    item: ItemKind::IsotopeLauncher,
    ammo: Ammo::Item(ItemKind::IsotopeCanister),
    range: 7,
    accuracy_at_1: 80,
    falloff: 6,
    damage: (30, 45),
    ticks: 18,
    splash: Some((15, 25)),
    ..BASE
};

/// Every gun, from the earliest to the latest.
pub const GUNS: &[Gun] = &[
    NAIL_DRIVER,
    PIPE_GUN,
    SCATTERGUN,
    RIFLE,
    BURST_CARBINE,
    SLUG_THROWER,
    ARC_CASTER,
    RIVERGLASS_LANCE,
    ISOTOPE_LAUNCHER,
];

/// A Damaged arm shoots at this share of its gun's accuracy, in percent.
pub const DAMAGED_ARM_ACCURACY: u32 = 75;

/// The gun an item is, if it is one.
pub fn gun(item: ItemKind) -> Option<&'static Gun> {
    GUNS.iter().find(|g| g.item == item)
}

/// The arms in the order their guns fire.
pub const ARMS: [BodyPartType; 2] = [BodyPartType::LeftArm, BodyPartType::RightArm];

/// Whether an arm can hold a gun: it is there.
fn arm_works(body: &Body, arm: BodyPartType) -> bool {
    body.parts
        .get(&arm)
        .is_some_and(|p| p.state != BodyPartState::Missing)
}

/// Why a gun couldn't be equipped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipError {
    NotAGun,
    NotCarried,
    /// Every working arm already holds a gun
    NoFreeArm,
}

impl std::fmt::Display for EquipError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            EquipError::NotAGun => "That isn't a gun.",
            EquipError::NotCarried => "You don't carry one.",
            EquipError::NoFreeArm => "Every working arm already holds a gun.",
        })
    }
}

/// Equip a carried gun into the first working arm with an empty slot; returns the arm.
pub fn equip(
    world: &mut World,
    entity: Entity,
    item: ItemKind,
) -> Result<BodyPartType, EquipError> {
    gun(item).ok_or(EquipError::NotAGun)?;
    let arm = {
        let body = world
            .get::<&Body>(entity)
            .map_err(|_| EquipError::NoFreeArm)?;
        ARMS.into_iter()
            .find(|&arm| arm_works(&body, arm) && !body.weapons.contains_key(&arm))
            .ok_or(EquipError::NoFreeArm)?
    };
    equip_into(world, entity, arm, item).map(|_| arm)
}

/// Equip a carried gun into `arm`, which must be working; a gun it already held goes back
/// into the inventory, and is returned.
pub fn equip_into(
    world: &mut World,
    entity: Entity,
    arm: BodyPartType,
    item: ItemKind,
) -> Result<Option<ItemKind>, EquipError> {
    gun(item).ok_or(EquipError::NotAGun)?;
    let works = world
        .get::<&Body>(entity)
        .is_ok_and(|b| ARMS.contains(&arm) && arm_works(&b, arm));
    if !works {
        return Err(EquipError::NoFreeArm);
    }
    let taken = world
        .get::<&mut Inventory>(entity)
        .map(|mut inv| inv.remove(item, 1))
        .unwrap_or(0);
    if taken == 0 {
        return Err(EquipError::NotCarried);
    }
    let old = world
        .get::<&mut Body>(entity)
        .ok()
        .and_then(|mut body| body.weapons.insert(arm, item));
    if let Some(old) = old {
        give_or_drop(world, entity, old);
    }
    Ok(old)
}

/// The best gun carried (the rifle before the pipe gun), if any.
pub fn best_carried_gun(world: &World, entity: Entity) -> Option<ItemKind> {
    let inv = world.get::<&Inventory>(entity).ok()?;
    GUNS.iter()
        .rev()
        .map(|g| g.item)
        .find(|&item| inv.count(item) > 0)
}

/// Take the gun out of an arm's slot and back into the inventory (onto the ground when it
/// is full). Returns the gun.
pub fn unequip(world: &mut World, entity: Entity, arm: BodyPartType) -> Option<ItemKind> {
    let item = world.get::<&mut Body>(entity).ok()?.weapons.remove(&arm)?;
    give_or_drop(world, entity, item);
    Some(item)
}

/// Guns held by arms that are now missing go into the inventory, or onto the ground when
/// it is full. Called when a part is destroyed.
pub fn release_lost_weapons(world: &mut World, res: &mut Resources, entity: Entity) {
    let lost: Vec<ItemKind> = {
        let Ok(mut body) = world.get::<&mut Body>(entity) else {
            return;
        };
        let missing: Vec<BodyPartType> = body
            .weapons
            .keys()
            .copied()
            .filter(|&arm| !arm_works(&body, arm))
            .collect();
        missing
            .into_iter()
            .filter_map(|arm| body.weapons.remove(&arm))
            .collect()
    };
    for item in lost {
        give_or_drop(world, entity, item);
        res.events.combat_event(
            format!(
                "Your {} falls from the lost arm.",
                crate::components::itemkind_name(item)
            ),
            res.time.tick,
        );
    }
}

fn give_or_drop(world: &mut World, entity: Entity, item: ItemKind) {
    let fits = world
        .get::<&mut Inventory>(entity)
        .map(|mut inv| inv.add_stack(crate::components::ItemStack::new(item, 1)))
        .unwrap_or(false);
    if !fits {
        if let Ok(pos) = world.get::<&Position>(entity).map(|p| *p) {
            crate::factory::spawn_drop(world, pos, crate::components::ItemStack::new(item, 1));
        }
    }
}

/// The equipped guns with ammo carried for them, in firing order, with the arm that holds
/// each. Two guns firing the same rounds need one each; guns on charge need the fullest
/// cell to cover a round for every one of them.
pub fn loaded_guns(world: &World, entity: Entity) -> Vec<(BodyPartType, &'static Gun)> {
    let (Ok(body), Ok(inv)) = (world.get::<&Body>(entity), world.get::<&Inventory>(entity)) else {
        return vec![];
    };
    let charge = crate::power_cells::fullest_charge(world, entity);
    let mut rounds: std::collections::BTreeMap<ItemKind, u32> = Default::default();
    let mut joules = 0u64;
    ARMS.into_iter()
        .filter(|arm| arm_works(&body, *arm))
        .filter_map(|arm| Some((arm, gun(*body.weapons.get(&arm)?)?)))
        .filter(|(_, g)| match g.ammo {
            Ammo::Item(item) => {
                let used = rounds.entry(item).or_insert(0);
                *used += 1;
                inv.count(item) >= *used
            }
            Ammo::Charge(j) => {
                joules += j;
                charge >= joules
            }
        })
        .collect()
}

/// Rounds of `gun` the entity can fire now: items carried, or whole rounds in the fullest
/// cell.
pub fn rounds_left(world: &World, entity: Entity, gun: &Gun) -> u32 {
    match gun.ammo {
        Ammo::Item(item) => world
            .get::<&Inventory>(entity)
            .map_or(0, |inv| inv.count(item)),
        Ammo::Charge(j) => (crate::power_cells::fullest_charge(world, entity) / j.max(1)) as u32,
    }
}

/// Use one round of `gun`'s ammo, leaving its casing; false with none left.
fn use_round(world: &mut World, entity: Entity, gun: &Gun) -> bool {
    match gun.ammo {
        Ammo::Item(item) => {
            let Ok(mut inv) = world.get::<&mut Inventory>(entity) else {
                return false;
            };
            if inv.remove(item, 1) == 0 {
                return false;
            }
            if let Some(casing) = gun.casing {
                inv.add(casing, 1);
            }
            true
        }
        Ammo::Charge(j) => crate::power_cells::use_power(world, entity, j).is_ok(),
    }
}

/// The Shoot move for this many guns: as slow as the slowest of them.
pub fn shoot_move(guns: &[(BodyPartType, &'static Gun)]) -> crate::moves::Move {
    let ticks = guns.iter().map(|(_, g)| g.ticks).max().unwrap_or(10);
    let n = guns.len();
    crate::moves::Move {
        name: if n > 1 {
            format!("Shoot ({n} guns)")
        } else {
            "Shoot".to_string()
        },
        move_type: crate::moves::MoveType::Shoot,
        energy_cost: 5,
        execution_time_ticks: ticks,
        splash_radius: None,
        damage: 0,
        description: "Fire every equipped, loaded gun".to_string(),
    }
}

/// A gun's chance to hit from `distance` tiles, held by an arm in `state`.
pub fn gun_hit_chance(gun: &Gun, distance: i64, state: BodyPartState) -> u32 {
    let chance = hit_chance(gun.accuracy_at_1, gun.falloff, distance);
    if state == BodyPartState::Damaged {
        chance * DAMAGED_ARM_ACCURACY / 100
    } else {
        chance
    }
}

/// Fire every equipped, loaded gun at `target`, in arm order. Each gun fires its burst,
/// using a round per shot (bursts stop when rounds run out), and every pellet rolls its
/// own hit; a piercing shot rolls against every enemy on its line, and a hit can stun or
/// splash by the gun's traits.
pub fn execute_shoot(world: &mut World, res: &mut Resources, shooter: Entity, target: Entity) {
    let guns = loaded_guns(world, shooter);
    if guns.is_empty() {
        res.events
            .combat_event("Click. Nothing to shoot with.", res.time.tick);
        return;
    }
    for (arm, gun) in guns {
        let state = world
            .get::<&Body>(shooter)
            .ok()
            .and_then(|b| b.parts.get(&arm).map(|p| p.state))
            .unwrap_or(BodyPartState::Functional);
        for _ in 0..gun.burst {
            // The target can fall to an earlier shot this turn
            if world.get::<&crate::components::Dead>(target).is_ok() {
                return;
            }
            if !use_round(world, shooter, gun) {
                break;
            }
            fire_round(world, res, shooter, target, gun, state);
        }
    }
}

/// Living creatures in the fight that a shot can strike: not the robot.
fn is_enemy(world: &World, entity: Entity) -> bool {
    world.get::<&crate::components::Combat>(entity).is_ok()
        && world.get::<&crate::components::Dead>(entity).is_err()
        && world.get::<&crate::components::Player>(entity).is_err()
}

/// The enemies a piercing shot from `from` toward `to` passes, nearest first: along the
/// line out to `range`, stopping at the first tile that blocks sight.
fn pierced(
    world: &World,
    res: &Resources,
    from: Position,
    to: Position,
    range: i64,
) -> Vec<Entity> {
    use crate::systems::fog_of_war::{blocks_sight, bresenham_line};
    let (dx, dy) = (to.x - from.x, to.y - from.y);
    let steps = dx.abs().max(dy.abs()).max(1);
    let scale = (range + steps - 1) / steps;
    let end = (from.x + dx * scale, from.y + dy * scale);
    let mut hit = Vec::new();
    for (x, y) in bresenham_line(from.x, from.y, end.0, end.1)
        .into_iter()
        .skip(1)
    {
        let p = Position { x, y, z: from.z };
        if tile_distance(from, p) > range || blocks_sight(world, res, x, y, from.z) {
            break;
        }
        for (e, q) in world.query::<&Position>().iter() {
            if *q == p && is_enemy(world, e) {
                hit.push(e);
            }
        }
    }
    hit
}

/// One round of `gun` at `target`: its pellets, or its piercing line.
fn fire_round(
    world: &mut World,
    res: &mut Resources,
    shooter: Entity,
    target: Entity,
    gun: &'static Gun,
    state: BodyPartState,
) {
    let (Ok(from), Ok(to)) = (
        world.get::<&Position>(shooter).map(|p| *p),
        world.get::<&Position>(target).map(|p| *p),
    ) else {
        return;
    };
    let name = crate::components::itemkind_name(gun.item);
    let targets = if gun.pierce {
        pierced(world, res, from, to, gun.range)
    } else {
        vec![target]
    };
    if targets.is_empty() {
        // Nothing on the line before a wall: the beam still fires and is spent
        fire(world, res, from, to, gun.range, 0);
        res.events
            .combat_event(format!("Your {name} shot hits the wall."), res.time.tick);
        return;
    }
    let mut hits = 0;
    for victim in targets {
        for _ in 0..gun.pellets {
            if world.get::<&crate::components::Dead>(victim).is_ok() {
                break;
            }
            let Ok(at) = world.get::<&Position>(victim).map(|p| *p) else {
                break;
            };
            let chance = gun_hit_chance(gun, tile_distance(from, at), state);
            let shot = fire(world, res, from, at, gun.range, chance);
            if shot != Shot::Hit {
                if gun.pellets == 1 {
                    res.events.combat_event(
                        format!("Your {name} shot {}.", shot.miss_words()),
                        res.time.tick,
                    );
                }
                continue;
            }
            hits += 1;
            strike(world, res, victim, at, gun, name);
        }
    }
    if gun.pellets > 1 {
        res.events.combat_event(
            format!("Your {name}: {hits} of {} pellets hit.", gun.pellets),
            res.time.tick,
        );
    }
}

/// A hit on `victim`: its damage, then a chance to stun, then splash on every other enemy
/// beside it.
fn strike(
    world: &mut World,
    res: &mut Resources,
    victim: Entity,
    at: Position,
    gun: &Gun,
    name: &str,
) {
    use rand::Rng;
    let damage = res.world_state.rng.gen_range(gun.damage.0..=gun.damage.1);
    crate::systems::combat_actions::apply_damage(
        world,
        res,
        victim,
        damage,
        &format!("your {name}"),
    );
    if gun.stun_percent > 0
        && res.world_state.rng.gen_range(0..100) < gun.stun_percent
        && world.get::<&crate::components::Dead>(victim).is_err()
    {
        world
            .insert_one(victim, crate::components::Stunned::new(STUN_TICKS))
            .ok();
        res.events
            .combat_event(format!("Your {name} stuns it!"), res.time.tick);
    }
    if let Some((low, high)) = gun.splash {
        let mut beside: Vec<(Position, Entity)> = world
            .query::<&Position>()
            .iter()
            .filter(|&(e, p)| {
                e != victim && p.z == at.z && tile_distance(*p, at) <= 1 && is_enemy(world, e)
            })
            .map(|(e, p)| (*p, e))
            .collect();
        beside.sort_by_key(|(p, e)| (p.y, p.x, e.to_bits()));
        for (_, e) in beside {
            let splash = res.world_state.rng.gen_range(low..=high);
            crate::systems::combat_actions::apply_damage(
                world,
                res,
                e,
                splash,
                &format!("your {name}'s blast"),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Health;
    use crate::model::body::BodyPart;
    use crate::test_support::{Handle, TestGame, TestWorld};
    use crate::tiles::TileKind;

    fn player(game: &TestGame) -> Entity {
        game.game.get_player_entity().unwrap()
    }

    /// The player at the origin carrying `items`, and a feral dog `x` tiles east.
    fn range(items: &[(ItemKind, u32)], x: i64) -> (TestGame, Handle) {
        let mut world = TestWorld::new(7)
            .fill((-12, -4, 0), (14, 4, 0), TileKind::Grass)
            .inventory(items);
        let dog = world.feral_dog(x, 0, 0);
        (world.build(), dog)
    }

    fn give_left_arm(game: &mut TestGame) {
        let p = player(game);
        let mut body = game.game.world.get::<&mut Body>(p).unwrap();
        let arm: &mut BodyPart = body.parts.get_mut(&BodyPartType::LeftArm).unwrap();
        arm.integrity = 100;
        arm.update_state_from_integrity();
    }

    fn weapons(game: &TestGame) -> Vec<(BodyPartType, ItemKind)> {
        let body = game.game.world.get::<&Body>(player(game)).unwrap();
        body.weapons.iter().map(|(a, g)| (*a, *g)).collect()
    }

    #[test]
    fn the_starting_robot_holds_one_gun_in_its_right_arm() {
        let (mut game, _) = range(&[(ItemKind::PipeGun, 2)], 3);
        let p = player(&game);
        assert_eq!(
            equip(&mut game.game.world, p, ItemKind::PipeGun),
            Ok(BodyPartType::RightArm)
        );
        assert_eq!(
            equip(&mut game.game.world, p, ItemKind::PipeGun),
            Err(EquipError::NoFreeArm),
            "the left arm is missing"
        );
        assert_eq!(
            weapons(&game),
            vec![(BodyPartType::RightArm, ItemKind::PipeGun)]
        );
        assert_eq!(game.player_has(ItemKind::PipeGun), 1);
    }

    #[test]
    fn equipping_into_a_full_arm_swaps_the_guns() {
        let (mut game, _) = range(&[(ItemKind::PipeGun, 1), (ItemKind::Rifle, 1)], 3);
        let p = player(&game);
        equip(&mut game.game.world, p, ItemKind::PipeGun).unwrap();
        assert_eq!(
            equip_into(
                &mut game.game.world,
                p,
                BodyPartType::RightArm,
                ItemKind::Rifle
            ),
            Ok(Some(ItemKind::PipeGun))
        );
        assert_eq!(
            weapons(&game),
            vec![(BodyPartType::RightArm, ItemKind::Rifle)]
        );
        assert_eq!(game.player_has(ItemKind::PipeGun), 1);
        assert_eq!(
            equip_into(
                &mut game.game.world,
                p,
                BodyPartType::LeftArm,
                ItemKind::PipeGun
            ),
            Err(EquipError::NoFreeArm),
            "a missing arm holds nothing"
        );
    }

    #[test]
    fn unequipping_puts_the_gun_back() {
        let (mut game, _) = range(&[(ItemKind::Rifle, 1)], 3);
        let p = player(&game);
        equip(&mut game.game.world, p, ItemKind::Rifle).unwrap();
        assert_eq!(game.player_has(ItemKind::Rifle), 0);
        assert_eq!(
            unequip(&mut game.game.world, p, BodyPartType::RightArm),
            Some(ItemKind::Rifle)
        );
        assert_eq!(game.player_has(ItemKind::Rifle), 1);
        assert!(weapons(&game).is_empty());
    }

    #[test]
    fn a_destroyed_arm_drops_its_gun_into_the_inventory() {
        let (mut game, _) = range(&[(ItemKind::PipeGun, 1)], 3);
        let p = player(&game);
        equip(&mut game.game.world, p, ItemKind::PipeGun).unwrap();
        {
            let mut body = game.game.world.get::<&mut Body>(p).unwrap();
            body.parts
                .get_mut(&BodyPartType::RightArm)
                .unwrap()
                .receive_damage(500, true);
        }
        release_lost_weapons(&mut game.game.world, &mut game.game.res, p);
        assert!(weapons(&game).is_empty());
        assert_eq!(game.player_has(ItemKind::PipeGun), 1);
    }

    #[test]
    fn removing_an_arm_takes_its_gun_with_it() {
        let (mut game, _) = range(&[(ItemKind::PipeGun, 1)], 3);
        let p = player(&game);
        equip(&mut game.game.world, p, ItemKind::PipeGun).unwrap();
        let items = game
            .game
            .world
            .get::<&mut Body>(p)
            .unwrap()
            .remove(BodyPartType::RightArm)
            .unwrap();
        assert!(items.contains(&(ItemKind::PipeGun, 1)));
    }

    #[test]
    fn a_wreck_keeps_the_guns_in_its_arms() {
        let (mut game, _) = range(&[(ItemKind::Rifle, 1)], 3);
        let p = player(&game);
        equip(&mut game.game.world, p, ItemKind::Rifle).unwrap();
        crate::death::player_died(&mut game.game.world, &mut game.game.res, p, "test".into());
        let wreck_has_rifle = game
            .game
            .world
            .query::<&Body>()
            .with::<&crate::components::Wreck>()
            .iter()
            .any(|(_, b)| b.weapons.values().any(|g| *g == ItemKind::Rifle));
        assert!(wreck_has_rifle);
    }

    #[test]
    fn hit_chances_follow_each_guns_numbers() {
        use BodyPartState::*;
        assert_eq!(gun_hit_chance(&PIPE_GUN, 1, Functional), 70);
        assert_eq!(gun_hit_chance(&PIPE_GUN, 3, Functional), 54);
        assert_eq!(gun_hit_chance(&RIFLE, 3, Functional), 82);
        assert_eq!(gun_hit_chance(&PIPE_GUN, 3, Damaged), 40);
        assert!(gun_hit_chance(&RIFLE, 10, Functional) > gun_hit_chance(&PIPE_GUN, 6, Functional));
    }

    #[test]
    fn shoot_is_offered_only_with_a_loaded_gun() {
        let (mut game, _) = range(&[(ItemKind::PipeGun, 1)], 3);
        let p = player(&game);
        equip(&mut game.game.world, p, ItemKind::PipeGun).unwrap();
        let has_shoot = |game: &TestGame| {
            crate::moves::player_moves(&game.game.world, p, false)
                .iter()
                .any(|m| m.move_type == crate::moves::MoveType::Shoot)
        };
        assert!(!has_shoot(&game), "no paper cartridges, no Shoot");
        game.game
            .world
            .get::<&mut Inventory>(p)
            .unwrap()
            .add(ItemKind::PaperCartridge, 3);
        assert!(has_shoot(&game));
    }

    fn shoot_once(game: &mut TestGame, target: Entity) {
        let p = player(game);
        execute_shoot(&mut game.game.world, &mut game.game.res, p, target);
    }

    #[test]
    fn a_shot_uses_one_cartridge_and_rolls_by_the_pipe_guns_chance() {
        let (mut game, dog) = range(&[(ItemKind::PipeGun, 1), (ItemKind::PaperCartridge, 3)], 3);
        let p = player(&game);
        equip(&mut game.game.world, p, ItemKind::PipeGun).unwrap();
        let target = game.entity(dog);
        shoot_once(&mut game, target);
        assert_eq!(game.player_has(ItemKind::PaperCartridge), 2);
        assert_eq!(game.player_has(ItemKind::ShellCasing), 0, "paper burns up");
        assert_eq!(game.game.res.shots.len(), 1);
    }

    #[test]
    fn hits_come_at_about_the_guns_chance() {
        let (mut game, dog) = range(
            &[(ItemKind::PipeGun, 1), (ItemKind::PaperCartridge, 400)],
            3,
        );
        let p = player(&game);
        equip(&mut game.game.world, p, ItemKind::PipeGun).unwrap();
        let target = game.entity(dog);
        let mut hits = 0;
        for _ in 0..400 {
            game.game.world.get::<&mut Health>(target).unwrap().current = 10_000;
            game.game.res.shots.clear();
            shoot_once(&mut game, target);
            hits += game.game.res.shots.iter().filter(|s| s.hit).count();
        }
        // 54% at 3 tiles
        assert!((180..=250).contains(&hits), "{hits} hits in 400");
    }

    #[test]
    fn two_arms_fire_two_guns_in_one_turn() {
        let (mut game, dog) = range(&[(ItemKind::PipeGun, 2), (ItemKind::PaperCartridge, 5)], 3);
        give_left_arm(&mut game);
        let p = player(&game);
        equip(&mut game.game.world, p, ItemKind::PipeGun).unwrap();
        equip(&mut game.game.world, p, ItemKind::PipeGun).unwrap();
        let guns = loaded_guns(&game.game.world, p);
        assert_eq!(guns.len(), 2);
        assert_eq!(shoot_move(&guns).name, "Shoot (2 guns)");
        let target = game.entity(dog);
        game.game.world.get::<&mut Health>(target).unwrap().current = 10_000;
        shoot_once(&mut game, target);
        assert_eq!(game.game.res.shots.len(), 2, "each gun fired");
        assert_eq!(game.player_has(ItemKind::PaperCartridge), 3);
    }

    #[test]
    fn two_guns_need_a_round_each() {
        let (mut game, _) = range(&[(ItemKind::PipeGun, 2), (ItemKind::PaperCartridge, 1)], 3);
        give_left_arm(&mut game);
        let p = player(&game);
        equip(&mut game.game.world, p, ItemKind::PipeGun).unwrap();
        equip(&mut game.game.world, p, ItemKind::PipeGun).unwrap();
        assert_eq!(loaded_guns(&game.game.world, p).len(), 1);
    }

    #[test]
    fn brass_rounds_leave_casings() {
        let (mut game, dog) = range(&[(ItemKind::Rifle, 1), (ItemKind::BrassCartridge, 2)], 3);
        let p = player(&game);
        equip(&mut game.game.world, p, ItemKind::Rifle).unwrap();
        let target = game.entity(dog);
        for _ in 0..2 {
            game.game.world.get::<&mut Health>(target).unwrap().current = 10_000;
            shoot_once(&mut game, target);
        }
        assert_eq!(game.player_has(ItemKind::BrassCartridge), 0);
        assert_eq!(game.player_has(ItemKind::ShellCasing), 2);
    }

    #[test]
    fn shooting_replays_the_same_for_the_same_seed() {
        let run = || {
            let (mut game, dog) = range(&[(ItemKind::Rifle, 1), (ItemKind::BrassCartridge, 6)], 5);
            let p = player(&game);
            equip(&mut game.game.world, p, ItemKind::Rifle).unwrap();
            let target = game.entity(dog);
            for _ in 0..6 {
                game.game.world.get::<&mut Health>(target).unwrap().current = 10_000;
                shoot_once(&mut game, target);
            }
            game.game.res.shots.clone()
        };
        assert_eq!(run(), run());
    }

    #[test]
    fn a_wall_stops_the_shot() {
        let (game, dog) = range(&[(ItemKind::Rifle, 1), (ItemKind::BrassCartridge, 1)], 4);
        let mut game = {
            let mut g = game;
            for y in -4..=4 {
                g.game
                    .res
                    .world_state
                    .world
                    .set_tile_cached(2, y, 0, TileKind::Rock);
            }
            g
        };
        let p = player(&game);
        equip(&mut game.game.world, p, ItemKind::Rifle).unwrap();
        let target = game.entity(dog);
        shoot_once(&mut game, target);
        assert!(!game.game.res.shots[0].hit);
    }

    /// The player carrying `items` with `gun` in the right arm, and dogs at `dogs`.
    fn armed(
        gun: ItemKind,
        items: &[(ItemKind, u32)],
        dogs: &[(i64, i64)],
    ) -> (TestGame, Vec<Handle>) {
        let mut world = TestWorld::new(7)
            .fill((-12, -4, 0), (16, 4, 0), TileKind::Grass)
            .inventory(items);
        let handles = dogs
            .iter()
            .map(|&(x, y)| world.feral_dog(x, y, 0))
            .collect();
        let mut game = world.build();
        let p = player(&game);
        game.game
            .world
            .get::<&mut Inventory>(p)
            .unwrap()
            .add(gun, 1);
        equip(&mut game.game.world, p, gun).unwrap();
        (game, handles)
    }

    fn tough(game: &mut TestGame, h: Handle) -> Entity {
        let e = game.entity(h);
        game.game.world.get::<&mut Health>(e).unwrap().current = 100_000;
        e
    }

    fn give_cell(game: &mut TestGame, kind: ItemKind, joules: u64) {
        let p = player(game);
        let mut stack = crate::components::ItemStack::new(kind, 1);
        stack.charge_j = Some(joules);
        game.game
            .world
            .get::<&mut Inventory>(p)
            .unwrap()
            .add_stack(stack);
    }

    #[test]
    fn each_new_gun_has_its_odds_at_one_and_three_tiles() {
        use BodyPartState::Functional as F;
        let expect = [
            (NAIL_DRIVER, 65, 45),
            (SCATTERGUN, 75, 35),
            (BURST_CARBINE, 75, 63),
            (SLUG_THROWER, 60, 40),
            (ARC_CASTER, 85, 75),
            (RIVERGLASS_LANCE, 95, 91),
            (ISOTOPE_LAUNCHER, 80, 68),
        ];
        for (g, at1, at3) in expect {
            assert_eq!(gun_hit_chance(&g, 1, F), at1, "{:?} at 1", g.item);
            assert_eq!(gun_hit_chance(&g, 3, F), at3, "{:?} at 3", g.item);
        }
    }

    #[test]
    fn every_scatter_pellet_rolls_its_own_hit() {
        let (mut game, dogs) = armed(
            ItemKind::Scattergun,
            &[(ItemKind::ScatterShell, 2)],
            &[(1, 0)],
        );
        let dog = tough(&mut game, dogs[0]);
        shoot_once(&mut game, dog);
        assert_eq!(
            game.player_has(ItemKind::ScatterShell),
            1,
            "one shell for five pellets"
        );
        assert_eq!(game.game.res.shots.len(), 5);
    }

    #[test]
    fn a_burst_fires_three_rounds_and_a_short_burst_fewer() {
        let (mut game, dogs) = armed(
            ItemKind::BurstCarbine,
            &[(ItemKind::BrassCartridge, 5)],
            &[(3, 0)],
        );
        let dog = tough(&mut game, dogs[0]);
        shoot_once(&mut game, dog);
        assert_eq!(game.player_has(ItemKind::BrassCartridge), 2);
        assert_eq!(game.player_has(ItemKind::ShellCasing), 3);
        assert_eq!(game.game.res.shots.len(), 3);
        game.game.res.shots.clear();
        shoot_once(&mut game, dog);
        assert_eq!(game.game.res.shots.len(), 2, "only two rounds left");
        assert_eq!(game.player_has(ItemKind::BrassCartridge), 0);
    }

    #[test]
    fn slugs_stun_about_three_hits_in_ten() {
        let (mut game, dogs) = armed(
            ItemKind::SlugThrower,
            &[(ItemKind::BrassSlug, 600)],
            &[(1, 0)],
        );
        let dog = tough(&mut game, dogs[0]);
        let (mut hits, mut stuns) = (0, 0);
        for _ in 0..600 {
            game.game
                .world
                .remove_one::<crate::components::Stunned>(dog)
                .ok();
            game.game.res.shots.clear();
            shoot_once(&mut game, dog);
            if game.game.res.shots.iter().any(|s| s.hit) {
                hits += 1;
                stuns += game
                    .game
                    .world
                    .get::<&crate::components::Stunned>(dog)
                    .is_ok() as u32;
            }
        }
        let share = stuns * 100 / hits;
        assert!((22..=38).contains(&share), "{stuns} stuns in {hits} hits");
    }

    #[test]
    fn the_arc_caster_draws_charge_and_isnt_offered_on_flat_cells() {
        let (mut game, dogs) = armed(ItemKind::ArcCaster, &[], &[(3, 0)]);
        let p = player(&game);
        give_cell(&mut game, ItemKind::CrudePowerCell, 0);
        assert!(
            loaded_guns(&game.game.world, p).is_empty(),
            "flat cell, no Shoot"
        );
        assert!(!crate::moves::player_moves(&game.game.world, p, false)
            .iter()
            .any(|m| m.move_type == crate::moves::MoveType::Shoot));
        give_cell(&mut game, ItemKind::CrudePowerCell, 2_000_000);
        assert_eq!(rounds_left(&game.game.world, p, &ARC_CASTER), 2);
        let dog = tough(&mut game, dogs[0]);
        shoot_once(&mut game, dog);
        assert_eq!(
            crate::power_cells::fullest_charge(&game.game.world, p),
            1_000_000
        );
    }

    #[test]
    fn the_lance_rolls_against_every_dog_on_its_line_until_a_wall() {
        let (mut game, dogs) = armed(
            ItemKind::RiverglassLance,
            &[],
            &[(2, 0), (5, 0), (9, 0), (4, 2)],
        );
        give_cell(&mut game, ItemKind::StandardPowerCell, 20_000_000);
        for y in -4..=4 {
            game.game
                .res
                .world_state
                .world
                .set_tile_cached(7, y, 0, TileKind::Rock);
        }
        let near = tough(&mut game, dogs[0]);
        tough(&mut game, dogs[1]);
        tough(&mut game, dogs[2]);
        tough(&mut game, dogs[3]);
        shoot_once(&mut game, near);
        let ends: Vec<i64> = game.game.res.shots.iter().map(|s| s.to.x).collect();
        assert_eq!(
            ends,
            vec![2, 5],
            "both dogs before the wall, not the one behind it or off the line"
        );
    }

    #[test]
    fn an_isotope_blast_hurts_the_dogs_beside_the_target_and_never_the_robot() {
        let (mut game, dogs) = armed(
            ItemKind::IsotopeLauncher,
            &[(ItemKind::IsotopeCanister, 30)],
            &[(3, 0), (4, 0), (6, 0)],
        );
        let target = tough(&mut game, dogs[0]);
        let beside = tough(&mut game, dogs[1]);
        let far = tough(&mut game, dogs[2]);
        let p = player(&game);
        let body_before = format!(
            "{:?}",
            game.game
                .world
                .get::<&Body>(p)
                .unwrap()
                .parts
                .get(&BodyPartType::Torso)
        );
        let tiles_before: Vec<TileKind> = (0..8).map(|x| game.tile(x, 0, 0)).collect();
        for _ in 0..30 {
            game.game.res.shots.clear();
            shoot_once(&mut game, target);
            if game.game.res.shots.iter().any(|s| s.hit) {
                break;
            }
        }
        let hp = |e: Entity| game.game.world.get::<&Health>(e).unwrap().current;
        assert!(hp(target) < 100_000);
        assert!(
            hp(beside) < 100_000,
            "the dog beside it is caught in the blast"
        );
        assert_eq!(hp(far), 100_000, "two tiles away is safe");
        let body_after = format!(
            "{:?}",
            game.game
                .world
                .get::<&Body>(p)
                .unwrap()
                .parts
                .get(&BodyPartType::Torso)
        );
        assert_eq!(body_before, body_after);
        let tiles_after: Vec<TileKind> = (0..8).map(|x| game.tile(x, 0, 0)).collect();
        assert_eq!(tiles_before, tiles_after, "the world is unchanged");
    }

    #[test]
    fn trait_shots_replay_the_same() {
        let run = || {
            let (mut game, dogs) = armed(
                ItemKind::Scattergun,
                &[(ItemKind::ScatterShell, 4)],
                &[(2, 0)],
            );
            let dog = tough(&mut game, dogs[0]);
            for _ in 0..4 {
                shoot_once(&mut game, dog);
            }
            game.game.res.shots.clone()
        };
        assert_eq!(run(), run());
    }
}

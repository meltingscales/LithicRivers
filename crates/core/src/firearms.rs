//! Guns: one weapon slot per working arm, and the Shoot move that fires every equipped,
//! loaded gun in one turn. Shots follow the same rules as enemy shots
//! (`systems::ranged::fire`): range, line of sight, and a hit chance that falls off with
//! distance, rolled from the world's seeded RNG.

use crate::components::{Inventory, ItemKind, Position};
use crate::model::body::{Body, BodyPartState, BodyPartType};
use crate::resources::Resources;
use crate::systems::ranged::{fire, hit_chance, tile_distance, Shot};
use hecs::{Entity, World};

/// One kind of gun and how it shoots.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gun {
    pub item: ItemKind,
    pub ammo: ItemKind,
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
}

pub const PIPE_GUN: Gun = Gun {
    item: ItemKind::PipeGun,
    ammo: ItemKind::PaperCartridge,
    range: 6,
    accuracy_at_1: 70,
    falloff: 8,
    damage: (10, 18),
    ticks: 12,
    casing: None,
};

pub const RIFLE: Gun = Gun {
    item: ItemKind::Rifle,
    ammo: ItemKind::BrassCartridge,
    range: 12,
    accuracy_at_1: 90,
    falloff: 4,
    damage: (16, 26),
    ticks: 10,
    casing: Some(ItemKind::ShellCasing),
};

pub const GUNS: &[Gun] = &[PIPE_GUN, RIFLE];

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
/// each. Two guns of one kind share the rounds, so each needs its own.
pub fn loaded_guns(world: &World, entity: Entity) -> Vec<(BodyPartType, &'static Gun)> {
    let (Ok(body), Ok(inv)) = (world.get::<&Body>(entity), world.get::<&Inventory>(entity)) else {
        return vec![];
    };
    let mut rounds: std::collections::BTreeMap<ItemKind, u32> = Default::default();
    ARMS.into_iter()
        .filter(|arm| arm_works(&body, *arm))
        .filter_map(|arm| Some((arm, gun(*body.weapons.get(&arm)?)?)))
        .filter(|(_, g)| {
            let used = rounds.entry(g.ammo).or_insert(0);
            *used += 1;
            inv.count(g.ammo) >= *used
        })
        .collect()
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

/// Fire every equipped, loaded gun at `target`, in arm order: each uses one round, rolls
/// its own hit, and a brass round leaves a shell casing.
pub fn execute_shoot(world: &mut World, res: &mut Resources, shooter: Entity, target: Entity) {
    use rand::Rng;
    let guns = loaded_guns(world, shooter);
    if guns.is_empty() {
        res.events
            .combat_event("Click. Nothing to shoot with.", res.time.tick);
        return;
    }
    for (arm, gun) in guns {
        // The target can fall to an earlier shot this turn
        if world.get::<&crate::components::Dead>(target).is_ok() {
            break;
        }
        let (Ok(from), Ok(to)) = (
            world.get::<&Position>(shooter).map(|p| *p),
            world.get::<&Position>(target).map(|p| *p),
        ) else {
            break;
        };
        let state = world
            .get::<&Body>(shooter)
            .ok()
            .and_then(|b| b.parts.get(&arm).map(|p| p.state))
            .unwrap_or(BodyPartState::Functional);
        if let Ok(mut inv) = world.get::<&mut Inventory>(shooter) {
            inv.remove(gun.ammo, 1);
            if let Some(casing) = gun.casing {
                inv.add(casing, 1);
            }
        }
        let name = crate::components::itemkind_name(gun.item);
        let chance = gun_hit_chance(gun, tile_distance(from, to), state);
        let shot = fire(world, res, from, to, gun.range, chance);
        if shot != Shot::Hit {
            res.events.combat_event(
                format!("Your {name} shot {}.", shot.miss_words()),
                res.time.tick,
            );
            continue;
        }
        let damage = res.world_state.rng.gen_range(gun.damage.0..=gun.damage.1);
        crate::systems::combat_actions::apply_damage(
            world,
            res,
            target,
            damage,
            &format!("your {name}"),
        );
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
}

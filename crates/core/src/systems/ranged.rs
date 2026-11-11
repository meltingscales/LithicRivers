//! Ranged combat: creatures that shoot from a distance (`RangedAttack`), and the rusty
//! military android that uses it.
//!
//! A shooter joins combat once the player stands within its range on its layer with a clear
//! line of sight (the fog of war's raycast: walls block it, creatures don't). Each shot is a
//! queued `EnemyShoot` action that uses one of the ammo items it carries and hits by
//! `hit_chance`, rolled from the world's seeded RNG.

use crate::components::{
    AndroidAI, BlocksMovement, Combat, Dead, Inventory, MilitaryAndroid, Position, RangedAttack,
};
use crate::moves::{CombatAction, QueuedAction};
use crate::pathfinding::Pathfinder;
use crate::resources::Resources;
use crate::systems::ecs_utils::{get_player_entity, get_player_position, is_stunned};
use crate::systems::fog_of_war::has_line_of_sight;
use hecs::{Entity, World};
use serde::{Deserialize, Serialize};

/// The lowest and highest a shot's chance to hit can be, in percent.
pub const MIN_HIT_CHANCE: u32 = 10;
pub const MAX_HIT_CHANCE: u32 = 95;
/// How far a military android sees the player, in tiles on its layer.
pub const ANDROID_SIGHT: i64 = 12;
/// The distance a military android with ammo tries to keep from the player.
pub const ANDROID_KEEP_MIN: i64 = 4;
pub const ANDROID_KEEP_MAX: i64 = 6;
/// Game ticks a shot's line stays on screen.
pub const SHOT_TRACE_TICKS: u64 = 2;

/// A shot that was just fired, for the client to draw its line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShotTrace {
    pub from: Position,
    pub to: Position,
    pub hit: bool,
    pub tick: u64,
}

/// Distance in tiles, counting diagonal steps as one (as movement does).
pub fn tile_distance(a: Position, b: Position) -> i64 {
    (a.x - b.x).abs().max((a.y - b.y).abs())
}

/// Percent chance for a shot to hit from `distance` tiles: the gun's accuracy at 1 tile,
/// less its falloff for every tile beyond, kept between 10% and 95%.
pub fn hit_chance(accuracy_at_1: u32, falloff: u32, distance: i64) -> u32 {
    let lost = falloff as i64 * (distance.max(1) - 1);
    (accuracy_at_1 as i64 - lost).clamp(MIN_HIT_CHANCE as i64, MAX_HIT_CHANCE as i64) as u32
}

/// How many shots a shooter has left: the ammo items it carries.
pub fn ammo_left(world: &World, shooter: Entity) -> u32 {
    let Ok(gun) = world.get::<&RangedAttack>(shooter) else {
        return 0;
    };
    world
        .get::<&Inventory>(shooter)
        .map(|inv| {
            inv.slots
                .iter()
                .filter(|s| s.kind == gun.ammo)
                .map(|s| s.qty)
                .sum()
        })
        .unwrap_or(0)
}

/// Whether `from` sees `to`: on the same layer with nothing that stops sight between them.
pub fn line_of_sight(world: &World, res: &Resources, from: Position, to: Position) -> bool {
    from.z == to.z && has_line_of_sight(world, res, from.x, from.y, to.x, to.y, from.z)
}

/// Whether a shooter can fire at `target` now: it has ammo, and `target` is within range
/// on its layer and in its line of sight.
pub fn can_shoot(world: &World, res: &Resources, shooter: Entity, target: Position) -> bool {
    let (Ok(gun), Ok(pos)) = (
        world.get::<&RangedAttack>(shooter),
        world.get::<&Position>(shooter),
    ) else {
        return false;
    };
    tile_distance(*pos, target) <= gun.range
        && ammo_left(world, shooter) > 0
        && line_of_sight(world, res, *pos, target)
}

/// Whether a combatant at `pos` fights the player at `player_pos`: beside them on their
/// layer, or able to shoot them.
pub fn engages(
    world: &World,
    res: &Resources,
    entity: Entity,
    pos: Position,
    player_pos: Position,
) -> bool {
    let adjacent =
        pos.z == player_pos.z && pos != player_pos && tile_distance(pos, player_pos) <= 1;
    adjacent || can_shoot(world, res, entity, player_pos)
}

/// Fire a queued shot: one ammo is used, and the shot hits by the hit chance at the
/// distance it is fired from. A wall that came between them stops it.
pub fn execute_enemy_shoot(
    world: &mut World,
    res: &mut Resources,
    shooter: Entity,
    target: Entity,
) {
    use rand::Rng;
    let (Ok(gun), Ok(from), Ok(to)) = (
        world.get::<&RangedAttack>(shooter).map(|g| *g),
        world.get::<&Position>(shooter).map(|p| *p),
        world.get::<&Position>(target).map(|p| *p),
    ) else {
        return;
    };
    if ammo_left(world, shooter) == 0 {
        return;
    }
    if let Ok(mut inv) = world.get::<&mut Inventory>(shooter) {
        inv.remove(gun.ammo, 1);
    }
    let name = shooter_name(world, shooter);
    let chance = hit_chance(gun.accuracy_at_1, gun.falloff, tile_distance(from, to));
    let shot = fire(world, res, from, to, gun.range, chance);
    if shot != Shot::Hit {
        res.events.combat_event(
            format!("The {name}'s shot {}.", shot.miss_words()),
            res.time.tick,
        );
        return;
    }
    let damage = res.world_state.rng.gen_range(8..=14);
    super::combat_actions::apply_damage(world, res, target, damage, &format!("{name}'s shot"));
}

/// How a shot went.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shot {
    Hit,
    Wide,
    /// Out of range, or something in the way
    Blocked,
}

impl Shot {
    /// How the combat log tells a miss: "goes wide".
    pub fn miss_words(self) -> &'static str {
        match self {
            Shot::Hit => "hits",
            Shot::Wide => "goes wide",
            Shot::Blocked => "hits the wall",
        }
    }
}

/// Fire one shot from `from` at `to` with `chance` percent to hit: it can't hit beyond
/// `range` or without a line of sight, and is rolled from the world's seeded RNG. Every
/// shot draws its line and makes its sound; the caller deals the damage.
pub fn fire(
    world: &World,
    res: &mut Resources,
    from: Position,
    to: Position,
    range: i64,
    chance: u32,
) -> Shot {
    use rand::Rng;
    let clear = tile_distance(from, to) <= range && line_of_sight(world, res, from, to);
    let shot = if !clear {
        Shot::Blocked
    } else if res.world_state.rng.gen_range(0..100) < chance {
        Shot::Hit
    } else {
        Shot::Wide
    };
    res.shots.push(ShotTrace {
        from,
        to,
        hit: shot == Shot::Hit,
        tick: res.time.tick,
    });
    res.play(crate::sound::SoundCue::Shot);
    shot
}

/// Take one queued step: away from the player when `away`, else toward where it last saw
/// them (or toward them).
pub fn execute_enemy_step(world: &mut World, res: &mut Resources, entity: Entity, away: bool) {
    let (Some(player_pos), Ok(pos)) = (
        get_player_position(world),
        world.get::<&Position>(entity).map(|p| *p),
    ) else {
        return;
    };
    let goal = if away {
        player_pos
    } else {
        world
            .get::<&AndroidAI>(entity)
            .ok()
            .and_then(|ai| ai.last_seen)
            .unwrap_or(player_pos)
    };
    if let Some(next) = next_step(world, res, pos, goal, player_pos, away) {
        if let Ok(mut p) = world.get::<&mut Position>(entity) {
            *p = next;
        }
    }
}

/// One tile toward `goal`, or away from it when `away`, never onto the player or a blocked
/// tile. Toward uses the dogs' pathfinding; away takes the open neighbor farthest from it.
fn next_step(
    world: &World,
    res: &Resources,
    pos: Position,
    goal: Position,
    player_pos: Position,
    away: bool,
) -> Option<Position> {
    let open = |p: Position| {
        p != player_pos
            && res.world_state.world.is_passable(p.x, p.y, p.z)
            && !world
                .query::<&Position>()
                .with::<&BlocksMovement>()
                .iter()
                .any(|(_, q)| *q == p)
    };
    if !away {
        // The goal may be the player's own tile: path to it, but never step onto it
        return Pathfinder::new(100)
            .planned_step(
                pos,
                goal,
                |p| p == goal || open(p),
                res.world_state.seed,
                res.time.tick,
            )
            .filter(|&p| open(p));
    }
    let mut best: Option<((i64, i64), Position)> = None;
    for dy in -1..=1 {
        for dx in -1..=1 {
            let p = Position {
                x: pos.x + dx,
                y: pos.y + dy,
                z: pos.z,
            };
            if (dx, dy) == (0, 0) || !open(p) {
                continue;
            }
            // Farthest first; of those, the one that strays least sideways
            let key = (
                tile_distance(p, goal),
                -((p.x - goal.x).abs() + (p.y - goal.y).abs()),
            );
            if best.is_none_or(|(b, _)| key > b) {
                best = Some((key, p));
            }
        }
    }
    best.filter(|((d, _), _)| *d > tile_distance(pos, goal))
        .map(|(_, p)| p)
}

/// The action a ranged creature takes next in combat, or `None` to fight like the others
/// (a melee attack). With ammo it backs off when the player is closer than it likes, else
/// shoots, or walks toward where it last saw them; out of ammo it closes in and strikes.
pub fn ranged_combat_action(
    world: &World,
    res: &mut Resources,
    entity: Entity,
) -> Option<QueuedAction> {
    use rand::Rng;
    world.get::<&RangedAttack>(entity).ok()?;
    let player = get_player_entity(world)?;
    let player_pos = get_player_position(world)?;
    let pos = *world.get::<&Position>(entity).ok()?;
    let distance = tile_distance(pos, player_pos);
    let has_ammo = ammo_left(world, entity) > 0;
    let sees = line_of_sight(world, res, pos, player_pos);
    let step = |away: bool, res: &mut Resources| {
        let ticks = res.world_state.rng.gen_range(6..=8);
        Some(queued(entity, CombatAction::EnemyStep { away }, ticks))
    };
    if has_ammo && sees {
        let backing_off = distance < ANDROID_KEEP_MIN
            && next_step(world, res, pos, player_pos, player_pos, true).is_some();
        if backing_off {
            return step(true, res);
        }
        if can_shoot(world, res, entity, player_pos) {
            let ticks = res.world_state.rng.gen_range(10..=14);
            return Some(queued(
                entity,
                CombatAction::EnemyShoot {
                    target_entity: player,
                },
                ticks,
            ));
        }
    }
    if distance > 1 {
        return step(false, res);
    }
    // Beside the player: a melee strike
    let damage = res.world_state.rng.gen_range(6..=12);
    let ticks = res.world_state.rng.gen_range(12..=16);
    Some(queued(
        entity,
        CombatAction::EnemyAttack {
            target_entity: player,
            damage,
        },
        ticks,
    ))
}

fn queued(entity: Entity, action: CombatAction, ticks: u64) -> QueuedAction {
    QueuedAction {
        entity,
        action,
        execution_time_ticks: ticks,
        remaining_time_ticks: ticks,
    }
}

/// What the combat log calls a shooter.
fn shooter_name(world: &World, entity: Entity) -> &'static str {
    if world.get::<&MilitaryAndroid>(entity).is_ok() {
        "android"
    } else {
        "enemy"
    }
}

/// Military androids outside combat: one that sees the player within `ANDROID_SIGHT`
/// tiles remembers where, and on every other turn walks toward that spot until it can
/// shoot (combat then starts) or, out of ammo, until it is beside them.
pub fn military_android_system(world: &mut World, res: &mut Resources) {
    let Some(player_pos) = get_player_position(world) else {
        return;
    };
    let mut androids: Vec<(Entity, Position)> = world
        .query::<&Position>()
        .with::<&MilitaryAndroid>()
        .iter()
        .map(|(e, p)| (e, *p))
        .collect();
    androids.sort_by_key(|(e, p)| (p.z, p.y, p.x, e.to_bits()));
    for (android, pos) in androids {
        let engaged = world.get::<&Combat>(android).is_ok_and(|c| c.triggered);
        if is_stunned(world, android) || world.get::<&Dead>(android).is_ok() || engaged {
            continue;
        }
        let sees = tile_distance(pos, player_pos) <= ANDROID_SIGHT
            && line_of_sight(world, res, pos, player_pos);
        let (turns, goal) = {
            let Ok(mut ai) = world.get::<&mut AndroidAI>(android) else {
                continue;
            };
            if sees {
                ai.last_seen = Some(player_pos);
            }
            ai.turns = ai.turns.wrapping_add(1);
            if ai.last_seen == Some(pos) {
                ai.last_seen = None;
            }
            (ai.turns, ai.last_seen)
        };
        let Some(goal) = goal else {
            continue;
        };
        if turns % 2 == 1 || engages(world, res, android, pos, player_pos) {
            continue;
        }
        if let Some(next) = next_step(world, res, pos, goal, player_pos, false) {
            if let Ok(mut p) = world.get::<&mut Position>(android) {
                *p = next;
            }
        }
    }
}

/// Forget shots whose line has been on screen long enough.
pub fn prune_shot_traces(res: &mut Resources) {
    let now = res.time.tick;
    res.shots
        .retain(|s| now.saturating_sub(s.tick) < SHOT_TRACE_TICKS);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat_state_manager::CombatStateManager;
    use crate::components::{EntityKind, Health, ItemKind};
    use crate::intent::PlayerIntent;
    use crate::test_support::{Handle, TestGame, TestWorld};
    use crate::tiles::TileKind;

    /// Open ground with the player at the origin and an android `x` tiles east.
    fn range(x: i64) -> (TestWorld, Handle) {
        let mut world = TestWorld::new(7)
            .fill((-12, -6, 0), (14, 6, 0), TileKind::Grass)
            .inventory(&[]);
        let android = world.military_android(x, 0, 0);
        (world, android)
    }

    fn wait(game: &mut TestGame, turns: usize) {
        for _ in 0..turns {
            game.game.res.player_state.intent = PlayerIntent::movement(0, 0, 0, 100);
            game.tick();
        }
    }

    fn ammo(game: &TestGame, android: Handle) -> u32 {
        ammo_left(&game.game.world, game.entity(android))
    }

    fn engaged(game: &TestGame, h: Handle) -> bool {
        game.game
            .world
            .get::<&Combat>(game.entity(h))
            .is_ok_and(|c| c.triggered)
    }

    #[test]
    fn hit_chance_falls_with_distance_within_bounds() {
        assert_eq!(hit_chance(85, 10, 1), 85);
        assert!(hit_chance(85, 10, 6) < hit_chance(85, 10, 2));
        assert_eq!(hit_chance(85, 10, 30), MIN_HIT_CHANCE);
        assert_eq!(hit_chance(200, 10, 1), MAX_HIT_CHANCE);
    }

    #[test]
    fn an_android_in_range_starts_combat_from_a_distance() {
        let (world, android) = range(6);
        let mut game = world.build();
        game.tick();
        assert!(CombatStateManager::is_in_combat(&game.game.res));
        assert!(engaged(&game, android));
        assert_eq!(
            game.pos(android).unwrap().x,
            6,
            "it didn't need to close in"
        );
    }

    #[test]
    fn a_wall_blocks_the_androids_shot_and_its_fight() {
        let (world, android) = range(5);
        let mut game = world.fill((2, -6, 0), (2, 6, 0), TileKind::Rock).build();
        let can = can_shoot(
            &game.game.world,
            &game.game.res,
            game.entity(android),
            game.player_pos(),
        );
        assert!(!can, "behind cover the android can't shoot");
        game.tick();
        assert!(!CombatStateManager::is_in_combat(&game.game.res));
    }

    #[test]
    fn a_feral_dog_out_of_reach_still_needs_to_close_in() {
        let mut world = TestWorld::new(7).fill((-6, -6, 0), (6, 6, 0), TileKind::Grass);
        let dog = world.feral_dog(5, 0, 0);
        let game = world.build();
        let player_pos = game.player_pos();
        let dog_e = game.entity(dog);
        assert!(!engages(
            &game.game.world,
            &game.game.res,
            dog_e,
            game.pos(dog).unwrap(),
            player_pos
        ));
    }

    #[test]
    fn each_shot_uses_one_ammo_and_out_of_ammo_it_closes_in() {
        let (world, android) = range(5);
        let mut game = world.build();
        let start = ammo(&game, android);
        assert!((5..=7).contains(&start), "rolled ammo {start}");
        // Leave it one shot
        let e = game.entity(android);
        game.game
            .world
            .get::<&mut Inventory>(e)
            .unwrap()
            .remove(ItemKind::BrassCartridge, start - 1);
        wait(&mut game, 30);
        assert_eq!(ammo(&game, android), 0, "it fired its last shot");
        let before = game.pos(android).unwrap().x;
        wait(&mut game, 30);
        assert!(
            game.pos(android).unwrap().x < before,
            "out of ammo it moves toward the robot"
        );
        assert_eq!(game.pos(android).unwrap().x, 1, "until it is beside them");
    }

    #[test]
    fn an_android_with_ammo_backs_away_to_keep_its_distance() {
        let (world, android) = range(2);
        let mut game = world.build();
        wait(&mut game, 30);
        let d = tile_distance(game.pos(android).unwrap(), game.player_pos());
        assert!(d >= ANDROID_KEEP_MIN, "it backed off to {d} tiles");
    }

    #[test]
    fn shots_replay_the_same_for_the_same_seed() {
        let run = || {
            let (world, _) = range(5);
            let mut game = world.build();
            wait(&mut game, 120);
            let player = game.game.get_player_entity().unwrap();
            let mut body: Vec<String> = game
                .game
                .world
                .get::<&crate::model::body::Body>(player)
                .unwrap()
                .parts
                .values()
                .map(|p| format!("{:?} {}", p.part_type, p.integrity))
                .collect();
            body.sort();
            (
                format!("{:?}", game.game.res.events.get_message_log()),
                body,
            )
        };
        assert_eq!(run(), run());
    }

    #[test]
    fn an_android_that_loses_sight_walks_to_where_it_last_saw_the_robot() {
        let (world, android) = range(10);
        let mut game = world.build();
        let e = game.entity(android);
        game.game.world.get::<&mut AndroidAI>(e).unwrap().last_seen =
            Some(Position { x: 8, y: 3, z: 0 });
        // Out of sight behind a wall, it still heads to that spot
        for y in -6..=6 {
            if y != 3 {
                game.game
                    .res
                    .world_state
                    .world
                    .set_tile_cached(9, y, 0, TileKind::Rock);
            }
        }
        wait(&mut game, 8);
        let p = game.pos(android).unwrap();
        assert!(p.x < 10, "it moved toward the last seen spot: {p:?}");
    }

    #[test]
    fn a_destroyed_android_leaves_its_ammo_and_scrap() {
        let (world, android) = range(5);
        let mut game = world.build();
        assert!(game.health(android) > Some(80), "sturdier than a feral dog");
        let e = game.entity(android);
        let carried = game.game.world.get::<&Inventory>(e).unwrap().slots.clone();
        game.game.world.get::<&mut Health>(e).unwrap().current = 0;
        crate::systems::handle_entity_death(&mut game.game.world, &mut game.game.res, e);
        let wreck = game
            .game
            .world
            .query::<(&EntityKind, &Inventory)>()
            .iter()
            .find(|(_, (k, _))| **k == EntityKind::Corpse)
            .map(|(_, (_, inv))| inv.slots.clone())
            .expect("a wreck remains");
        assert_eq!(wreck, carried);
        for item in [
            ItemKind::BrassCartridge,
            ItemKind::ScrapElectronics,
            ItemKind::IronScrap,
        ] {
            assert!(
                wreck.iter().any(|s| s.kind == item),
                "{item:?} in {wreck:?}"
            );
        }
    }
}

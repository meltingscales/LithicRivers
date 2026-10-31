//! Spawner blocks: while the player is near, a spawner grows one creature on a timer and
//! lets it out on open ground beside it, up to a cap of its own. Wild spawners, placed by
//! worldgen in ruins, have a fixed creature and need nothing.

use hecs::World;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};

use crate::components::{Player, Position};
use crate::factory::{Facing, FactoryKind, FACTORY_STEP_TICKS, TICKS_PER_SECOND};
use crate::resources::Resources;
use crate::spawn_utils::EnemyKind;

/// A spawner lets out a creature every 30 game seconds.
pub const SPAWN_INTERVAL_TICKS: u64 = 30 * TICKS_PER_SECOND;
/// A spawner only grows creatures while the player is within this many tiles.
pub const SPAWNER_RANGE: i64 = 24;
/// Creatures appear within this many tiles of their spawner.
pub const SPAWNER_RADIUS: i64 = 4;
/// At most this many of a spawner's creatures are alive at once.
pub const SPAWNER_CAP: usize = 4;
/// Tiles tried when looking for open ground beside a spawner.
const SPAWN_TRIES: usize = 16;

/// A spawner block's state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spawner {
    /// A wild spawner's creature; `None` for one the player placed
    pub wild: Option<EnemyKind>,
    /// Ticks grown toward the next creature
    pub elapsed: u64,
}

impl Spawner {
    /// The creature it grows now, if it can grow one.
    pub fn creature(&self) -> Option<EnemyKind> {
        self.wild
    }
}

/// Marks a creature that came from the spawner at this position, so the spawner can
/// count its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpawnedBy(pub Position);

/// The creature a wild spawner on a layer grows: dogs on the surface and above, slimes
/// underground.
pub fn wild_creature(z: i64) -> EnemyKind {
    if z >= 0 {
        EnemyKind::FeralDog
    } else {
        EnemyKind::NuclearSlime
    }
}

/// Place a wild spawner that grows `creature`.
pub fn place_wild(
    world: &mut World,
    res: &mut Resources,
    pos: Position,
    creature: EnemyKind,
) -> Result<hecs::Entity, crate::factory::PlaceError> {
    let entity = crate::factory::place(world, res, FactoryKind::Spawner, pos, Facing::North)?;
    world
        .get::<&mut Spawner>(entity)
        .expect("spawners have a Spawner")
        .wild = Some(creature);
    Ok(entity)
}

fn distance(a: Position, b: Position) -> i64 {
    (a.x - b.x).abs().max((a.y - b.y).abs())
}

/// One factory step for every spawner.
pub fn step_spawners(world: &mut World, res: &mut Resources, step_tick: u64) {
    let Some(player) = world
        .query::<&Position>()
        .with::<&Player>()
        .iter()
        .next()
        .map(|(_, p)| *p)
    else {
        return;
    };
    let mut due = Vec::new();
    for (entity, (pos, spawner)) in world.query_mut::<(&Position, &mut Spawner)>() {
        if spawner.creature().is_none()
            || pos.z != player.z
            || distance(*pos, player) > SPAWNER_RANGE
        {
            continue;
        }
        spawner.elapsed = (spawner.elapsed + FACTORY_STEP_TICKS).min(SPAWN_INTERVAL_TICKS);
        if spawner.elapsed >= SPAWN_INTERVAL_TICKS {
            due.push((entity, *pos));
        }
    }
    for (entity, pos) in due {
        let own = world
            .query::<&SpawnedBy>()
            .iter()
            .filter(|(_, by)| by.0 == pos)
            .count();
        if own >= SPAWNER_CAP {
            continue;
        }
        let Some(creature) = world
            .get::<&Spawner>(entity)
            .ok()
            .and_then(|s| s.creature())
        else {
            continue;
        };
        if spawn_beside(world, res, pos, creature, step_tick) {
            world
                .get::<&mut Spawner>(entity)
                .expect("still a spawner")
                .elapsed = 0;
        }
    }
}

/// Let a creature out on open ground near a spawner, picked from the seed, the spawner's
/// position and the tick. Returns whether one came out.
fn spawn_beside(
    world: &mut World,
    res: &mut Resources,
    at: Position,
    creature: EnemyKind,
    tick: u64,
) -> bool {
    let seed = res.world_state.world.mix_coords(at.x, at.y, at.z)
        ^ tick.wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ 0x5BA_3E6;
    let mut rng = ChaCha20Rng::seed_from_u64(seed);
    for _ in 0..SPAWN_TRIES {
        let pos = Position {
            x: at.x + rng.gen_range(-SPAWNER_RADIUS..=SPAWNER_RADIUS),
            y: at.y + rng.gen_range(-SPAWNER_RADIUS..=SPAWNER_RADIUS),
            z: at.z,
        };
        let spawned =
            crate::spawn_utils::spawn_enemy(world, &mut res.world_state.world, creature, pos);
        if let Ok(entity) = spawned {
            world
                .insert_one(entity, SpawnedBy(at))
                .expect("just spawned");
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{EntityKind, ItemKind};
    use crate::test_support::{TestGame, TestWorld};
    use crate::tiles::TileKind;

    fn arena() -> TestWorld {
        TestWorld::new(7).fill((-12, -12, 0), (12, 12, 0), TileKind::Grass)
    }

    fn at(x: i64, y: i64) -> Position {
        Position { x, y, z: 0 }
    }

    fn dogs(t: &TestGame) -> usize {
        t.game
            .world
            .query::<&EntityKind>()
            .iter()
            .filter(|(_, k)| **k == EntityKind::FeralDog)
            .count()
    }

    /// Let game time pass in factory steps, with the player standing still.
    fn pass(t: &mut TestGame, ticks: u64) {
        t.game.res.time.tick += ticks;
        crate::factory::run_due_steps(&mut t.game.world, &mut t.game.res);
    }

    #[test]
    fn a_wild_spawner_lets_out_a_creature_every_30_seconds_nearby() {
        let mut t = arena().build();
        place_wild(
            &mut t.game.world,
            &mut t.game.res,
            at(8, 0),
            EnemyKind::FeralDog,
        )
        .unwrap();
        pass(&mut t, SPAWN_INTERVAL_TICKS - FACTORY_STEP_TICKS);
        assert_eq!(dogs(&t), 0, "not yet");
        pass(&mut t, FACTORY_STEP_TICKS);
        assert_eq!(dogs(&t), 1);
        let dog = t
            .game
            .world
            .query::<(&Position, &SpawnedBy)>()
            .iter()
            .map(|(_, (p, by))| (*p, *by))
            .next()
            .unwrap();
        assert_eq!(dog.1, SpawnedBy(at(8, 0)));
        assert!(distance(dog.0, at(8, 0)) <= SPAWNER_RADIUS);
        pass(&mut t, SPAWN_INTERVAL_TICKS);
        assert_eq!(dogs(&t), 2);
    }

    #[test]
    fn a_spawner_stops_at_its_cap_and_refills_when_one_dies() {
        let mut t = arena().build();
        place_wild(
            &mut t.game.world,
            &mut t.game.res,
            at(8, 0),
            EnemyKind::FeralDog,
        )
        .unwrap();
        pass(&mut t, 10 * SPAWN_INTERVAL_TICKS);
        assert_eq!(dogs(&t), SPAWNER_CAP);
        let one = t
            .game
            .world
            .query::<&SpawnedBy>()
            .iter()
            .map(|(e, _)| e)
            .next()
            .unwrap();
        t.game.world.despawn(one).unwrap();
        pass(&mut t, FACTORY_STEP_TICKS);
        assert_eq!(dogs(&t), SPAWNER_CAP, "a full timer refills at once");
    }

    #[test]
    fn a_spawner_sleeps_while_the_player_is_far() {
        let mut t = arena().player_at(-20, 0, 0).build();
        place_wild(
            &mut t.game.world,
            &mut t.game.res,
            at(8, 0),
            EnemyKind::FeralDog,
        )
        .unwrap();
        pass(&mut t, 10 * SPAWN_INTERVAL_TICKS);
        assert_eq!(dogs(&t), 0);
    }

    #[test]
    fn breaking_a_wild_spawner_gives_a_spawner_item() {
        let mut t = arena().inventory(&[]).build();
        place_wild(
            &mut t.game.world,
            &mut t.game.res,
            at(1, 0),
            EnemyKind::FeralDog,
        )
        .unwrap();
        let player = t.game.get_player_entity().unwrap();
        let broken =
            crate::factory::break_block(&mut t.game.world, &mut t.game.res, at(1, 0), player);
        assert_eq!(broken, Some(FactoryKind::Spawner));
        assert_eq!(t.player_inventory(), vec![(ItemKind::Spawner, 1)]);
    }

    #[test]
    fn forest_outposts_bring_a_wild_dog_spawner() {
        let mut game = TestWorld::new(12345).build().game;
        let mut found = None;
        'search: for cy in 0..60 {
            for cx in 0..60 {
                game.ensure_chunk_with_pending_structures(cx, cy, 0);
                game.place_wild_spawners();
                found = game
                    .world
                    .query::<(&Position, &Spawner)>()
                    .iter()
                    .map(|(_, (p, s))| (*p, *s))
                    .next();
                if found.is_some() {
                    break 'search;
                }
            }
        }
        let (pos, spawner) = found.expect("some forest chunk has an outpost");
        assert_eq!(pos.z, 0, "on the outpost's ground floor");
        assert_eq!(spawner.wild, Some(EnemyKind::FeralDog));
        assert_eq!(
            game.res.world_state.world.get_tile_cached(pos.x, pos.y, 0),
            TileKind::FactoryBlock
        );
    }

    #[test]
    fn a_spawner_replays_the_same() {
        let run = || {
            let mut t = arena().build();
            place_wild(
                &mut t.game.world,
                &mut t.game.res,
                at(8, 0),
                EnemyKind::FeralDog,
            )
            .unwrap();
            pass(&mut t, 4 * SPAWN_INTERVAL_TICKS);
            let mut found: Vec<(i64, i64)> = t
                .game
                .world
                .query::<(&Position, &SpawnedBy)>()
                .iter()
                .map(|(_, (p, _))| (p.x, p.y))
                .collect();
            found.sort();
            found
        };
        assert_eq!(run(), run());
    }
}

//! Creatures that keep appearing where they belong. Every 10 game seconds each spawn rule
//! is checked for the chunks around the player: a seeded roll picks whether a chunk gets a
//! creature, which then appears on open ground out of the player's sight, unless the kind
//! is already at its cap near the player. A chunk that spawned rests for a while.

use crate::components::{BlocksMovement, EntityKind, LightSource, Player, Position};
use crate::resources::{Resources, WorldType};
use crate::spawn_utils::EnemyKind;
use crate::systems::ecs_utils::get_player_position;
use crate::world::{BiomeBand, DepthBand, CHUNK_SIZE, CHUNK_SIZE_Z};
use hecs::World;
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;

/// Spawn rules are checked once per this many ticks (10 game seconds).
pub const SPAWN_CHECK_TICKS: u64 = 1000;
/// After a creature spawns in a chunk, the chunk rests this long (5 game minutes).
pub const CHUNK_COOLDOWN_TICKS: u64 = 30 * SPAWN_CHECK_TICKS;
/// Creatures never appear this close to the player.
pub const SPAWN_MIN_DISTANCE: i64 = 16;
/// How far the player sees in daylight; creatures appear only beyond it or out of line
/// of sight. Wider than the world view of a usual window.
pub const SIGHT_RADIUS: i64 = 48;
/// Tiles tried in a chunk when looking for open ground.
const SPAWN_TRIES: usize = 16;
/// Nothing spawns naturally this close to a placed light.
pub const LIGHT_SAFE_RADIUS: i64 = 8;
/// How far a creature must be able to wander from a tile for the tile to count as open
/// country rather than the inside of a fenced or walled base.
pub const ENCLOSURE_RADIUS: i64 = 64;

/// Where a rule's creature belongs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpawnPlace {
    /// A surface biome
    Biome(BiomeBand),
    /// A layer of the underground
    Band(DepthBand),
}

impl SpawnPlace {
    /// Whether creatures of this place can appear on the layer at all.
    fn on_layer(self, z: i64) -> bool {
        match self {
            SpawnPlace::Biome(_) => crate::world::depth_band(z) == DepthBand::Surface,
            SpawnPlace::Band(band) => crate::world::depth_band(z) == band,
        }
    }

    fn contains(self, res: &Resources, pos: Position) -> bool {
        self.on_layer(pos.z)
            && match self {
                SpawnPlace::Biome(biome) => {
                    res.world_state
                        .world
                        .biome_for(pos.x as f64, pos.y as f64, pos.z as f64)
                        == biome
                }
                SpawnPlace::Band(_) => true,
            }
    }
}

/// Where a creature kind keeps appearing, how often, and how many may be near the player.
#[derive(Debug, Clone, Copy)]
pub struct SpawnRule {
    pub kind: EnemyKind,
    pub place: SpawnPlace,
    /// Chance, in percent, that a chunk gets a creature on a check
    pub chance_percent: u32,
    /// The same at night
    pub night_chance_percent: u32,
    /// At most this many of the kind within `radius` tiles of the player
    pub cap: usize,
    pub radius: i64,
}

pub const SPAWN_RULES: &[SpawnRule] = &[
    // Feral dogs roam the forests, and more of them come out at night
    SpawnRule {
        kind: EnemyKind::FeralDog,
        place: SpawnPlace::Biome(BiomeBand::Forest),
        chance_percent: 15,
        night_chance_percent: 30,
        cap: 6,
        radius: 64,
    },
    // Nuclear slimes ooze through the shallows, and more often through the deep
    SpawnRule {
        kind: EnemyKind::NuclearSlime,
        place: SpawnPlace::Band(DepthBand::Shallows),
        chance_percent: 10,
        night_chance_percent: 10,
        cap: 4,
        radius: 64,
    },
    SpawnRule {
        kind: EnemyKind::NuclearSlime,
        place: SpawnPlace::Band(DepthBand::Deep),
        chance_percent: 20,
        night_chance_percent: 20,
        cap: 4,
        radius: 64,
    },
];

fn entity_kind(kind: EnemyKind) -> EntityKind {
    match kind {
        EnemyKind::FeralDog => EntityKind::FeralDog,
        EnemyKind::NuclearSlime => EntityKind::NuclearSlime,
    }
}

/// Distance in tiles on a layer, counting diagonal steps as one.
fn distance(a: Position, b: Position) -> i64 {
    (a.x - b.x).abs().max((a.y - b.y).abs())
}

/// Whether the player can see a tile: on their layer, in line of sight, and within their
/// light, or within `SIGHT_RADIUS` on the surface by day.
pub fn player_sees(world: &World, res: &Resources, player: Position, pos: Position) -> bool {
    if pos.z != player.z || distance(pos, player) > SIGHT_RADIUS {
        return false;
    }
    let daylight = pos.z >= 0 && !res.time.is_night();
    let lit = daylight
        || crate::systems::fog_of_war::is_tile_illuminated(world, res, pos.x, pos.y, pos.z);
    lit && crate::systems::fog_of_war::has_line_of_sight(
        world, res, player.x, player.y, pos.x, pos.y, pos.z,
    )
}

/// Whether a placed light (any light that isn't the player's) is near a tile.
fn near_light(world: &World, pos: Position) -> bool {
    world
        .query::<&Position>()
        .with::<&LightSource>()
        .without::<&Player>()
        .iter()
        .any(|(_, p)| p.z == pos.z && distance(*p, pos) <= LIGHT_SAFE_RADIUS)
}

/// Whether a tile is shut in by fences and walls: walking from it over passable ground
/// never gets `ENCLOSURE_RADIUS` tiles away. Ground not generated yet counts as a way out.
pub fn enclosed(res: &Resources, start: Position) -> bool {
    let world = &res.world_state.world;
    let mut seen = std::collections::HashSet::from([(start.x, start.y)]);
    let mut queue = std::collections::VecDeque::from([(start.x, start.y)]);
    while let Some((x, y)) = queue.pop_front() {
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (nx, ny) = (x + dx, y + dy);
            if !seen.insert((nx, ny)) {
                continue;
            }
            let pos = Position {
                x: nx,
                y: ny,
                z: start.z,
            };
            let generated = world.has_chunk(
                nx.div_euclid(CHUNK_SIZE),
                ny.div_euclid(CHUNK_SIZE),
                start.z.div_euclid(CHUNK_SIZE_Z),
            );
            if !generated || distance(pos, start) > ENCLOSURE_RADIUS {
                return false;
            }
            if world
                .get_tile_cached_no_worldgen(nx, ny, start.z)
                .is_passable()
            {
                queue.push_back((nx, ny));
            }
        }
    }
    true
}

pub fn creature_spawn_system(world: &mut World, res: &mut Resources) {
    if res.world_state.world.world_type != WorldType::Normal {
        return;
    }
    // Time jumps by whole actions, so check once per 10-second period crossed
    let period = res.time.tick / SPAWN_CHECK_TICKS;
    if period <= res.spawn_check_period {
        return;
    }
    res.spawn_check_period = period;
    let Some(player) = get_player_position(world) else {
        return;
    };
    let tick = res.time.tick;
    res.spawn_cooldowns.retain(|_, until| *until > tick);

    let (pcx, pcy, cz) = (
        player.x.div_euclid(CHUNK_SIZE),
        player.y.div_euclid(CHUNK_SIZE),
        player.z.div_euclid(CHUNK_SIZE_Z),
    );
    for (index, rule) in SPAWN_RULES.iter().enumerate() {
        if !rule.place.on_layer(player.z) {
            continue;
        }
        for cy in pcy - 1..=pcy + 1 {
            for cx in pcx - 1..=pcx + 1 {
                let chunk = (cx, cy, cz);
                if res.spawn_cooldowns.contains_key(&chunk) {
                    continue;
                }
                if let Some(pos) = roll_spawn(world, res, rule, index, chunk, period, player) {
                    crate::spawn_utils::spawn_enemy(
                        world,
                        &mut res.world_state.world,
                        rule.kind,
                        pos,
                    )
                    .expect("the spawn tile was checked");
                    res.spawn_cooldowns
                        .insert(chunk, tick + CHUNK_COOLDOWN_TICKS);
                }
            }
        }
    }
}

/// Where a rule spawns its creature in a chunk this period, if anywhere.
fn roll_spawn(
    world: &World,
    res: &mut Resources,
    rule: &SpawnRule,
    index: usize,
    (cx, cy, cz): (i64, i64, i64),
    period: u64,
    player: Position,
) -> Option<Position> {
    let seed = res.world_state.world.mix_coords(cx, cy, cz)
        ^ period.wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (index as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F)
        ^ 0x5_1A4E_C4A7;
    let mut rng = ChaCha20Rng::seed_from_u64(seed);
    let chance = if res.time.is_night() {
        rule.night_chance_percent
    } else {
        rule.chance_percent
    };
    if rng.gen_range(0..100) >= chance {
        return None;
    }
    let kind = entity_kind(rule.kind);
    let near = world
        .query::<(&EntityKind, &Position)>()
        .iter()
        .filter(|(_, (k, p))| {
            **k == kind && p.z == player.z && distance(**p, player) <= rule.radius
        })
        .count();
    if near >= rule.cap {
        return None;
    }
    for _ in 0..SPAWN_TRIES {
        let pos = Position {
            x: cx * CHUNK_SIZE + rng.gen_range(0..CHUNK_SIZE),
            y: cy * CHUNK_SIZE + rng.gen_range(0..CHUNK_SIZE),
            z: cz,
        };
        let dist = distance(pos, player);
        if dist < SPAWN_MIN_DISTANCE || dist > rule.radius || !rule.place.contains(res, pos) {
            continue;
        }
        let open = res
            .world_state
            .world
            .get_tile_cached(pos.x, pos.y, pos.z)
            .is_passable();
        let free = !world
            .query::<&Position>()
            .with::<&BlocksMovement>()
            .iter()
            .any(|(_, p)| *p == pos);
        if open
            && free
            && !near_light(world, pos)
            && !player_sees(world, res, player, pos)
            && !enclosed(res, pos)
        {
            return Some(pos);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{Inventory, ItemKind, NuclearSlime};
    use crate::intent::PlayerAction;
    use crate::test_support::TestWorld;
    use crate::tiles::TileKind;

    fn waste(world: &World, slime: hecs::Entity) -> u32 {
        world
            .get::<&Inventory>(slime)
            .map(|inv| {
                inv.slots
                    .iter()
                    .filter(|s| s.kind == ItemKind::NuclearWaste)
                    .map(|s| s.qty)
                    .sum()
            })
            .unwrap_or(0)
    }

    #[test]
    fn a_slime_is_tougher_than_a_dog() {
        let mut world = TestWorld::new(5);
        let slime = world.nuclear_slime(3, 0, 0);
        let dog = world.feral_dog(-3, 0, 0);
        let game = world.build();
        assert!(game.health(slime) > game.health(dog));
    }

    #[test]
    fn about_one_slime_in_twenty_carries_waste_and_always_the_same_one() {
        let game = TestWorld::new(5).build().game;
        let mut world = World::new();
        let mut carriers = 0;
        for i in 0..2000 {
            let slime = crate::spawn_utils::world_spawn_nuclear_slime(
                &mut world,
                i,
                i * 7,
                -20,
                &game.res.world_state.world,
            );
            carriers += u32::from(waste(&world, slime) > 0);
        }
        let percent = carriers as f64 / 20.0;
        assert!((3.0..=7.0).contains(&percent), "{percent}% carry waste");
        // Same seed, same spot, same slime
        let again = TestWorld::new(5).build().game;
        let mut other = World::new();
        for i in 0..200 {
            let a = crate::spawn_utils::world_spawn_nuclear_slime(
                &mut world,
                i,
                3,
                -20,
                &game.res.world_state.world,
            );
            let b = crate::spawn_utils::world_spawn_nuclear_slime(
                &mut other,
                i,
                3,
                -20,
                &again.res.world_state.world,
            );
            assert_eq!(waste(&world, a), waste(&other, b));
        }
    }

    #[test]
    fn a_slime_closes_in_slowly_then_fights() {
        let mut world = TestWorld::new(5).fill((-8, -8, 0), (8, 8, 0), TileKind::Grass);
        let slime = world.nuclear_slime(4, 0, 0);
        let mut t = world.build();
        for _ in 0..5 {
            t.act(PlayerAction::Move {
                dx: 0,
                dy: 0,
                dz: 0,
            });
            t.tick();
        }
        assert!(
            t.pos(slime).unwrap().x > 1,
            "not adjacent yet after 5 turns"
        );
        t.act(PlayerAction::Move {
            dx: 0,
            dy: 0,
            dz: 0,
        });
        t.tick();
        assert_eq!(t.pos(slime).unwrap().x, 1, "adjacent after 6 turns");
        t.act(PlayerAction::Move {
            dx: 0,
            dy: 0,
            dz: 0,
        });
        t.tick();
        assert!(t.game.res.player_state.combat_active);
    }

    #[test]
    fn a_slime_leaves_a_corpse_only_if_it_carries_waste() {
        let game = TestWorld::new(5).build().game;
        let mut world = World::new();
        let mut res = game.res;
        let mut carrier = None;
        let mut empty = None;
        for i in 0..400 {
            let slime = crate::spawn_utils::world_spawn_nuclear_slime(
                &mut world,
                i,
                0,
                -20,
                &res.world_state.world,
            );
            if waste(&world, slime) > 0 {
                carrier.get_or_insert(slime);
            } else {
                empty.get_or_insert(slime);
            }
        }
        let (carrier, empty) = (carrier.unwrap(), empty.unwrap());
        let corpses = |world: &World| {
            world
                .query::<&EntityKind>()
                .iter()
                .filter(|(_, k)| **k == EntityKind::Corpse)
                .count()
        };
        crate::systems::handle_entity_death(&mut world, &mut res, empty);
        assert_eq!(corpses(&world), 0);
        assert!(world.get::<&NuclearSlime>(empty).is_err(), "it dissolved");
        crate::systems::handle_entity_death(&mut world, &mut res, carrier);
        assert_eq!(corpses(&world), 1);
        let corpse = world
            .query::<(&EntityKind, &Inventory)>()
            .iter()
            .find(|(_, (k, _))| **k == EntityKind::Corpse)
            .map(|(_, (_, inv))| inv.slots.clone())
            .unwrap();
        assert_eq!(corpse[0].kind, ItemKind::NuclearWaste);
    }

    /// Let `checks` spawn checks pass, each 10 game seconds after the last.
    fn run_checks(t: &mut crate::test_support::TestGame, checks: usize) {
        for _ in 0..checks {
            t.game.res.time.tick += SPAWN_CHECK_TICKS;
            creature_spawn_system(&mut t.game.world, &mut t.game.res);
        }
    }

    fn count(t: &crate::test_support::TestGame, kind: EntityKind) -> usize {
        t.game
            .world
            .query::<&EntityKind>()
            .iter()
            .filter(|(_, k)| **k == kind)
            .count()
    }

    fn positions(t: &crate::test_support::TestGame, kind: EntityKind) -> Vec<Position> {
        let mut found: Vec<Position> = t
            .game
            .world
            .query::<(&EntityKind, &Position)>()
            .iter()
            .filter(|(_, (k, _))| **k == kind)
            .map(|(_, (_, p))| *p)
            .collect();
        found.sort_by_key(|p| (p.x, p.y, p.z));
        found
    }

    /// A spot on the surface with forest for 64 tiles around it.
    fn forest_spot(seed: u64) -> (i64, i64) {
        let world = crate::world::GameWorld::new(100, 100, seed);
        let forest = |x: i64, y: i64| world.biome_for(x as f64, y as f64, 0.0) == BiomeBand::Forest;
        (0..400)
            .flat_map(|i| (0..400).map(move |j| (i * 50 - 10_000, j * 50 - 10_000)))
            .find(|&(x, y)| {
                [(-64, -64), (-64, 64), (64, -64), (64, 64), (0, 0)]
                    .iter()
                    .all(|(dx, dy)| forest(x + dx, y + dy))
            })
            .expect("the world has a forest")
    }

    /// The player in the middle of an open forest clearing.
    fn forest(seed: u64) -> crate::test_support::TestGame {
        let (x, y) = forest_spot(seed);
        TestWorld::new(seed)
            .player_at(x, y, 0)
            .fill((x - 70, y - 70, 0), (x + 70, y + 70, 0), TileKind::Grass)
            .build()
    }

    #[test]
    fn slimes_spawn_underground_and_never_on_the_surface() {
        let slimes = |z: i64| {
            let mut t = TestWorld::new(12345).player_at(0, 0, z).build();
            run_checks(&mut t, 100);
            count(&t, EntityKind::NuclearSlime)
        };
        assert_eq!(slimes(0), 0);
        assert!(slimes(-20) > 0);
    }

    #[test]
    fn dogs_refill_a_cleared_forest_up_to_the_cap_and_never_in_sight() {
        let mut t = forest(7);
        let player = t.game.get_player_position().unwrap();
        for _ in 0..200 {
            let before = positions(&t, EntityKind::FeralDog);
            run_checks(&mut t, 1);
            for pos in positions(&t, EntityKind::FeralDog) {
                if !before.contains(&pos) {
                    assert!(distance(pos, player) >= SPAWN_MIN_DISTANCE);
                    assert!(!player_sees(&t.game.world, &t.game.res, player, pos));
                }
            }
        }
        let dogs = count(&t, EntityKind::FeralDog);
        assert!(dogs > 0, "dogs came");
        assert!(dogs <= 6, "{dogs} dogs is over the cap");

        // Clear the forest: dogs come back
        let dogs: Vec<hecs::Entity> = t
            .game
            .world
            .query::<&crate::components::FeralDog>()
            .iter()
            .map(|(e, _)| e)
            .collect();
        for dog in dogs {
            t.game.world.despawn(dog).unwrap();
        }
        run_checks(&mut t, 200);
        assert!(count(&t, EntityKind::FeralDog) > 0, "dogs came back");
    }

    #[test]
    fn nothing_spawns_while_the_kind_is_at_its_cap() {
        let (x, y) = forest_spot(7);
        let mut world = TestWorld::new(7).player_at(x, y, 0).fill(
            (x - 70, y - 70, 0),
            (x + 70, y + 70, 0),
            TileKind::Grass,
        );
        for i in 0..6 {
            world.feral_dog(x + 20 + i, y, 0);
        }
        let mut t = world.build();
        run_checks(&mut t, 300);
        assert_eq!(count(&t, EntityKind::FeralDog), 6);
    }

    #[test]
    fn a_chunk_rests_after_a_spawn() {
        let mut t = forest(7);
        let mut checks = 0;
        while count(&t, EntityKind::FeralDog) == 0 {
            run_checks(&mut t, 1);
            checks += 1;
            assert!(checks < 500, "no dog ever came");
        }
        let tick = t.game.res.time.tick;
        assert!(t
            .game
            .res
            .spawn_cooldowns
            .values()
            .all(|until| *until == tick + CHUNK_COOLDOWN_TICKS));
        assert_eq!(
            t.game.res.spawn_cooldowns.len(),
            count(&t, EntityKind::FeralDog),
            "one resting chunk per dog"
        );
        // The resting chunks get nothing more until their rest ends
        let rested = t.game.res.spawn_cooldowns.clone();
        run_checks(&mut t, 29);
        for chunk in rested.keys() {
            assert_eq!(t.game.res.spawn_cooldowns[chunk], rested[chunk]);
        }
    }

    /// Let `nights` spawn checks pass, each at midnight a day after the last.
    fn run_night_checks(t: &mut crate::test_support::TestGame, nights: usize) {
        for _ in 0..nights {
            let time = &mut t.game.res.time;
            time.tick += crate::game_time::DAY_TICKS;
            time.tick += time.ticks_until(0, 0);
            assert!(time.is_night());
            creature_spawn_system(&mut t.game.world, &mut t.game.res);
        }
    }

    #[test]
    fn what_the_player_sees() {
        let mut t = forest(7);
        let p = t.game.get_player_position().unwrap();
        let at = |dx: i64| Position {
            x: p.x + dx,
            y: p.y,
            z: 0,
        };
        let sees = |t: &crate::test_support::TestGame, dx: i64| {
            player_sees(&t.game.world, &t.game.res, p, at(dx))
        };
        assert!(sees(&t, 30), "open ground by day");
        assert!(!sees(&t, SIGHT_RADIUS + 1), "too far");
        t.game
            .res
            .world_state
            .world
            .set_tile_cached(p.x + 10, p.y, 0, TileKind::Rock);
        assert!(!sees(&t, 30), "behind a rock");
        t.game.res.time.tick += t.game.res.time.ticks_until(0, 0);
        assert!(!sees(&t, -30), "out of the player's light at night");
        assert!(sees(&t, -1), "in the player's light at night");
    }

    #[test]
    fn nothing_spawns_near_a_placed_light() {
        let (x, y) = forest_spot(7);
        let mut t = TestWorld::new(7)
            .player_at(x, y, 0)
            .fill((x - 70, y - 70, 0), (x + 70, y + 70, 0), TileKind::Grass)
            .build();
        // Without lights, dogs come
        let mut unlit = forest(7);
        run_checks(&mut unlit, 200);
        assert!(count(&unlit, EntityKind::FeralDog) > 0);
        // Lights every 16 tiles leave no tile more than 8 from one
        for lx in (-64..=64).step_by(16) {
            for ly in (-64..=64).step_by(16) {
                t.game.world.spawn((
                    Position {
                        x: x + lx,
                        y: y + ly,
                        z: 0,
                    },
                    LightSource::new(8),
                ));
            }
        }
        run_checks(&mut t, 200);
        assert_eq!(count(&t, EntityKind::FeralDog), 0);
    }

    #[test]
    fn nothing_spawns_inside_a_fenced_base() {
        let pen_dogs = |fenced: bool| {
            let mut t = forest(7);
            let p = t.game.get_player_position().unwrap();
            if fenced {
                for i in -30..=30 {
                    for (fx, fy) in [(i, -30), (i, 30), (-30, i), (30, i)] {
                        let pos = Position {
                            x: p.x + fx,
                            y: p.y + fy,
                            z: 0,
                        };
                        if !t.game.res.factory_index.contains_key(&pos) {
                            crate::factory::place(
                                &mut t.game.world,
                                &mut t.game.res,
                                crate::factory::FactoryKind::Fence,
                                pos,
                                crate::factory::Facing::North,
                            )
                            .unwrap();
                        }
                    }
                }
                assert!(enclosed(&t.game.res, Position { x: p.x + 20, ..p }));
                assert!(!enclosed(&t.game.res, Position { x: p.x + 40, ..p }));
            }
            run_night_checks(&mut t, 300);
            positions(&t, EntityKind::FeralDog)
                .into_iter()
                .filter(|d| distance(*d, p) < 30)
                .count()
        };
        assert!(
            pen_dogs(false) > 0,
            "without the fence, dogs come that close"
        );
        assert_eq!(pen_dogs(true), 0);
    }

    #[test]
    fn the_same_play_gets_the_same_spawns() {
        let run = || {
            let mut t = forest(7);
            run_checks(&mut t, 200);
            positions(&t, EntityKind::FeralDog)
        };
        let dogs = run();
        assert!(!dogs.is_empty());
        assert_eq!(run(), dogs);
    }

    #[test]
    fn a_long_action_checks_spawns_once() {
        let mut t = forest(7);
        t.game.res.time.tick += 50 * SPAWN_CHECK_TICKS;
        creature_spawn_system(&mut t.game.world, &mut t.game.res);
        assert_eq!(t.game.res.spawn_check_period, 50);
        creature_spawn_system(&mut t.game.world, &mut t.game.res);
        assert_eq!(
            t.game.res.spawn_check_period, 50,
            "not again in the same period"
        );
    }

    #[test]
    fn a_slime_fight_replays_the_same() {
        let fight = || {
            let mut world = TestWorld::new(5).fill((-8, -8, 0), (8, 8, 0), TileKind::Grass);
            world.nuclear_slime(2, 0, 0);
            let mut t = world.build();
            for _ in 0..40 {
                t.act(PlayerAction::Move {
                    dx: 0,
                    dy: 0,
                    dz: 0,
                });
                t.tick();
            }
            // The fight's log: who hit what, for how much, and when
            t.game
                .res
                .events
                .get_message_log()
                .messages
                .iter()
                .map(|m| m.text.clone())
                .collect::<Vec<String>>()
        };
        let log = fight();
        assert!(
            log.iter().any(|m| m.contains("from enemy attack")),
            "the slime attacked: {log:?}"
        );
        assert_eq!(fight(), log);
    }
}

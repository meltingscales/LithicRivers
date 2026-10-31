//! Spawner blocks: while the player is near, a spawner grows one creature on a timer and
//! lets it out on open ground beside it, up to a cap of its own. Wild spawners, placed by
//! worldgen in ruins, have a fixed creature and need nothing. A spawner the player places
//! grows only on grid power, using one meat slurry and one charge of its embryo for each
//! creature; the embryo sets the creature. Belts and extractors feed it both.

use hecs::World;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};

use crate::components::{ItemKind, ItemStack, Player, Position};
use crate::factory::{Facing, FactoryKind, FACTORY_STEP_TICKS, TICKS_PER_SECOND};
use crate::power::PowerConsumer;
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
/// What a placed spawner draws while it has slurry and an embryo, in watts.
pub const SPAWNER_W: u64 = 60_000;
/// The most meat slurry a spawner holds.
pub const SLURRY_CAPACITY: u32 = 16;
/// The most embryos (all of one kind) a spawner holds, counting the one growing.
pub const EMBRYO_CAPACITY: u32 = 4;

/// A spawner block's state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spawner {
    /// A wild spawner's creature; `None` for one the player placed
    pub wild: Option<EnemyKind>,
    /// Ticks grown toward the next creature
    pub elapsed: u64,
    /// Meat slurry held
    #[serde(default)]
    pub slurry: u32,
    /// The kind of embryo in the embryo slot
    #[serde(default)]
    pub embryo: Option<ItemKind>,
    /// Embryos held, counting the one growing
    #[serde(default)]
    pub embryos: u32,
    /// Creatures the growing embryo has left in it
    #[serde(default)]
    pub charges: u32,
}

impl Spawner {
    /// The creature it grows now, if it can grow one: a wild spawner's own, or for a
    /// placed one, its embryo's while it has slurry.
    pub fn creature(&self) -> Option<EnemyKind> {
        if self.wild.is_some() {
            return self.wild;
        }
        let (creature, _) = embryo_kind(self.embryo?)?;
        (self.slurry > 0 && self.embryos > 0).then_some(creature)
    }

    /// Whether it would take `item` now: slurry while it has room, and embryos of the
    /// kind it holds (any kind when empty). Wild spawners take nothing.
    pub fn accepts(&self, item: ItemKind) -> bool {
        if self.wild.is_some() {
            return false;
        }
        if item == ItemKind::MeatSlurry {
            return self.slurry < SLURRY_CAPACITY;
        }
        embryo_kind(item).is_some()
            && (self.embryos == 0 || (self.embryo == Some(item) && self.embryos < EMBRYO_CAPACITY))
    }

    /// Store an item it accepts.
    pub fn insert(&mut self, item: ItemKind) {
        if item == ItemKind::MeatSlurry {
            self.slurry += 1;
        } else if let Some((_, grade)) = embryo_kind(item) {
            if self.embryos == 0 {
                self.embryo = Some(item);
                self.charges = grade.charges();
            }
            self.embryos += 1;
        }
    }

    /// Use one slurry and one charge for a creature that came out of a placed spawner; a
    /// spent embryo is used up and the next one starts.
    fn use_up(&mut self) {
        if self.wild.is_some() {
            return;
        }
        self.slurry -= 1;
        self.charges -= 1;
        if self.charges == 0 {
            self.embryos -= 1;
            match (self.embryos, self.embryo.and_then(embryo_kind)) {
                (0, _) | (_, None) => {
                    self.embryo = None;
                    self.embryos = 0;
                }
                (_, Some((_, grade))) => self.charges = grade.charges(),
            }
        }
    }

    /// What breaking it gives back besides the block: its slurry and embryos (a growing
    /// embryo comes back whole).
    pub fn contents(&self) -> Vec<ItemStack> {
        let mut items = Vec::new();
        if self.slurry > 0 {
            items.push(ItemStack::new(ItemKind::MeatSlurry, self.slurry));
        }
        if let (Some(embryo), 1..) = (self.embryo, self.embryos) {
            items.push(ItemStack::new(embryo, self.embryos));
        }
        items
    }
}

impl Spawner {
    /// What the look panel says about it: what it grows, or what it lacks, and how long
    /// until the next creature while it grows.
    pub fn status_lines(&self) -> Vec<String> {
        let grows = self
            .wild
            .or_else(|| self.embryo.and_then(embryo_kind).map(|(c, _)| c));
        let mut lines = Vec::new();
        match grows {
            Some(creature) => lines.push(format!("Grows {}", creature.plural())),
            None => lines.push("Needs an embryo".to_string()),
        }
        if self.wild.is_none() {
            lines.push(format!("Slurry {}, embryos {}", self.slurry, self.embryos));
            if grows.is_some() {
                lines.push(format!("{} left in embryo", self.charges));
            }
            if self.slurry == 0 {
                lines.push("Needs meat slurry".to_string());
            }
        }
        if self.creature().is_some() {
            let left = SPAWN_INTERVAL_TICKS - self.elapsed;
            lines.push(format!("Next in {}s", left.div_ceil(TICKS_PER_SECOND)));
        }
        lines
    }
}

/// Placed spawners want power while they have slurry and an embryo.
pub fn update_demand(world: &mut World) {
    for (_, (spawner, consumer)) in world.query_mut::<(&Spawner, &mut PowerConsumer)>() {
        consumer.active = spawner.wild.is_none() && spawner.creature().is_some();
    }
}

/// Marks a creature that came from the spawner at this position, so the spawner can
/// count its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpawnedBy(pub Position);

/// How good an embryo is: a better one lasts for more spawns, and drops more rarely.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EmbryoGrade {
    Poor,
    Flawed,
    Flawless,
}

impl EmbryoGrade {
    /// Best first, the order a death rolls them in.
    pub const ALL: [EmbryoGrade; 3] = [
        EmbryoGrade::Flawless,
        EmbryoGrade::Flawed,
        EmbryoGrade::Poor,
    ];

    /// How many creatures an embryo of this grade grows before it is spent.
    pub fn charges(self) -> u32 {
        match self {
            EmbryoGrade::Poor => 2,
            EmbryoGrade::Flawed => 4,
            EmbryoGrade::Flawless => 8,
        }
    }

    /// Chance, in percent, that a dying creature drops an embryo of this grade.
    pub fn drop_percent(self) -> u32 {
        match self {
            EmbryoGrade::Poor => 10,
            EmbryoGrade::Flawed => 3,
            EmbryoGrade::Flawless => 1,
        }
    }
}

/// The embryo item for a creature and grade.
pub fn embryo(creature: EnemyKind, grade: EmbryoGrade) -> ItemKind {
    use EmbryoGrade::*;
    match (creature, grade) {
        (EnemyKind::FeralDog, Poor) => ItemKind::FeralDogEmbryoPoor,
        (EnemyKind::FeralDog, Flawed) => ItemKind::FeralDogEmbryoFlawed,
        (EnemyKind::FeralDog, Flawless) => ItemKind::FeralDogEmbryoFlawless,
        (EnemyKind::NuclearSlime, Poor) => ItemKind::NuclearSlimeEmbryoPoor,
        (EnemyKind::NuclearSlime, Flawed) => ItemKind::NuclearSlimeEmbryoFlawed,
        (EnemyKind::NuclearSlime, Flawless) => ItemKind::NuclearSlimeEmbryoFlawless,
    }
}

/// The creature and grade of an embryo item, if it is one.
pub fn embryo_kind(item: ItemKind) -> Option<(EnemyKind, EmbryoGrade)> {
    EnemyKind::ALL
        .iter()
        .flat_map(|&c| EmbryoGrade::ALL.map(|g| (c, g)))
        .find(|&(c, g)| embryo(c, g) == item)
}

/// Roll whether a dying creature drops an embryo of its own kind: at most one, rolled
/// from the seed, where it died and when, so a farm's drops vary but replay the same.
pub fn roll_embryo(
    game_world: &crate::world::GameWorld,
    creature: EnemyKind,
    at: Position,
    tick: u64,
) -> Option<ItemKind> {
    let seed = game_world.mix_coords(at.x, at.y, at.z)
        ^ tick.wrapping_mul(0xC2B2_AE3D_27D4_EB4F)
        ^ 0xE3B_7105;
    let roll = ChaCha20Rng::seed_from_u64(seed).gen_range(0..100);
    let mut below = 0;
    for grade in EmbryoGrade::ALL {
        below += grade.drop_percent();
        if roll < below {
            return Some(embryo(creature, grade));
        }
    }
    None
}

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
    for (entity, (pos, spawner, consumer)) in
        world.query_mut::<(&Position, &mut Spawner, Option<&PowerConsumer>)>()
    {
        if spawner.creature().is_none()
            || pos.z != player.z
            || distance(*pos, player) > SPAWNER_RANGE
        {
            continue;
        }
        // Wild spawners grow at full speed; placed ones as fast as their power allows
        let grown = match (spawner.wild, consumer) {
            (None, Some(consumer)) => {
                FACTORY_STEP_TICKS * u64::from(consumer.satisfaction_permille) / 1000
            }
            (None, None) => 0,
            (Some(_), _) => FACTORY_STEP_TICKS,
        };
        spawner.elapsed = (spawner.elapsed + grown).min(SPAWN_INTERVAL_TICKS);
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
            let mut spawner = world.get::<&mut Spawner>(entity).expect("still a spawner");
            spawner.elapsed = 0;
            spawner.use_up();
        }
    }
}

/// Let a creature out near a spawner, on open ground it could walk to from the spawner
/// within `SPAWNER_RADIUS` (so a fenced spawner keeps its creatures in), picked from the
/// seed, the spawner's position and the tick. Returns whether one came out.
fn spawn_beside(
    world: &mut World,
    res: &mut Resources,
    at: Position,
    creature: EnemyKind,
    tick: u64,
) -> bool {
    let open = reachable_ground(world, res, at);
    if open.is_empty() {
        return false;
    }
    let seed = res.world_state.world.mix_coords(at.x, at.y, at.z)
        ^ tick.wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ 0x5BA_3E6;
    let pos = open[ChaCha20Rng::seed_from_u64(seed).gen_range(0..open.len())];
    let spawned = crate::spawn_utils::spawn_enemy(world, &mut res.world_state.world, creature, pos)
        .expect("the tile was checked");
    world
        .insert_one(spawned, SpawnedBy(at))
        .expect("just spawned");
    true
}

/// Free, passable tiles a creature could walk to from a spawner within `SPAWNER_RADIUS`,
/// in the order a walk outward finds them.
fn reachable_ground(world: &World, res: &mut Resources, at: Position) -> Vec<Position> {
    let occupied: std::collections::HashSet<Position> = world
        .query::<&Position>()
        .with::<&crate::components::BlocksMovement>()
        .iter()
        .map(|(_, p)| *p)
        .collect();
    let mut seen = std::collections::HashSet::from([at]);
    let mut queue = std::collections::VecDeque::from([at]);
    let mut open = Vec::new();
    while let Some(here) = queue.pop_front() {
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let next = Position {
                x: here.x + dx,
                y: here.y + dy,
                z: here.z,
            };
            if distance(next, at) > SPAWNER_RADIUS || !seen.insert(next) {
                continue;
            }
            let passable = res
                .world_state
                .world
                .get_tile_cached(next.x, next.y, next.z)
                .is_passable();
            if passable {
                queue.push_back(next);
                if !occupied.contains(&next) {
                    open.push(next);
                }
            }
        }
    }
    open
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::EntityKind;
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
    fn creatures_drop_embryos_of_their_kind_at_about_the_set_rates() {
        let game = TestWorld::new(5).build().game;
        let world = &game.res.world_state.world;
        let mut by_grade = std::collections::HashMap::new();
        for i in 0..10_000 {
            let at = Position { x: i, y: 3, z: 0 };
            if let Some(item) = roll_embryo(world, EnemyKind::NuclearSlime, at, i as u64 * 7) {
                let (creature, grade) = embryo_kind(item).unwrap();
                assert_eq!(creature, EnemyKind::NuclearSlime);
                *by_grade.entry(grade).or_insert(0) += 1;
            }
        }
        for grade in EmbryoGrade::ALL {
            let percent = by_grade[&grade] as f64 / 100.0;
            let expected = grade.drop_percent() as f64;
            assert!(
                (percent - expected).abs() <= expected * 0.3 + 0.3,
                "{grade:?}: {percent}%"
            );
        }
    }

    #[test]
    fn the_same_deaths_drop_the_same_embryos() {
        let deaths = |seed: u64| {
            let mut t = arena().build();
            t.game.res.world_state.world = crate::world::GameWorld::new(10, 10, seed);
            let mut drops = Vec::new();
            for i in 0..300 {
                let dog = crate::spawn_utils::world_spawn_feraldog(
                    &mut t.game.world,
                    i % 7,
                    i / 7,
                    0,
                    &t.game.res.world_state.world,
                );
                t.game.res.time.tick = 1000 + i as u64 * 13;
                crate::systems::handle_entity_death(&mut t.game.world, &mut t.game.res, dog);
                let embryos: Vec<ItemKind> = t
                    .game
                    .world
                    .query::<&crate::components::Inventory>()
                    .with::<&EntityKind>()
                    .iter()
                    .flat_map(|(_, inv)| inv.slots.clone())
                    .map(|s| s.kind)
                    .filter(|k| embryo_kind(*k).is_some())
                    .collect();
                drops.push(embryos.len());
            }
            drops
        };
        let drops = deaths(5);
        assert!(drops.last().unwrap() > &0, "some dog dropped an embryo");
        assert_eq!(deaths(5), drops, "same seed, same drops");
        assert_ne!(deaths(6), drops, "another seed, other drops");
    }

    #[test]
    fn an_embryo_item_knows_its_creature_and_grade() {
        for creature in EnemyKind::ALL {
            for grade in EmbryoGrade::ALL {
                assert_eq!(
                    embryo_kind(embryo(*creature, grade)),
                    Some((*creature, grade))
                );
            }
        }
        assert_eq!(embryo_kind(ItemKind::Meat), None);
        assert!(EmbryoGrade::Poor.charges() < EmbryoGrade::Flawed.charges());
        assert!(EmbryoGrade::Flawed.charges() < EmbryoGrade::Flawless.charges());
    }

    /// A placed spawner at (0, 2), on a pole fed by a generator holding `fuel` biochar.
    fn farm(fuel: u32) -> (TestGame, hecs::Entity) {
        let mut t = arena().build();
        let g = &mut t.game;
        let put = |g: &mut crate::game::Game, kind, pos| {
            crate::factory::place(&mut g.world, &mut g.res, kind, pos, Facing::North).unwrap()
        };
        let generator = put(g, FactoryKind::BurnerGenerator, at(-3, 2));
        g.world
            .get::<&mut crate::components::Inventory>(generator)
            .unwrap()
            .add(ItemKind::Biochar, fuel);
        put(g, FactoryKind::PowerPole, at(-2, 2));
        let spawner = put(g, FactoryKind::Spawner, at(0, 2));
        (t, spawner)
    }

    fn feed(t: &mut TestGame, spawner: hecs::Entity, item: ItemKind, qty: u32) {
        for _ in 0..qty {
            assert!(crate::factory::insert(&mut t.game.world, spawner, item));
        }
    }

    fn state(t: &TestGame, spawner: hecs::Entity) -> Spawner {
        *t.game.world.get::<&Spawner>(spawner).unwrap()
    }

    fn count(t: &TestGame, kind: EntityKind) -> usize {
        t.game
            .world
            .query::<&EntityKind>()
            .iter()
            .filter(|(_, k)| **k == kind)
            .count()
    }

    #[test]
    fn a_fed_powered_spawner_grows_until_its_embryo_is_spent() {
        let (mut t, spawner) = farm(20);
        feed(&mut t, spawner, ItemKind::MeatSlurry, 5);
        feed(&mut t, spawner, ItemKind::FeralDogEmbryoPoor, 1);
        pass(&mut t, SPAWN_INTERVAL_TICKS);
        assert_eq!(dogs(&t), 1);
        assert_eq!(state(&t, spawner).slurry, 4, "one slurry a creature");
        pass(&mut t, 5 * SPAWN_INTERVAL_TICKS);
        assert_eq!(dogs(&t), 2, "a poor embryo grows two");
        let spent = state(&t, spawner);
        assert_eq!((spent.slurry, spent.embryos, spent.embryo), (3, 0, None));
        let consumer = *t.game.world.get::<&PowerConsumer>(spawner).unwrap();
        assert!(!consumer.active, "with no embryo it draws nothing");
    }

    #[test]
    fn an_unpowered_spawner_waits_and_keeps_what_it_holds() {
        let (mut t, spawner) = farm(0);
        feed(&mut t, spawner, ItemKind::MeatSlurry, 2);
        feed(&mut t, spawner, ItemKind::FeralDogEmbryoFlawed, 1);
        pass(&mut t, 5 * SPAWN_INTERVAL_TICKS);
        assert_eq!(dogs(&t), 0);
        let held = state(&t, spawner);
        assert_eq!((held.slurry, held.embryos, held.charges), (2, 1, 4));
        // Power comes back
        let generator = t.game.res.factory_index[&at(-3, 2)];
        t.game
            .world
            .get::<&mut crate::components::Inventory>(generator)
            .unwrap()
            .add(ItemKind::Biochar, 10);
        pass(&mut t, SPAWN_INTERVAL_TICKS + FACTORY_STEP_TICKS);
        assert_eq!(dogs(&t), 1);
    }

    #[test]
    fn a_spawner_out_of_slurry_stops_and_its_embryo_keeps_its_charges() {
        let (mut t, spawner) = farm(20);
        feed(&mut t, spawner, ItemKind::MeatSlurry, 1);
        feed(&mut t, spawner, ItemKind::FeralDogEmbryoFlawed, 1);
        pass(&mut t, 5 * SPAWN_INTERVAL_TICKS);
        assert_eq!(dogs(&t), 1);
        let held = state(&t, spawner);
        assert_eq!((held.slurry, held.embryos, held.charges), (0, 1, 3));
        assert_eq!(held.creature(), None);
    }

    #[test]
    fn the_embryo_sets_the_creature() {
        let (mut t, spawner) = farm(20);
        feed(&mut t, spawner, ItemKind::MeatSlurry, 1);
        feed(&mut t, spawner, ItemKind::NuclearSlimeEmbryoPoor, 1);
        assert!(
            !crate::factory::accepts(&t.game.world, spawner, ItemKind::FeralDogEmbryoPoor),
            "one kind of embryo at a time"
        );
        pass(&mut t, SPAWN_INTERVAL_TICKS);
        assert_eq!(count(&t, EntityKind::NuclearSlime), 1);
        assert_eq!(dogs(&t), 0);
    }

    #[test]
    fn a_spawned_embryo_makes_way_for_the_next() {
        let (mut t, spawner) = farm(40);
        feed(&mut t, spawner, ItemKind::MeatSlurry, 10);
        feed(&mut t, spawner, ItemKind::FeralDogEmbryoPoor, 2);
        pass(&mut t, 4 * SPAWN_INTERVAL_TICKS);
        assert_eq!(dogs(&t), 4, "two poor embryos grow four");
        assert_eq!(state(&t, spawner).embryos, 0);
    }

    #[test]
    fn an_extractor_feeds_a_spawner_from_a_chest() {
        let (mut t, spawner) = farm(20);
        let g = &mut t.game;
        let chest = crate::factory::place(
            &mut g.world,
            &mut g.res,
            FactoryKind::Chest,
            at(2, 2),
            Facing::North,
        )
        .unwrap();
        {
            let mut inv = g
                .world
                .get::<&mut crate::components::Inventory>(chest)
                .unwrap();
            inv.add(ItemKind::MeatSlurry, 3);
            inv.add(ItemKind::FeralDogEmbryoPoor, 1);
        }
        crate::factory::place(
            &mut g.world,
            &mut g.res,
            FactoryKind::Extractor,
            at(1, 2),
            Facing::West,
        )
        .unwrap();
        pass(&mut t, 20 * FACTORY_STEP_TICKS);
        let fed = state(&t, spawner);
        assert_eq!((fed.slurry, fed.embryos), (3, 1));
    }

    #[test]
    fn breaking_a_placed_spawner_gives_back_what_it_held() {
        let (mut t, spawner) = farm(0);
        feed(&mut t, spawner, ItemKind::MeatSlurry, 3);
        feed(&mut t, spawner, ItemKind::FeralDogEmbryoFlawless, 2);
        let player = t.game.get_player_entity().unwrap();
        t.game
            .world
            .get::<&mut crate::components::Inventory>(player)
            .unwrap()
            .slots
            .clear();
        crate::factory::break_block(&mut t.game.world, &mut t.game.res, at(0, 2), player);
        assert_eq!(t.player_inventory(), vec![(ItemKind::Spawner, 1)]);
        let mut dropped = t.items_at(0, 2, 0);
        dropped.sort();
        assert_eq!(
            dropped,
            vec![
                (ItemKind::MeatSlurry, 3),
                (ItemKind::FeralDogEmbryoFlawless, 2)
            ]
        );
    }

    #[test]
    fn the_look_panel_names_what_a_spawner_grows_and_lacks() {
        let wild = Spawner {
            wild: Some(EnemyKind::NuclearSlime),
            elapsed: SPAWN_INTERVAL_TICKS - 250,
            ..Default::default()
        };
        assert_eq!(
            wild.status_lines(),
            vec!["Grows nuclear slimes", "Next in 3s"]
        );
        let mut placed = Spawner::default();
        assert_eq!(
            placed.status_lines(),
            vec![
                "Needs an embryo",
                "Slurry 0, embryos 0",
                "Needs meat slurry"
            ]
        );
        placed.insert(ItemKind::FeralDogEmbryoFlawed);
        assert_eq!(
            placed.status_lines(),
            vec![
                "Grows feral dogs",
                "Slurry 0, embryos 1",
                "4 left in embryo",
                "Needs meat slurry"
            ]
        );
    }

    #[test]
    fn a_fenced_spawner_lets_creatures_out_only_inside_the_fence() {
        let mut t = arena().build();
        let g = &mut t.game;
        place_wild(&mut g.world, &mut g.res, at(8, 0), EnemyKind::FeralDog).unwrap();
        // A 3x3 pen around the spawner: room for eight dogs, all inside
        for x in 6..=10 {
            for y in -2..=2 {
                if x == 6 || x == 10 || y == -2 || y == 2 {
                    crate::factory::place(
                        &mut g.world,
                        &mut g.res,
                        FactoryKind::Fence,
                        at(x, y),
                        Facing::North,
                    )
                    .unwrap();
                }
            }
        }
        pass(&mut t, 10 * SPAWN_INTERVAL_TICKS);
        let dogs: Vec<Position> = t
            .game
            .world
            .query::<&Position>()
            .with::<&SpawnedBy>()
            .iter()
            .map(|(_, p)| *p)
            .collect();
        assert_eq!(dogs.len(), SPAWNER_CAP);
        assert!(
            dogs.iter()
                .all(|p| (7..=9).contains(&p.x) && (-1..=1).contains(&p.y)),
            "{dogs:?}"
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

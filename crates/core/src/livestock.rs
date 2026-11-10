//! Animal farming: feeding and breeding sheep, and the blocks that turn a pen into a farm.
//!
//! - **Breeding:** a sheep fed a carrot or mash is `Fed` for 2 game minutes. Two fed adults
//!   within 3 tiles breed: a lamb appears on an open tile beside them and both rest for 5
//!   minutes (`BreedCooldown`). Lambs grow up in 5 minutes. No sheep breed where 16 or more
//!   already stand within 8 tiles.
//! - **Auto-feeder:** holds feed and feeds the nearest hungry adult within 3 tiles every 20
//!   game seconds. It needs no power.
//! - **Animal counter:** counts creatures by kind in a square around itself (5x5 to 15x15).
//! - **Livestock processor:** a salvaged SapienCorp slaughter unit. While powered it deals
//!   10 damage every 2 game seconds to every creature on the 3 tiles in front of it, and
//!   keeps what they drop. It never touches the player or people.
//! - **Bio-vat:** grows biomechanical embryos from meat, mash and scrap electronics on
//!   power. Each batch rolls a grade from the world seed, the vat's position and its batch
//!   count; 2 extra meat improve the odds.
//!
//! All of it runs in the factory step, after machines.

use hecs::{Entity, World};
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

use crate::components::{
    BlocksMovement, Dead, EntityKind, Health, Inventory, ItemKind, ItemStack, Position, Sheep,
    SpriteRef,
};
use crate::factory::{FactoryBlock, FactoryKind, FACTORY_STEP_TICKS, TICKS_PER_SECOND};
use crate::power::PowerConsumer;
use crate::resources::Resources;

const MINUTE: u64 = 60 * TICKS_PER_SECOND;
/// How long one feeding keeps a sheep ready to breed.
pub const FED_TICKS: u64 = 2 * MINUTE;
/// How long a sheep rests after breeding before it can breed again.
pub const BREED_COOLDOWN_TICKS: u64 = 5 * MINUTE;
/// How long a lamb takes to grow into a sheep.
pub const LAMB_GROW_TICKS: u64 = 5 * MINUTE;
/// Two fed sheep breed when this close (in tiles, either way).
pub const BREED_RADIUS: i64 = 3;
/// No breeding where this many sheep stand within `CROWD_RADIUS` tiles.
pub const CROWD_CAP: usize = 16;
pub const CROWD_RADIUS: i64 = 8;
/// A lamb's health; it reaches a sheep's when it grows up.
pub const LAMB_HEALTH: u32 = 25;
pub const SHEEP_HEALTH: u32 = 50;

/// Feeders reach this far and feed one sheep this often.
pub const FEEDER_RADIUS: i64 = 3;
pub const FEEDER_PERIOD_TICKS: u64 = 20 * TICKS_PER_SECOND;
pub const FEEDER_SLOTS: usize = 2;

/// Counter squares: 5x5 up to 15x15, in steps of two.
pub const COUNTER_SIZES: [i64; 6] = [5, 7, 9, 11, 13, 15];

/// The processor's damage, how often it strikes at full power, and what it draws.
pub const PROCESSOR_DAMAGE: u32 = 10;
pub const PROCESSOR_PERIOD_TICKS: u64 = 2 * TICKS_PER_SECOND;
pub const PROCESSOR_W: u64 = 60_000;
pub const PROCESSOR_SLOTS: usize = 4;

/// A bio-vat batch, how long it takes at full power, and what the vat draws.
pub const BATCH_MEAT: u32 = 4;
pub const BATCH_MASH: u32 = 2;
pub const BATCH_ELECTRONICS: u32 = 1;
/// Meat beyond a batch's that improves its odds, if the vat has it.
pub const BONUS_MEAT: u32 = 2;
pub const BATCH_TICKS: u64 = 2 * MINUTE;
pub const BIO_VAT_W: u64 = 120_000;
pub const BIO_VAT_SLOTS: usize = 6;
/// Embryos a bio-vat holds before it stops.
pub const BIO_VAT_OUTPUT_CAP: u32 = 8;

/// A sheep that has eaten, until `until_tick`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fed {
    pub until_tick: u64,
}

/// A sheep resting after breeding, until `until_tick`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BreedCooldown {
    pub until_tick: u64,
}

/// A young sheep, until it grows up at `grows_at_tick`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lamb {
    pub grows_at_tick: u64,
}

/// An auto-feeder's timer: it next feeds at `next_tick`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Feeder {
    pub next_tick: u64,
}

/// An animal counter: the side of its square and what it last counted, by kind.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnimalCounter {
    pub size: i64,
    pub counts: Vec<(EntityKind, u32)>,
}

impl AnimalCounter {
    pub fn new() -> Self {
        Self {
            size: COUNTER_SIZES[0],
            counts: Vec::new(),
        }
    }

    /// The next square size, wrapping from the largest back to the smallest.
    pub fn next_size(&mut self) {
        let i = COUNTER_SIZES.iter().position(|&s| s == self.size);
        self.size = COUNTER_SIZES[i.map_or(0, |i| (i + 1) % COUNTER_SIZES.len())];
    }

    /// "3 sheep, 1 feral dog", or "no animals".
    pub fn summary(&self) -> String {
        if self.counts.is_empty() {
            return "no animals".to_string();
        }
        self.counts
            .iter()
            .map(|&(kind, n)| format!("{n} {}", kind_name(kind, n)))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// The livestock processor's progress towards its next strike, in thousandths of a tick at
/// full power.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Processor {
    pub progress: u64,
}

/// A bio-vat's progress on its current batch and how many batches it has finished.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BioVat {
    pub progress: u64,
    pub batches: u64,
}

/// An embryo grade from a bio-vat.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grade {
    Deformed,
    Wrinkly,
    Perfect,
}

impl Grade {
    pub fn item(self) -> ItemKind {
        match self {
            Grade::Deformed => ItemKind::BiomechEmbryoDeformed,
            Grade::Wrinkly => ItemKind::BiomechEmbryoWrinkly,
            Grade::Perfect => ItemKind::BiomechEmbryoPerfect,
        }
    }
}

/// Chances in percent (deformed, wrinkly, perfect): 70/25/5, and 60/30/10 with the bonus
/// meat.
pub fn grade_odds(bonus: bool) -> (u32, u32, u32) {
    if bonus {
        (60, 30, 10)
    } else {
        (70, 25, 5)
    }
}

/// The grade of batch number `batch` of the vat at `pos`.
pub fn roll_grade(res: &Resources, pos: Position, batch: u64, bonus: bool) -> Grade {
    let world = &res.world_state.world;
    let seed = world.mix_coords(pos.x, pos.y, pos.z) ^ batch.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    let roll = rand_chacha::ChaCha20Rng::seed_from_u64(seed).gen_range(0..100);
    let (deformed, wrinkly, _) = grade_odds(bonus);
    if roll < deformed {
        Grade::Deformed
    } else if roll < deformed + wrinkly {
        Grade::Wrinkly
    } else {
        Grade::Perfect
    }
}

/// Whether sheep eat this.
pub fn is_feed(item: ItemKind) -> bool {
    matches!(item, ItemKind::Carrot | ItemKind::Mash)
}

/// Whether a bio-vat takes this as an ingredient.
pub fn is_vat_input(item: ItemKind) -> bool {
    matches!(
        item,
        ItemKind::Meat | ItemKind::Mash | ItemKind::ScrapElectronics
    )
}

pub fn is_embryo(item: ItemKind) -> bool {
    matches!(
        item,
        ItemKind::BiomechEmbryoDeformed
            | ItemKind::BiomechEmbryoWrinkly
            | ItemKind::BiomechEmbryoPerfect
    )
}

fn kind_name(kind: EntityKind, n: u32) -> &'static str {
    match (kind, n == 1) {
        (EntityKind::Sheep, _) => "sheep",
        (EntityKind::FeralDog, true) => "feral dog",
        (EntityKind::FeralDog, false) => "feral dogs",
        (EntityKind::NuclearSlime, true) => "nuclear slime",
        (EntityKind::NuclearSlime, false) => "nuclear slimes",
        (EntityKind::MiningBot, true) => "mining bot",
        (EntityKind::MiningBot, false) => "mining bots",
        _ => "other",
    }
}

/// Whether farm blocks act on this kind: animals and enemies, never the player, people
/// or corpses.
pub fn is_animal(kind: EntityKind) -> bool {
    matches!(
        kind,
        EntityKind::Sheep | EntityKind::FeralDog | EntityKind::NuclearSlime | EntityKind::MiningBot
    )
}

/// Why a sheep won't eat.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedError {
    NotFeed,
    NotASheep,
    /// Lambs don't breed, so they aren't fed
    Lamb,
    /// Already fed, or resting after breeding
    NotHungry,
}

impl FeedError {
    pub fn message(self) -> &'static str {
        match self {
            FeedError::NotFeed => "Sheep only eat carrots and mash",
            FeedError::NotASheep => "Only sheep can be fed",
            FeedError::Lamb => "The lamb is too young to breed",
            FeedError::NotHungry => "The sheep isn't hungry",
        }
    }
}

/// Whether `sheep` would eat now.
pub fn hungry(world: &World, sheep: Entity, tick: u64) -> Result<(), FeedError> {
    if world.get::<&Sheep>(sheep).is_err() || world.get::<&Dead>(sheep).is_ok() {
        return Err(FeedError::NotASheep);
    }
    if world.get::<&Lamb>(sheep).is_ok() {
        return Err(FeedError::Lamb);
    }
    let fed = world.get::<&Fed>(sheep).is_ok_and(|f| f.until_tick > tick);
    let resting = world
        .get::<&BreedCooldown>(sheep)
        .is_ok_and(|c| c.until_tick > tick);
    if fed || resting {
        return Err(FeedError::NotHungry);
    }
    Ok(())
}

/// Feed `sheep` one `item`. Does not touch any inventory.
pub fn feed(
    world: &mut World,
    res: &Resources,
    sheep: Entity,
    item: ItemKind,
) -> Result<(), FeedError> {
    if !is_feed(item) {
        return Err(FeedError::NotFeed);
    }
    hungry(world, sheep, res.time.tick)?;
    let _ = world.insert_one(
        sheep,
        Fed {
            until_tick: res.time.tick + FED_TICKS,
        },
    );
    Ok(())
}

/// The first feed in `holder`'s inventory.
pub fn first_feed(world: &World, holder: Entity) -> Option<ItemKind> {
    world
        .get::<&Inventory>(holder)
        .ok()?
        .slots
        .iter()
        .find(|s| s.qty > 0 && is_feed(s.kind))
        .map(|s| s.kind)
}

/// The player feeds `sheep` the first feed they carry, which leaves their inventory.
pub fn player_feeds(
    world: &mut World,
    res: &Resources,
    player: Entity,
    sheep: Entity,
) -> Result<ItemKind, FeedError> {
    let item = first_feed(world, player).ok_or(FeedError::NotFeed)?;
    feed(world, res, sheep, item)?;
    take_one(world, player, item);
    Ok(item)
}

/// Spawn a lamb at `pos`, growing up `LAMB_GROW_TICKS` after `tick`.
pub fn spawn_lamb(world: &mut World, pos: Position, tick: u64) -> Entity {
    let lamb = crate::spawn_utils::world_spawn_sheep(world, pos.x, pos.y, pos.z);
    let _ = world.insert(
        lamb,
        (
            Lamb {
                grows_at_tick: tick + LAMB_GROW_TICKS,
            },
            Health::new(LAMB_HEALTH),
            SpriteRef::new("entities", "lamb"),
        ),
    );
    lamb
}

fn chebyshev(a: Position, b: Position) -> i64 {
    (a.x - b.x).abs().max((a.y - b.y).abs())
}

fn near(a: Position, b: Position, r: i64) -> bool {
    a.z == b.z && chebyshev(a, b) <= r
}

/// Whether a creature could stand at `pos`: walkable, with no blocking entity or factory
/// block.
fn open_tile(world: &World, res: &mut Resources, pos: Position) -> bool {
    res.world_state
        .world
        .get_tile_cached(pos.x, pos.y, pos.z)
        .is_passable()
        && !res.factory_index.contains_key(&pos)
        && !world
            .query::<&Position>()
            .with::<&BlocksMovement>()
            .iter()
            .any(|(_, p)| *p == pos)
}

/// Grow lambs, let feeding and rest wear off, and breed fed pairs.
fn step_breeding(world: &mut World, res: &mut Resources, tick: u64) {
    let grown: Vec<Entity> = world
        .query::<&Lamb>()
        .iter()
        .filter(|(_, l)| l.grows_at_tick <= tick)
        .map(|(e, _)| e)
        .collect();
    for sheep in grown {
        let _ = world.remove_one::<Lamb>(sheep);
        if let Ok(mut health) = world.get::<&mut Health>(sheep) {
            health.max = SHEEP_HEALTH;
            health.current += SHEEP_HEALTH - LAMB_HEALTH;
        }
        let _ = world.insert_one(sheep, SpriteRef::new("entities", "sheep"));
    }
    let worn_off: Vec<Entity> = world
        .query::<&Fed>()
        .iter()
        .filter(|(_, f)| f.until_tick <= tick)
        .map(|(e, _)| e)
        .collect();
    for sheep in worn_off {
        let _ = world.remove_one::<Fed>(sheep);
    }
    let rested: Vec<Entity> = world
        .query::<&BreedCooldown>()
        .iter()
        .filter(|(_, c)| c.until_tick <= tick)
        .map(|(e, _)| e)
        .collect();
    for sheep in rested {
        let _ = world.remove_one::<BreedCooldown>(sheep);
    }

    let mut ready: Vec<(Entity, Position)> = world
        .query::<&Position>()
        .with::<(&Sheep, &Fed)>()
        .without::<&Lamb>()
        .without::<&BreedCooldown>()
        .without::<&Dead>()
        .iter()
        .map(|(e, p)| (e, *p))
        .collect();
    ready.sort_by_key(|(_, p)| (p.z, p.y, p.x));
    let sheep: Vec<Position> = world
        .query::<&Position>()
        .with::<&Sheep>()
        .without::<&Dead>()
        .iter()
        .map(|(_, p)| *p)
        .collect();
    let mut used = vec![false; ready.len()];
    for i in 0..ready.len() {
        for j in i + 1..ready.len() {
            if used[i] || used[j] {
                continue;
            }
            let ((a, pa), (b, pb)) = (ready[i], ready[j]);
            if !near(pa, pb, BREED_RADIUS) {
                continue;
            }
            let crowd = sheep.iter().filter(|&&p| near(p, pa, CROWD_RADIUS)).count();
            if crowd >= CROWD_CAP {
                continue;
            }
            let Some(spot) = lamb_spot(world, res, pa, pb, tick) else {
                continue;
            };
            used[i] = true;
            used[j] = true;
            spawn_lamb(world, spot, tick);
            for parent in [a, b] {
                let _ = world.remove_one::<Fed>(parent);
                let _ = world.insert_one(
                    parent,
                    BreedCooldown {
                        until_tick: tick + BREED_COOLDOWN_TICKS,
                    },
                );
            }
        }
    }
}

/// An open tile beside either parent, chosen by a roll seeded by the first parent's tile
/// and the tick.
fn lamb_spot(
    world: &World,
    res: &mut Resources,
    a: Position,
    b: Position,
    tick: u64,
) -> Option<Position> {
    let mut spots: Vec<Position> = Vec::new();
    for parent in [a, b] {
        for dy in -1..=1 {
            for dx in -1..=1 {
                let p = Position {
                    x: parent.x + dx,
                    y: parent.y + dy,
                    z: parent.z,
                };
                if !spots.contains(&p) && open_tile(world, res, p) {
                    spots.push(p);
                }
            }
        }
    }
    if spots.is_empty() {
        return None;
    }
    let seed = res.world_state.world.mix_coords(a.x, a.y, a.z) ^ tick;
    let i = rand_chacha::ChaCha20Rng::seed_from_u64(seed).gen_range(0..spots.len());
    Some(spots[i])
}

/// Every factory block of `kind` with its position, in position order.
fn blocks(world: &World, kind: FactoryKind) -> Vec<(Entity, Position, FactoryBlock)> {
    let mut found: Vec<_> = world
        .query::<(&Position, &FactoryBlock)>()
        .iter()
        .filter(|(_, (_, b))| b.kind == kind)
        .map(|(e, (p, b))| (e, *p, *b))
        .collect();
    found.sort_by_key(|(_, p, _)| (p.z, p.y, p.x));
    found
}

fn take_one(world: &mut World, holder: Entity, item: ItemKind) -> bool {
    let Ok(mut inv) = world.get::<&mut Inventory>(holder) else {
        return false;
    };
    let Some(i) = inv.slots.iter().position(|s| s.kind == item && s.qty > 0) else {
        return false;
    };
    inv.slots[i].qty -= 1;
    if inv.slots[i].qty == 0 {
        inv.slots.remove(i);
    }
    true
}

fn count(inv: &Inventory, item: ItemKind) -> u32 {
    inv.slots
        .iter()
        .filter(|s| s.kind == item)
        .map(|s| s.qty)
        .sum()
}

/// Feeders with feed whose time has come feed the nearest hungry adult in reach.
fn step_feeders(world: &mut World, res: &mut Resources, tick: u64) {
    for (feeder, pos, _) in blocks(world, FactoryKind::AutoFeeder) {
        let due = world
            .get::<&Feeder>(feeder)
            .is_ok_and(|f| f.next_tick <= tick);
        if !due {
            continue;
        }
        let Some(item) = world
            .get::<&Inventory>(feeder)
            .ok()
            .and_then(|inv| inv.slots.iter().find(|s| is_feed(s.kind)).map(|s| s.kind))
        else {
            continue;
        };
        let mut hungry_sheep: Vec<(Entity, Position)> = world
            .query::<&Position>()
            .with::<&Sheep>()
            .iter()
            .filter(|(_, p)| near(**p, pos, FEEDER_RADIUS))
            .map(|(e, p)| (e, *p))
            .filter(|&(e, _)| hungry(world, e, tick).is_ok())
            .collect();
        hungry_sheep.sort_by_key(|&(_, p)| (chebyshev(p, pos), p.y, p.x));
        let Some(&(sheep, _)) = hungry_sheep.first() else {
            continue;
        };
        if feed(world, res, sheep, item).is_ok() {
            take_one(world, feeder, item);
            if let Ok(mut f) = world.get::<&mut Feeder>(feeder) {
                f.next_tick = tick + FEEDER_PERIOD_TICKS;
            }
        }
    }
}

/// Every counter counts the creatures in its square, by kind.
fn step_counters(world: &mut World) {
    let creatures: Vec<(EntityKind, Position)> = world
        .query::<(&EntityKind, &Position)>()
        .without::<&Dead>()
        .iter()
        .filter(|(_, (k, _))| is_animal(**k))
        .map(|(_, (k, p))| (*k, *p))
        .collect();
    for (counter, (pos, c)) in world.query_mut::<(&Position, &mut AnimalCounter)>() {
        let _ = counter;
        let half = c.size / 2;
        let mut counts: Vec<(EntityKind, u32)> = Vec::new();
        for &(kind, p) in &creatures {
            if near(p, *pos, half) {
                match counts.iter_mut().find(|(k, _)| *k == kind) {
                    Some((_, n)) => *n += 1,
                    None => counts.push((kind, 1)),
                }
            }
        }
        counts.sort_by_key(|&(k, _)| k as u8);
        c.counts = counts;
    }
}

/// The 3 tiles in front of a processor: straight ahead and either side of it.
pub fn processor_targets(block: FactoryBlock, pos: Position) -> [Position; 3] {
    let (dx, dy) = block.facing.offset();
    let front = Position {
        x: pos.x + dx,
        y: pos.y + dy,
        z: pos.z,
    };
    // Across the facing: swap the axes
    let (sx, sy) = (dy, dx);
    [
        Position {
            x: front.x - sx,
            y: front.y - sy,
            ..front
        },
        front,
        Position {
            x: front.x + sx,
            y: front.y + sy,
            ..front
        },
    ]
}

/// The animals standing in front of a processor, in position order.
fn processor_victims(world: &World, block: FactoryBlock, pos: Position) -> Vec<Entity> {
    let targets = processor_targets(block, pos);
    let mut found: Vec<(Entity, Position)> = world
        .query::<(&EntityKind, &Position)>()
        .with::<&Health>()
        .without::<&Dead>()
        .iter()
        .filter(|(_, (k, p))| is_animal(**k) && targets.contains(p))
        .map(|(e, (_, p))| (e, *p))
        .collect();
    found.sort_by_key(|&(e, p)| (p.y, p.x, e.to_bits()));
    found.into_iter().map(|(e, _)| e).collect()
}

/// Whether a bio-vat holds a full batch and has room for an embryo.
fn vat_ready(inv: &Inventory) -> bool {
    count(inv, ItemKind::Meat) >= BATCH_MEAT
        && count(inv, ItemKind::Mash) >= BATCH_MASH
        && count(inv, ItemKind::ScrapElectronics) >= BATCH_ELECTRONICS
        && inv
            .slots
            .iter()
            .filter(|s| is_embryo(s.kind))
            .map(|s| s.qty)
            .sum::<u32>()
            < BIO_VAT_OUTPUT_CAP
}

/// Before power is shared: processors want power only with an animal in front, and
/// bio-vats only with a full batch.
pub fn update_demand(world: &mut World) {
    for (processor, pos, block) in blocks(world, FactoryKind::LivestockProcessor) {
        let working = !processor_victims(world, block, pos).is_empty();
        if let Ok(mut c) = world.get::<&mut PowerConsumer>(processor) {
            c.active = working;
        }
    }
    for (vat, (inv, consumer)) in world
        .query_mut::<(&Inventory, &mut PowerConsumer)>()
        .with::<&BioVat>()
    {
        let _ = vat;
        consumer.active = vat_ready(inv);
    }
}

fn satisfaction(world: &World, e: Entity) -> u64 {
    world.get::<&PowerConsumer>(e).map_or(0, |c| {
        if c.active {
            c.satisfaction_permille as u64
        } else {
            0
        }
    })
}

/// Processors strike what is in front of them and collect what it drops.
fn step_processors(world: &mut World, res: &mut Resources) {
    for (processor, pos, block) in blocks(world, FactoryKind::LivestockProcessor) {
        let gained = satisfaction(world, processor) * FACTORY_STEP_TICKS;
        let strike = {
            let Ok(mut p) = world.get::<&mut Processor>(processor) else {
                continue;
            };
            p.progress += gained;
            if gained == 0 {
                p.progress = 0;
            }
            let strike = p.progress >= PROCESSOR_PERIOD_TICKS * 1000;
            if strike {
                p.progress -= PROCESSOR_PERIOD_TICKS * 1000;
            }
            strike
        };
        if !strike {
            continue;
        }
        for victim in processor_victims(world, block, pos) {
            let Ok(victim_pos) = world.get::<&Position>(victim).map(|p| *p) else {
                continue;
            };
            let dead = {
                let Ok(mut health) = world.get::<&mut Health>(victim) else {
                    continue;
                };
                health.damage(PROCESSOR_DAMAGE);
                health.current == 0
            };
            if dead {
                crate::systems::handle_machine_kill(world, res, victim);
                collect_corpses(world, processor, victim_pos);
            }
        }
    }
}

/// Move everything corpses at `pos` hold into the processor, and clear them away.
fn collect_corpses(world: &mut World, processor: Entity, pos: Position) {
    let corpses: Vec<Entity> = world
        .query::<(&EntityKind, &Position)>()
        .iter()
        .filter(|(_, (k, p))| **k == EntityKind::Corpse && **p == pos)
        .map(|(e, _)| e)
        .collect();
    for corpse in corpses {
        let items = world
            .get::<&Inventory>(corpse)
            .map(|inv| inv.slots.clone())
            .unwrap_or_default();
        if let Ok(mut inv) = world.get::<&mut Inventory>(processor) {
            for stack in items {
                inv.add(stack.kind, stack.qty);
            }
        }
        let _ = world.despawn(corpse);
    }
}

/// Bio-vats work their batch at their power and grow an embryo when it is done.
fn step_bio_vats(world: &mut World, res: &mut Resources) {
    for (vat, pos, _) in blocks(world, FactoryKind::BioVat) {
        let ready = world
            .get::<&Inventory>(vat)
            .is_ok_and(|inv| vat_ready(&inv));
        let gained = if ready {
            satisfaction(world, vat) * FACTORY_STEP_TICKS
        } else {
            0
        };
        let done = {
            let Ok(mut v) = world.get::<&mut BioVat>(vat) else {
                continue;
            };
            if !ready {
                v.progress = 0;
                continue;
            }
            v.progress += gained;
            v.progress >= BATCH_TICKS * 1000
        };
        if !done {
            continue;
        }
        let batch = {
            let mut v = world.get::<&mut BioVat>(vat).expect("checked above");
            v.progress = 0;
            v.batches += 1;
            v.batches
        };
        let bonus = world
            .get::<&Inventory>(vat)
            .is_ok_and(|inv| count(&inv, ItemKind::Meat) >= BATCH_MEAT + BONUS_MEAT);
        let meat = BATCH_MEAT + if bonus { BONUS_MEAT } else { 0 };
        for (item, qty) in [
            (ItemKind::Meat, meat),
            (ItemKind::Mash, BATCH_MASH),
            (ItemKind::ScrapElectronics, BATCH_ELECTRONICS),
        ] {
            for _ in 0..qty {
                take_one(world, vat, item);
            }
        }
        let grade = roll_grade(res, pos, batch, bonus);
        if let Ok(mut inv) = world.get::<&mut Inventory>(vat) {
            inv.add(grade.item(), 1);
        }
    }
}

/// How far a bio-vat's batch is, in percent.
pub fn vat_progress_percent(world: &World, vat: Entity) -> u64 {
    world
        .get::<&BioVat>(vat)
        .map_or(0, |v| v.progress * 100 / (BATCH_TICKS * 1000))
}

/// Whether a bio-vat holds a full batch and room for its embryo.
pub fn vat_has_batch(world: &World, vat: Entity) -> bool {
    world
        .get::<&Inventory>(vat)
        .is_ok_and(|inv| vat_ready(&inv))
}

/// One factory step of farming, after power is shared.
pub fn step(world: &mut World, res: &mut Resources, tick: u64) {
    step_breeding(world, res, tick);
    step_feeders(world, res, tick);
    step_processors(world, res);
    step_bio_vats(world, res);
    step_counters(world);
}

/// Whether the block `kind` takes `item` from the player or a belt, however full it is.
pub fn takes(kind: FactoryKind, item: ItemKind) -> bool {
    match kind {
        FactoryKind::AutoFeeder => is_feed(item),
        FactoryKind::BioVat => is_vat_input(item),
        _ => false,
    }
}

/// Whether `target` (a feeder or bio-vat) takes `item` right now.
pub fn accepts(world: &World, target: Entity, kind: FactoryKind, item: ItemKind) -> bool {
    takes(kind, item)
        && world
            .get::<&Inventory>(target)
            .is_ok_and(|inv| inv.has_room_for(item))
}

/// Lines for the look panel: a counter's count, or a sheep's state.
pub fn look_lines(world: &World, entity: Entity, tick: u64) -> Vec<String> {
    if let Ok(c) = world.get::<&AnimalCounter>(entity) {
        return vec![
            format!("Counts a {0}x{0} square", c.size),
            capitalize(&c.summary()),
        ];
    }
    if world.get::<&Sheep>(entity).is_err() {
        return Vec::new();
    }
    let mut lines = Vec::new();
    if let Ok(lamb) = world.get::<&Lamb>(entity) {
        let left = lamb.grows_at_tick.saturating_sub(tick);
        lines.push(format!(
            "A lamb; grows up in {}s",
            left.div_ceil(TICKS_PER_SECOND)
        ));
    } else if world.get::<&Fed>(entity).is_ok() {
        lines.push("Fed; looking for a mate".to_string());
    } else if let Ok(rest) = world.get::<&BreedCooldown>(entity) {
        let left = rest.until_tick.saturating_sub(tick);
        lines.push(format!("Resting {}s", left.div_ceil(TICKS_PER_SECOND)));
    } else {
        lines.push("Hungry".to_string());
    }
    lines
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    c.next()
        .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
        .unwrap_or_default()
}

/// What a sheep carries when it dies: meat and maybe leather; a lamb, a little meat.
pub const SHEEP_LOOT: &[crate::crafting::CreatureLoot] = &[
    crate::crafting::CreatureLoot {
        item: ItemKind::Meat,
        chance_percent: 100,
        min: 1,
        max: 2,
    },
    crate::crafting::CreatureLoot {
        item: ItemKind::Leather,
        chance_percent: 100,
        min: 0,
        max: 1,
    },
];
pub const LAMB_LOOT: &[crate::crafting::CreatureLoot] = &[crate::crafting::CreatureLoot {
    item: ItemKind::Meat,
    chance_percent: 100,
    min: 1,
    max: 1,
}];

/// Roll what a dying sheep drops, seeded by where and when it died.
pub fn sheep_drops(world: &World, res: &Resources, sheep: Entity, pos: Position) -> Vec<ItemStack> {
    let table = if world.get::<&Lamb>(sheep).is_ok() {
        LAMB_LOOT
    } else {
        SHEEP_LOOT
    };
    let seed = res.world_state.world.mix_coords(pos.x, pos.y, pos.z) ^ res.time.tick;
    let mut rng = rand_chacha::ChaCha20Rng::seed_from_u64(seed);
    crate::crafting::roll_loot(table, &mut rng)
        .into_iter()
        .filter(|&(_, qty)| qty > 0)
        .map(|(item, qty)| ItemStack::new(item, qty))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::{self, Facing};
    use crate::intent::PlayerIntent;
    use crate::test_support::{TestGame, TestWorld};
    use crate::tiles::TileKind;

    fn at(x: i64, y: i64) -> Position {
        Position { x, y, z: 0 }
    }

    /// Grass from (-10, -10) to (10, 10), the player out of the way at (0, 30).
    fn meadow() -> TestWorld {
        TestWorld::new(4)
            .player_at(0, 30, 0)
            .fill((-10, -10, 0), (10, 10, 0), TileKind::Grass)
    }

    /// Advance the game by `ticks` in one waiting action.
    fn wait(game: &mut TestGame, ticks: u64) {
        game.game.res.player_state.intent = PlayerIntent::movement(0, 0, 0, ticks);
        game.tick();
    }

    fn sheep(game: &TestGame) -> Vec<Entity> {
        game.game
            .world
            .query::<&Sheep>()
            .without::<&Dead>()
            .iter()
            .map(|(e, _)| e)
            .collect()
    }

    fn lambs(game: &TestGame) -> usize {
        game.game.world.query::<&Lamb>().iter().count()
    }

    /// Keep sheep where they are: the wander system would move them out of reach.
    fn hobble(game: &mut TestGame) {
        for e in sheep(game) {
            let _ = game
                .game
                .world
                .insert_one(e, crate::components::BattleDelay::new(u64::MAX / 2));
        }
    }

    fn put(game: &mut TestGame, kind: FactoryKind, pos: Position, facing: Facing) -> Entity {
        factory::place(&mut game.game.world, &mut game.game.res, kind, pos, facing).expect("placed")
    }

    fn give(game: &mut TestGame, e: Entity, item: ItemKind, qty: u32) {
        game.game
            .world
            .get::<&mut Inventory>(e)
            .unwrap()
            .add(item, qty);
    }

    fn held(game: &TestGame, e: Entity, item: ItemKind) -> u32 {
        count(&game.game.world.get::<&Inventory>(e).unwrap(), item)
    }

    /// Two generators of logs and a pole at `pole`, powering 180 kW around it.
    fn power(game: &mut TestGame, pole: Position) {
        put(game, FactoryKind::PowerPole, pole, Facing::North);
        for dy in [1, -1] {
            let g = put(
                game,
                FactoryKind::BurnerGenerator,
                at(pole.x, pole.y + dy),
                Facing::North,
            );
            give(game, g, ItemKind::Log, 20);
        }
    }

    #[test]
    fn two_fed_sheep_breed_a_lamb_that_grows_up() {
        let mut world = meadow();
        let a = world.sheep(0, 0, 0);
        let b = world.sheep(2, 0, 0);
        let mut game = world.build();
        hobble(&mut game);
        let (a, b) = (game.entity(a), game.entity(b));
        let res = &game.game.res;
        assert_eq!(
            feed(&mut game.game.world, res, a, ItemKind::Wheat),
            Err(FeedError::NotFeed)
        );
        feed(&mut game.game.world, res, a, ItemKind::Carrot).unwrap();
        assert_eq!(
            feed(&mut game.game.world, res, a, ItemKind::Carrot),
            Err(FeedError::NotHungry)
        );
        wait(&mut game, FACTORY_STEP_TICKS);
        assert_eq!(lambs(&game), 0, "one fed sheep doesn't breed");
        let res = &game.game.res;
        feed(&mut game.game.world, res, b, ItemKind::Mash).unwrap();
        wait(&mut game, FACTORY_STEP_TICKS);
        assert_eq!(lambs(&game), 1);
        for parent in [a, b] {
            assert!(game.game.world.get::<&Fed>(parent).is_err());
            assert_eq!(
                hungry(&game.game.world, parent, game.game.res.time.tick),
                Err(FeedError::NotHungry),
                "resting"
            );
        }
        let lamb = game
            .game
            .world
            .query::<&Lamb>()
            .iter()
            .map(|(e, _)| e)
            .next()
            .unwrap();
        assert_eq!(
            game.game.world.get::<&Health>(lamb).unwrap().max,
            LAMB_HEALTH
        );
        assert_eq!(
            hungry(&game.game.world, lamb, game.game.res.time.tick),
            Err(FeedError::Lamb)
        );
        wait(&mut game, LAMB_GROW_TICKS);
        assert_eq!(lambs(&game), 0, "grown up");
        assert_eq!(
            game.game.world.get::<&Health>(lamb).unwrap().max,
            SHEEP_HEALTH
        );
        assert_eq!(
            hungry(&game.game.world, a, game.game.res.time.tick),
            Ok(()),
            "rested after 5 minutes"
        );
    }

    #[test]
    fn feeding_wears_off_and_sheep_too_far_apart_dont_breed() {
        let mut world = meadow();
        let a = world.sheep(0, 0, 0);
        let b = world.sheep(4, 0, 0);
        let mut game = world.build();
        hobble(&mut game);
        for h in [a, b] {
            let e = game.entity(h);
            let res = &game.game.res;
            feed(&mut game.game.world, res, e, ItemKind::Carrot).unwrap();
        }
        wait(&mut game, FED_TICKS + FACTORY_STEP_TICKS);
        assert_eq!(lambs(&game), 0, "four tiles apart");
        assert_eq!(
            game.game.world.query::<&Fed>().iter().count(),
            0,
            "worn off"
        );
    }

    #[test]
    fn crowds_stop_breeding() {
        let mut world = meadow();
        for i in 0..CROWD_CAP as i64 {
            world.sheep(i % 6 - 3, i / 6 - 3, 0);
        }
        let mut game = world.build();
        hobble(&mut game);
        for e in sheep(&game) {
            let res = &game.game.res;
            feed(&mut game.game.world, res, e, ItemKind::Carrot).unwrap();
        }
        wait(&mut game, FACTORY_STEP_TICKS);
        assert_eq!(
            lambs(&game),
            0,
            "{CROWD_CAP} sheep within {CROWD_RADIUS} tiles"
        );
    }

    #[test]
    fn breeding_is_deterministic() {
        let run = || {
            let mut world = meadow();
            let a = world.sheep(0, 0, 0);
            let b = world.sheep(1, 1, 0);
            let mut game = world.build();
            hobble(&mut game);
            for h in [a, b] {
                let e = game.entity(h);
                let res = &game.game.res;
                feed(&mut game.game.world, res, e, ItemKind::Carrot).unwrap();
            }
            wait(&mut game, FACTORY_STEP_TICKS);
            let spot = game
                .game
                .world
                .query::<&Position>()
                .with::<&Lamb>()
                .iter()
                .map(|(_, p)| *p)
                .next()
                .unwrap();
            spot
        };
        let spot = run();
        assert_eq!(run(), spot);
        assert!(chebyshev(spot, at(0, 0)) <= 1 || chebyshev(spot, at(1, 1)) <= 1);
    }

    #[test]
    fn a_feeder_feeds_the_nearest_hungry_sheep_every_20_seconds() {
        let mut world = meadow();
        let near_sheep = world.sheep(1, 0, 0);
        let far_sheep = world.sheep(3, 0, 0);
        let out_of_reach = world.sheep(-4, 0, 0);
        let mut game = world.build();
        hobble(&mut game);
        let feeder = put(&mut game, FactoryKind::AutoFeeder, at(0, 0), Facing::North);
        assert!(factory::accepts(&game.game.world, feeder, ItemKind::Mash));
        assert!(!factory::accepts(&game.game.world, feeder, ItemKind::Wheat));
        give(&mut game, feeder, ItemKind::Carrot, 5);
        wait(&mut game, FACTORY_STEP_TICKS);
        let fed = |game: &TestGame, h| game.game.world.get::<&Fed>(game.entity(h)).is_ok();
        assert!(fed(&game, near_sheep));
        assert!(!fed(&game, far_sheep), "one at a time");
        wait(&mut game, FEEDER_PERIOD_TICKS + 2 * FACTORY_STEP_TICKS);
        assert!(!fed(&game, out_of_reach));
        // The two fed sheep bred, so the third feeding goes to nobody in reach
        assert_eq!(lambs(&game), 1);
        assert_eq!(held(&game, feeder, ItemKind::Carrot), 3);
    }

    #[test]
    fn a_counter_counts_animals_by_kind_in_its_square() {
        let mut world = meadow();
        world.sheep(1, 0, 0);
        world.sheep(-2, 2, 0);
        world.sheep(4, 0, 0);
        world.feral_dog(0, -2, 0);
        let mut game = world.build();
        hobble(&mut game);
        let counter = put(
            &mut game,
            FactoryKind::AnimalCounter,
            at(0, 0),
            Facing::North,
        );
        wait(&mut game, FACTORY_STEP_TICKS);
        let counts = |game: &TestGame| {
            let c = game.game.world.get::<&AnimalCounter>(counter).unwrap();
            AnimalCounter::clone(&c)
        };
        assert_eq!(counts(&game).summary(), "2 sheep, 1 feral dog");
        game.game
            .world
            .get::<&mut AnimalCounter>(counter)
            .unwrap()
            .next_size();
        wait(&mut game, FACTORY_STEP_TICKS);
        assert_eq!(counts(&game).size, 7);
        assert_eq!(
            counts(&game).counts,
            [(EntityKind::Sheep, 2), (EntityKind::FeralDog, 1)]
        );
        let mut c = AnimalCounter::new();
        for _ in 0..COUNTER_SIZES.len() {
            c.next_size();
        }
        assert_eq!(c.size, COUNTER_SIZES[0], "wraps around");
    }

    #[test]
    fn the_processor_culls_what_is_in_front_and_keeps_the_drops() {
        let mut world = meadow().player_at(3, 0, 0);
        let victim = world.sheep(0, -1, 0);
        let beside = world.sheep(-1, -1, 0);
        let behind = world.sheep(0, 1, 0);
        let mut game = world.build();
        hobble(&mut game);
        let processor = put(
            &mut game,
            FactoryKind::LivestockProcessor,
            at(0, 0),
            Facing::North,
        );
        power(&mut game, at(3, 3));
        // 50 health at 10 every 2 game seconds: dead in about 10 seconds
        wait(&mut game, 12 * TICKS_PER_SECOND);
        for h in [victim, beside] {
            assert!(!game.game.world.contains(game.entity(h)), "culled");
        }
        assert!(
            game.game.world.contains(game.entity(behind)),
            "behind is safe"
        );
        let meat = held(&game, processor, ItemKind::Meat);
        assert!((2..=4).contains(&meat), "1-2 meat a sheep: {meat}");
        assert_eq!(
            game.game
                .world
                .query::<&EntityKind>()
                .iter()
                .filter(|(_, k)| **k == EntityKind::Corpse)
                .count(),
            0,
            "corpses collected"
        );
        assert!(!factory::accepts(
            &game.game.world,
            processor,
            ItemKind::Meat
        ));
    }

    #[test]
    fn the_processor_never_harms_the_player() {
        let mut game = meadow().player_at(0, -1, 0).build();
        put(
            &mut game,
            FactoryKind::LivestockProcessor,
            at(0, 0),
            Facing::North,
        );
        power(&mut game, at(3, 3));
        let before = game
            .game
            .get_player_component::<Health>()
            .map(|h| h.current);
        wait(&mut game, 30 * TICKS_PER_SECOND);
        let after = game
            .game
            .get_player_component::<Health>()
            .map(|h| h.current);
        assert_eq!(before, after);
        let processor = game.game.res.factory_index[&at(0, 0)];
        assert!(
            !game
                .game
                .world
                .get::<&PowerConsumer>(processor)
                .unwrap()
                .active,
            "nothing to strike, no demand"
        );
    }

    #[test]
    fn a_powered_bio_vat_grows_an_embryo_from_a_batch() {
        let mut game = meadow().build();
        let vat = put(&mut game, FactoryKind::BioVat, at(0, 0), Facing::North);
        power(&mut game, at(2, 0));
        assert!(!factory::accepts(
            &game.game.world,
            vat,
            ItemKind::BiomechEmbryoPerfect
        ));
        give(&mut game, vat, ItemKind::Meat, 4);
        give(&mut game, vat, ItemKind::Mash, 2);
        wait(&mut game, BATCH_TICKS);
        assert_eq!(
            vat_progress_percent(&game.game.world, vat),
            0,
            "no electronics, no batch"
        );
        give(&mut game, vat, ItemKind::ScrapElectronics, 1);
        wait(&mut game, BATCH_TICKS + 2 * FACTORY_STEP_TICKS);
        let inv = game.game.world.get::<&Inventory>(vat).unwrap().clone();
        let embryos: u32 = inv
            .slots
            .iter()
            .filter(|s| is_embryo(s.kind))
            .map(|s| s.qty)
            .sum();
        assert_eq!(embryos, 1);
        assert!(
            inv.slots.iter().all(|s| is_embryo(s.kind)),
            "the batch is used up"
        );
        assert_eq!(
            crate::logistics::peek(&game.game.world, vat, None).map(is_embryo),
            Some(true)
        );
    }

    #[test]
    fn extractors_take_only_embryos_from_a_bio_vat() {
        let mut game = meadow().build();
        let vat = put(&mut game, FactoryKind::BioVat, at(0, 0), Facing::North);
        give(&mut game, vat, ItemKind::Meat, 4);
        assert_eq!(crate::logistics::peek(&game.game.world, vat, None), None);
        give(&mut game, vat, ItemKind::BiomechEmbryoWrinkly, 1);
        assert_eq!(
            crate::logistics::peek(&game.game.world, vat, None),
            Some(ItemKind::BiomechEmbryoWrinkly)
        );
    }

    #[test]
    fn grades_follow_their_odds_and_bonus_meat_helps() {
        let game = meadow().build();
        let tally = |bonus| {
            let mut n = [0u32; 3];
            for batch in 0..4000 {
                let g = roll_grade(&game.game.res, at(3, 7), batch, bonus);
                n[g as usize] += 1;
            }
            n.map(|c| c as f64 / 40.0)
        };
        let plain = tally(false);
        let rich = tally(true);
        for (got, want) in plain.iter().zip([70.0, 25.0, 5.0]) {
            assert!((got - want).abs() < 3.0, "{plain:?}");
        }
        for (got, want) in rich.iter().zip([60.0, 30.0, 10.0]) {
            assert!((got - want).abs() < 3.0, "{rich:?}");
        }
        assert_eq!(
            roll_grade(&game.game.res, at(3, 7), 9, false),
            roll_grade(&game.game.res, at(3, 7), 9, false)
        );
    }

    #[test]
    fn sheep_drop_meat_and_maybe_leather() {
        let mut world = meadow();
        let s = world.sheep(1, 0, 0);
        let mut game = world.build();
        let e = game.entity(s);
        crate::systems::handle_entity_death(&mut game.game.world, &mut game.game.res, e);
        let corpse: Vec<ItemStack> = game
            .game
            .world
            .query::<(&EntityKind, &Inventory)>()
            .iter()
            .find(|(_, (k, _))| **k == EntityKind::Corpse)
            .map(|(_, (_, inv))| inv.slots.clone())
            .unwrap();
        let meat = corpse
            .iter()
            .find(|s| s.kind == ItemKind::Meat)
            .map_or(0, |s| s.qty);
        assert!((1..=2).contains(&meat));
        assert!(corpse
            .iter()
            .all(|s| matches!(s.kind, ItemKind::Meat | ItemKind::Leather)));
    }
}

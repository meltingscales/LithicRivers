//! The power grid: burner generators turn fuel into power, power poles join generators and
//! consumers into networks, and each factory step a network shares its generators' output
//! among its consumers.
//!
//! All energy is whole joules and satisfaction is per-mille, so the simulation stays exact
//! and deterministic.

use std::collections::HashMap;

use hecs::{Entity, World};
use serde::{Deserialize, Serialize};

use crate::components::{Inventory, ItemKind, Position};
use crate::factory::FactoryKind;
pub use crate::factory::STEPS_PER_FACTORY_SECOND;
use crate::resources::Resources;

/// A burner generator's most output, in watts (joules per factory second).
pub const BURNER_MAX_W: u64 = 90_000;
/// Fuel item stacks a burner generator holds.
pub const BURNER_FUEL_SLOTS: usize = 4;
/// Poles link to other poles this many tiles away (Chebyshev distance).
pub const WIRE_REACH: i64 = 8;
/// Poles power generators and consumers this many tiles away (Chebyshev distance).
pub const SUPPLY_RADIUS: i64 = 3;

/// Energy in one fuel item, in joules, or `None` if it isn't fuel.
pub fn fuel_energy(item: ItemKind) -> Option<u64> {
    match item {
        ItemKind::Biochar => Some(4_000_000),
        ItemKind::Log => Some(2_000_000),
        ItemKind::WoodenPlank => Some(500_000),
        ItemKind::Stick => Some(250_000),
        ItemKind::WoodenShavings => Some(100_000),
        _ => None,
    }
}

/// A burner generator. Its fuel items are in its `Inventory`; `buffer_j` is the energy left
/// in the item it is burning.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Generator {
    pub buffer_j: u64,
}

/// A power pole: links to poles within `WIRE_REACH`, powers blocks within `SUPPLY_RADIUS`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PowerPole;

/// A machine that runs on power. `satisfaction_permille` is what it got last step, out of
/// 1000; machines scale their progress by it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PowerConsumer {
    pub demand_w: u64,
    /// Whether it wants power right now (an idle machine doesn't)
    pub active: bool,
    pub satisfaction_permille: u16,
}

impl PowerConsumer {
    pub fn new(demand_w: u64) -> Self {
        Self {
            demand_w,
            active: true,
            satisfaction_permille: 0,
        }
    }

    /// A consumer that wants no power until it has work.
    pub fn idle(demand_w: u64) -> Self {
        Self {
            active: false,
            ..Self::new(demand_w)
        }
    }
}

/// One connected network.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Network {
    /// The lowest pole position: stable across rebuilds and loads
    pub id: Position,
    pub poles: Vec<Entity>,
    pub generators: Vec<Entity>,
    pub consumers: Vec<Entity>,
    /// Last step's figures, in watts
    pub supply_w: u64,
    pub demand_w: u64,
}

/// The networks, derived from the factory blocks and rebuilt when `dirty`. Saved anyway
/// (entity ids survive saving), so last step's figures and the wires look the same after a
/// load.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerNetworks {
    pub dirty: bool,
    pub networks: Vec<Network>,
    /// Pole pairs within wire reach, for drawing wires
    pub links: Vec<(Position, Position)>,
}

impl Default for PowerNetworks {
    fn default() -> Self {
        Self {
            dirty: true,
            networks: Vec::new(),
            links: Vec::new(),
        }
    }
}

impl PowerNetworks {
    /// The network a pole, generator or consumer belongs to.
    pub fn network_of(&self, entity: Entity) -> Option<&Network> {
        self.networks.iter().find(|n| {
            n.poles.contains(&entity)
                || n.generators.contains(&entity)
                || n.consumers.contains(&entity)
        })
    }
}

fn chebyshev(a: Position, b: Position) -> i64 {
    (a.x - b.x).abs().max((a.y - b.y).abs())
}

fn sort_key(p: &Position) -> (i64, i64, i64) {
    (p.z, p.y, p.x)
}

fn find(parent: &mut [usize], i: usize) -> usize {
    let mut root = i;
    while parent[root] != root {
        root = parent[root];
    }
    let mut node = i;
    while parent[node] != root {
        let next = parent[node];
        parent[node] = root;
        node = next;
    }
    root
}

fn union(parent: &mut [usize], a: usize, b: usize) {
    let (ra, rb) = (find(parent, a), find(parent, b));
    // Keep the lower index (lower position) as the root, so ids are stable
    if ra < rb {
        parent[rb] = ra;
    } else if rb < ra {
        parent[ra] = rb;
    }
}

/// Rebuild every network from the poles, generators and consumers in the world.
pub fn rebuild(world: &World) -> PowerNetworks {
    let mut poles: Vec<(Entity, Position)> = world
        .query::<&Position>()
        .with::<&PowerPole>()
        .iter()
        .map(|(e, p)| (e, *p))
        .collect();
    poles.sort_by_key(|(_, p)| sort_key(p));
    let mut parent: Vec<usize> = (0..poles.len()).collect();
    let mut links = Vec::new();
    for i in 0..poles.len() {
        for j in i + 1..poles.len() {
            let (a, b) = (poles[i].1, poles[j].1);
            if a.z == b.z && chebyshev(a, b) <= WIRE_REACH {
                union(&mut parent, i, j);
                links.push((a, b));
            }
        }
    }

    // Each block joins (and merges) the networks of every pole covering it
    let mut members: Vec<(Entity, Position, bool)> = world
        .query::<&Position>()
        .with::<&Generator>()
        .iter()
        .map(|(e, p)| (e, *p, true))
        .chain(
            world
                .query::<&Position>()
                .with::<&PowerConsumer>()
                .iter()
                .map(|(e, p)| (e, *p, false)),
        )
        .collect();
    members.sort_by_key(|(_, p, _)| sort_key(p));
    let mut attached: Vec<(Entity, bool, usize)> = Vec::new();
    for &(entity, pos, is_generator) in &members {
        let covering: Vec<usize> = (0..poles.len())
            .filter(|&i| poles[i].1.z == pos.z && chebyshev(poles[i].1, pos) <= SUPPLY_RADIUS)
            .collect();
        for pair in covering.windows(2) {
            union(&mut parent, pair[0], pair[1]);
        }
        if let Some(&first) = covering.first() {
            attached.push((entity, is_generator, first));
        }
    }

    let mut by_root: HashMap<usize, Network> = HashMap::new();
    for (i, &(entity, _)) in poles.iter().enumerate() {
        let root = find(&mut parent, i);
        let network = by_root.entry(root).or_insert_with(|| Network {
            id: poles[root].1,
            poles: Vec::new(),
            generators: Vec::new(),
            consumers: Vec::new(),
            supply_w: 0,
            demand_w: 0,
        });
        network.poles.push(entity);
    }
    for (entity, is_generator, pole) in attached {
        let root = find(&mut parent, pole);
        let network = by_root.get_mut(&root).expect("every pole has a network");
        if is_generator {
            network.generators.push(entity);
        } else {
            network.consumers.push(entity);
        }
    }
    let mut networks: Vec<Network> = by_root.into_values().collect();
    networks.sort_by_key(|n| sort_key(&n.id));
    PowerNetworks {
        dirty: false,
        networks,
        links,
    }
}

/// Joules a generator can give this step: up to its maximum, from its buffer plus the next
/// fuel item it would have to start.
fn available_j(world: &World, generator: Entity) -> u64 {
    let max_step = BURNER_MAX_W / STEPS_PER_FACTORY_SECOND;
    let buffer = world.get::<&Generator>(generator).map_or(0, |g| g.buffer_j);
    let next = world
        .get::<&Inventory>(generator)
        .ok()
        .and_then(|inv| inv.slots.iter().find_map(|s| fuel_energy(s.kind)))
        .unwrap_or(0);
    max_step.min(buffer + next)
}

/// Take `joules` from a generator, starting its next fuel item if the buffer runs short.
fn draw(world: &mut World, generator: Entity, joules: u64) {
    let buffer = world.get::<&Generator>(generator).map_or(0, |g| g.buffer_j);
    let mut buffer = buffer;
    if buffer < joules {
        if let Ok(mut inv) = world.get::<&mut Inventory>(generator) {
            if let Some(slot) = inv.slots.iter().position(|s| fuel_energy(s.kind).is_some()) {
                buffer += fuel_energy(inv.slots[slot].kind).unwrap_or(0);
                inv.slots[slot].qty -= 1;
                if inv.slots[slot].qty == 0 {
                    inv.slots.remove(slot);
                }
            }
        }
    }
    if let Ok(mut g) = world.get::<&mut Generator>(generator) {
        g.buffer_j = buffer.saturating_sub(joules);
    }
}

/// One factory step of the grid: rebuild networks if needed, then share each network's
/// generator output among its active consumers.
pub fn step_power(world: &mut World, res: &mut Resources) {
    if res.power.dirty {
        res.power = rebuild(world);
    }
    // Consumers outside every network get nothing
    for (_, consumer) in world.query::<&mut PowerConsumer>().iter() {
        consumer.satisfaction_permille = 0;
    }
    let mut networks = std::mem::take(&mut res.power.networks);
    for network in &mut networks {
        let demand_j: u64 = network
            .consumers
            .iter()
            .filter_map(|&c| world.get::<&PowerConsumer>(c).ok().map(|c| *c))
            .filter(|c| c.active)
            .map(|c| c.demand_w / STEPS_PER_FACTORY_SECOND)
            .sum();
        let available: Vec<u64> = network
            .generators
            .iter()
            .map(|&g| available_j(world, g))
            .collect();
        let total_available: u64 = available.iter().sum();
        let supplied = demand_j.min(total_available);
        // An idle network is fully satisfied
        let satisfaction = (supplied * 1000).checked_div(demand_j).unwrap_or(1000) as u16;
        // Generators share in proportion to what they can give; the rounding remainder
        // goes to the first generators, one joule each
        if supplied > 0 {
            let mut shares: Vec<u64> = available
                .iter()
                .map(|&a| supplied * a / total_available)
                .collect();
            let mut remainder = supplied - shares.iter().sum::<u64>();
            for (share, &a) in shares.iter_mut().zip(&available) {
                if remainder == 0 {
                    break;
                }
                if *share < a {
                    *share += 1;
                    remainder -= 1;
                }
            }
            for (&generator, &share) in network.generators.iter().zip(&shares) {
                if share > 0 {
                    draw(world, generator, share);
                }
            }
        }
        for &c in &network.consumers {
            if let Ok(mut consumer) = world.get::<&mut PowerConsumer>(c) {
                consumer.satisfaction_permille = if consumer.active { satisfaction } else { 0 };
            }
        }
        network.supply_w = supplied * STEPS_PER_FACTORY_SECOND;
        network.demand_w = demand_j * STEPS_PER_FACTORY_SECOND;
    }
    res.power.networks = networks;
}

/// Whether a factory block kind belongs to the grid (placing or breaking it changes networks).
pub fn is_grid_block(kind: FactoryKind) -> bool {
    matches!(
        kind,
        FactoryKind::BurnerGenerator
            | FactoryKind::PowerPole
            | FactoryKind::AutoPlanter
            | FactoryKind::AutoHarvester
            | FactoryKind::ElectricFurnace
    ) || kind.is_test_consumer()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::{self, accepts, insert, Facing, TEST_CONSUMER_W};
    use crate::test_support::TestWorld;
    use crate::tiles::TileKind;
    use crate::Game;

    fn at(x: i64, y: i64) -> Position {
        Position { x, y, z: 0 }
    }

    fn grid_game() -> Game {
        TestWorld::new(4).player_at(0, 20, 0).build().game
    }

    fn put(game: &mut Game, kind: FactoryKind, pos: Position) -> Entity {
        game.res
            .world_state
            .world
            .set_tile_cached(pos.x, pos.y, pos.z, TileKind::Grass);
        factory::place(&mut game.world, &mut game.res, kind, pos, Facing::North).expect("placed")
    }

    fn fuel(game: &mut Game, generator: Entity, item: ItemKind, qty: u32) {
        game.world
            .get::<&mut Inventory>(generator)
            .unwrap()
            .add(item, qty);
    }

    /// Fuel energy left in a generator: items plus its buffer.
    fn energy(game: &Game, generator: Entity) -> u64 {
        let items: u64 = game
            .world
            .get::<&Inventory>(generator)
            .unwrap()
            .slots
            .iter()
            .map(|s| fuel_energy(s.kind).unwrap() * s.qty as u64)
            .sum();
        items + game.world.get::<&Generator>(generator).unwrap().buffer_j
    }

    /// Pass `seconds` factory seconds, one at a time.
    fn pass_seconds(game: &mut Game, seconds: u64) {
        for _ in 0..seconds {
            game.res.player_state.intent = crate::intent::PlayerIntent::movement(
                0,
                0,
                0,
                crate::factory::FACTORY_SECOND_TICKS,
            );
            game.tick();
        }
    }

    fn satisfaction(game: &Game, consumer: Entity) -> u16 {
        game.world
            .get::<&PowerConsumer>(consumer)
            .unwrap()
            .satisfaction_permille
    }

    // --- 1.1 fuel ----------------------------------------------------------------------

    #[test]
    fn generators_take_only_fuel() {
        let mut game = grid_game();
        let generator = put(&mut game, FactoryKind::BurnerGenerator, at(0, 0));
        assert!(accepts(&game.world, generator, ItemKind::Log));
        assert!(accepts(&game.world, generator, ItemKind::WoodenShavings));
        assert!(!accepts(&game.world, generator, ItemKind::Stone));
        assert!(!insert(&mut game.world, generator, ItemKind::Stone));
        assert_eq!(fuel_energy(ItemKind::Log), Some(2_000_000));
        assert_eq!(fuel_energy(ItemKind::WoodenPlank), Some(500_000));
        assert_eq!(fuel_energy(ItemKind::Stick), Some(250_000));
        assert_eq!(fuel_energy(ItemKind::WoodenShavings), Some(100_000));
        assert_eq!(fuel_energy(ItemKind::Stone), None);
    }

    // --- 1.2 networks ------------------------------------------------------------------

    fn same_network(game: &mut Game, a: Entity, b: Entity) -> bool {
        game.res.power = rebuild(&game.world);
        let na = game.res.power.network_of(a).map(|n| n.id);
        na.is_some() && na == game.res.power.network_of(b).map(|n| n.id)
    }

    #[test]
    fn poles_within_reach_link_and_farther_ones_do_not() {
        let mut game = grid_game();
        let a = put(&mut game, FactoryKind::PowerPole, at(0, 0));
        let b = put(&mut game, FactoryKind::PowerPole, at(7, 0));
        let c = put(&mut game, FactoryKind::PowerPole, at(16, 0));
        let generator = put(&mut game, FactoryKind::BurnerGenerator, at(1, 1));
        let consumer = put(&mut game, FactoryKind::TestConsumer, at(8, 1));
        let far = put(&mut game, FactoryKind::TestConsumer, at(17, 1));
        assert!(same_network(&mut game, a, b), "7 apart");
        assert!(same_network(&mut game, generator, consumer));
        assert!(!same_network(&mut game, b, c), "9 apart");
        assert!(!same_network(&mut game, generator, far));
        // A block outside every supply area belongs to no network
        let alone = put(&mut game, FactoryKind::TestConsumer, at(40, 0));
        game.res.power = rebuild(&game.world);
        assert!(game.res.power.network_of(alone).is_none());
    }

    #[test]
    fn a_middle_pole_merges_networks_and_removing_it_splits_them() {
        let mut game = grid_game();
        let a = put(&mut game, FactoryKind::PowerPole, at(0, 0));
        let c = put(&mut game, FactoryKind::PowerPole, at(12, 0));
        assert!(!same_network(&mut game, a, c));
        put(&mut game, FactoryKind::PowerPole, at(6, 0));
        assert!(same_network(&mut game, a, c));
        let p = game.get_player_entity().unwrap();
        factory::break_block(&mut game.world, &mut game.res, at(6, 0), p);
        assert!(
            game.res.power.dirty,
            "breaking a pole marks networks for rebuild"
        );
        assert!(!same_network(&mut game, a, c));
    }

    #[test]
    fn poles_link_across_a_chunk_edge() {
        let mut game = grid_game();
        let a = put(&mut game, FactoryKind::PowerPole, at(60, 0));
        let b = put(&mut game, FactoryKind::PowerPole, at(66, 0));
        assert!(same_network(&mut game, a, b));
        let c = put(&mut game, FactoryKind::PowerPole, at(-3, 0));
        let d = put(&mut game, FactoryKind::PowerPole, at(3, 0));
        assert!(same_network(&mut game, c, d), "across 0");
        assert_eq!(
            game.res.power.network_of(c).unwrap().id,
            at(-3, 0),
            "lowest pole is the id"
        );
    }

    // --- 1.3 distribution --------------------------------------------------------------

    /// A pole with a generator and `consumers` test consumers next to it.
    fn small_grid(game: &mut Game, consumers: i64) -> (Entity, Vec<Entity>) {
        put(game, FactoryKind::PowerPole, at(0, 0));
        let generator = put(game, FactoryKind::BurnerGenerator, at(-1, 0));
        let consumers = (0..consumers)
            .map(|i| put(game, FactoryKind::TestConsumer, at(1, i - 1)))
            .collect();
        (generator, consumers)
    }

    #[test]
    fn an_idle_grid_burns_nothing() {
        let mut game = grid_game();
        let (generator, consumers) = small_grid(&mut game, 1);
        game.world
            .get::<&mut PowerConsumer>(consumers[0])
            .unwrap()
            .active = false;
        fuel(&mut game, generator, ItemKind::Log, 5);
        pass_seconds(&mut game, 600);
        assert_eq!(energy(&game, generator), 5 * 2_000_000);
        let logs = game.world.get::<&Inventory>(generator).unwrap().slots[0].qty;
        assert_eq!(logs, 5);
    }

    #[test]
    fn partial_load_burns_proportionally_and_keeps_the_remainder() {
        let mut game = grid_game();
        let (generator, consumers) = small_grid(&mut game, 1);
        assert_eq!(
            TEST_CONSUMER_W * 2,
            BURNER_MAX_W,
            "half the generator's output"
        );
        fuel(&mut game, generator, ItemKind::Log, 5);
        pass_seconds(&mut game, 60);
        assert_eq!(satisfaction(&game, consumers[0]), 1000);
        assert_eq!(
            5 * 2_000_000 - energy(&game, generator),
            TEST_CONSUMER_W * 60
        );
        let buffer = game.world.get::<&Generator>(generator).unwrap().buffer_j;
        assert_eq!(
            buffer,
            2 * 2_000_000 - TEST_CONSUMER_W * 60,
            "the second log is partly used"
        );
    }

    #[test]
    fn a_shortage_gives_every_consumer_the_same_fraction() {
        let mut game = grid_game();
        let (generator, consumers) = small_grid(&mut game, 4); // 180 kW against 90 kW
        fuel(&mut game, generator, ItemKind::Log, 5);
        pass_seconds(&mut game, 2);
        for &c in &consumers {
            assert_eq!(satisfaction(&game, c), 500);
        }
        let network = game.res.power.network_of(generator).unwrap();
        assert_eq!(
            (network.supply_w, network.demand_w),
            (BURNER_MAX_W, 4 * TEST_CONSUMER_W)
        );
    }

    #[test]
    fn generators_share_the_load() {
        let mut game = grid_game();
        let (first, _) = small_grid(&mut game, 1);
        let second = put(&mut game, FactoryKind::BurnerGenerator, at(-1, 1));
        fuel(&mut game, first, ItemKind::Log, 2);
        fuel(&mut game, second, ItemKind::Log, 2);
        pass_seconds(&mut game, 10);
        let used = |g| 2 * 2_000_000 - energy(&game, g);
        assert_eq!(used(first), TEST_CONSUMER_W * 10 / 2);
        assert_eq!(used(second), TEST_CONSUMER_W * 10 / 2);
    }

    #[test]
    fn out_of_fuel_stops_consumers_until_refuelled() {
        let mut game = grid_game();
        let (generator, consumers) = small_grid(&mut game, 1);
        fuel(&mut game, generator, ItemKind::Stick, 1); // 250 kJ: about 5.5 s at 45 kW
        pass_seconds(&mut game, 10);
        assert_eq!(satisfaction(&game, consumers[0]), 0, "out of fuel");
        fuel(&mut game, generator, ItemKind::Log, 1);
        pass_seconds(&mut game, 1);
        assert_eq!(satisfaction(&game, consumers[0]), 1000, "resumed");
    }

    #[test]
    fn networks_survive_a_reload() {
        let mut game = grid_game();
        let (generator, consumers) = small_grid(&mut game, 1);
        fuel(&mut game, generator, ItemKind::Log, 1);
        pass_seconds(&mut game, 1);
        let saved = crate::save_load::save_to_string(&game, &serde_json::Value::Null).unwrap();
        let before = game.res.power.networks.clone();
        crate::save_load::load_from_str(&mut game, &saved).unwrap();
        assert_eq!(
            game.res.power.networks, before,
            "networks and their figures are saved"
        );
        pass_seconds(&mut game, 1);
        assert_eq!(satisfaction(&game, consumers[0]), 1000);
    }
}

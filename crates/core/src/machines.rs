//! Tree-farm machines: the auto-planter plants acorns around itself and the auto-harvester
//! chops grown trees around itself and pushes the wood into the block it faces.
//!
//! A machine's items are in its `Inventory` (the planter's acorns, the harvester's output),
//! capped at `MACHINE_CAPACITY` items. Each factory step it demands power only if it has
//! work, and adds its power satisfaction (per-mille) to its progress; it acts when the
//! progress reaches its action cost.

use std::collections::HashSet;

use hecs::{Entity, World};
use serde::{Deserialize, Serialize};

use crate::components::{Inventory, ItemKind, Position};
use crate::factory::{FactoryBlock, FactoryKind};
use crate::power::{PowerConsumer, STEPS_PER_SECOND};
use crate::resources::Resources;
use crate::tiles::TileKind;

/// Machines work tiles within this many tiles of themselves (a 5x5 square).
pub const MACHINE_RADIUS: i64 = 2;
/// Items a machine holds: the planter's acorns, the harvester's output.
pub const MACHINE_CAPACITY: u32 = 16;
/// Item stacks a machine's inventory has room for.
pub const MACHINE_SLOTS: usize = 4;
/// What each machine draws while it works, in watts.
/// Planting costs 0.24 MJ and a chop 1.08 MJ, against about 5 MJ of logs from a tree, so a
/// farm returns about 4x its fuel. Together they need 240 kW: more than one burner
/// generator (90 kW) gives, so power planning matters.
pub const PLANTER_W: u64 = 60_000;
pub const HARVESTER_W: u64 = 180_000;
/// Game seconds per action at full power.
pub const PLANT_SECONDS: u64 = 4;
pub const CHOP_SECONDS: u64 = 6;

/// A machine's progress towards its next action, in per-mille steps: full power adds 1000
/// each factory step.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Machine {
    pub progress: u64,
}

/// What a machine kind draws while it works.
pub fn demand_w(kind: FactoryKind) -> u64 {
    match kind {
        FactoryKind::AutoHarvester => HARVESTER_W,
        _ => PLANTER_W,
    }
}

/// Progress a machine kind needs for one action.
pub fn action_cost(kind: FactoryKind) -> u64 {
    let seconds = match kind {
        FactoryKind::AutoHarvester => CHOP_SECONDS,
        _ => PLANT_SECONDS,
    };
    seconds * STEPS_PER_SECOND * 1000
}

fn held(world: &World, machine: Entity) -> u32 {
    world
        .get::<&Inventory>(machine)
        .map_or(0, |inv| inv.slots.iter().map(|s| s.qty).sum())
}

/// Whether a machine takes `item` into its input: planters take acorns up to their capacity;
/// harvesters take nothing.
pub fn accepts_input(world: &World, machine: Entity, kind: FactoryKind, item: ItemKind) -> bool {
    kind == FactoryKind::AutoPlanter
        && item == ItemKind::Acorn
        && held(world, machine) < MACHINE_CAPACITY
}

/// The most items one chopped tree can drop.
fn max_tree_drop() -> u32 {
    crate::crafting::mining_drop(TileKind::Tree)
        .map_or(0, |d| d.items.iter().map(|&(_, _, max)| max).sum())
}

/// The tiles a machine at `center` works, in its fixed order: row by row from the
/// north-west corner, skipping its own tile.
pub fn work_area(center: Position) -> impl Iterator<Item = Position> {
    (-MACHINE_RADIUS..=MACHINE_RADIUS)
        .flat_map(|dy| (-MACHINE_RADIUS..=MACHINE_RADIUS).map(move |dx| (dx, dy)))
        .filter(|&d| d != (0, 0))
        .map(move |(dx, dy)| Position {
            x: center.x + dx,
            y: center.y + dy,
            z: center.z,
        })
}

/// The tile a machine would act on next, if it has work: for a planter with acorns, the
/// first free grass or dirt tile; for a harvester with room for a tree's drops, the first
/// grown tree.
fn work_target(
    world: &World,
    res: &mut Resources,
    occupied: &HashSet<Position>,
    machine: Entity,
    kind: FactoryKind,
    pos: Position,
) -> Option<Position> {
    let tiles = &mut res.world_state.world;
    match kind {
        FactoryKind::AutoPlanter => {
            if held(world, machine) == 0 {
                return None;
            }
            work_area(pos).find(|p| {
                matches!(
                    tiles.get_tile_cached(p.x, p.y, p.z),
                    TileKind::Grass | TileKind::Dirt
                ) && !occupied.contains(p)
            })
        }
        FactoryKind::AutoHarvester => {
            if held(world, machine) + max_tree_drop() > MACHINE_CAPACITY {
                return None;
            }
            work_area(pos).find(|p| tiles.get_tile_cached(p.x, p.y, p.z) == TileKind::Tree)
        }
        _ => None,
    }
}

/// Every machine with its kind and position, in position order.
fn machines(world: &World) -> Vec<(Entity, FactoryKind, Position, crate::factory::Facing)> {
    let mut found: Vec<_> = world
        .query::<(&Position, &FactoryBlock)>()
        .with::<&Machine>()
        .iter()
        .map(|(e, (p, b))| (e, b.kind, *p, b.facing))
        .collect();
    found.sort_by_key(|(_, _, p, _)| (p.z, p.y, p.x));
    found
}

fn occupied_positions(world: &World) -> HashSet<Position> {
    world.query::<&Position>().iter().map(|(_, p)| *p).collect()
}

/// Before power is shared: each machine wants power only if it has work.
pub fn update_demand(world: &mut World, res: &mut Resources) {
    let machines = machines(world);
    if machines.is_empty() {
        return;
    }
    let occupied = occupied_positions(world);
    for (machine, kind, pos, _) in machines {
        let working = work_target(world, res, &occupied, machine, kind, pos).is_some();
        if let Ok(mut consumer) = world.get::<&mut PowerConsumer>(machine) {
            consumer.active = working;
        }
    }
}

/// After power is shared: machines with work progress by their satisfaction and act when
/// done; harvesters then push one item into the block they face.
pub fn step_machines(world: &mut World, res: &mut Resources) {
    let machines = machines(world);
    if machines.is_empty() {
        return;
    }
    let mut occupied = occupied_positions(world);
    for (machine, kind, pos, facing) in machines {
        // Re-checked here: the player may have changed the tile since demand was set
        if let Some(target) = work_target(world, res, &occupied, machine, kind, pos) {
            let satisfaction = world
                .get::<&PowerConsumer>(machine)
                .map_or(0, |c| c.satisfaction_permille as u64);
            let done = {
                let mut m = world.get::<&mut Machine>(machine).expect("machine");
                m.progress += satisfaction;
                let done = m.progress >= action_cost(kind);
                if done {
                    m.progress = 0;
                }
                done
            };
            if done {
                act(world, res, machine, kind, target);
                occupied.insert(target);
            }
        }
        if kind == FactoryKind::AutoHarvester {
            push_output(world, res, machine, pos, facing);
        }
    }
}

fn act(world: &mut World, res: &mut Resources, machine: Entity, kind: FactoryKind, at: Position) {
    match kind {
        FactoryKind::AutoPlanter => {
            if crate::plants::plant(world, res, at).is_ok() {
                take_one(world, machine, ItemKind::Acorn);
            }
        }
        FactoryKind::AutoHarvester => {
            let drops = crate::crafting::mine_tile(res, at).unwrap_or_default();
            if let Ok(mut inv) = world.get::<&mut Inventory>(machine) {
                for (item, qty) in drops {
                    inv.add(item, qty);
                }
            }
        }
        _ => {}
    }
}

fn take_one(world: &mut World, machine: Entity, item: ItemKind) {
    if let Ok(mut inv) = world.get::<&mut Inventory>(machine) {
        if let Some(i) = inv.slots.iter().position(|s| s.kind == item && s.qty > 0) {
            inv.slots[i].qty -= 1;
            if inv.slots[i].qty == 0 {
                inv.slots.remove(i);
            }
        }
    }
}

/// Push the first output item into the factory block in front, if it accepts it.
fn push_output(
    world: &mut World,
    res: &Resources,
    machine: Entity,
    pos: Position,
    facing: crate::factory::Facing,
) {
    let Some(item) = world
        .get::<&Inventory>(machine)
        .ok()
        .and_then(|inv| inv.slots.first().map(|s| s.kind))
    else {
        return;
    };
    let (dx, dy) = facing.offset();
    let front = Position {
        x: pos.x + dx,
        y: pos.y + dy,
        z: pos.z,
    };
    let Some(&target) = res.factory_index.get(&front) else {
        return;
    };
    if crate::factory::insert_from(world, target, item, Some(facing)) {
        take_one(world, machine, item);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::{self, BeltSlot, Facing};
    use crate::intent::{PlayerAction, PlayerIntent};
    use crate::plants::Growth;
    use crate::test_support::{TestGame, TestWorld};
    use crate::world::CHUNK_SIZE;

    fn at(x: i64, y: i64) -> Position {
        Position { x, y, z: 0 }
    }

    /// Grass from (-6, -6) to (6, 6), the player out of the way at (0, 20).
    fn field() -> TestGame {
        TestWorld::new(11)
            .player_at(0, 20, 0)
            .fill((-6, -6, 0), (6, 6, 0), TileKind::Grass)
            .build()
    }

    fn put(game: &mut TestGame, kind: FactoryKind, pos: Position, facing: Facing) -> Entity {
        let g = &mut game.game;
        g.res
            .world_state
            .world
            .set_tile_cached(pos.x, pos.y, pos.z, TileKind::Grass);
        factory::place(&mut g.world, &mut g.res, kind, pos, facing).expect("placed")
    }

    /// A pole at `pole` and two generators beside it (180 kW, enough to run a harvester at
    /// full speed) with ten logs each; returns the first generator.
    fn power(game: &mut TestGame, pole: Position) -> Entity {
        put(game, FactoryKind::PowerPole, pole, Facing::North);
        let generators = [at(pole.x, pole.y + 1), at(pole.x, pole.y - 1)]
            .map(|pos| put(game, FactoryKind::BurnerGenerator, pos, Facing::North));
        for g in generators {
            give(game, g, ItemKind::Log, 10);
        }
        generators[0]
    }

    fn give(game: &mut TestGame, entity: Entity, item: ItemKind, qty: u32) {
        game.game
            .world
            .get::<&mut Inventory>(entity)
            .unwrap()
            .add(item, qty);
    }

    fn count(game: &TestGame, entity: Entity, item: ItemKind) -> u32 {
        game.game
            .world
            .get::<&Inventory>(entity)
            .unwrap()
            .slots
            .iter()
            .filter(|s| s.kind == item)
            .map(|s| s.qty)
            .sum()
    }

    fn seconds(game: &mut TestGame, n: u64) {
        game.game.res.player_state.intent =
            PlayerIntent::movement(0, 0, 0, n * crate::factory::TICKS_PER_SECOND);
        game.tick();
    }

    fn saplings(game: &TestGame) -> Vec<Position> {
        let mut found: Vec<Position> = game
            .game
            .world
            .query::<&Position>()
            .with::<&Growth>()
            .iter()
            .map(|(_, p)| *p)
            .collect();
        found.sort_by_key(|p| (p.y, p.x));
        found
    }

    fn active(game: &TestGame, machine: Entity) -> bool {
        game.game
            .world
            .get::<&PowerConsumer>(machine)
            .unwrap()
            .active
    }

    // --- 1.2 auto-planter --------------------------------------------------------------

    #[test]
    fn planter_plants_in_fixed_order_once_per_action() {
        let mut game = field();
        let planter = put(&mut game, FactoryKind::AutoPlanter, at(0, 0), Facing::North);
        power(&mut game, at(3, 0));
        give(&mut game, planter, ItemKind::Acorn, 3);
        seconds(&mut game, 3 * PLANT_SECONDS - 1);
        assert_eq!(saplings(&game).len(), 2, "two before the third action ends");
        seconds(&mut game, 1);
        assert_eq!(saplings(&game), [at(-2, -2), at(-1, -2), at(0, -2)]);
        assert_eq!(count(&game, planter, ItemKind::Acorn), 0);
        seconds(&mut game, 1);
        assert!(!active(&game, planter), "no acorns, no demand");
    }

    #[test]
    fn a_belt_feeds_acorns_into_a_planter() {
        let mut game = field();
        let planter = put(&mut game, FactoryKind::AutoPlanter, at(0, 0), Facing::North);
        let belt = put(&mut game, FactoryKind::Belt, at(-1, 0), Facing::East);
        *game.game.world.get::<&mut BeltSlot>(belt).unwrap() = BeltSlot {
            item: Some(ItemKind::Acorn),
            progress: 0,
        };
        seconds(&mut game, 1);
        assert_eq!(count(&game, planter, ItemKind::Acorn), 1);
        assert!(
            !factory::accepts(&game.game.world, planter, ItemKind::Log),
            "only acorns"
        );
        give(&mut game, planter, ItemKind::Acorn, MACHINE_CAPACITY - 1);
        assert!(
            !factory::accepts(&game.game.world, planter, ItemKind::Acorn),
            "full at {MACHINE_CAPACITY}"
        );
        assert!(saplings(&game).is_empty(), "unpowered machines don't work");
    }

    #[test]
    fn planter_without_acorns_or_free_tiles_does_nothing() {
        let mut game = field();
        let planter = put(&mut game, FactoryKind::AutoPlanter, at(0, 0), Facing::North);
        let generator = power(&mut game, at(3, 0));
        seconds(&mut game, 10);
        assert!(!active(&game, planter));
        // Acorns but every tile in reach is rock
        for p in work_area(at(0, 0)) {
            game.game
                .res
                .world_state
                .world
                .set_tile_cached(p.x, p.y, 0, TileKind::Rock);
        }
        give(&mut game, planter, ItemKind::Acorn, 3);
        seconds(&mut game, 10);
        assert!(!active(&game, planter));
        assert!(saplings(&game).is_empty());
        assert_eq!(count(&game, planter, ItemKind::Acorn), 3);
        assert_eq!(count(&game, generator, ItemKind::Log), 10, "no fuel burned");
    }

    // --- 1.3 auto-harvester ------------------------------------------------------------

    /// What the player gets for chopping a tree at `pos` in the same world.
    fn manual_drops(pos: Position) -> Vec<(ItemKind, u32)> {
        let mut game = TestWorld::new(11)
            .player_at(pos.x + 1, pos.y, 0)
            .fill((-6, -6, 0), (6, 6, 0), TileKind::Grass)
            .tile(pos.x, pos.y, 0, TileKind::Tree)
            .inventory(&[])
            .auto_pickup(true)
            .build();
        game.act(PlayerAction::MineAt {
            x: pos.x,
            y: pos.y,
            z: 0,
        });
        game.tick();
        let mut drops = game.items_at(pos.x, pos.y, 0);
        drops.sort();
        drops
    }

    #[test]
    fn harvester_chops_onto_a_belt_into_a_chest_with_manual_drops() {
        let mut game = field();
        game.game
            .res
            .world_state
            .world
            .set_tile_cached(-2, -2, 0, TileKind::Tree);
        let harvester = put(
            &mut game,
            FactoryKind::AutoHarvester,
            at(0, 0),
            Facing::East,
        );
        put(&mut game, FactoryKind::Belt, at(1, 0), Facing::East);
        let chest = put(&mut game, FactoryKind::Chest, at(2, 0), Facing::North);
        power(&mut game, at(0, 3));
        seconds(&mut game, CHOP_SECONDS);
        assert_eq!(
            game.tile(-2, -2, 0),
            TileKind::Dirt,
            "chopped in one action"
        );
        seconds(&mut game, 10);
        let mut delivered: Vec<(ItemKind, u32)> = [ItemKind::Acorn, ItemKind::Log]
            .into_iter()
            .map(|k| (k, count(&game, chest, k)))
            .collect();
        delivered.sort();
        assert_eq!(delivered, manual_drops(at(-2, -2)));
        assert_eq!(
            game.game
                .world
                .get::<&Inventory>(harvester)
                .unwrap()
                .slots
                .len(),
            0
        );
        assert!(!active(&game, harvester), "no trees left");
    }

    #[test]
    fn a_full_harvester_stops_chopping() {
        let mut game = field();
        game.game
            .res
            .world_state
            .world
            .set_tile_cached(-2, -2, 0, TileKind::Tree);
        let harvester = put(
            &mut game,
            FactoryKind::AutoHarvester,
            at(0, 0),
            Facing::East,
        );
        power(&mut game, at(0, 3));
        give(&mut game, harvester, ItemKind::Log, MACHINE_CAPACITY - 5);
        assert!(
            !factory::accepts(&game.game.world, harvester, ItemKind::Log),
            "harvesters take nothing in"
        );
        seconds(&mut game, 10);
        assert_eq!(
            game.tile(-2, -2, 0),
            TileKind::Tree,
            "no room for its drops"
        );
        assert!(!active(&game, harvester));
        // A chest in front makes room
        put(&mut game, FactoryKind::Chest, at(1, 0), Facing::North);
        seconds(&mut game, 10);
        assert_eq!(game.tile(-2, -2, 0), TileKind::Dirt);
    }

    #[test]
    fn harvester_and_player_chop_the_same_tree_once() {
        let mut game = field();
        game.game
            .res
            .world_state
            .world
            .set_tile_cached(-2, -2, 0, TileKind::Tree);
        let harvester = put(
            &mut game,
            FactoryKind::AutoHarvester,
            at(0, 0),
            Facing::East,
        );
        power(&mut game, at(0, 3));
        seconds(&mut game, 2);
        game.act(PlayerAction::MineAt { x: -2, y: -2, z: 0 });
        game.tick();
        seconds(&mut game, 5);
        assert_eq!(
            game.game
                .world
                .get::<&Inventory>(harvester)
                .unwrap()
                .slots
                .len(),
            0,
            "the player got the tree"
        );
    }

    // --- 1.4 power ---------------------------------------------------------------------

    #[test]
    fn idle_machines_demand_nothing() {
        let mut game = field();
        put(
            &mut game,
            FactoryKind::AutoHarvester,
            at(0, 0),
            Facing::East,
        );
        put(
            &mut game,
            FactoryKind::AutoPlanter,
            at(-3, -3),
            Facing::North,
        );
        let generator = power(&mut game, at(0, 3));
        seconds(&mut game, 60);
        assert_eq!(count(&game, generator, ItemKind::Log), 10);
        let network = game.game.res.power.network_of(generator).unwrap();
        assert_eq!(network.demand_w, 0);
    }

    #[test]
    fn half_power_doubles_action_time_and_no_power_stops_work() {
        // A harvester (180 kW) on one 90 kW generator gets half
        let mut game = field();
        put(
            &mut game,
            FactoryKind::AutoHarvester,
            at(3, 3),
            Facing::North,
        );
        game.game
            .res
            .world_state
            .world
            .set_tile_cached(1, 1, 0, TileKind::Tree);
        put(&mut game, FactoryKind::PowerPole, at(0, 0), Facing::North);
        let generator = put(
            &mut game,
            FactoryKind::BurnerGenerator,
            at(-3, 3),
            Facing::North,
        );
        give(&mut game, generator, ItemKind::Log, 10);
        assert_eq!(HARVESTER_W, 2 * crate::power::BURNER_MAX_W);
        seconds(&mut game, 2 * CHOP_SECONDS - 1);
        assert_eq!(game.tile(1, 1, 0), TileKind::Tree, "not yet at half speed");
        seconds(&mut game, 1);
        assert_eq!(
            game.tile(1, 1, 0),
            TileKind::Dirt,
            "twice the full-power time"
        );

        // No pole: nothing happens
        let mut game = field();
        game.game
            .res
            .world_state
            .world
            .set_tile_cached(-2, -2, 0, TileKind::Tree);
        put(
            &mut game,
            FactoryKind::AutoHarvester,
            at(0, 0),
            Facing::North,
        );
        seconds(&mut game, 30);
        assert_eq!(game.tile(-2, -2, 0), TileKind::Tree);
    }

    #[test]
    fn harvester_area_spans_four_chunks() {
        let edge = CHUNK_SIZE - 1; // 63
        let mut game = TestWorld::new(11).player_at(0, 20, 0).build();
        let trees = [
            at(edge + 1, edge + 1),
            at(edge + 2, edge + 1),
            at(edge + 1, edge + 2),
            at(edge + 2, edge + 2),
            at(edge + 1, edge - 1),
            at(edge - 1, edge + 1),
        ];
        for t in trees {
            game.game
                .res
                .world_state
                .world
                .set_tile_cached(t.x, t.y, 0, TileKind::Tree);
        }
        put(
            &mut game,
            FactoryKind::AutoHarvester,
            at(edge, edge),
            Facing::North,
        );
        let chest = put(
            &mut game,
            FactoryKind::Chest,
            at(edge, edge - 1),
            Facing::North,
        );
        power(&mut game, at(edge - 3, edge));
        seconds(&mut game, CHOP_SECONDS * trees.len() as u64);
        for t in trees {
            assert_eq!(game.tile(t.x, t.y, 0), TileKind::Dirt, "{t:?}");
        }
        seconds(&mut game, 5);
        let logs = count(&game, chest, ItemKind::Log);
        assert!((2 * 6..=3 * 6).contains(&logs), "{logs} logs");
    }
}

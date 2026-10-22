//! Belt logistics: extractors move items out of one block into another, belt routers split
//! and sort belt lines, and belts carry a player standing on them.

use hecs::{Entity, World};
use serde::{Deserialize, Serialize};

use crate::components::{BlocksMovement, Inventory, ItemKind, Player, Position};
use crate::factory::{
    insert_from, BeltSlot, Facing, FactoryBlock, FactoryKind, BELT_STEPS_PER_TILE,
};
use crate::furnace::{Furnace, Slot};
use crate::resources::Resources;
use crate::tiles::TileKind;

/// Factory steps between an extractor's moves (one game second).
pub const EXTRACTOR_STEPS: u8 = 5;

/// An extractor or filter extractor. A plain extractor never has a filter; a filter
/// extractor moves nothing until one is set.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Extractor {
    pub filter: Option<ItemKind>,
    pub progress: u8,
}

/// A belt router: holds the one item that arrived from behind until an output takes it.
/// Without a filter, items go to the left, straight and right outputs in turn; with one,
/// items of that kind go left and everything else straight on.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Router {
    pub item: Option<ItemKind>,
    /// The output to try first next time: 0 left, 1 straight, 2 right
    pub next: u8,
    pub filter: Option<ItemKind>,
}

fn step_from(pos: Position, facing: Facing) -> Position {
    let (dx, dy) = facing.offset();
    Position {
        x: pos.x + dx,
        y: pos.y + dy,
        z: pos.z,
    }
}

fn left_of(facing: Facing) -> Facing {
    facing.rotate_cw().rotate_cw().rotate_cw()
}

/// The item an extractor would take from `source` (a filter limits it to one kind): a
/// furnace's output, or the first matching stack of a block's inventory. Belts give nothing.
pub fn peek(world: &World, source: Entity, filter: Option<ItemKind>) -> Option<ItemKind> {
    let matches = |item: ItemKind| filter.is_none_or(|f| f == item);
    if world.get::<&BeltSlot>(source).is_ok() {
        return None;
    }
    if let Ok(furnace) = world.get::<&Furnace>(source) {
        return furnace.output.map(|s| s.kind).filter(|&k| matches(k));
    }
    let inv = world.get::<&Inventory>(source).ok()?;
    inv.slots
        .iter()
        .find(|s| s.qty > 0 && matches(s.kind))
        .map(|s| s.kind)
}

/// Take one `item` out of `source`, as `peek` found it.
fn remove_one(world: &mut World, source: Entity, item: ItemKind) {
    if let Ok(mut furnace) = world.get::<&mut Furnace>(source) {
        furnace.take(Slot::Output);
        return;
    }
    if let Ok(mut inv) = world.get::<&mut Inventory>(source) {
        if let Some(i) = inv.slots.iter().position(|s| s.kind == item && s.qty > 0) {
            inv.slots[i].qty -= 1;
            if inv.slots[i].qty == 0 {
                inv.slots.remove(i);
            }
        }
    }
}

/// Whether the router `target` (facing `facing`) takes an item arriving along `heading`:
/// only from behind, and only while it is empty.
pub fn router_accepts(
    world: &World,
    target: Entity,
    facing: Facing,
    heading: Option<Facing>,
) -> bool {
    heading == Some(facing) && world.get::<&Router>(target).is_ok_and(|r| r.item.is_none())
}

fn blocks_in_order<T: hecs::Component>(world: &World) -> Vec<(Entity, Position, Facing)> {
    let mut found: Vec<(Entity, Position, Facing)> = world
        .query::<(&Position, &FactoryBlock)>()
        .with::<&T>()
        .iter()
        .map(|(e, (p, b))| (e, *p, b.facing))
        .collect();
    found.sort_by_key(|(_, p, _)| (p.z, p.y, p.x));
    found
}

/// One factory step for every extractor: every `EXTRACTOR_STEPS` steps it moves one item
/// from the block behind it into the block in front, only if the front block takes it.
pub fn step_extractors(world: &mut World, res: &Resources) {
    for (entity, pos, facing) in blocks_in_order::<Extractor>(world) {
        let (kind, filter) = {
            let block = *world.get::<&FactoryBlock>(entity).expect("block");
            let mut ex = world.get::<&mut Extractor>(entity).expect("extractor");
            ex.progress = (ex.progress + 1).min(EXTRACTOR_STEPS);
            if ex.progress < EXTRACTOR_STEPS {
                continue;
            }
            (block.kind, ex.filter)
        };
        if kind == FactoryKind::FilterExtractor && filter.is_none() {
            continue;
        }
        let behind = step_from(pos, facing.opposite());
        let front = step_from(pos, facing);
        let (Some(&source), Some(&target)) = (
            res.factory_index.get(&behind),
            res.factory_index.get(&front),
        ) else {
            continue;
        };
        let Some(item) = peek(world, source, filter) else {
            continue;
        };
        // Only commit the take once the front block has taken the item
        if insert_from(world, target, item, Some(facing)) {
            remove_one(world, source, item);
            world
                .get::<&mut Extractor>(entity)
                .expect("extractor")
                .progress = 0;
        }
    }
}

/// One factory step for every router: hand its item to the next output that takes it.
pub fn step_routers(world: &mut World, res: &Resources) {
    for (entity, pos, facing) in blocks_in_order::<Router>(world) {
        let router = *world.get::<&Router>(entity).expect("router");
        let Some(item) = router.item else {
            continue;
        };
        let outputs = [left_of(facing), facing, facing.rotate_cw()];
        let order: Vec<usize> = match router.filter {
            Some(f) if f == item => vec![0],
            Some(_) => vec![1],
            None => (0..3).map(|i| (router.next as usize + i) % 3).collect(),
        };
        for i in order {
            let direction = outputs[i];
            let Some(&target) = res.factory_index.get(&step_from(pos, direction)) else {
                continue;
            };
            if insert_from(world, target, item, Some(direction)) {
                let mut r = world.get::<&mut Router>(entity).expect("router");
                r.item = None;
                r.next = ((i + 1) % 3) as u8;
                break;
            }
        }
    }
}

/// The item filter of a filter extractor or belt router (`None` if unset or not one).
pub fn filter(world: &World, block: Entity) -> Option<ItemKind> {
    world
        .get::<&Extractor>(block)
        .map(|e| e.filter)
        .or_else(|_| world.get::<&Router>(block).map(|r| r.filter))
        .ok()
        .flatten()
}

/// Set the item filter of a filter extractor or belt router.
pub fn set_filter(world: &mut World, block: Entity, filter: Option<ItemKind>) {
    if let Ok(mut e) = world.get::<&mut Extractor>(block) {
        e.filter = filter;
    } else if let Ok(mut r) = world.get::<&mut Router>(block) {
        r.filter = filter;
    }
}

/// Whether the player can stand at `pos`: a passable tile, or a belt.
pub fn walkable(res: &mut Resources, world: &World, pos: Position) -> bool {
    let tile = res.world_state.world.get_tile_cached(pos.x, pos.y, pos.z);
    tile.is_passable()
        || (tile == TileKind::FactoryBlock
            && res
                .factory_index
                .get(&pos)
                .is_some_and(|&e| world.get::<&BeltSlot>(e).is_ok()))
}

/// After the belts move: a player standing on a belt is carried one tile along it every
/// `BELT_STEPS_PER_TILE` steps, if the next tile is walkable and nothing blocks it.
pub fn carry_player(world: &mut World, res: &mut Resources) {
    let Some((player, pos)) = world
        .query::<&Position>()
        .with::<&Player>()
        .iter()
        .next()
        .map(|(e, p)| (e, *p))
    else {
        return;
    };
    let belt_facing = res
        .factory_index
        .get(&pos)
        .and_then(|&e| {
            world
                .get::<&BeltSlot>(e)
                .ok()
                .and(world.get::<&FactoryBlock>(e).ok())
        })
        // Vertical conveyors carry items, not the player
        .filter(|b| b.kind == FactoryKind::Belt)
        .map(|b| b.facing);
    let Some(facing) = belt_facing else {
        res.belt_ride_steps = 0;
        return;
    };
    res.belt_ride_steps += 1;
    if res.belt_ride_steps < BELT_STEPS_PER_TILE {
        return;
    }
    let next = step_from(pos, facing);
    let blocked = world
        .query::<&Position>()
        .with::<&BlocksMovement>()
        .iter()
        .any(|(e, p)| e != player && *p == next);
    if blocked || !walkable(res, world, next) {
        return;
    }
    res.belt_ride_steps = 0;
    if let Ok(mut p) = world.get::<&mut Position>(player) {
        *p = next;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::DroppedItem;
    use crate::factory::{self, FACTORY_STEP_TICKS};
    use crate::intent::{PlayerAction, PlayerIntent};
    use crate::test_support::{TestGame, TestWorld};
    use crate::Game;

    fn at(x: i64, y: i64) -> Position {
        Position { x, y, z: 0 }
    }

    fn game() -> Game {
        TestWorld::new(6)
            .player_at(0, 20, 0)
            .fill((-6, -6, 0), (8, 8, 0), TileKind::Grass)
            .build()
            .game
    }

    fn put(game: &mut Game, kind: FactoryKind, pos: Position, facing: Facing) -> Entity {
        factory::place(&mut game.world, &mut game.res, kind, pos, facing).expect("placed")
    }

    fn give(game: &mut Game, e: Entity, item: ItemKind, qty: u32) {
        game.world.get::<&mut Inventory>(e).unwrap().add(item, qty);
    }

    fn count(game: &Game, e: Entity, item: ItemKind) -> u32 {
        game.world
            .get::<&Inventory>(e)
            .map(|inv| {
                inv.slots
                    .iter()
                    .filter(|s| s.kind == item)
                    .map(|s| s.qty)
                    .sum()
            })
            .unwrap_or(0)
    }

    fn steps(game: &mut Game, n: u64) {
        game.res.player_state.intent = PlayerIntent::movement(0, 0, 0, n * FACTORY_STEP_TICKS);
        game.tick();
    }

    /// Items of `item` anywhere: inventories, belts, routers, furnaces and the ground.
    fn total(game: &Game, item: ItemKind) -> u32 {
        let inventories: u32 = game
            .world
            .query::<&Inventory>()
            .without::<&Player>()
            .iter()
            .flat_map(|(_, inv)| inv.slots.clone())
            .filter(|s| s.kind == item)
            .map(|s| s.qty)
            .sum();
        let belts = game
            .world
            .query::<&BeltSlot>()
            .iter()
            .filter(|(_, b)| b.item == Some(item))
            .count() as u32;
        let routers = game
            .world
            .query::<&Router>()
            .iter()
            .filter(|(_, r)| r.item == Some(item))
            .count() as u32;
        let lying: u32 = game
            .world
            .query::<&DroppedItem>()
            .iter()
            .filter(|(_, d)| d.kind == item)
            .map(|(_, d)| d.qty)
            .sum();
        inventories + belts + routers + lying
    }

    // --- 1.1 taking -------------------------------------------------------------------

    #[test]
    fn peek_reads_each_kind_of_source() {
        let mut g = game();
        let chest = put(&mut g, FactoryKind::Chest, at(0, 0), Facing::North);
        give(&mut g, chest, ItemKind::Stone, 1);
        give(&mut g, chest, ItemKind::Log, 2);
        assert_eq!(peek(&g.world, chest, None), Some(ItemKind::Stone));
        assert_eq!(
            peek(&g.world, chest, Some(ItemKind::Log)),
            Some(ItemKind::Log)
        );
        assert_eq!(peek(&g.world, chest, Some(ItemKind::Acorn)), None);
        let generator = put(
            &mut g,
            FactoryKind::BurnerGenerator,
            at(1, 0),
            Facing::North,
        );
        give(&mut g, generator, ItemKind::Stick, 1);
        assert_eq!(peek(&g.world, generator, None), Some(ItemKind::Stick));
        let furnace = put(&mut g, FactoryKind::CrudeFurnace, at(2, 0), Facing::North);
        g.world.get::<&mut Furnace>(furnace).unwrap().output =
            Some(crate::components::ItemStack::new(ItemKind::Biochar, 2));
        assert_eq!(peek(&g.world, furnace, None), Some(ItemKind::Biochar));
        let belt = put(&mut g, FactoryKind::Belt, at(3, 0), Facing::North);
        *g.world.get::<&mut BeltSlot>(belt).unwrap() = BeltSlot {
            item: Some(ItemKind::Log),
            progress: 0,
        };
        assert_eq!(peek(&g.world, belt, None), None, "belts give nothing");
    }

    // --- 1.2 extractors ---------------------------------------------------------------

    #[test]
    fn extractor_moves_a_chest_onto_a_belt_line() {
        let mut g = game();
        let source = put(&mut g, FactoryKind::Chest, at(0, 0), Facing::North);
        give(&mut g, source, ItemKind::Log, 3);
        put(&mut g, FactoryKind::Extractor, at(1, 0), Facing::East);
        put(&mut g, FactoryKind::Belt, at(2, 0), Facing::East);
        put(&mut g, FactoryKind::Belt, at(3, 0), Facing::East);
        let end = put(&mut g, FactoryKind::Chest, at(4, 0), Facing::North);
        steps(&mut g, 15); // three game seconds
        assert_eq!(count(&g, source, ItemKind::Log), 0, "one a second");
        steps(&mut g, 10);
        assert_eq!(count(&g, end, ItemKind::Log), 3);
        assert_eq!(total(&g, ItemKind::Log), 3, "nothing lost or duplicated");
    }

    #[test]
    fn a_refused_item_stays_in_the_source() {
        let mut g = game();
        let source = put(&mut g, FactoryKind::Chest, at(0, 0), Facing::North);
        give(&mut g, source, ItemKind::Stone, 2);
        put(&mut g, FactoryKind::Extractor, at(1, 0), Facing::East);
        // A generator in front refuses stone
        put(
            &mut g,
            FactoryKind::BurnerGenerator,
            at(2, 0),
            Facing::North,
        );
        steps(&mut g, 50);
        assert_eq!(count(&g, source, ItemKind::Stone), 2);
        // Nothing in front: nothing moves either
        let lone = put(&mut g, FactoryKind::Chest, at(0, 3), Facing::North);
        give(&mut g, lone, ItemKind::Log, 1);
        put(&mut g, FactoryKind::Extractor, at(1, 3), Facing::East);
        steps(&mut g, 50);
        assert_eq!(count(&g, lone, ItemKind::Log), 1);
    }

    #[test]
    fn filter_extractor_moves_only_its_item() {
        let mut g = game();
        let source = put(&mut g, FactoryKind::Chest, at(0, 0), Facing::North);
        give(&mut g, source, ItemKind::Log, 3);
        give(&mut g, source, ItemKind::Acorn, 2);
        let ex = put(&mut g, FactoryKind::FilterExtractor, at(1, 0), Facing::East);
        let planter = put(&mut g, FactoryKind::AutoPlanter, at(2, 0), Facing::North);
        steps(&mut g, 25);
        assert_eq!(
            count(&g, planter, ItemKind::Acorn),
            0,
            "no filter, no moves"
        );
        g.world.get::<&mut Extractor>(ex).unwrap().filter = Some(ItemKind::Acorn);
        steps(&mut g, 25);
        assert_eq!(count(&g, planter, ItemKind::Acorn), 2);
        assert_eq!(count(&g, source, ItemKind::Log), 3, "logs stay");
    }

    // --- 1.3 routers ------------------------------------------------------------------

    fn router_rig(g: &mut Game) -> (Entity, [Entity; 3]) {
        put(g, FactoryKind::Belt, at(0, 2), Facing::East);
        put(g, FactoryKind::Belt, at(1, 2), Facing::East);
        let router = put(g, FactoryKind::BeltRouter, at(2, 2), Facing::East);
        put(g, FactoryKind::Belt, at(2, 1), Facing::North);
        put(g, FactoryKind::Belt, at(3, 2), Facing::East);
        put(g, FactoryKind::Belt, at(2, 3), Facing::South);
        let left = put(g, FactoryKind::Chest, at(2, 0), Facing::North);
        let straight = put(g, FactoryKind::Chest, at(4, 2), Facing::North);
        let right = put(g, FactoryKind::Chest, at(2, 4), Facing::North);
        (router, [left, straight, right])
    }

    fn drop_on(g: &mut Game, pos: Position, item: ItemKind, qty: u32) {
        g.world.spawn((
            pos,
            DroppedItem { kind: item, qty },
            crate::components::SpriteRef::new("items", "log"),
        ));
    }

    #[test]
    fn a_router_splits_evenly() {
        let mut g = game();
        let (_, chests) = router_rig(&mut g);
        drop_on(&mut g, at(0, 2), ItemKind::Log, 6);
        steps(&mut g, 100);
        for c in chests {
            assert_eq!(count(&g, c, ItemKind::Log), 2);
        }
    }

    #[test]
    fn a_router_skips_blocked_outputs() {
        let mut g = game();
        let (_, [_, straight, right]) = router_rig(&mut g);
        // Replace the left output belt with a full chest right next to the router
        let p = g.get_player_entity().unwrap();
        factory::break_block(&mut g.world, &mut g.res, at(2, 1), p);
        let left = put(&mut g, FactoryKind::Chest, at(2, 1), Facing::North);
        g.world.get::<&mut Inventory>(left).unwrap().max_slots = 1;
        give(&mut g, left, ItemKind::Stone, 1);
        drop_on(&mut g, at(0, 2), ItemKind::Log, 4);
        steps(&mut g, 100);
        assert_eq!(count(&g, left, ItemKind::Log), 0, "full: skipped");
        assert_eq!(
            count(&g, straight, ItemKind::Log) + count(&g, right, ItemKind::Log),
            4
        );
        assert_eq!(total(&g, ItemKind::Log), 4);
    }

    #[test]
    fn a_filtered_router_sorts_one_kind_left() {
        let mut g = game();
        let (router, [left, straight, right]) = router_rig(&mut g);
        g.world.get::<&mut Router>(router).unwrap().filter = Some(ItemKind::Acorn);
        drop_on(&mut g, at(0, 2), ItemKind::Acorn, 2);
        drop_on(&mut g, at(1, 2), ItemKind::Log, 3);
        steps(&mut g, 100);
        assert_eq!(count(&g, left, ItemKind::Acorn), 2);
        assert_eq!(count(&g, straight, ItemKind::Log), 3);
        assert_eq!(
            count(&g, right, ItemKind::Log) + count(&g, right, ItemKind::Acorn),
            0
        );
    }

    #[test]
    fn routers_take_items_only_from_behind() {
        let mut g = game();
        let router = put(&mut g, FactoryKind::BeltRouter, at(0, 0), Facing::East);
        assert!(!factory::accepts_from(
            &g.world,
            router,
            ItemKind::Log,
            Some(Facing::South)
        ));
        assert!(!factory::accepts(&g.world, router, ItemKind::Log));
        assert!(factory::accepts_from(
            &g.world,
            router,
            ItemKind::Log,
            Some(Facing::East)
        ));
    }

    // --- 1.4 riding belts -------------------------------------------------------------

    fn rider() -> TestGame {
        TestWorld::new(6)
            .fill((-6, -6, 0), (8, 8, 0), TileKind::Grass)
            .inventory(&[])
            .build()
    }

    #[test]
    fn walking_onto_a_belt() {
        let mut t = rider();
        // A south-facing belt into a chest, so the ride can't carry the player away
        put(&mut t.game, FactoryKind::Belt, at(1, 0), Facing::South);
        put(&mut t.game, FactoryKind::Chest, at(1, 1), Facing::North);
        t.act(PlayerAction::Move {
            dx: 1,
            dy: 0,
            dz: 0,
        });
        t.tick();
        assert_eq!(t.player_pos(), at(1, 0), "belts are walkable");
    }

    #[test]
    fn a_belt_carries_the_player_two_tiles_a_second() {
        let mut t = rider();
        for x in 1..=3 {
            put(&mut t.game, FactoryKind::Belt, at(x, 0), Facing::East);
        }
        let p = t.game.get_player_entity().unwrap();
        *t.game.world.get::<&mut Position>(p).unwrap() = at(1, 0);
        steps(&mut t.game, 5);
        assert_eq!(t.player_pos(), at(3, 0));
    }

    #[test]
    fn riding_stops_at_an_obstacle_and_other_blocks_stay_solid() {
        let mut t = rider();
        put(&mut t.game, FactoryKind::Belt, at(1, 0), Facing::East);
        put(&mut t.game, FactoryKind::Chest, at(2, 0), Facing::North);
        t.act(PlayerAction::Move {
            dx: 1,
            dy: 0,
            dz: 0,
        });
        t.tick();
        steps(&mut t.game, 20);
        assert_eq!(t.player_pos(), at(1, 0), "the chest blocks the ride");
        t.act(PlayerAction::Move {
            dx: 1,
            dy: 0,
            dz: 0,
        });
        t.tick();
        assert_eq!(t.player_pos(), at(1, 0), "chests aren't walkable");
    }

    fn g_index(g: &Game, origin: Position, dx: i64, dy: i64) -> Entity {
        g.res.factory_index[&Position {
            x: origin.x + dx,
            y: origin.y + dy,
            z: origin.z,
        }]
    }

    #[test]
    fn the_showcase_tree_farm_replants_from_its_own_acorns() {
        let mut settings =
            crate::resources::WorldSettings::defaults(&crate::game_config::GameConfig::new());
        settings.world_type = crate::resources::WorldType::FactoryShowcase;
        let mut g = Game::new_with_settings(5, settings);
        let (_, origin) = crate::showcase::layout()
            .into_iter()
            .find(|(e, _)| e.name == "Self-feeding tree farm")
            .unwrap();
        let block = |dx, dy| g_index(&g, origin, dx, dy);
        let (planter, chest) = (block(3, 2), block(3, 4));
        // 30 game minutes: the 4 starting acorns run out, then the chopped trees' acorns come
        // back through the filter extractor
        let (mut ran_out, mut refilled) = (false, false);
        for _ in 0..180 {
            g.pass_time();
            let acorns = count(&g, planter, ItemKind::Acorn);
            ran_out |= acorns == 0;
            refilled |= ran_out && acorns > 0;
        }
        assert!(
            ran_out && refilled,
            "the planter was refilled from its own harvest"
        );
        assert!(
            count(&g, chest, ItemKind::Log) > 0,
            "the harvester filled the chest"
        );
    }
}

//! Factory blocks: placeable 1x1 blocks backed by an entity on a `TileKind::FactoryBlock`
//! marker tile, so they can hold a facing, an inventory and progress. Factory systems advance
//! in fixed steps of elapsed game ticks, wherever the blocks are.

use std::collections::HashMap;

use hecs::{Entity, World};
use serde::{Deserialize, Serialize};

use crate::components::{
    itemkind_name, itemkind_sprite_name, DroppedItem, Inventory, ItemKind, Position, SpriteRef,
};
use crate::resources::Resources;
use crate::tiles::TileKind;

/// Game ticks per game second.
pub const TICKS_PER_SECOND: u64 = 100;
/// Game ticks per factory step (five steps per game second).
pub const FACTORY_STEP_TICKS: u64 = 20;
/// Item stacks a chest holds.
pub const CHEST_SLOTS: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Facing {
    #[default]
    North,
    East,
    South,
    West,
}

impl Facing {
    pub fn rotate_cw(self) -> Self {
        match self {
            Facing::North => Facing::East,
            Facing::East => Facing::South,
            Facing::South => Facing::West,
            Facing::West => Facing::North,
        }
    }

    /// The (dx, dy) step this facing points along; north is -y.
    pub fn offset(self) -> (i64, i64) {
        match self {
            Facing::North => (0, -1),
            Facing::East => (1, 0),
            Facing::South => (0, 1),
            Facing::West => (-1, 0),
        }
    }

    pub fn arrow(self) -> char {
        match self {
            Facing::North => '^',
            Facing::East => '>',
            Facing::South => 'v',
            Facing::West => '<',
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Facing::North => "north",
            Facing::East => "east",
            Facing::South => "south",
            Facing::West => "west",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FactoryKind {
    Chest,
    /// A directional block with no behavior, for tests of facing and rotation.
    #[cfg(any(test, feature = "test-support"))]
    TestArrow,
}

impl FactoryKind {
    /// The kind an item places, if it is a factory block.
    pub fn from_item(item: ItemKind) -> Option<Self> {
        match item {
            ItemKind::Chest => Some(FactoryKind::Chest),
            _ => None,
        }
    }

    /// The item breaking this block gives back.
    pub fn item(self) -> Option<ItemKind> {
        match self {
            FactoryKind::Chest => Some(ItemKind::Chest),
            #[cfg(any(test, feature = "test-support"))]
            FactoryKind::TestArrow => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            FactoryKind::Chest => "chest",
            #[cfg(any(test, feature = "test-support"))]
            FactoryKind::TestArrow => "test arrow",
        }
    }

    /// Whether the block's facing matters (and so whether it can be rotated).
    pub fn directional(self) -> bool {
        match self {
            FactoryKind::Chest => false,
            #[cfg(any(test, feature = "test-support"))]
            FactoryKind::TestArrow => true,
        }
    }

    /// The sprite (category "factory") drawn for this kind and facing.
    pub fn sprite(self, facing: Facing) -> SpriteRef {
        let _ = facing; // No directional sprites yet; belts will add `belt_<facing>`
        match self {
            FactoryKind::Chest => SpriteRef::new("factory", "chest"),
            #[cfg(any(test, feature = "test-support"))]
            FactoryKind::TestArrow => SpriteRef::new("factory", "chest"),
        }
    }
}

/// A placed factory block. The entity also has a `Position`, a `SpriteRef`, and per-kind
/// components (chests: an `Inventory` with `CHEST_SLOTS` slots).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactoryBlock {
    pub kind: FactoryKind,
    pub facing: Facing,
}

/// Elapsed-time bookkeeping for factory steps.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactoryClock {
    /// Game tick when steps were last run
    pub last_tick: u64,
    /// Ticks since then that didn't fill a whole step
    pub carry: u64,
    /// Steps run so far
    pub steps: u64,
}

/// Factory blocks by position. Derived from the entities, so it isn't saved; rebuild it
/// after replacing the world.
pub type FactoryIndex = HashMap<Position, Entity>;

pub fn rebuild_index(world: &World) -> FactoryIndex {
    world
        .query::<(&Position, &FactoryBlock)>()
        .iter()
        .map(|(e, (pos, _))| (*pos, e))
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaceError {
    /// A solid tile, a factory block or another entity is there
    Occupied,
}

/// Whether a block can be placed at `pos`: a passable tile with no entity on it. Shared by
/// plain blocks and factory blocks.
pub fn can_place_at(world: &World, res: &mut Resources, pos: Position) -> bool {
    let tile = res.world_state.world.get_tile_cached(pos.x, pos.y, pos.z);
    tile.is_passable() && !world.query::<&Position>().iter().any(|(_, p)| *p == pos)
}

/// Place a factory block. Does not touch the player's inventory.
pub fn place(
    world: &mut World,
    res: &mut Resources,
    kind: FactoryKind,
    pos: Position,
    facing: Facing,
) -> Result<Entity, PlaceError> {
    if !can_place_at(world, res, pos) {
        return Err(PlaceError::Occupied);
    }
    res.world_state
        .world
        .set_tile_cached(pos.x, pos.y, pos.z, TileKind::FactoryBlock);
    let mut builder = hecs::EntityBuilder::new();
    builder.add(pos);
    builder.add(FactoryBlock { kind, facing });
    builder.add(kind.sprite(facing));
    if kind == FactoryKind::Chest {
        builder.add(Inventory {
            max_slots: CHEST_SLOTS,
            ..Default::default()
        });
    }
    let entity = world.spawn(builder.build());
    res.factory_index.insert(pos, entity);
    Ok(entity)
}

/// Break the factory block at `pos`: its item goes to `receiver`'s inventory and everything
/// it held is dropped on its tile. Returns the kind broken, or `None` if nothing is there.
pub fn break_block(
    world: &mut World,
    res: &mut Resources,
    pos: Position,
    receiver: Entity,
) -> Option<FactoryKind> {
    let entity = res.factory_index.remove(&pos)?;
    let block = *world.get::<&FactoryBlock>(entity).ok()?;
    let contents = world
        .get::<&Inventory>(entity)
        .map(|inv| inv.slots.clone())
        .unwrap_or_default();
    let _ = world.despawn(entity);
    res.world_state
        .world
        .set_tile_cached(pos.x, pos.y, pos.z, TileKind::Air);
    if let Some(item) = block.kind.item() {
        if let Ok(mut inv) = world.get::<&mut Inventory>(receiver) {
            inv.add(item, 1);
        }
    }
    for stack in contents {
        world.spawn((
            pos,
            DroppedItem {
                kind: stack.kind,
                qty: stack.qty,
            },
            SpriteRef::new("items", itemkind_sprite_name(stack.kind)),
        ));
    }
    Some(block.kind)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotateResult {
    Rotated(Facing),
    NotDirectional(FactoryKind),
    NoBlock,
}

/// Rotate the factory block at `pos` one step clockwise.
pub fn rotate(world: &mut World, res: &Resources, pos: Position) -> RotateResult {
    let Some(&entity) = res.factory_index.get(&pos) else {
        return RotateResult::NoBlock;
    };
    let Ok(mut block) = world.get::<&mut FactoryBlock>(entity) else {
        return RotateResult::NoBlock;
    };
    if !block.kind.directional() {
        return RotateResult::NotDirectional(block.kind);
    }
    block.facing = block.facing.rotate_cw();
    let (kind, facing) = (block.kind, block.facing);
    drop(block);
    let _ = world.insert_one(entity, kind.sprite(facing));
    RotateResult::Rotated(facing)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferError {
    /// The source has no such stack
    Empty,
    /// The destination has no room for a new stack
    Full,
}

/// Move one item from stack `slot` of `from`'s inventory to `to`'s inventory.
pub fn move_one_item(
    world: &mut World,
    from: Entity,
    to: Entity,
    slot: usize,
) -> Result<ItemKind, TransferError> {
    let kind = {
        let inv = world
            .get::<&Inventory>(from)
            .map_err(|_| TransferError::Empty)?;
        inv.slots
            .get(slot)
            .filter(|s| s.qty > 0)
            .ok_or(TransferError::Empty)?
            .kind
    };
    {
        let mut dest = world
            .get::<&mut Inventory>(to)
            .map_err(|_| TransferError::Full)?;
        let has_stack = dest.slots.iter().any(|s| s.kind == kind);
        if !has_stack && dest.slots.len() >= dest.max_slots {
            return Err(TransferError::Full);
        }
        dest.add(kind, 1);
    }
    let mut src = world.get::<&mut Inventory>(from).expect("checked above");
    src.slots[slot].qty -= 1;
    if src.slots[slot].qty == 0 {
        src.slots.remove(slot);
    }
    Ok(kind)
}

/// Run every factory step that fits in the ticks elapsed since the last call. Factory systems
/// (none yet; later changes add growth, generators, power, machines and belts, in that order)
/// run over all factory blocks, wherever they are.
pub fn run_due_steps(_world: &mut World, res: &mut Resources) {
    let clock = &mut res.factory_clock;
    let elapsed = res.time.tick.saturating_sub(clock.last_tick) + clock.carry;
    clock.last_tick = res.time.tick;
    clock.carry = elapsed % FACTORY_STEP_TICKS;
    let steps = elapsed / FACTORY_STEP_TICKS;
    for _ in 0..steps {
        clock.steps += 1;
    }
}

/// A short description of a placement failure for the message log.
pub fn place_error_message(error: PlaceError, item: ItemKind) -> String {
    match error {
        PlaceError::Occupied => {
            format!("Can't place {}: the tile is occupied", itemkind_name(item))
        }
    }
}

impl crate::Game {
    /// The tile at `pos` as factory code sees it. A chunk it touches first is generated the
    /// same way as when the player walks there, including its pending structures.
    pub fn factory_tile(&mut self, pos: Position) -> TileKind {
        let (cx, cy, cz) = crate::Game::world_to_chunk_coords(pos.x, pos.y, pos.z);
        self.ensure_chunk_with_pending_structures(cx, cy, cz);
        self.res
            .world_state
            .world
            .get_tile_cached(pos.x, pos.y, pos.z)
    }

    /// The tile and factory block (if any) one step from `pos` along `facing`. All factory
    /// neighbor access goes through here, so chunk edges and negative coordinates behave
    /// like any other position.
    pub fn factory_neighbor(
        &mut self,
        pos: Position,
        facing: Facing,
    ) -> (Position, TileKind, Option<Entity>) {
        let (dx, dy) = facing.offset();
        let next = Position {
            x: pos.x + dx,
            y: pos.y + dy,
            z: pos.z,
        };
        let tile = self.factory_tile(next);
        (next, tile, self.res.factory_index.get(&next).copied())
    }

    /// Pass 10 game seconds in 100-tick waits, as if the player waited; stops after the wait
    /// in which combat starts. Returns every tick result combined.
    pub fn pass_time(&mut self) -> crate::game::GameTickResult {
        const SECONDS: u64 = 10;
        let mut result = crate::game::GameTickResult::NoAction;
        for _ in 0..SECONDS {
            self.res.player_state.intent =
                crate::intent::PlayerIntent::movement(0, 0, 0, TICKS_PER_SECOND);
            result |= self.tick();
            if result.contains(crate::game::GameTickResult::CombatTriggered) {
                break;
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TestWorld;
    use crate::Game;

    fn arena() -> TestWorld {
        TestWorld::new(7).fill((-6, -6, 0), (6, 6, 0), TileKind::Grass)
    }

    fn at(x: i64, y: i64) -> Position {
        Position { x, y, z: 0 }
    }

    fn player(game: &Game) -> Entity {
        game.get_player_entity().unwrap()
    }

    fn place_in(game: &mut Game, kind: FactoryKind, pos: Position, facing: Facing) -> Entity {
        place(&mut game.world, &mut game.res, kind, pos, facing).expect("placed")
    }

    #[test]
    fn placing_puts_an_entity_on_a_marker_tile() {
        let mut game = arena().build().game;
        let chest = place_in(&mut game, FactoryKind::Chest, at(1, 0), Facing::North);
        assert_eq!(
            game.res.world_state.world.get_tile_cached(1, 0, 0),
            TileKind::FactoryBlock
        );
        assert_eq!(game.res.factory_index.get(&at(1, 0)), Some(&chest));
        assert_eq!(
            game.world.get::<&Inventory>(chest).unwrap().max_slots,
            CHEST_SLOTS
        );
        assert!(!TileKind::FactoryBlock.is_passable(), "blocks movement");
    }

    #[test]
    fn occupied_targets_are_refused() {
        let mut world = arena();
        world.sheep(2, 0, 0);
        let mut game = world.build().game;
        place_in(&mut game, FactoryKind::Chest, at(1, 0), Facing::North);
        for pos in [at(1, 0), at(2, 0), at(0, 0)] {
            let r = place(
                &mut game.world,
                &mut game.res,
                FactoryKind::Chest,
                pos,
                Facing::North,
            );
            assert_eq!(r, Err(PlaceError::Occupied), "{pos:?}");
        }
        game.res
            .world_state
            .world
            .set_tile_cached(3, 0, 0, TileKind::Rock);
        let r = place(
            &mut game.world,
            &mut game.res,
            FactoryKind::Chest,
            at(3, 0),
            Facing::North,
        );
        assert_eq!(r, Err(PlaceError::Occupied), "solid tile");
    }

    #[test]
    fn breaking_returns_the_item_and_spills_contents() {
        let mut game = arena().inventory(&[]).build().game;
        let chest = place_in(&mut game, FactoryKind::Chest, at(1, 0), Facing::North);
        game.world
            .get::<&mut Inventory>(chest)
            .unwrap()
            .add(ItemKind::Log, 10);

        let p = player(&game);
        assert_eq!(
            break_block(&mut game.world, &mut game.res, at(1, 0), p),
            Some(FactoryKind::Chest)
        );
        assert!(!game.world.contains(chest));
        assert!(game.res.factory_index.is_empty());
        assert!(game
            .res
            .world_state
            .world
            .get_tile_cached(1, 0, 0)
            .is_passable());
        let inv = game.world.get::<&Inventory>(p).unwrap().slots.clone();
        assert_eq!(inv.len(), 1);
        assert_eq!((inv[0].kind, inv[0].qty), (ItemKind::Chest, 1));
        let dropped: Vec<(Position, ItemKind, u32)> = game
            .world
            .query::<(&Position, &DroppedItem)>()
            .iter()
            .map(|(_, (p, d))| (*p, d.kind, d.qty))
            .collect();
        assert_eq!(dropped, [(at(1, 0), ItemKind::Log, 10)]);
        assert_eq!(
            break_block(&mut game.world, &mut game.res, at(1, 0), p),
            None
        );
    }

    #[test]
    fn mining_a_factory_block_breaks_it() {
        let mut test = arena().inventory(&[]).build();
        place_in(&mut test.game, FactoryKind::Chest, at(1, 0), Facing::North);
        test.act(crate::intent::PlayerAction::MineAt { x: 1, y: 0, z: 0 });
        test.tick();
        assert!(test.game.res.factory_index.is_empty());
        assert_eq!(test.player_has(ItemKind::Chest), 1);
    }

    #[test]
    fn rotation_turns_directional_blocks_clockwise_only() {
        let mut game = arena().build().game;
        let arrow = place_in(&mut game, FactoryKind::TestArrow, at(1, 0), Facing::North);
        place_in(&mut game, FactoryKind::Chest, at(-1, 0), Facing::North);
        assert_eq!(
            rotate(&mut game.world, &game.res, at(1, 0)),
            RotateResult::Rotated(Facing::East)
        );
        assert_eq!(
            game.world.get::<&FactoryBlock>(arrow).unwrap().facing,
            Facing::East
        );
        assert_eq!(
            rotate(&mut game.world, &game.res, at(-1, 0)),
            RotateResult::NotDirectional(FactoryKind::Chest)
        );
        assert_eq!(
            rotate(&mut game.world, &game.res, at(0, 1)),
            RotateResult::NoBlock
        );
        let mut facing = Facing::North;
        for expected in [Facing::East, Facing::South, Facing::West, Facing::North] {
            facing = facing.rotate_cw();
            assert_eq!(facing, expected);
        }
    }

    #[test]
    fn moving_items_respects_chest_capacity() {
        let mut game = arena().inventory(&[]).build().game;
        let chest = place_in(&mut game, FactoryKind::Chest, at(1, 0), Facing::North);
        let p = player(&game);
        {
            let mut inv = game.world.get::<&mut Inventory>(p).unwrap();
            inv.max_slots = 40;
            for kind in [ItemKind::Log, ItemKind::Stick] {
                inv.add(kind, 5);
            }
        }
        for _ in 0..5 {
            assert_eq!(
                move_one_item(&mut game.world, p, chest, 0),
                Ok(ItemKind::Log)
            );
        }
        let chest_inv = game.world.get::<&Inventory>(chest).unwrap().slots.clone();
        assert_eq!((chest_inv[0].kind, chest_inv[0].qty), (ItemKind::Log, 5));
        let player_inv = game.world.get::<&Inventory>(p).unwrap().slots.clone();
        assert_eq!(player_inv.len(), 1, "the emptied stack is gone");

        game.world.get::<&mut Inventory>(chest).unwrap().max_slots = 1;
        assert_eq!(
            move_one_item(&mut game.world, p, chest, 0),
            Err(TransferError::Full)
        );
        assert_eq!(
            move_one_item(&mut game.world, p, chest, 7),
            Err(TransferError::Empty)
        );
    }

    #[test]
    fn factory_steps_depend_only_on_elapsed_time() {
        let mut one = arena().build();
        one.game.res.player_state.intent = crate::intent::PlayerIntent::movement(0, 0, 0, 1000);
        one.tick();
        let mut five = arena().build();
        for _ in 0..5 {
            five.game.res.player_state.intent = crate::intent::PlayerIntent::movement(0, 0, 0, 200);
            five.tick();
        }
        assert_eq!(one.game.res.time.tick, five.game.res.time.tick);
        assert_eq!(one.game.res.factory_clock, five.game.res.factory_clock);
        assert_eq!(one.game.res.factory_clock.steps, 1000 / FACTORY_STEP_TICKS);

        // Leftover ticks carry over instead of being lost
        let mut odd = arena().build();
        for _ in 0..3 {
            odd.game.res.player_state.intent = crate::intent::PlayerIntent::movement(0, 0, 0, 30);
            odd.tick();
        }
        assert_eq!(odd.game.res.factory_clock.steps, 90 / FACTORY_STEP_TICKS);
        assert_eq!(odd.game.res.factory_clock.carry, 90 % FACTORY_STEP_TICKS);
    }

    #[test]
    fn neighbors_across_chunk_edges_and_negative_coordinates() {
        use crate::world::CHUNK_SIZE;
        let mut game = TestWorld::new(7).player_at(10, 10, 0).build().game;
        let edge = CHUNK_SIZE - 1; // 63, the last column of chunk 0
        for (from, facing, to) in [
            (at(edge, 0), Facing::East, at(edge + 1, 0)),
            (at(0, 0), Facing::West, at(-1, 0)),
            (at(-5, 0), Facing::East, at(-4, 0)),
            (at(0, 0), Facing::North, at(0, -1)),
        ] {
            game.res
                .world_state
                .world
                .set_tile_cached(to.x, to.y, to.z, TileKind::Grass);
            let target = place_in(&mut game, FactoryKind::Chest, to, Facing::North);
            let (pos, tile, entity) = game.factory_neighbor(from, facing);
            assert_eq!(pos, to);
            assert_eq!(tile, TileKind::FactoryBlock);
            assert_eq!(entity, Some(target), "{from:?} {facing:?}");
        }
    }

    /// Every tile of chunk (cx, cy, 0), from a world that already has it.
    fn chunk_tiles(game: &Game, cx: i64, cy: i64) -> Vec<TileKind> {
        use crate::world::CHUNK_SIZE;
        let world = &game.res.world_state.world;
        (0..CHUNK_SIZE * CHUNK_SIZE)
            .map(|i| {
                world.get_tile_cached_no_worldgen(
                    cx * CHUNK_SIZE + i % CHUNK_SIZE,
                    cy * CHUNK_SIZE + i / CHUNK_SIZE,
                    0,
                )
            })
            .collect()
    }

    #[test]
    fn machine_generated_chunk_matches_a_walked_to_chunk() {
        use crate::world::CHUNK_SIZE;
        // A chunk far from the start, with a structure queued in it, that nobody has visited
        let (cx, cy) = (-40, 37);
        let origin = at(cx * CHUNK_SIZE + 10, cy * CHUNK_SIZE + 10);
        let queue = |game: &mut Game| {
            game.res
                .pending_structures
                .push(crate::resources::PendingStructure {
                    name: "sapiencorp-bunker.lrstructure".to_string(),
                    x: origin.x,
                    y: origin.y,
                    z: 0,
                    bury_structure: false,
                });
        };

        let mut by_machine = Game::new(12345);
        queue(&mut by_machine);
        assert!(!by_machine.res.world_state.world.has_chunk(cx, cy, 0));
        by_machine.factory_tile(at(origin.x + 30, origin.y + 30));

        let mut walked = Game::new(12345);
        queue(&mut walked);
        let p = player(&walked);
        *walked.world.get::<&mut Position>(p).unwrap() = at(origin.x + 30, origin.y + 30);
        walked.tick();

        let machine_tiles = chunk_tiles(&by_machine, cx, cy);
        assert_eq!(machine_tiles, chunk_tiles(&walked, cx, cy));
        let plain = Game::new(12345);
        let mut plain = plain;
        plain.res.world_state.world.ensure_chunk(cx, cy, 0);
        assert_ne!(
            machine_tiles,
            chunk_tiles(&plain, cx, cy),
            "the structure was placed"
        );
    }
}

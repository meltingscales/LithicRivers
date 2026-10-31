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
/// Game ticks per factory second, the unit block rates and power figures are measured in.
/// Factories run at a third of game speed (one factory second is three game seconds), so
/// they don't outpace a player whose every step takes two game seconds.
pub const FACTORY_SECOND_TICKS: u64 = 3 * TICKS_PER_SECOND;
/// Factory steps per factory second.
pub const STEPS_PER_FACTORY_SECOND: u64 = 5;
/// Game ticks per factory step.
pub const FACTORY_STEP_TICKS: u64 = FACTORY_SECOND_TICKS / STEPS_PER_FACTORY_SECOND;
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
    pub fn opposite(self) -> Self {
        self.rotate_cw().rotate_cw()
    }

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, strum::EnumIter)]
pub enum FactoryKind {
    Chest,
    /// Wooden conveyor: carries one item a tile in its facing direction
    Belt,
    /// Burns fuel items to power its network
    BurnerGenerator,
    /// Joins generators and consumers into a network
    PowerPole,
    /// Plants acorns around itself
    AutoPlanter,
    /// Chops grown trees around itself and outputs the wood in its facing direction
    AutoHarvester,
    /// Burns fuel to smelt wood into biochar; fed from its back and sides
    CrudeFurnace,
    /// Moves items from the block behind it into the block in front
    Extractor,
    /// An extractor that moves only one item kind
    FilterExtractor,
    /// Splits a belt line in turn, or sorts one item kind to the left
    BeltRouter,
    /// Carries one item to the block on the layer above
    VerticalConveyorUp,
    /// Carries one item to the block on the layer below
    VerticalConveyorDown,
    /// Smelts on grid power instead of fuel; fed from its back and sides
    ElectricFurnace,
    /// Holds a spare body of crude parts, with no power
    CrudeVat,
    /// Holds a spare body of crude or standard parts, on a trickle of power
    StandardVat,
    /// Holds a spare body of any grade, on a lot of power
    AdvancedVat,
    /// Keeps creatures and the player out, but not sight or light
    Fence,
    /// Keeps creatures out; the player walks through it
    Gate,
    /// Grows creatures and lets them out beside it
    Spawner,
    /// A consumer that does nothing but draw power, for grid tests
    #[cfg(any(test, feature = "test-support"))]
    TestConsumer,
    /// A directional block with no behavior, for tests of facing and rotation.
    #[cfg(any(test, feature = "test-support"))]
    TestArrow,
}

impl FactoryKind {
    /// The kind an item places, if it is a factory block.
    pub fn from_item(item: ItemKind) -> Option<Self> {
        match item {
            ItemKind::Chest => Some(FactoryKind::Chest),
            ItemKind::WoodenConveyor => Some(FactoryKind::Belt),
            ItemKind::BurnerGenerator => Some(FactoryKind::BurnerGenerator),
            ItemKind::PowerPole => Some(FactoryKind::PowerPole),
            ItemKind::AutoPlanter => Some(FactoryKind::AutoPlanter),
            ItemKind::AutoHarvester => Some(FactoryKind::AutoHarvester),
            ItemKind::CrudeFurnace => Some(FactoryKind::CrudeFurnace),
            ItemKind::ElectricFurnace => Some(FactoryKind::ElectricFurnace),
            ItemKind::CrudeVat => Some(FactoryKind::CrudeVat),
            ItemKind::StandardVat => Some(FactoryKind::StandardVat),
            ItemKind::AdvancedVat => Some(FactoryKind::AdvancedVat),
            ItemKind::Extractor => Some(FactoryKind::Extractor),
            ItemKind::FilterExtractor => Some(FactoryKind::FilterExtractor),
            ItemKind::BeltRouter => Some(FactoryKind::BeltRouter),
            ItemKind::VerticalConveyor => Some(FactoryKind::VerticalConveyorUp),
            ItemKind::Fence => Some(FactoryKind::Fence),
            ItemKind::Gate => Some(FactoryKind::Gate),
            ItemKind::Spawner => Some(FactoryKind::Spawner),
            _ => None,
        }
    }

    /// The item breaking this block gives back.
    pub fn item(self) -> Option<ItemKind> {
        match self {
            FactoryKind::Chest => Some(ItemKind::Chest),
            FactoryKind::Belt => Some(ItemKind::WoodenConveyor),
            FactoryKind::BurnerGenerator => Some(ItemKind::BurnerGenerator),
            FactoryKind::PowerPole => Some(ItemKind::PowerPole),
            FactoryKind::AutoPlanter => Some(ItemKind::AutoPlanter),
            FactoryKind::AutoHarvester => Some(ItemKind::AutoHarvester),
            FactoryKind::CrudeFurnace => Some(ItemKind::CrudeFurnace),
            FactoryKind::ElectricFurnace => Some(ItemKind::ElectricFurnace),
            FactoryKind::CrudeVat => Some(ItemKind::CrudeVat),
            FactoryKind::StandardVat => Some(ItemKind::StandardVat),
            FactoryKind::AdvancedVat => Some(ItemKind::AdvancedVat),
            FactoryKind::Extractor => Some(ItemKind::Extractor),
            FactoryKind::FilterExtractor => Some(ItemKind::FilterExtractor),
            FactoryKind::BeltRouter => Some(ItemKind::BeltRouter),
            FactoryKind::VerticalConveyorUp | FactoryKind::VerticalConveyorDown => {
                Some(ItemKind::VerticalConveyor)
            }
            FactoryKind::Fence => Some(ItemKind::Fence),
            FactoryKind::Gate => Some(ItemKind::Gate),
            FactoryKind::Spawner => Some(ItemKind::Spawner),
            #[cfg(any(test, feature = "test-support"))]
            FactoryKind::TestConsumer => None,
            #[cfg(any(test, feature = "test-support"))]
            FactoryKind::TestArrow => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            FactoryKind::Chest => "chest",
            FactoryKind::Belt => "wooden conveyor",
            FactoryKind::BurnerGenerator => "burner generator",
            FactoryKind::PowerPole => "power pole",
            FactoryKind::AutoPlanter => "auto-planter",
            FactoryKind::AutoHarvester => "auto-harvester",
            FactoryKind::CrudeFurnace => "crude furnace",
            FactoryKind::ElectricFurnace => "electric furnace",
            FactoryKind::CrudeVat => "crude resurrection vat",
            FactoryKind::StandardVat => "standard resurrection vat",
            FactoryKind::AdvancedVat => "advanced resurrection vat",
            FactoryKind::Extractor => "extractor",
            FactoryKind::FilterExtractor => "filter extractor",
            FactoryKind::BeltRouter => "belt router",
            FactoryKind::VerticalConveyorUp => "vertical conveyor (up)",
            FactoryKind::VerticalConveyorDown => "vertical conveyor (down)",
            FactoryKind::Fence => "fence",
            FactoryKind::Gate => "gate",
            FactoryKind::Spawner => "spawner",
            #[cfg(any(test, feature = "test-support"))]
            FactoryKind::TestConsumer => "test consumer",
            #[cfg(any(test, feature = "test-support"))]
            FactoryKind::TestArrow => "test arrow",
        }
    }

    /// Whether the block's facing matters (and so whether it can be rotated).
    pub fn directional(self) -> bool {
        match self {
            FactoryKind::Chest => false,
            FactoryKind::Belt
            | FactoryKind::AutoHarvester
            | FactoryKind::CrudeFurnace
            | FactoryKind::ElectricFurnace
            | FactoryKind::Extractor
            | FactoryKind::FilterExtractor
            | FactoryKind::BeltRouter => true,
            FactoryKind::AutoPlanter => false,
            FactoryKind::BurnerGenerator | FactoryKind::PowerPole => false,
            FactoryKind::CrudeVat | FactoryKind::StandardVat | FactoryKind::AdvancedVat => false,
            // Up or down instead of a facing: see `rotate`
            FactoryKind::VerticalConveyorUp | FactoryKind::VerticalConveyorDown => false,
            FactoryKind::Fence | FactoryKind::Gate | FactoryKind::Spawner => false,
            #[cfg(any(test, feature = "test-support"))]
            FactoryKind::TestConsumer => false,
            #[cfg(any(test, feature = "test-support"))]
            FactoryKind::TestArrow => true,
        }
    }

    /// Whether sight and light pass over the block, though nothing walks through it.
    pub fn see_through(self) -> bool {
        matches!(self, FactoryKind::Fence | FactoryKind::Gate)
    }

    /// Kinds that exist only in tests (always false outside tests).
    pub fn is_test_only(self) -> bool {
        #[cfg(any(test, feature = "test-support"))]
        return matches!(self, FactoryKind::TestConsumer | FactoryKind::TestArrow);
        #[cfg(not(any(test, feature = "test-support")))]
        false
    }

    /// The test-only power consumer (always false outside tests).
    pub fn is_test_consumer(self) -> bool {
        #[cfg(any(test, feature = "test-support"))]
        return self == FactoryKind::TestConsumer;
        #[cfg(not(any(test, feature = "test-support")))]
        false
    }

    /// The sprite (category "factory") drawn for this kind and facing.
    pub fn sprite(self, facing: Facing) -> SpriteRef {
        match self {
            FactoryKind::Chest => SpriteRef::new("factory", "chest"),
            FactoryKind::Belt => SpriteRef::new("factory", &format!("belt_{}", facing.name())),
            FactoryKind::BurnerGenerator => SpriteRef::new("factory", "burner_generator"),
            FactoryKind::PowerPole => SpriteRef::new("factory", "power_pole"),
            FactoryKind::AutoPlanter => SpriteRef::new("factory", "auto_planter"),
            FactoryKind::AutoHarvester => {
                SpriteRef::new("factory", &format!("auto_harvester_{}", facing.name()))
            }
            FactoryKind::CrudeFurnace | FactoryKind::ElectricFurnace => {
                crate::furnace::sprite(self, facing, false)
            }
            FactoryKind::Extractor => {
                SpriteRef::new("factory", &format!("extractor_{}", facing.name()))
            }
            FactoryKind::FilterExtractor => {
                SpriteRef::new("factory", &format!("filter_extractor_{}", facing.name()))
            }
            FactoryKind::BeltRouter => {
                SpriteRef::new("factory", &format!("belt_router_{}", facing.name()))
            }
            FactoryKind::VerticalConveyorUp => SpriteRef::new("factory", "vertical_conveyor_up"),
            FactoryKind::CrudeVat => SpriteRef::new("factory", "crude_vat"),
            FactoryKind::StandardVat => SpriteRef::new("factory", "standard_vat"),
            FactoryKind::AdvancedVat => SpriteRef::new("factory", "advanced_vat"),
            FactoryKind::VerticalConveyorDown => {
                SpriteRef::new("factory", "vertical_conveyor_down")
            }
            FactoryKind::Fence => SpriteRef::new("factory", "fence"),
            FactoryKind::Gate => SpriteRef::new("factory", "gate"),
            FactoryKind::Spawner => SpriteRef::new("factory", "spawner"),
            #[cfg(any(test, feature = "test-support"))]
            FactoryKind::TestConsumer => SpriteRef::new("factory", "chest"),
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
    match kind {
        FactoryKind::Chest => {
            builder.add(Inventory {
                max_slots: CHEST_SLOTS,
                ..Default::default()
            });
        }
        FactoryKind::Belt | FactoryKind::VerticalConveyorUp | FactoryKind::VerticalConveyorDown => {
            builder.add(BeltSlot::default());
        }
        FactoryKind::BurnerGenerator => {
            builder.add(crate::power::Generator::default());
            builder.add(Inventory {
                max_slots: crate::power::BURNER_FUEL_SLOTS,
                ..Default::default()
            });
        }
        FactoryKind::PowerPole => {
            builder.add(crate::power::PowerPole);
        }
        FactoryKind::CrudeFurnace => {
            builder.add(crate::furnace::Furnace::default());
        }
        FactoryKind::ElectricFurnace => {
            builder.add(crate::furnace::Furnace::default());
            builder.add(crate::power::PowerConsumer::idle(
                crate::furnace::ELECTRIC_FURNACE_W,
            ));
        }
        FactoryKind::Extractor | FactoryKind::FilterExtractor => {
            builder.add(crate::logistics::Extractor::default());
        }
        FactoryKind::CrudeVat | FactoryKind::StandardVat | FactoryKind::AdvancedVat => {
            builder.add(Inventory {
                max_slots: crate::vat::VAT_SLOTS,
                ..Default::default()
            });
            if let Some(demand_w) = crate::vat::demand_w(kind) {
                builder.add(crate::power::PowerConsumer::idle(demand_w));
            }
        }
        FactoryKind::BeltRouter => {
            builder.add(crate::logistics::Router::default());
        }
        FactoryKind::AutoPlanter | FactoryKind::AutoHarvester => {
            builder.add(crate::machines::Machine::default());
            builder.add(crate::power::PowerConsumer::idle(
                crate::machines::demand_w(kind),
            ));
            builder.add(Inventory {
                max_slots: crate::machines::MACHINE_SLOTS,
                ..Default::default()
            });
        }
        #[cfg(any(test, feature = "test-support"))]
        FactoryKind::TestConsumer => {
            builder.add(crate::power::PowerConsumer::new(TEST_CONSUMER_W));
        }
        FactoryKind::Fence | FactoryKind::Gate => {}
        FactoryKind::Spawner => {
            builder.add(crate::spawner::Spawner::default());
            builder.add(crate::power::PowerConsumer::idle(crate::spawner::SPAWNER_W));
        }
        #[cfg(any(test, feature = "test-support"))]
        FactoryKind::TestArrow => {}
    }
    let entity = world.spawn(builder.build());
    res.factory_index.insert(pos, entity);
    if crate::power::is_grid_block(kind) {
        res.power.dirty = true;
    }
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
    let mut contents = world
        .get::<&Inventory>(entity)
        .map(|inv| inv.slots.clone())
        .unwrap_or_default();
    if let Some(item) = world.get::<&BeltSlot>(entity).ok().and_then(|b| b.item) {
        contents.push(crate::components::ItemStack::new(item, 1));
    }
    if let Ok(furnace) = world.get::<&crate::furnace::Furnace>(entity) {
        contents.extend(furnace.slots().into_iter().flatten());
    }
    if let Ok(spawner) = world.get::<&crate::spawner::Spawner>(entity) {
        contents.extend(spawner.contents());
    }
    if let Some(item) = world
        .get::<&crate::logistics::Router>(entity)
        .ok()
        .and_then(|r| r.item)
    {
        contents.push(crate::components::ItemStack::new(item, 1));
    }
    let _ = world.despawn(entity);
    if crate::power::is_grid_block(block.kind) {
        res.power.dirty = true;
    }
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
    /// A vertical conveyor switched between up and down; this is its new kind
    Flipped(FactoryKind),
    NotDirectional(FactoryKind),
    NoBlock,
}

impl FactoryKind {
    /// Which way a vertical conveyor carries items: +1 up, -1 down, `None` for other blocks.
    pub fn lift(self) -> Option<i64> {
        match self {
            FactoryKind::VerticalConveyorUp => Some(1),
            FactoryKind::VerticalConveyorDown => Some(-1),
            _ => None,
        }
    }
}

/// Rotate the factory block at `pos` one step clockwise, or flip a vertical conveyor
/// between up and down.
pub fn rotate(world: &mut World, res: &Resources, pos: Position) -> RotateResult {
    let Some(&entity) = res.factory_index.get(&pos) else {
        return RotateResult::NoBlock;
    };
    let Ok(mut block) = world.get::<&mut FactoryBlock>(entity) else {
        return RotateResult::NoBlock;
    };
    if block.kind.lift().is_some() {
        block.kind = match block.kind {
            FactoryKind::VerticalConveyorUp => FactoryKind::VerticalConveyorDown,
            _ => FactoryKind::VerticalConveyorUp,
        };
        let (kind, facing) = (block.kind, block.facing);
        drop(block);
        let _ = world.insert_one(entity, kind.sprite(facing));
        return RotateResult::Flipped(kind);
    }
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

/// Move every item from `from`'s inventory to `to`'s, stack by stack, stopping at the
/// first stack `to` has no room for. Returns how many items moved.
pub fn move_all_items(world: &mut World, from: Entity, to: Entity) -> u32 {
    let mut moved = 0;
    loop {
        let Some(stack) = world
            .get::<&Inventory>(from)
            .ok()
            .and_then(|inv| inv.slots.first().copied())
        else {
            return moved;
        };
        {
            let Ok(mut dest) = world.get::<&mut Inventory>(to) else {
                return moved;
            };
            let has_stack = dest.slots.iter().any(|s| s.kind == stack.kind);
            if !has_stack && dest.slots.len() >= dest.max_slots {
                return moved;
            }
            dest.add(stack.kind, stack.qty);
        }
        world
            .get::<&mut Inventory>(from)
            .expect("read above")
            .slots
            .remove(0);
        moved += stack.qty;
    }
}

/// Run every factory step that fits in the ticks elapsed since the last call. Each step runs
/// the factory systems in a fixed order (growth, machine and furnace demand, power, machines, furnaces,
/// extractors, routers, belts, then carrying the player along belts)
/// over everything, wherever it is.
pub fn run_due_steps(world: &mut World, res: &mut Resources) {
    let clock = &mut res.factory_clock;
    let elapsed = res.time.tick.saturating_sub(clock.last_tick) + clock.carry;
    // The tick of the first step due: the start of the elapsed span plus one step
    let first_step_tick = res.time.tick.saturating_sub(elapsed) + FACTORY_STEP_TICKS;
    clock.last_tick = res.time.tick;
    clock.carry = elapsed % FACTORY_STEP_TICKS;
    let steps = elapsed / FACTORY_STEP_TICKS;
    for i in 0..steps {
        let step_tick = first_step_tick + i * FACTORY_STEP_TICKS;
        crate::plants::grow_saplings(world, res, step_tick);
        crate::machines::update_demand(world, res);
        crate::furnace::update_demand(world);
        crate::vat::update_demand(world);
        crate::spawner::update_demand(world);
        crate::power::step_power(world, res);
        crate::machines::step_machines(world, res);
        crate::furnace::step_furnaces(world, res);
        crate::logistics::step_extractors(world, res);
        crate::logistics::step_routers(world, res);
        step_belts(world, res);
        crate::logistics::carry_player(world, res);
        crate::spawner::step_spawners(world, res, step_tick);
        res.factory_clock.steps += 1;
    }
}

/// What the test-only consumer draws, in watts.
#[cfg(any(test, feature = "test-support"))]
pub const TEST_CONSUMER_W: u64 = 45_000;

/// Steps an item spends on a belt before it moves on (under half a factory second).
pub const BELT_STEPS_PER_TILE: u8 = 2;
/// Steps a player rides on a belt before it carries them a tile: every step, so belts stay
/// quick to walk on (about 1.7 tiles per game second).
pub const BELT_RIDE_STEPS_PER_TILE: u8 = 1;

/// What a belt carries: at most one item, and how many steps it has been on this belt.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BeltSlot {
    pub item: Option<ItemKind>,
    pub progress: u8,
}

/// Whether a block of this kind ever takes `item` from the player, however full it is:
/// generators take fuel, planters acorns, furnaces fuel or something smeltable, and
/// harvesters and extractors nothing. Chests, belts and routers take anything.
pub fn takes(kind: FactoryKind, item: ItemKind) -> bool {
    match kind {
        FactoryKind::BurnerGenerator => crate::power::fuel_energy(item).is_some(),
        FactoryKind::AutoPlanter => item == ItemKind::Acorn,
        FactoryKind::CrudeFurnace => {
            crate::power::fuel_energy(item).is_some()
                || crate::furnace::smelting_recipe(item).is_some()
        }
        FactoryKind::ElectricFurnace => crate::furnace::smelting_recipe(item).is_some(),
        FactoryKind::CrudeVat | FactoryKind::StandardVat | FactoryKind::AdvancedVat => {
            crate::vat::takes(kind, item)
        }
        FactoryKind::AutoHarvester | FactoryKind::Extractor | FactoryKind::FilterExtractor => false,
        FactoryKind::Spawner => {
            item == ItemKind::MeatSlurry || crate::spawner::embryo_kind(item).is_some()
        }
        FactoryKind::Fence | FactoryKind::Gate => false,
        _ => true,
    }
}

/// What a machine takes, for telling the player their item list is filtered; `None` for
/// blocks that take anything (chests).
pub fn takes_label(kind: FactoryKind) -> Option<&'static str> {
    match kind {
        FactoryKind::BurnerGenerator => Some("fuel"),
        FactoryKind::AutoPlanter => Some("acorns"),
        FactoryKind::CrudeFurnace => Some("fuel/smeltables"),
        FactoryKind::ElectricFurnace => Some("smeltables"),
        FactoryKind::CrudeVat => Some("crude body parts"),
        FactoryKind::StandardVat => Some("crude/standard body parts"),
        FactoryKind::AdvancedVat => Some("body parts"),
        FactoryKind::AutoHarvester | FactoryKind::Extractor | FactoryKind::FilterExtractor => {
            Some("nothing")
        }
        _ => None,
    }
}

/// Whether `target` (a factory block) would take `item` handed to it with no direction (by
/// the player's transfer window). See `accepts_from`.
pub fn accepts(world: &World, target: Entity, item: ItemKind) -> bool {
    accepts_from(world, target, item, None)
}

/// Whether `target` (a factory block) would take `item` right now, arriving while moving
/// along `heading` (a belt or machine pushing it; `None` for the player): an empty belt, a
/// chest with room for it, a generator's fuel slots, a machine's input, or the furnace slot
/// for the side it arrives at.
pub fn accepts_from(
    world: &World,
    target: Entity,
    item: ItemKind,
    heading: Option<Facing>,
) -> bool {
    if let Ok(belt) = world.get::<&BeltSlot>(target) {
        return belt.item.is_none();
    }
    let Ok(block) = world.get::<&FactoryBlock>(target).map(|b| *b) else {
        return false;
    };
    match block.kind {
        FactoryKind::CrudeFurnace | FactoryKind::ElectricFurnace => {
            return crate::furnace::accepts(world, target, block, item, heading)
        }
        FactoryKind::BeltRouter => {
            return crate::logistics::router_accepts(world, target, block.facing, heading)
        }
        FactoryKind::Spawner => {
            return world
                .get::<&crate::spawner::Spawner>(target)
                .is_ok_and(|s| s.accepts(item))
        }
        // Extractors only pass items through
        FactoryKind::Extractor | FactoryKind::FilterExtractor => return false,
        FactoryKind::BurnerGenerator if crate::power::fuel_energy(item).is_none() => return false,
        FactoryKind::AutoPlanter | FactoryKind::AutoHarvester => {
            return crate::machines::accepts_input(world, target, block.kind, item)
        }
        FactoryKind::CrudeVat | FactoryKind::StandardVat | FactoryKind::AdvancedVat => {
            return crate::vat::accepts(world, target, item)
        }
        _ => {}
    }
    world.get::<&Inventory>(target).is_ok_and(|inv| {
        inv.slots.iter().any(|s| s.kind == item) || inv.slots.len() < inv.max_slots
    })
}

/// Give `item` to `target` if it accepts it, with no direction. Returns whether it did.
pub fn insert(world: &mut World, target: Entity, item: ItemKind) -> bool {
    insert_from(world, target, item, None)
}

/// Give `item` to `target`, arriving while moving along `heading`, if it accepts it.
/// Returns whether it did.
pub fn insert_from(
    world: &mut World,
    target: Entity,
    item: ItemKind,
    heading: Option<Facing>,
) -> bool {
    if !accepts_from(world, target, item, heading) {
        return false;
    }
    let block = world.get::<&FactoryBlock>(target).map(|b| *b).ok();
    if let (Ok(mut furnace), Some(block)) =
        (world.get::<&mut crate::furnace::Furnace>(target), block)
    {
        furnace.insert(block, item, heading);
        return true;
    }
    if let Ok(mut router) = world.get::<&mut crate::logistics::Router>(target) {
        router.item = Some(item);
        return true;
    }
    if let Ok(mut spawner) = world.get::<&mut crate::spawner::Spawner>(target) {
        spawner.insert(item);
        return true;
    }
    if let Ok(mut belt) = world.get::<&mut BeltSlot>(target) {
        *belt = BeltSlot {
            item: Some(item),
            progress: 0,
        };
        return true;
    }
    if let Ok(mut inv) = world.get::<&mut Inventory>(target) {
        inv.add(item, 1);
        return true;
    }
    false
}

/// Move one item from stack `slot` of `from`'s inventory into the factory block `to`, using
/// the same handoff as belts.
pub fn move_one_item_to_block(
    world: &mut World,
    from: Entity,
    to: Entity,
    slot: usize,
) -> Result<ItemKind, TransferError> {
    let kind = world
        .get::<&Inventory>(from)
        .ok()
        .and_then(|inv| inv.slots.get(slot).filter(|s| s.qty > 0).map(|s| s.kind))
        .ok_or(TransferError::Empty)?;
    if !insert(world, to, kind) {
        return Err(TransferError::Full);
    }
    let mut inv = world.get::<&mut Inventory>(from).expect("checked above");
    inv.slots[slot].qty -= 1;
    if inv.slots[slot].qty == 0 {
        inv.slots.remove(slot);
    }
    Ok(kind)
}

/// Belts in processing order: each belt after the belt it feeds, so a full line shifts by
/// one tile in a single step. Ties and loops go by position.
fn belt_order(world: &World, res: &Resources) -> Vec<(Entity, Position, FactoryBlock)> {
    let mut belts: Vec<(Entity, Position, FactoryBlock)> = world
        .query::<(&Position, &FactoryBlock)>()
        .with::<&BeltSlot>()
        .iter()
        .map(|(e, (p, b))| (e, *p, *b))
        .collect();
    belts.sort_by_key(|(_, p, _)| (p.z, p.y, p.x));
    let by_entity: HashMap<Entity, (Position, FactoryBlock)> =
        belts.iter().map(|(e, p, b)| (*e, (*p, *b))).collect();
    let next = |pos: Position, block: FactoryBlock| {
        res.factory_index
            .get(&belt_target(pos, block).0)
            .copied()
            .filter(|e| by_entity.contains_key(e))
    };
    // 0 = not seen, 1 = on the current path (a loop), 2 = placed in the order
    let mut state: HashMap<Entity, u8> = HashMap::new();
    let mut order = Vec::with_capacity(belts.len());
    for &(start, _, _) in &belts {
        let mut path = Vec::new();
        let mut current = Some(start);
        while let Some(e) = current {
            if state.get(&e).copied().unwrap_or(0) != 0 {
                break;
            }
            state.insert(e, 1);
            path.push(e);
            let (pos, block) = by_entity[&e];
            current = next(pos, block);
        }
        for e in path.into_iter().rev() {
            state.insert(e, 2);
            let (pos, block) = by_entity[&e];
            order.push((e, pos, block));
        }
    }
    order
}

/// Where a belt-like block hands its item, and the heading it arrives with: a belt passes
/// it on along its facing, a vertical conveyor to the layer above or below, arriving with
/// no heading (as if put in by hand).
fn belt_target(pos: Position, block: FactoryBlock) -> (Position, Option<Facing>) {
    if let Some(dz) = block.kind.lift() {
        return (
            Position {
                z: pos.z + dz,
                ..pos
            },
            None,
        );
    }
    let (dx, dy) = block.facing.offset();
    let target = Position {
        x: pos.x + dx,
        y: pos.y + dy,
        z: pos.z,
    };
    (target, Some(block.facing))
}

/// One factory step for every belt: advance items, hand them on, then load dropped items.
fn step_belts(world: &mut World, res: &mut Resources) {
    let order = belt_order(world, res);
    for &(belt, pos, block) in &order {
        let Some(item) = ({
            let mut slot = world.get::<&mut BeltSlot>(belt).expect("belt has a slot");
            match slot.item {
                Some(item) => {
                    slot.progress = slot.progress.saturating_add(1);
                    (slot.progress >= BELT_STEPS_PER_TILE).then_some(item)
                }
                None => None,
            }
        }) else {
            continue;
        };
        let (target, heading) = belt_target(pos, block);
        let Some(&target) = res.factory_index.get(&target) else {
            continue;
        };
        if insert_from(world, target, item, heading) {
            *world.get::<&mut BeltSlot>(belt).expect("belt has a slot") = BeltSlot::default();
        }
    }
    // Empty belts pick up one item lying on their tile
    for &(belt, pos, _) in &order {
        if world.get::<&BeltSlot>(belt).is_ok_and(|s| s.item.is_some()) {
            continue;
        }
        let lying = world
            .query::<(&Position, &DroppedItem)>()
            .iter()
            .find(|(_, (p, d))| **p == pos && d.qty > 0)
            .map(|(e, (_, d))| (e, d.kind));
        let Some((drop, kind)) = lying else {
            continue;
        };
        let emptied = {
            let mut d = world.get::<&mut DroppedItem>(drop).expect("found above");
            d.qty -= 1;
            d.qty == 0
        };
        if emptied {
            let _ = world.despawn(drop);
        }
        *world.get::<&mut BeltSlot>(belt).expect("belt has a slot") = BeltSlot {
            item: Some(kind),
            progress: 0,
        };
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
    use crate::intent::PlayerAction;
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

    /// Fence in the ring of tiles from `(x0, y0)` to `(x1, y1)`, with a gate at `gate`.
    fn pen(game: &mut Game, (x0, y0): (i64, i64), (x1, y1): (i64, i64), gate: Option<Position>) {
        for x in x0..=x1 {
            for y in y0..=y1 {
                if x == x0 || x == x1 || y == y0 || y == y1 {
                    let kind = if Some(at(x, y)) == gate {
                        FactoryKind::Gate
                    } else {
                        FactoryKind::Fence
                    };
                    place_in(game, kind, at(x, y), Facing::North);
                }
            }
        }
    }

    fn wait(t: &mut crate::test_support::TestGame, turns: usize) {
        for _ in 0..turns {
            t.act(PlayerAction::Move {
                dx: 0,
                dy: 0,
                dz: 0,
            });
            t.tick();
        }
    }

    #[test]
    fn a_fenced_dog_cannot_reach_the_player_who_still_sees_it() {
        let mut world = arena();
        let dog = world.feral_dog(4, 0, 0);
        let mut t = world.build();
        pen(&mut t.game, (3, -1), (5, 1), None);
        wait(&mut t, 20);
        assert_eq!(t.pos(dog), Some(at(4, 0)), "the dog stays in its pen");
        assert!(!t.game.res.player_state.combat_active);
        assert!(crate::systems::fog_of_war::has_line_of_sight(
            &t.game.world,
            &t.game.res,
            0,
            0,
            4,
            0,
            0
        ));
        // A wall of rock, unlike a fence, hides it
        t.game
            .res
            .world_state
            .world
            .set_tile_cached(3, 0, 0, TileKind::Rock);
        t.game.res.factory_index.remove(&at(3, 0));
        assert!(!crate::systems::fog_of_war::has_line_of_sight(
            &t.game.world,
            &t.game.res,
            0,
            0,
            4,
            0,
            0
        ));
    }

    #[test]
    fn the_player_walks_through_a_gate_and_the_player_only() {
        // Into an empty pen through its gate
        let mut t = arena().player_at(1, 0, 0).build();
        pen(&mut t.game, (2, -2), (6, 2), Some(at(2, 0)));
        let east = PlayerAction::Move {
            dx: 1,
            dy: 0,
            dz: 0,
        };
        t.act(east.clone());
        t.tick();
        assert_eq!(t.player_pos(), at(2, 0), "onto the gate");
        t.act(east);
        t.tick();
        assert_eq!(t.player_pos(), at(3, 0), "into the pen");
        // But not through a fence
        t.act(PlayerAction::Move {
            dx: 0,
            dy: -1,
            dz: 0,
        });
        t.tick();
        t.act(PlayerAction::Move {
            dx: 0,
            dy: -1,
            dz: 0,
        });
        t.tick();
        assert_eq!(t.player_pos(), at(3, -1), "the fence stops the player");

        // A dog in a gated pen can't leave by the gate
        let mut world = arena().player_at(-3, 0, 0);
        let dog = world.feral_dog(4, 0, 0);
        let mut t = world.build();
        pen(&mut t.game, (2, -2), (6, 2), Some(at(2, 0)));
        wait(&mut t, 30);
        let pos = t.pos(dog).unwrap();
        assert!(
            (3..=5).contains(&pos.x) && (-1..=1).contains(&pos.y),
            "{pos:?}"
        );
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
    fn taking_all_moves_every_stack_that_fits() {
        let mut game = arena().inventory(&[]).build().game;
        let chest = place_in(&mut game, FactoryKind::Chest, at(1, 0), Facing::North);
        let p = player(&game);
        {
            let mut inv = game.world.get::<&mut Inventory>(chest).unwrap();
            inv.add(ItemKind::Log, 10);
            inv.add(ItemKind::Stone, 3);
            inv.add(ItemKind::Stick, 2);
        }
        game.world.get::<&mut Inventory>(p).unwrap().max_slots = 2;
        assert_eq!(
            move_all_items(&mut game.world, chest, p),
            13,
            "logs and stone"
        );
        let left = game.world.get::<&Inventory>(chest).unwrap().slots.clone();
        assert_eq!(
            (left.len(), left[0].kind),
            (1, ItemKind::Stick),
            "no room for sticks"
        );
        game.world.get::<&mut Inventory>(p).unwrap().max_slots = 3;
        assert_eq!(move_all_items(&mut game.world, chest, p), 2);
        assert_eq!(move_all_items(&mut game.world, chest, p), 0, "empty");
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

    // --- Belts -----------------------------------------------------------------------

    fn belt_item(game: &Game, pos: Position) -> Option<ItemKind> {
        let e = game.res.factory_index[&pos];
        game.world.get::<&BeltSlot>(e).unwrap().item
    }

    fn put_on_belt(game: &mut Game, pos: Position, item: ItemKind) {
        let e = game.res.factory_index[&pos];
        *game.world.get::<&mut BeltSlot>(e).unwrap() = BeltSlot {
            item: Some(item),
            progress: 0,
        };
    }

    fn belts(game: &mut Game, from: Position, len: i64, facing: Facing) {
        let (dx, dy) = facing.offset();
        for i in 0..len {
            let pos = Position {
                x: from.x + dx * i,
                y: from.y + dy * i,
                z: from.z,
            };
            game.res
                .world_state
                .world
                .set_tile_cached(pos.x, pos.y, pos.z, TileKind::Grass);
            place_in(game, FactoryKind::Belt, pos, facing);
        }
    }

    /// Run `n` factory steps (20 ticks each) as waiting actions.
    fn steps(game: &mut Game, n: u64) {
        game.res.player_state.intent =
            crate::intent::PlayerIntent::movement(0, 0, 0, n * FACTORY_STEP_TICKS);
        game.tick();
    }

    /// Items on belts, in chests and lying around.
    fn item_count(game: &Game, kind: ItemKind) -> u32 {
        let on_belts = game
            .world
            .query::<&BeltSlot>()
            .iter()
            .filter(|(_, b)| b.item == Some(kind))
            .count() as u32;
        let in_chests: u32 = game
            .world
            .query::<(&FactoryBlock, &Inventory)>()
            .iter()
            .flat_map(|(_, (_, inv))| inv.slots.clone())
            .filter(|s| s.kind == kind)
            .map(|s| s.qty)
            .sum();
        let lying: u32 = game
            .world
            .query::<&DroppedItem>()
            .iter()
            .filter(|(_, d)| d.kind == kind)
            .map(|(_, d)| d.qty)
            .sum();
        on_belts + in_chests + lying
    }

    fn belt_game() -> Game {
        TestWorld::new(5).player_at(0, 10, 0).build().game
    }

    #[test]
    fn handoff_into_belts_and_chests() {
        let mut game = belt_game();
        belts(&mut game, at(0, 0), 1, Facing::East);
        let chest = place_in(&mut game, FactoryKind::Chest, at(1, 0), Facing::North);
        let belt = game.res.factory_index[&at(0, 0)];
        assert!(accepts(&game.world, belt, ItemKind::Log));
        assert!(insert(&mut game.world, belt, ItemKind::Log));
        assert!(
            !accepts(&game.world, belt, ItemKind::Stick),
            "a belt holds one item"
        );
        assert!(insert(&mut game.world, chest, ItemKind::Log));
        game.world.get::<&mut Inventory>(chest).unwrap().max_slots = 1;
        assert!(
            accepts(&game.world, chest, ItemKind::Log),
            "stacks onto its log"
        );
        assert!(
            !accepts(&game.world, chest, ItemKind::Stick),
            "no room for a new stack"
        );
        let player = game.get_player_entity().unwrap();
        assert!(
            !accepts(&game.world, player, ItemKind::Log),
            "only factory blocks"
        );
    }

    #[test]
    fn items_move_one_tile_every_half_second() {
        let mut game = belt_game();
        belts(&mut game, at(0, 0), 3, Facing::East);
        put_on_belt(&mut game, at(0, 0), ItemKind::Log);
        steps(&mut game, 5); // one game second
        assert_eq!(belt_item(&game, at(2, 0)), Some(ItemKind::Log));
        assert_eq!(belt_item(&game, at(0, 0)), None);
    }

    #[test]
    fn corner_and_delivery_into_a_chest() {
        let mut game = belt_game();
        belts(&mut game, at(0, 0), 2, Facing::East); // (0,0) (1,0) east
        belts(&mut game, at(2, 0), 2, Facing::South); // (2,0) (2,1) south
        let chest = place_in(&mut game, FactoryKind::Chest, at(2, 2), Facing::North);
        put_on_belt(&mut game, at(0, 0), ItemKind::Log);
        steps(&mut game, 8); // four tiles
        let inv = game.world.get::<&Inventory>(chest).unwrap().slots.clone();
        assert_eq!(inv.len(), 1);
        assert_eq!((inv[0].kind, inv[0].qty), (ItemKind::Log, 1));
        assert_eq!(item_count(&game, ItemKind::Log), 1);
    }

    #[test]
    fn full_chest_backs_up_the_line_and_it_resumes() {
        let mut game = belt_game();
        belts(&mut game, at(0, 0), 3, Facing::East);
        let chest = place_in(&mut game, FactoryKind::Chest, at(3, 0), Facing::North);
        {
            let mut inv = game.world.get::<&mut Inventory>(chest).unwrap();
            inv.max_slots = 1;
            inv.add(ItemKind::Stone, 1);
        }
        for x in 0..3 {
            put_on_belt(&mut game, at(x, 0), ItemKind::Log);
        }
        steps(&mut game, 20);
        for x in 0..3 {
            assert_eq!(
                belt_item(&game, at(x, 0)),
                Some(ItemKind::Log),
                "waits at {x}"
            );
        }
        game.world
            .get::<&mut Inventory>(chest)
            .unwrap()
            .slots
            .clear();
        steps(&mut game, 20);
        assert_eq!(
            item_count(&game, ItemKind::Log),
            3,
            "nothing lost or duplicated"
        );
        let inv = game.world.get::<&Inventory>(chest).unwrap().slots.clone();
        assert_eq!((inv[0].kind, inv[0].qty), (ItemKind::Log, 3));
    }

    #[test]
    fn two_lines_merge_without_losing_items() {
        let mut game = belt_game();
        belts(&mut game, at(0, 0), 2, Facing::East); // into (2,0)
        belts(&mut game, at(2, -2), 2, Facing::South); // into (2,0)
        belts(&mut game, at(2, 0), 2, Facing::East);
        let chest = place_in(&mut game, FactoryKind::Chest, at(4, 0), Facing::North);
        for pos in [at(0, 0), at(1, 0), at(2, -2), at(2, -1)] {
            put_on_belt(&mut game, pos, ItemKind::Log);
        }
        steps(&mut game, 40);
        let inv = game.world.get::<&Inventory>(chest).unwrap().slots.clone();
        assert_eq!((inv[0].kind, inv[0].qty), (ItemKind::Log, 4));
    }

    #[test]
    fn a_loop_with_a_gap_keeps_turning() {
        let mut game = belt_game();
        // A 2x2 loop: (0,0) E -> (1,0) S -> (1,1) W -> (0,1) N -> (0,0)
        for (pos, facing) in [
            (at(0, 0), Facing::East),
            (at(1, 0), Facing::South),
            (at(1, 1), Facing::West),
            (at(0, 1), Facing::North),
        ] {
            belts(&mut game, pos, 1, facing);
        }
        put_on_belt(&mut game, at(0, 0), ItemKind::Log);
        put_on_belt(&mut game, at(1, 0), ItemKind::Stick);
        let mut seen = std::collections::BTreeSet::new();
        for _ in 0..8 {
            steps(&mut game, 2);
            assert_eq!(item_count(&game, ItemKind::Log), 1);
            assert_eq!(item_count(&game, ItemKind::Stick), 1);
            for pos in [at(0, 0), at(1, 0), at(1, 1), at(0, 1)] {
                if belt_item(&game, pos) == Some(ItemKind::Log) {
                    seen.insert((pos.x, pos.y));
                }
            }
        }
        assert_eq!(seen.len(), 4, "the log visits every belt");
    }

    #[test]
    fn empty_belts_pick_up_dropped_items() {
        let mut game = belt_game();
        belts(&mut game, at(0, 0), 2, Facing::East);
        game.world.spawn((
            at(0, 0),
            DroppedItem {
                kind: ItemKind::Log,
                qty: 2,
            },
            SpriteRef::new("items", "log"),
        ));
        steps(&mut game, 1);
        assert_eq!(belt_item(&game, at(0, 0)), Some(ItemKind::Log));
        steps(&mut game, 2);
        assert_eq!(belt_item(&game, at(1, 0)), Some(ItemKind::Log));
        assert_eq!(
            belt_item(&game, at(0, 0)),
            Some(ItemKind::Log),
            "second log loaded"
        );
        assert_eq!(item_count(&game, ItemKind::Log), 2);
    }

    #[test]
    fn belt_lines_deliver_the_same_across_chunk_edges() {
        use crate::world::CHUNK_SIZE;
        let steps_to_deliver = |start_x: i64| {
            let mut game = belt_game();
            belts(&mut game, at(start_x, 0), 5, Facing::East);
            let end = at(start_x + 5, 0);
            game.res
                .world_state
                .world
                .set_tile_cached(end.x, 0, 0, TileKind::Grass);
            let chest = place_in(&mut game, FactoryKind::Chest, end, Facing::North);
            put_on_belt(&mut game, at(start_x, 0), ItemKind::Log);
            (1..100)
                .find(|_| {
                    steps(&mut game, 1);
                    !game
                        .world
                        .get::<&Inventory>(chest)
                        .unwrap()
                        .slots
                        .is_empty()
                })
                .expect("delivered")
        };
        let inside = steps_to_deliver(2);
        assert_eq!(
            steps_to_deliver(CHUNK_SIZE - 2),
            inside,
            "x = 62..66, chest at 67"
        );
        assert_eq!(steps_to_deliver(-3), inside, "x = -3..1, across 0");
    }

    #[test]
    fn breaking_a_belt_drops_what_it_carried() {
        let mut game = belt_game();
        belts(&mut game, at(0, 0), 1, Facing::East);
        put_on_belt(&mut game, at(0, 0), ItemKind::Log);
        let p = player(&game);
        assert_eq!(
            break_block(&mut game.world, &mut game.res, at(0, 0), p),
            Some(FactoryKind::Belt)
        );
        assert_eq!(item_count(&game, ItemKind::Log), 1);
    }

    // --- Vertical conveyors ----------------------------------------------------------

    fn at_z(x: i64, y: i64, z: i64) -> Position {
        Position { x, y, z }
    }

    /// Clear the tile, then place a block on it.
    fn place_open(game: &mut Game, kind: FactoryKind, pos: Position, facing: Facing) -> Entity {
        game.res
            .world_state
            .world
            .set_tile_cached(pos.x, pos.y, pos.z, TileKind::Air);
        place_in(game, kind, pos, facing)
    }

    fn chest_logs(game: &Game, chest: Entity) -> u32 {
        game.world
            .get::<&Inventory>(chest)
            .unwrap()
            .slots
            .iter()
            .filter(|s| s.kind == ItemKind::Log)
            .map(|s| s.qty)
            .sum()
    }

    #[test]
    fn a_belt_lifts_a_log_into_a_chest_on_the_layer_above() {
        let mut game = belt_game();
        belts(&mut game, at(0, 0), 1, Facing::East);
        place_open(
            &mut game,
            FactoryKind::VerticalConveyorUp,
            at(1, 0),
            Facing::North,
        );
        let chest = place_open(&mut game, FactoryKind::Chest, at_z(1, 0, 1), Facing::North);
        put_on_belt(&mut game, at(0, 0), ItemKind::Log);
        steps(&mut game, 4);
        assert_eq!(chest_logs(&game, chest), 1);
        assert_eq!(
            item_count(&game, ItemKind::Log),
            1,
            "nothing lost or copied"
        );
    }

    #[test]
    fn stacked_conveyors_climb_several_layers() {
        let mut game = belt_game();
        for z in 0..3 {
            place_open(
                &mut game,
                FactoryKind::VerticalConveyorUp,
                at_z(0, 0, z),
                Facing::North,
            );
        }
        let chest = place_open(&mut game, FactoryKind::Chest, at_z(0, 0, 3), Facing::North);
        for _ in 0..3 {
            put_on_belt(&mut game, at(0, 0), ItemKind::Log);
            steps(&mut game, 2);
        }
        steps(&mut game, 6);
        assert_eq!(chest_logs(&game, chest), 3);
        assert_eq!(item_count(&game, ItemKind::Log), 3);
    }

    #[test]
    fn a_down_conveyor_lowers_onto_a_belt() {
        let mut game = belt_game();
        place_open(
            &mut game,
            FactoryKind::VerticalConveyorDown,
            at_z(0, 0, 1),
            Facing::North,
        );
        belts(&mut game, at(0, 0), 1, Facing::East);
        let chest = place_open(&mut game, FactoryKind::Chest, at(1, 0), Facing::North);
        put_on_belt(&mut game, at_z(0, 0, 1), ItemKind::Log);
        steps(&mut game, 6);
        assert_eq!(chest_logs(&game, chest), 1);
    }

    #[test]
    fn rock_above_holds_the_item() {
        let mut game = belt_game();
        place_open(
            &mut game,
            FactoryKind::VerticalConveyorUp,
            at(0, 0),
            Facing::North,
        );
        game.res
            .world_state
            .world
            .set_tile_cached(0, 0, 1, TileKind::Rock);
        put_on_belt(&mut game, at(0, 0), ItemKind::Log);
        steps(&mut game, 10);
        assert_eq!(belt_item(&game, at(0, 0)), Some(ItemKind::Log));
        assert_eq!(item_count(&game, ItemKind::Log), 1);
    }

    #[test]
    fn rotating_flips_up_and_down_and_breaking_gives_one_item() {
        let mut game = belt_game();
        let lift = place_open(
            &mut game,
            FactoryKind::VerticalConveyorUp,
            at(0, 0),
            Facing::North,
        );
        assert_eq!(
            rotate(&mut game.world, &game.res, at(0, 0)),
            RotateResult::Flipped(FactoryKind::VerticalConveyorDown)
        );
        assert_eq!(
            game.world.get::<&SpriteRef>(lift).unwrap().name,
            "vertical_conveyor_down"
        );
        assert_eq!(
            rotate(&mut game.world, &game.res, at(0, 0)),
            RotateResult::Flipped(FactoryKind::VerticalConveyorUp)
        );
        let p = player(&game);
        assert_eq!(
            break_block(&mut game.world, &mut game.res, at(0, 0), p),
            Some(FactoryKind::VerticalConveyorUp)
        );
        let inv = game.world.get::<&Inventory>(p).unwrap();
        assert!(inv
            .slots
            .iter()
            .any(|s| s.kind == ItemKind::VerticalConveyor && s.qty == 1));
    }

    #[test]
    fn standing_on_a_vertical_conveyor_does_not_carry_the_player() {
        let mut game = belt_game();
        place_open(
            &mut game,
            FactoryKind::VerticalConveyorUp,
            at(0, 0),
            Facing::East,
        );
        game.res
            .world_state
            .world
            .set_tile_cached(1, 0, 0, TileKind::Grass);
        let p = player(&game);
        *game.world.get::<&mut Position>(p).unwrap() = at(0, 0);
        steps(&mut game, 6);
        assert_eq!(game.get_player_position(), Some(at(0, 0)));
    }
}

//! Smelting: the shared smelting table and the crude furnace, an unpowered block that burns
//! fuel to turn wood into biochar.
//!
//! A furnace has three single-stack slots. Items arriving from its back go to the input,
//! from either side to the fuel, and finished items leave through its front. Fuel energy is
//! whole joules, drawn only while smelting; a partly burned item's energy stays in
//! `buffer_j`.

use hecs::{Entity, World};
use serde::{Deserialize, Serialize};

use crate::components::{ItemKind, ItemStack, Position, SpriteRef};
use crate::factory::{Facing, FactoryBlock};
use crate::power::{fuel_energy, STEPS_PER_SECOND};
use crate::resources::Resources;

/// Items a furnace slot holds.
pub const FURNACE_SLOT_MAX: u32 = 32;
/// Fuel energy a crude furnace burns while smelting, in watts.
pub const CRUDE_FURNACE_W: u64 = 90_000;

/// One row of the smelting table: `input_count` of `input` become `output_count` of
/// `output` after `seconds` game seconds of smelting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SmeltingRecipe {
    pub input: ItemKind,
    pub input_count: u32,
    pub output: ItemKind,
    pub output_count: u32,
    pub seconds: u64,
}

impl SmeltingRecipe {
    /// Factory steps one smelt takes at full speed.
    pub fn steps(&self) -> u64 {
        self.seconds * STEPS_PER_SECOND
    }
}

const fn biochar(input: ItemKind, input_count: u32, seconds: u64) -> SmeltingRecipe {
    SmeltingRecipe {
        input,
        input_count,
        output: ItemKind::Biochar,
        output_count: 1,
        seconds,
    }
}

/// The smelting table, shared by every furnace.
pub const SMELTING: &[SmeltingRecipe] = &[
    biochar(ItemKind::Log, 1, 4),
    biochar(ItemKind::WoodenPlank, 2, 3),
    biochar(ItemKind::Stick, 4, 3),
    biochar(ItemKind::Acorn, 4, 5),
    biochar(ItemKind::WoodenShavings, 8, 2),
];

pub fn smelting_recipe(input: ItemKind) -> Option<&'static SmeltingRecipe> {
    SMELTING.iter().find(|r| r.input == input)
}

/// Which furnace slot a stack is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    Fuel,
    Input,
    Output,
}

impl Slot {
    pub const ALL: [Slot; 3] = [Slot::Fuel, Slot::Input, Slot::Output];

    pub fn name(self) -> &'static str {
        match self {
            Slot::Fuel => "Fuel",
            Slot::Input => "Input",
            Slot::Output => "Output",
        }
    }
}

/// Why a furnace slot refuses an item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    NotFuel,
    NotSmeltable,
    /// Only the furnace puts items in its output
    Output,
    /// The slot holds another item or is full
    Full,
}

impl Refusal {
    pub fn message(self, item: ItemKind) -> String {
        let name = crate::components::itemkind_name(item);
        match self {
            Refusal::NotFuel => format!("{name} isn't fuel"),
            Refusal::NotSmeltable => format!("{name} can't be smelted"),
            Refusal::Output => "Only finished items go in the output".to_string(),
            Refusal::Full => "That slot is full".to_string(),
        }
    }
}

/// A furnace's slots and smelting state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Furnace {
    pub fuel: Option<ItemStack>,
    pub input: Option<ItemStack>,
    pub output: Option<ItemStack>,
    /// Factory steps smelted so far of the current smelt
    pub progress: u64,
    /// Energy left in the fuel item being burned, in joules
    pub buffer_j: u64,
    /// Whether it smelted last step (drawn lit)
    pub lit: bool,
}

fn fits(slot: &Option<ItemStack>, item: ItemKind, qty: u32) -> bool {
    match slot {
        None => qty <= FURNACE_SLOT_MAX,
        Some(s) => s.kind == item && s.qty + qty <= FURNACE_SLOT_MAX,
    }
}

fn add(slot: &mut Option<ItemStack>, item: ItemKind, qty: u32) {
    match slot {
        Some(s) => s.qty += qty,
        None => *slot = Some(ItemStack::new(item, qty)),
    }
}

/// Take `qty` from a slot, emptying it at zero.
fn remove(slot: &mut Option<ItemStack>, qty: u32) {
    if let Some(s) = slot {
        s.qty = s.qty.saturating_sub(qty);
        if s.qty == 0 {
            *slot = None;
        }
    }
}

impl Furnace {
    pub fn slots(&self) -> [Option<ItemStack>; 3] {
        [self.fuel, self.input, self.output]
    }

    pub fn slot(&self, slot: Slot) -> Option<ItemStack> {
        match slot {
            Slot::Fuel => self.fuel,
            Slot::Input => self.input,
            Slot::Output => self.output,
        }
    }

    fn slot_mut(&mut self, slot: Slot) -> &mut Option<ItemStack> {
        match slot {
            Slot::Fuel => &mut self.fuel,
            Slot::Input => &mut self.input,
            Slot::Output => &mut self.output,
        }
    }

    /// Whether `slot` takes one `item` right now, and why not.
    pub fn check(&self, slot: Slot, item: ItemKind) -> Result<(), Refusal> {
        match slot {
            Slot::Fuel if fuel_energy(item).is_none() => return Err(Refusal::NotFuel),
            Slot::Input if smelting_recipe(item).is_none() => return Err(Refusal::NotSmeltable),
            Slot::Output => return Err(Refusal::Output),
            _ => {}
        }
        if fits(&self.slot(slot), item, 1) {
            Ok(())
        } else {
            Err(Refusal::Full)
        }
    }

    /// Put one `item` into `slot`, if it takes it.
    pub fn put(&mut self, slot: Slot, item: ItemKind) -> Result<(), Refusal> {
        self.check(slot, item)?;
        add(self.slot_mut(slot), item, 1);
        Ok(())
    }

    /// Take one item out of `slot`.
    pub fn take(&mut self, slot: Slot) -> Option<ItemKind> {
        let kind = self.slot(slot)?.kind;
        remove(self.slot_mut(slot), 1);
        Some(kind)
    }

    /// The slot an item moving along `heading` arrives at, for a furnace facing `facing`:
    /// the input from behind, the fuel from either side, nothing from the front or with no
    /// direction.
    pub fn slot_for(facing: Facing, heading: Option<Facing>) -> Option<Slot> {
        let heading = heading?;
        if heading == facing {
            Some(Slot::Input)
        } else if heading == facing.opposite() {
            None
        } else {
            Some(Slot::Fuel)
        }
    }

    /// Insert one item arriving along `heading`; the caller checked `accepts`.
    pub fn insert(&mut self, facing: Facing, item: ItemKind, heading: Option<Facing>) {
        if let Some(slot) = Self::slot_for(facing, heading) {
            let _ = self.put(slot, item);
        }
    }

    /// The recipe it can smelt now: the input holds a full batch and the output has room.
    pub fn ready_recipe(&self) -> Option<&'static SmeltingRecipe> {
        let input = self.input?;
        let recipe = smelting_recipe(input.kind)?;
        (input.qty >= recipe.input_count && fits(&self.output, recipe.output, recipe.output_count))
            .then_some(recipe)
    }

    /// Fuel energy left: the buffer plus every fuel item.
    pub fn energy_left_j(&self) -> u64 {
        let items = self
            .fuel
            .map_or(0, |s| fuel_energy(s.kind).unwrap_or(0) * s.qty as u64);
        self.buffer_j + items
    }

    /// One factory step: smelt if there's a full batch, room for the result and fuel for
    /// the step. Without fuel it pauses, keeping its progress; with nothing to smelt it
    /// resets.
    pub fn step(&mut self, step_j: u64) {
        let Some(recipe) = self.ready_recipe() else {
            self.progress = 0;
            self.lit = false;
            return;
        };
        if self.buffer_j < step_j {
            if let Some(energy) = self.fuel.and_then(|s| fuel_energy(s.kind)) {
                self.buffer_j += energy;
                remove(&mut self.fuel, 1);
            }
        }
        if self.buffer_j < step_j {
            self.lit = false;
            return;
        }
        self.buffer_j -= step_j;
        self.lit = true;
        self.progress += 1;
        if self.progress >= recipe.steps() {
            self.progress = 0;
            remove(&mut self.input, recipe.input_count);
            add(&mut self.output, recipe.output, recipe.output_count);
        }
    }
}

/// Why moving an item between the player and a furnace slot failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferError {
    /// Nothing to move
    Empty,
    /// The furnace slot refuses it
    Refused(Refusal),
    /// The player's inventory has no room
    InventoryFull,
}

/// Move one item from stack `player_slot` of `player`'s inventory into `slot` of `furnace`.
pub fn store(
    world: &mut World,
    player: Entity,
    furnace: Entity,
    player_slot: usize,
    slot: Slot,
) -> Result<ItemKind, TransferError> {
    let item = world
        .get::<&crate::components::Inventory>(player)
        .ok()
        .and_then(|inv| {
            inv.slots
                .get(player_slot)
                .filter(|s| s.qty > 0)
                .map(|s| s.kind)
        })
        .ok_or(TransferError::Empty)?;
    world
        .get::<&mut Furnace>(furnace)
        .map_err(|_| TransferError::Empty)?
        .put(slot, item)
        .map_err(TransferError::Refused)?;
    let mut inv = world
        .get::<&mut crate::components::Inventory>(player)
        .expect("checked above");
    inv.slots[player_slot].qty -= 1;
    if inv.slots[player_slot].qty == 0 {
        inv.slots.remove(player_slot);
    }
    Ok(item)
}

/// Move one item out of `slot` of `furnace` into `player`'s inventory.
pub fn take(
    world: &mut World,
    furnace: Entity,
    slot: Slot,
    player: Entity,
) -> Result<ItemKind, TransferError> {
    let item = world
        .get::<&Furnace>(furnace)
        .ok()
        .and_then(|f| f.slot(slot))
        .map(|s| s.kind)
        .ok_or(TransferError::Empty)?;
    {
        let mut inv = world
            .get::<&mut crate::components::Inventory>(player)
            .map_err(|_| TransferError::InventoryFull)?;
        let has_stack = inv.slots.iter().any(|s| s.kind == item);
        if !has_stack && inv.slots.len() >= inv.max_slots {
            return Err(TransferError::InventoryFull);
        }
        inv.add(item, 1);
    }
    world
        .get::<&mut Furnace>(furnace)
        .expect("checked above")
        .take(slot);
    Ok(item)
}

/// Whether the furnace `target` (facing `facing`) takes `item` arriving along `heading`.
pub fn accepts(
    world: &World,
    target: Entity,
    facing: Facing,
    item: ItemKind,
    heading: Option<Facing>,
) -> bool {
    let Some(slot) = Furnace::slot_for(facing, heading) else {
        return false;
    };
    world
        .get::<&Furnace>(target)
        .is_ok_and(|f| f.check(slot, item).is_ok())
}

/// The furnace sprite for its facing and whether it is lit.
pub fn sprite(facing: Facing, lit: bool) -> SpriteRef {
    let state = if lit { "lit" } else { "unlit" };
    SpriteRef::new(
        "factory",
        &format!("crude_furnace_{state}_{}", facing.name()),
    )
}

/// One factory step for every furnace, in position order: smelt, redraw when it lights up
/// or goes out, and push one finished item out its front.
pub fn step_furnaces(world: &mut World, res: &mut Resources) {
    let mut furnaces: Vec<(Entity, Position, Facing)> = world
        .query::<(&Position, &FactoryBlock)>()
        .with::<&Furnace>()
        .iter()
        .map(|(e, (p, b))| (e, *p, b.facing))
        .collect();
    if furnaces.is_empty() {
        return;
    }
    furnaces.sort_by_key(|(_, p, _)| (p.z, p.y, p.x));
    let step_j = CRUDE_FURNACE_W / STEPS_PER_SECOND;
    for (entity, pos, facing) in furnaces {
        let (was_lit, lit, output) = {
            let mut furnace = world.get::<&mut Furnace>(entity).expect("furnace");
            let was_lit = furnace.lit;
            furnace.step(step_j);
            (was_lit, furnace.lit, furnace.output.map(|s| s.kind))
        };
        if lit != was_lit {
            let _ = world.insert_one(entity, sprite(facing, lit));
        }
        let Some(item) = output else {
            continue;
        };
        let (dx, dy) = facing.offset();
        let front = Position {
            x: pos.x + dx,
            y: pos.y + dy,
            z: pos.z,
        };
        let Some(&target) = res.factory_index.get(&front) else {
            continue;
        };
        if crate::factory::insert_from(world, target, item, Some(facing)) {
            let mut furnace = world.get::<&mut Furnace>(entity).expect("furnace");
            remove(&mut furnace.output, 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{DroppedItem, Inventory};
    use crate::factory::{self, FactoryKind, FACTORY_STEP_TICKS, TICKS_PER_SECOND};
    use crate::intent::PlayerIntent;
    use crate::test_support::TestWorld;
    use crate::tiles::TileKind;
    use crate::Game;

    fn at(x: i64, y: i64) -> Position {
        Position { x, y, z: 0 }
    }

    fn game() -> Game {
        TestWorld::new(2).player_at(0, 20, 0).build().game
    }

    fn put(game: &mut Game, kind: FactoryKind, pos: Position, facing: Facing) -> Entity {
        game.res
            .world_state
            .world
            .set_tile_cached(pos.x, pos.y, pos.z, TileKind::Grass);
        factory::place(&mut game.world, &mut game.res, kind, pos, facing).expect("placed")
    }

    fn furnace(game: &Game, e: Entity) -> Furnace {
        *game.world.get::<&Furnace>(e).unwrap()
    }

    fn load(game: &mut Game, e: Entity, slot: Slot, item: ItemKind, qty: u32) {
        let mut f = game.world.get::<&mut Furnace>(e).unwrap();
        for _ in 0..qty {
            f.put(slot, item).unwrap();
        }
    }

    fn steps(game: &mut Game, n: u64) {
        game.res.player_state.intent = PlayerIntent::movement(0, 0, 0, n * FACTORY_STEP_TICKS);
        game.tick();
    }

    fn seconds(game: &mut Game, n: u64) {
        steps(game, n * TICKS_PER_SECOND / FACTORY_STEP_TICKS);
    }

    fn qty(slot: Option<ItemStack>) -> u32 {
        slot.map_or(0, |s| s.qty)
    }

    // --- 1.1 table and biochar ----------------------------------------------------------

    #[test]
    fn smelting_table_and_biochar_fuel() {
        let expected = [
            (ItemKind::Log, 1, 4),
            (ItemKind::WoodenPlank, 2, 3),
            (ItemKind::Stick, 4, 3),
            (ItemKind::Acorn, 4, 5),
            (ItemKind::WoodenShavings, 8, 2),
        ];
        for (input, count, seconds) in expected {
            let r = smelting_recipe(input).unwrap();
            assert_eq!(
                (r.input_count, r.output, r.output_count, r.seconds),
                (count, ItemKind::Biochar, 1, seconds)
            );
        }
        assert!(smelting_recipe(ItemKind::Stone).is_none());
        assert!(fuel_energy(ItemKind::Biochar) > fuel_energy(ItemKind::Log));
        let mut g = game();
        let generator = put(
            &mut g,
            FactoryKind::BurnerGenerator,
            at(0, 0),
            Facing::North,
        );
        assert!(factory::accepts(&g.world, generator, ItemKind::Biochar));
        let f = Furnace::default();
        assert_eq!(
            f.check(Slot::Input, ItemKind::Stone),
            Err(Refusal::NotSmeltable)
        );
        assert_eq!(
            Refusal::NotSmeltable.message(ItemKind::Stone),
            "Stone can't be smelted"
        );
        assert_eq!(f.check(Slot::Fuel, ItemKind::Stone), Err(Refusal::NotFuel));
        assert_eq!(
            f.check(Slot::Output, ItemKind::Biochar),
            Err(Refusal::Output)
        );
    }

    // --- 1.2 smelting and fuel ----------------------------------------------------------

    #[test]
    fn smelts_a_log_in_four_seconds() {
        let mut g = game();
        let f = put(&mut g, FactoryKind::CrudeFurnace, at(0, 0), Facing::East);
        load(&mut g, f, Slot::Input, ItemKind::Log, 1);
        load(&mut g, f, Slot::Fuel, ItemKind::Log, 1);
        steps(&mut g, 19);
        assert_eq!(qty(furnace(&g, f).output), 0, "not yet");
        assert!(furnace(&g, f).lit);
        steps(&mut g, 1);
        let state = furnace(&g, f);
        assert_eq!(state.input, None);
        assert_eq!(state.output, Some(ItemStack::new(ItemKind::Biochar, 1)));
        assert_eq!(
            state.buffer_j,
            2_000_000 - 20 * (CRUDE_FURNACE_W / STEPS_PER_SECOND)
        );
        steps(&mut g, 1);
        assert!(!furnace(&g, f).lit, "out of input");
    }

    #[test]
    fn batches_wait_for_a_full_batch() {
        let mut g = game();
        let f = put(&mut g, FactoryKind::CrudeFurnace, at(0, 0), Facing::East);
        load(&mut g, f, Slot::Input, ItemKind::WoodenPlank, 3);
        load(&mut g, f, Slot::Fuel, ItemKind::Log, 1);
        seconds(&mut g, 10);
        let state = furnace(&g, f);
        assert_eq!(qty(state.input), 1, "one plank waits for another");
        assert_eq!(qty(state.output), 1);
    }

    #[test]
    fn an_idle_furnace_keeps_its_fuel() {
        let mut g = game();
        let f = put(&mut g, FactoryKind::CrudeFurnace, at(0, 0), Facing::East);
        load(&mut g, f, Slot::Fuel, ItemKind::Log, 3);
        seconds(&mut g, 600);
        assert_eq!(furnace(&g, f).energy_left_j(), 3 * 2_000_000);
        assert_eq!(qty(furnace(&g, f).fuel), 3);
    }

    #[test]
    fn out_of_fuel_pauses_and_resumes_keeping_progress_and_energy() {
        let mut g = game();
        let f = put(&mut g, FactoryKind::CrudeFurnace, at(0, 0), Facing::East);
        load(&mut g, f, Slot::Input, ItemKind::Log, 1);
        // Shavings give 100 kJ: about 5 steps of 18 kJ
        load(&mut g, f, Slot::Fuel, ItemKind::WoodenShavings, 1);
        steps(&mut g, 10);
        let paused = furnace(&g, f);
        assert_eq!(paused.progress, 5);
        assert_eq!(paused.buffer_j, 100_000 - 5 * 18_000, "partial energy kept");
        assert!(!paused.lit);
        load(&mut g, f, Slot::Fuel, ItemKind::Log, 1);
        steps(&mut g, 15);
        assert_eq!(qty(furnace(&g, f).output), 1, "resumed from step 5");
    }

    // --- 1.3 side routing ---------------------------------------------------------------

    fn belt_with(game: &mut Game, pos: Position, facing: Facing, item: ItemKind, qty: u32) {
        put(game, FactoryKind::Belt, pos, facing);
        game.world.spawn((
            pos,
            DroppedItem { kind: item, qty },
            SpriteRef::new("items", "log"),
        ));
    }

    fn chest_count(game: &Game, chest: Entity, item: ItemKind) -> u32 {
        game.world
            .get::<&Inventory>(chest)
            .unwrap()
            .slots
            .iter()
            .filter(|s| s.kind == item)
            .map(|s| s.qty)
            .sum()
    }

    #[test]
    fn a_belt_fed_furnace_fuels_itself_from_the_side() {
        let run = |x0: i64| {
            let mut g = game();
            belt_with(&mut g, at(x0, 0), Facing::East, ItemKind::Log, 3);
            let f = put(
                &mut g,
                FactoryKind::CrudeFurnace,
                at(x0 + 1, 0),
                Facing::East,
            );
            belt_with(&mut g, at(x0 + 1, -1), Facing::South, ItemKind::Biochar, 1);
            put(&mut g, FactoryKind::Belt, at(x0 + 2, 0), Facing::East);
            let chest = put(&mut g, FactoryKind::Chest, at(x0 + 3, 0), Facing::North);
            seconds(&mut g, 20);
            assert_eq!(chest_count(&g, chest, ItemKind::Biochar), 3, "x0 = {x0}");
            assert_eq!(furnace(&g, f).input, None);
            assert_eq!(
                chest_count(&g, chest, ItemKind::Log),
                0,
                "logs went to the input"
            );
        };
        run(0);
        // Across a chunk edge: furnace at 63, belt out at 64
        run(crate::world::CHUNK_SIZE - 2);
    }

    #[test]
    fn wrong_side_items_stay_on_the_belt() {
        let mut g = game();
        let f = put(&mut g, FactoryKind::CrudeFurnace, at(1, 0), Facing::East);
        belt_with(&mut g, at(1, -1), Facing::South, ItemKind::Stone, 1);
        // In through the front: refused even though logs are fuel and smeltable
        belt_with(&mut g, at(2, 0), Facing::West, ItemKind::Log, 1);
        seconds(&mut g, 5);
        let state = furnace(&g, f);
        assert_eq!((state.fuel, state.input), (None, None));
        let side = g.res.factory_index[&at(1, -1)];
        assert_eq!(
            g.world.get::<&factory::BeltSlot>(side).unwrap().item,
            Some(ItemKind::Stone),
            "stone isn't fuel"
        );
        assert!(
            !factory::accepts(&g.world, f, ItemKind::Log),
            "no direction: refused"
        );
    }

    #[test]
    fn the_player_stores_and_takes_by_slot() {
        let mut g = TestWorld::new(2)
            .player_at(0, 20, 0)
            .inventory(&[(ItemKind::Log, 6), (ItemKind::Stone, 1)])
            .build()
            .game;
        let f = put(&mut g, FactoryKind::CrudeFurnace, at(0, 0), Facing::East);
        let p = g.get_player_entity().unwrap();
        for _ in 0..4 {
            assert_eq!(store(&mut g.world, p, f, 0, Slot::Input), Ok(ItemKind::Log));
        }
        for _ in 0..2 {
            assert_eq!(store(&mut g.world, p, f, 0, Slot::Fuel), Ok(ItemKind::Log));
        }
        // The emptied log stack is gone, so stone is now stack 0
        assert_eq!(
            store(&mut g.world, p, f, 0, Slot::Input),
            Err(TransferError::Refused(Refusal::NotSmeltable))
        );
        seconds(&mut g, 4);
        assert_eq!(
            take(&mut g.world, f, Slot::Output, p),
            Ok(ItemKind::Biochar)
        );
        assert_eq!(
            take(&mut g.world, f, Slot::Output, p),
            Err(TransferError::Empty)
        );
        assert_eq!(qty(furnace(&g, f).input), 3);
    }

    #[test]
    fn breaking_a_furnace_drops_its_slots() {
        let mut g = game();
        let f = put(&mut g, FactoryKind::CrudeFurnace, at(0, 0), Facing::East);
        load(&mut g, f, Slot::Input, ItemKind::Log, 2);
        load(&mut g, f, Slot::Fuel, ItemKind::Stick, 3);
        let p = g.get_player_entity().unwrap();
        factory::break_block(&mut g.world, &mut g.res, at(0, 0), p);
        let mut dropped: Vec<(ItemKind, u32)> = g
            .world
            .query::<&DroppedItem>()
            .iter()
            .map(|(_, d)| (d.kind, d.qty))
            .collect();
        dropped.sort();
        assert_eq!(dropped, [(ItemKind::Log, 2), (ItemKind::Stick, 3)]);
    }
}

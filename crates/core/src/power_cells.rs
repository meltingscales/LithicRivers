//! Stored power: battery blocks on the grid, and power cells the robot carries.
//!
//! A battery charges from its network's surplus and discharges into a deficit (see
//! `power::step_power`), and charges the cell in its slot from what it stores. Cells are
//! items that never stack, so each keeps its own charge (`ItemStack::charge_j`). The robot's
//! fusion core trickles charge into the first carried cell that isn't full, and gear draws
//! from the fullest carried cell (`use_power`).
//!
//! All energy is whole joules, so it stays exact and deterministic.

use hecs::{Entity, World};
use serde::{Deserialize, Serialize};

use crate::components::{Inventory, ItemKind, ItemStack, Player};
use crate::factory::{FACTORY_STEP_TICKS, STEPS_PER_FACTORY_SECOND, TICKS_PER_SECOND};
use crate::model::body::{Body, BodyPartState, BodyPartType};
use crate::power::PowerConsumer;

/// The most a battery stores, in joules.
pub const BATTERY_CAPACITY_J: u64 = 40_000_000;
/// The fastest a battery charges or discharges on its network, in watts.
pub const BATTERY_RATE_W: u64 = 90_000;
/// The fastest a battery charges the cell in its slot, in watts.
pub const CELL_CHARGE_W: u64 = 90_000;
/// Item slots a battery has: one, for a cell.
pub const BATTERY_SLOTS: usize = 1;

/// What an item charger draws from its network while a cell in it isn't full, in watts.
pub const ITEM_CHARGER_W: u64 = 50_000;
/// Cell slots an item charger has.
pub const ITEM_CHARGER_SLOTS: usize = 2;

/// Whether an inventory holds a cell that isn't full.
fn has_unfull_cell(inv: &Inventory) -> bool {
    inv.slots.iter().any(|s| room(s) > 0)
}

/// Before power is shared: an item charger wants power only while a cell in it isn't full.
pub fn update_charger_demand(world: &mut World) {
    for (_, (block, inv, consumer)) in world.query_mut::<(
        &crate::factory::FactoryBlock,
        &Inventory,
        &mut PowerConsumer,
    )>() {
        if block.kind == crate::factory::FactoryKind::ItemCharger {
            consumer.active = has_unfull_cell(inv);
        }
    }
}

/// After power is shared: each item charger puts what its network delivered into its
/// cells, in slot order.
pub fn step_chargers(world: &mut World) {
    for (_, (block, inv, consumer)) in world.query_mut::<(
        &crate::factory::FactoryBlock,
        &mut Inventory,
        &PowerConsumer,
    )>() {
        if block.kind != crate::factory::FactoryKind::ItemCharger || !consumer.active {
            continue;
        }
        let mut joules = consumer.demand_w / STEPS_PER_FACTORY_SECOND
            * u64::from(consumer.satisfaction_permille)
            / 1000;
        for cell in inv.slots.iter_mut() {
            let moved = joules.min(room(cell));
            if moved > 0 {
                cell.charge_j = Some(charge(cell) + moved);
                joules -= moved;
            }
        }
    }
}

/// A battery block's stored energy.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Battery {
    pub stored_j: u64,
}

/// The charge of a power cell lying on the ground (a dropped cell's `DroppedItem` has
/// no room for it).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CellCharge(pub u64);

/// What a power cell holds when full, or `None` if the item isn't a cell.
pub fn cell_capacity(item: ItemKind) -> Option<u64> {
    match item {
        ItemKind::CrudePowerCell => Some(2_000_000),
        ItemKind::StandardPowerCell => Some(20_000_000),
        ItemKind::AdvancedPowerCell => Some(100_000_000),
        _ => None,
    }
}

/// Whether an item is a power cell.
pub fn is_cell(item: ItemKind) -> bool {
    cell_capacity(item).is_some()
}

/// A cell stack's charge (an empty cell has none recorded).
pub fn charge(stack: &ItemStack) -> u64 {
    stack.charge_j.unwrap_or(0)
}

/// Room left in a cell stack, in joules (0 for anything that isn't a cell).
fn room(stack: &ItemStack) -> u64 {
    cell_capacity(stack.kind).map_or(0, |cap| cap.saturating_sub(charge(stack)))
}

/// A cell's charge for the player, short enough for item lists: "1.2/2 MJ".
pub fn charge_label(stack: &ItemStack) -> Option<String> {
    let cap = cell_capacity(stack.kind)?;
    let held = crate::teleporter::format_mj(charge(stack));
    Some(format!(
        "{}/{}",
        held.trim_end_matches(" MJ"),
        crate::teleporter::format_mj(cap)
    ))
}

/// After power is shared: each battery charges the cell in its slot from what it stores.
pub fn charge_slot_cells(world: &mut World) {
    let per_step = CELL_CHARGE_W / STEPS_PER_FACTORY_SECOND;
    for (_, (battery, inv)) in world.query_mut::<(&mut Battery, &mut Inventory)>() {
        let Some(cell) = inv.slots.first_mut() else {
            continue;
        };
        let moved = per_step.min(room(cell)).min(battery.stored_j);
        if moved > 0 {
            battery.stored_j -= moved;
            cell.charge_j = Some(charge(cell) + moved);
        }
    }
}

/// What the robot's fusion core charges a carried cell with per game second, by its state.
pub fn core_charge_per_second(state: BodyPartState) -> u64 {
    match state {
        BodyPartState::Missing => 0,
        BodyPartState::Damaged => 20,
        BodyPartState::Functional => 50,
        BodyPartState::Enhanced => 100,
    }
}

/// Each factory step: the player's fusion core charges the first carried cell that isn't
/// full, at its state's rate.
pub fn trickle_charge(world: &mut World) {
    for (_, (body, inv)) in world
        .query_mut::<(&Body, &mut Inventory)>()
        .with::<&Player>()
    {
        let state = body
            .parts
            .get(&BodyPartType::PowerSource)
            .map_or(BodyPartState::Missing, |p| p.state);
        let joules = core_charge_per_second(state) * FACTORY_STEP_TICKS / TICKS_PER_SECOND;
        if let Some(cell) = inv.slots.iter_mut().find(|s| room(s) > 0) {
            cell.charge_j = Some(charge(cell) + joules.min(room(cell)));
        }
    }
}

/// Why gear couldn't draw power.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoCharge {
    /// How much more the fullest cell would need
    pub short_j: u64,
}

impl NoCharge {
    pub fn message(&self) -> String {
        format!(
            "No charge: your fullest power cell is {} short.",
            crate::teleporter::format_mj(self.short_j)
        )
    }
}

/// The charge in the fullest cell `entity` carries (0 with none).
pub fn fullest_charge(world: &World, entity: Entity) -> u64 {
    world.get::<&Inventory>(entity).map_or(0, |inv| {
        inv.slots
            .iter()
            .filter(|s| is_cell(s.kind))
            .map(charge)
            .max()
            .unwrap_or(0)
    })
}

/// Draw `joules` for gear from the fullest cell `entity` carries (the first, on a tie).
/// Refused, drawing nothing, if no single cell holds enough.
pub fn use_power(world: &mut World, entity: Entity, joules: u64) -> Result<(), NoCharge> {
    let mut inv = world
        .get::<&mut Inventory>(entity)
        .map_err(|_| NoCharge { short_j: joules })?;
    let fullest = inv
        .slots
        .iter_mut()
        .filter(|s| is_cell(s.kind))
        .reduce(|best, s| if charge(s) > charge(best) { s } else { best });
    match fullest {
        Some(cell) if charge(cell) >= joules => {
            cell.charge_j = Some(charge(cell) - joules);
            Ok(())
        }
        Some(cell) => Err(NoCharge {
            short_j: joules - charge(cell),
        }),
        None => Err(NoCharge { short_j: joules }),
    }
}

/// A battery's status for its window: stored energy, and what it is doing on its network.
pub fn battery_status(world: &World, res: &crate::resources::Resources, battery: Entity) -> String {
    let stored = world.get::<&Battery>(battery).map_or(0, |b| b.stored_j);
    let doing = match res.power.network_of(battery) {
        None => "no power pole in reach".to_string(),
        Some(network) => match network.battery_flows.iter().find(|(e, _)| *e == battery) {
            Some((_, w)) if *w > 0 => format!("charging at {} kW", w / 1000),
            Some((_, w)) if *w < 0 => format!("discharging at {} kW", -w / 1000),
            _ => "idle".to_string(),
        },
    };
    format!(
        "{} / {}, {doing}",
        crate::teleporter::format_mj(stored),
        crate::teleporter::format_mj(BATTERY_CAPACITY_J)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Position;
    use crate::factory::{self, Facing, FactoryKind};
    use crate::power::PowerConsumer;
    use crate::test_support::TestWorld;
    use crate::tiles::TileKind;
    use crate::Game;

    fn at(x: i64, y: i64) -> Position {
        Position { x, y, z: 0 }
    }

    fn game() -> Game {
        TestWorld::new(4)
            .fill((-12, -12, 0), (12, 12, 0), TileKind::Grass)
            .player_at(0, 10, 0)
            .inventory(&[])
            .build()
            .game
    }

    fn put(game: &mut Game, kind: FactoryKind, pos: Position) -> Entity {
        factory::place(&mut game.world, &mut game.res, kind, pos, Facing::North).expect("placed")
    }

    fn pass_factory_seconds(game: &mut Game, seconds: u64) {
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

    fn stored(game: &Game, battery: Entity) -> u64 {
        game.world.get::<&Battery>(battery).unwrap().stored_j
    }

    fn cell(kind: ItemKind, charge_j: u64) -> ItemStack {
        ItemStack {
            charge_j: Some(charge_j),
            ..ItemStack::new(kind, 1)
        }
    }

    fn player_cells(game: &Game) -> Vec<(ItemKind, u64)> {
        let player = game.get_player_entity().unwrap();
        game.world
            .get::<&Inventory>(player)
            .unwrap()
            .slots
            .iter()
            .filter(|s| is_cell(s.kind))
            .map(|s| (s.kind, charge(s)))
            .collect()
    }

    fn give_player(game: &mut Game, stack: ItemStack) {
        let player = game.get_player_entity().unwrap();
        assert!(game
            .world
            .get::<&mut Inventory>(player)
            .unwrap()
            .add_stack(stack));
    }

    /// A generator with `biochar` fuel, a pole, a battery and a 45 kW consumer.
    fn grid(game: &mut Game, biochar: u32) -> (Entity, Entity, Entity) {
        let generator = put(game, FactoryKind::BurnerGenerator, at(0, 0));
        game.world
            .get::<&mut Inventory>(generator)
            .unwrap()
            .add(ItemKind::Biochar, biochar);
        put(game, FactoryKind::PowerPole, at(1, 0));
        let battery = put(game, FactoryKind::Battery, at(2, 0));
        let consumer = put(game, FactoryKind::TestConsumer, at(1, 1));
        (generator, battery, consumer)
    }

    // --- 1.1 batteries ------------------------------------------------------------------

    #[test]
    fn a_battery_stores_the_surplus_at_its_rate() {
        let mut game = game();
        let (_, battery, _) = grid(&mut game, 50);
        pass_factory_seconds(&mut game, 10);
        // 90 kW from the burner, 45 kW to the consumer: 45 kW spare, under the 90 kW rate
        assert_eq!(stored(&game, battery), 45_000 * 10);
        let network = game.res.power.network_of(battery).unwrap();
        assert_eq!(network.stored_j, 450_000);
        assert_eq!(network.battery_flows, vec![(battery, 45_000)]);
    }

    #[test]
    fn a_charged_battery_keeps_machines_running_through_a_fuel_gap() {
        let mut game = game();
        let (_, battery, consumer) = grid(&mut game, 0);
        game.world.get::<&mut Battery>(battery).unwrap().stored_j = 450_000;
        pass_factory_seconds(&mut game, 1);
        let c = *game.world.get::<&PowerConsumer>(consumer).unwrap();
        assert_eq!(c.satisfaction_permille, 1000, "the battery covers it all");
        assert_eq!(stored(&game, battery), 450_000 - 45_000);
        let network = game.res.power.network_of(battery).unwrap();
        assert_eq!(network.supply_w, 45_000);
        assert_eq!(network.battery_flows, vec![(battery, -45_000)]);
        // Until it runs dry
        pass_factory_seconds(&mut game, 9);
        assert_eq!(stored(&game, battery), 0);
        pass_factory_seconds(&mut game, 1);
        let c = *game.world.get::<&PowerConsumer>(consumer).unwrap();
        assert_eq!(c.satisfaction_permille, 0);
    }

    #[test]
    fn a_battery_discharges_no_faster_than_its_rate() {
        let mut game = game();
        let (_, battery, _) = grid(&mut game, 0);
        for x in 3..=5 {
            put(&mut game, FactoryKind::TestConsumer, at(x - 3, 2));
        }
        // 4 consumers want 180 kW; the battery gives 90 kW
        game.world.get::<&mut Battery>(battery).unwrap().stored_j = BATTERY_CAPACITY_J;
        pass_factory_seconds(&mut game, 1);
        assert_eq!(stored(&game, battery), BATTERY_CAPACITY_J - BATTERY_RATE_W);
    }

    #[test]
    fn a_full_battery_stops_charging() {
        let mut game = game();
        let (generator, battery, _) = grid(&mut game, 50);
        game.world.get::<&mut Battery>(battery).unwrap().stored_j = BATTERY_CAPACITY_J;
        let fuel = |g: &Game| {
            g.world.get::<&Inventory>(generator).unwrap().slots[0].qty as u64 * 4_000_000
                + g.world
                    .get::<&crate::power::Generator>(generator)
                    .unwrap()
                    .buffer_j
        };
        let before = fuel(&game);
        pass_factory_seconds(&mut game, 10);
        assert_eq!(stored(&game, battery), BATTERY_CAPACITY_J);
        assert_eq!(
            before - fuel(&game),
            45_000 * 10,
            "only the consumer burns fuel"
        );
    }

    #[test]
    fn stored_energy_and_cells_survive_a_save_and_reload() {
        let mut game = game();
        let (_, battery, _) = grid(&mut game, 0);
        game.world.get::<&mut Battery>(battery).unwrap().stored_j = 1_234_567;
        give_player(&mut game, cell(ItemKind::StandardPowerCell, 7_654_321));
        let saved = crate::save_load::save_to_string(&game, &serde_json::Value::Null).unwrap();
        game.world.get::<&mut Battery>(battery).unwrap().stored_j = 0;
        crate::save_load::load_from_str(&mut game, &saved).expect("loaded");
        let battery = game.res.factory_index[&at(2, 0)];
        assert_eq!(stored(&game, battery), 1_234_567);
        assert_eq!(
            player_cells(&game),
            vec![(ItemKind::StandardPowerCell, 7_654_321)]
        );
    }

    // --- 1.2 cells ----------------------------------------------------------------------

    #[test]
    fn cells_never_stack_and_keep_their_own_charge() {
        let mut inv = Inventory::default();
        inv.add(ItemKind::CrudePowerCell, 2);
        assert!(inv.add_stack(cell(ItemKind::CrudePowerCell, 500)));
        assert_eq!(inv.slots.len(), 3);
        assert!(inv.slots.iter().all(|s| s.qty == 1));
        assert_eq!(charge(&inv.slots[2]), 500);
        let taken = inv.take_one(2).unwrap();
        assert_eq!(taken.charge_j, Some(500));
        assert_eq!(inv.slots.len(), 2);
        // Other items still stack
        inv.add(ItemKind::Log, 2);
        inv.add(ItemKind::Log, 3);
        assert_eq!(inv.slots.len(), 3);
    }

    #[test]
    fn cell_labels_show_charge_and_capacity() {
        assert_eq!(
            crate::components::stack_name(&cell(ItemKind::CrudePowerCell, 1_250_000)),
            "Crude Power Cell (1.3/2 MJ)"
        );
        assert_eq!(
            crate::components::stack_name(&ItemStack::new(ItemKind::Log, 3)),
            "Log"
        );
    }

    #[test]
    fn stacks_without_a_charge_save_as_before() {
        let json = serde_json::to_string(&ItemStack::new(ItemKind::Log, 2)).unwrap();
        assert_eq!(json, r#"{"kind":"Log","qty":2}"#);
        let old: ItemStack = serde_json::from_str(r#"{"kind":"Log","qty":2}"#).unwrap();
        assert_eq!(old, ItemStack::new(ItemKind::Log, 2));
    }

    #[test]
    fn moving_a_cell_between_inventories_and_the_ground_keeps_its_charge() {
        let mut game = game();
        let chest = put(&mut game, FactoryKind::Chest, at(0, 9));
        let player = game.get_player_entity().unwrap();
        // Full, so the fusion core's trickle doesn't change it on the way
        give_player(&mut game, cell(ItemKind::CrudePowerCell, 2_000_000));
        factory::move_one_item_to_block(&mut game.world, player, chest, 0).unwrap();
        assert_eq!(
            game.world.get::<&Inventory>(chest).unwrap().slots[0].charge_j,
            Some(2_000_000)
        );
        factory::move_one_item(&mut game.world, chest, player, 0).unwrap();
        assert_eq!(
            player_cells(&game),
            vec![(ItemKind::CrudePowerCell, 2_000_000)]
        );
        // Dropped and picked up again
        let taken = game
            .world
            .get::<&mut Inventory>(player)
            .unwrap()
            .take_one(0)
            .unwrap();
        factory::spawn_drop(&mut game.world, at(1, 10), taken);
        game.world
            .get::<&mut Inventory>(player)
            .unwrap()
            .auto_pickup = true;
        game.queue_player_move(1, 0);
        game.tick();
        assert_eq!(
            player_cells(&game),
            vec![(ItemKind::CrudePowerCell, 2_000_000)]
        );
    }

    #[test]
    fn belts_and_extractors_leave_cells_alone() {
        let mut game = game();
        let chest = put(&mut game, FactoryKind::Chest, at(-3, 0));
        put(&mut game, FactoryKind::Extractor, at(-2, 0));
        let belt = put(&mut game, FactoryKind::Belt, at(-1, 0));
        assert!(!factory::accepts(
            &game.world,
            belt,
            ItemKind::CrudePowerCell
        ));
        game.world
            .get::<&mut Inventory>(chest)
            .unwrap()
            .add(ItemKind::CrudePowerCell, 1);
        assert_eq!(crate::logistics::peek(&game.world, chest, None), None);
    }

    // --- 1.3 charging cells -------------------------------------------------------------

    #[test]
    fn a_battery_charges_the_cell_in_its_slot_until_full_or_empty() {
        let mut game = game();
        let battery = put(&mut game, FactoryKind::Battery, at(5, 5));
        let player = game.get_player_entity().unwrap();
        give_player(&mut game, cell(ItemKind::StandardPowerCell, 0));
        assert!(!factory::accepts(&game.world, battery, ItemKind::Log));
        factory::move_one_item_to_block(&mut game.world, player, battery, 0).unwrap();
        game.world.get::<&mut Battery>(battery).unwrap().stored_j = 900_000;
        pass_factory_seconds(&mut game, 5);
        let slot = game.world.get::<&Inventory>(battery).unwrap().slots[0];
        assert_eq!(charge(&slot), CELL_CHARGE_W * 5);
        assert_eq!(stored(&game, battery), 900_000 - CELL_CHARGE_W * 5);
        pass_factory_seconds(&mut game, 10);
        let slot = game.world.get::<&Inventory>(battery).unwrap().slots[0];
        assert_eq!(charge(&slot), 900_000, "until the battery is empty");
        assert_eq!(stored(&game, battery), 0);
        // A nearly full cell stops at its capacity
        game.world.get::<&mut Inventory>(battery).unwrap().slots[0].charge_j =
            Some(20_000_000 - 1_000);
        game.world.get::<&mut Battery>(battery).unwrap().stored_j = 900_000;
        pass_factory_seconds(&mut game, 1);
        let slot = game.world.get::<&Inventory>(battery).unwrap().slots[0];
        assert_eq!(charge(&slot), 20_000_000);
        assert_eq!(stored(&game, battery), 899_000);
    }

    #[test]
    fn the_fusion_core_trickles_charge_by_its_state() {
        assert_eq!(core_charge_per_second(BodyPartState::Damaged), 20);
        assert_eq!(core_charge_per_second(BodyPartState::Functional), 50);
        assert_eq!(core_charge_per_second(BodyPartState::Enhanced), 100);
        for (state, per_second) in [
            (BodyPartState::Damaged, 20),
            (BodyPartState::Functional, 50),
            (BodyPartState::Enhanced, 100),
        ] {
            let mut game = game();
            let player = game.get_player_entity().unwrap();
            game.world
                .get::<&mut Body>(player)
                .unwrap()
                .parts
                .get_mut(&BodyPartType::PowerSource)
                .unwrap()
                .state = state;
            // A full cell first: the charge goes to the first one that isn't full
            give_player(&mut game, cell(ItemKind::CrudePowerCell, 2_000_000));
            give_player(&mut game, cell(ItemKind::CrudePowerCell, 0));
            // One game minute is 20 factory seconds
            pass_factory_seconds(&mut game, 20);
            assert_eq!(
                player_cells(&game),
                vec![
                    (ItemKind::CrudePowerCell, 2_000_000),
                    (ItemKind::CrudePowerCell, per_second * 60)
                ],
                "{state:?}"
            );
        }
    }

    // --- 1.4 gear -----------------------------------------------------------------------

    #[test]
    fn gear_draws_from_the_fullest_cell_and_refuses_when_none_has_enough() {
        let mut game = game();
        let player = game.get_player_entity().unwrap();
        assert_eq!(
            use_power(&mut game.world, player, 10),
            Err(NoCharge { short_j: 10 })
        );
        give_player(&mut game, cell(ItemKind::CrudePowerCell, 1_000_000));
        give_player(&mut game, cell(ItemKind::StandardPowerCell, 3_000_000));
        give_player(&mut game, cell(ItemKind::CrudePowerCell, 2_000_000));
        assert_eq!(fullest_charge(&game.world, player), 3_000_000);
        use_power(&mut game.world, player, 1_500_000).unwrap();
        assert_eq!(
            player_cells(&game),
            vec![
                (ItemKind::CrudePowerCell, 1_000_000),
                (ItemKind::StandardPowerCell, 1_500_000),
                (ItemKind::CrudePowerCell, 2_000_000),
            ]
        );
        let err = use_power(&mut game.world, player, 2_500_000).unwrap_err();
        assert_eq!(err, NoCharge { short_j: 500_000 });
        assert_eq!(
            err.message(),
            "No charge: your fullest power cell is 0.5 MJ short."
        );
        assert_eq!(
            fullest_charge(&game.world, player),
            2_000_000,
            "nothing drawn"
        );
    }

    #[test]
    fn an_item_charger_fills_its_cells_in_slot_order_from_the_grid() {
        let mut game = game();
        let rtg = put(&mut game, FactoryKind::Rtg, at(0, 0));
        put(&mut game, FactoryKind::PowerPole, at(1, 0));
        let charger = put(&mut game, FactoryKind::ItemCharger, at(2, 0));
        assert!(!factory::accepts(&game.world, charger, ItemKind::Log));
        // Nothing to charge: it draws nothing
        pass_factory_seconds(&mut game, 1);
        assert!(!game.world.get::<&PowerConsumer>(charger).unwrap().active);
        {
            let mut inv = game.world.get::<&mut Inventory>(charger).unwrap();
            inv.add_stack(cell(ItemKind::CrudePowerCell, 1_990_000));
            inv.add_stack(cell(ItemKind::CrudePowerCell, 0));
        }
        pass_factory_seconds(&mut game, 1);
        let charges: Vec<u64> = game
            .world
            .get::<&Inventory>(charger)
            .unwrap()
            .slots
            .iter()
            .map(charge)
            .collect();
        // 50 kW: the first cell takes what it has room for, the second the rest
        assert_eq!(charges, vec![2_000_000, ITEM_CHARGER_W - 10_000]);
        assert!(game.world.contains(rtg));
    }

    #[test]
    fn an_rtg_powers_its_network_forever_and_burns_nothing() {
        let mut game = game();
        put(&mut game, FactoryKind::Rtg, at(0, 0));
        put(&mut game, FactoryKind::PowerPole, at(1, 0));
        let consumer = put(&mut game, FactoryKind::TestConsumer, at(2, 0));
        pass_factory_seconds(&mut game, 200);
        let c = *game.world.get::<&PowerConsumer>(consumer).unwrap();
        assert_eq!(c.satisfaction_permille, 1000);
        let network = game.res.power.network_of(consumer).unwrap();
        assert_eq!(network.supply_w, crate::factory::TEST_CONSUMER_W);
    }
}

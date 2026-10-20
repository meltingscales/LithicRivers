//! Headless tests for factory blocks in the client: placing and using chests, rotating,
//! and passing time.

use crate::app_state::BuildMode;
use crate::headless::{parse_script, state_json, Runner, EXIT_OK};
use lithicrivers_core::components::{Inventory, ItemKind, Position};
use lithicrivers_core::factory::{self, Facing, FactoryBlock, FactoryKind};

/// From the World tab at (0, 0, 0): Place mode with a chest on hotbar slot F1.
const READY_TO_PLACE: &str = "skip-splash\nkey ui.MENU_PREV\ncheat tp 0 0 0\n\
    cheat giveitem chest 1\nkey build.TOGGLE_BREAK_PLACE_MODE 2\n\
    key hotbar.HOTBAR_SLOT_1 2\nkey movement.MOVE_SOUTH\nkey ui.MENU_ACTIVATE\n";
/// From Place mode: back to movement mode (through Rotate).
const TO_MOVEMENT: &str = "key build.TOGGLE_BREAK_PLACE_MODE 2\n";
const WEST: Position = Position { x: -1, y: 0, z: 0 };

fn run(runner: &mut Runner, text: &str) {
    let report = runner.run_steps(&parse_script(text).unwrap().steps);
    assert_eq!(report.status, EXIT_OK, "{:?}", report.error);
}

fn after(text: &str) -> Runner {
    let mut runner = Runner::new(12345, (100, 30));
    run(&mut runner, text);
    runner
}

fn count(runner: &Runner, entity: hecs::Entity, kind: ItemKind) -> u32 {
    runner
        .app
        .core
        .game
        .world
        .get::<&Inventory>(entity)
        .map(|inv| {
            inv.slots
                .iter()
                .filter(|s| s.kind == kind)
                .map(|s| s.qty)
                .sum()
        })
        .unwrap_or(0)
}

fn last_message(runner: &Runner) -> String {
    let log = &runner.app.core.game.res.events.get_message_log().messages;
    log.last().map(|m| m.text.clone()).unwrap_or_default()
}

fn block_at(runner: &Runner, pos: Position) -> hecs::Entity {
    *runner
        .app
        .core
        .game
        .res
        .factory_index
        .get(&pos)
        .expect("a block there")
}

#[test]
fn chest_stores_and_returns_items() {
    let mut runner = after(&format!(
        "{READY_TO_PLACE}key build.PLACE_WEST\n{TO_MOVEMENT}"
    ));
    let chest = block_at(&runner, WEST);
    let player = runner.app.core.game.get_player_entity().unwrap();
    assert_eq!(
        count(&runner, player, ItemKind::Chest),
        0,
        "the chest was used"
    );

    // Store 3 logs: the player's log stack is the third (Nail, Acorn, Log, ...)
    run(
        &mut runner,
        "key action.INTERACT\nexpect-text \"Chest Items (0/16)\"\nkey movement.MOVE_EAST\n\
         key movement.MOVE_SOUTH 2\nkey ui.MENU_ACTIVATE 3",
    );
    assert_eq!(count(&runner, chest, ItemKind::Log), 3);
    assert_eq!(count(&runner, player, ItemKind::Log), 0);

    // Take one back from the chest panel
    run(&mut runner, "key movement.MOVE_WEST\nkey ui.MENU_ACTIVATE");
    assert_eq!(count(&runner, chest, ItemKind::Log), 2);
    assert_eq!(count(&runner, player, ItemKind::Log), 1);
    assert!(last_message(&runner).contains("Took Log"));
}

#[test]
fn breaking_a_chest_gives_it_back_and_spills_its_contents() {
    let mut runner = after(&format!("{READY_TO_PLACE}key build.PLACE_WEST\n"));
    let chest = block_at(&runner, WEST);
    runner
        .app
        .core
        .game
        .world
        .get::<&mut Inventory>(chest)
        .unwrap()
        .add(ItemKind::Log, 10);
    // Break mode is one step back around the cycle: Place -> Rotate -> Movement -> Break
    run(
        &mut runner,
        "key build.TOGGLE_BREAK_PLACE_MODE 3\nkey build.BREAK_WEST",
    );
    let player = runner.app.core.game.get_player_entity().unwrap();
    assert_eq!(count(&runner, player, ItemKind::Chest), 1);
    let logs: u32 = runner
        .app
        .core
        .game
        .world
        .query::<(&Position, &lithicrivers_core::components::DroppedItem)>()
        .iter()
        .filter(|(_, (p, d))| **p == WEST && d.kind == ItemKind::Log)
        .map(|(_, (_, d))| d.qty)
        .sum();
    assert_eq!(logs, 10);
}

#[test]
fn rotate_before_placing_sets_the_facing_and_shows_it() {
    let mut runner = after(&format!("{READY_TO_PLACE}key build.ROTATE_PLACEMENT 2"));
    assert_eq!(runner.app.panels.build.placement_facing, Facing::South);
    assert!(
        runner.screen_text().contains("facing v south"),
        "hotbar shows the facing"
    );
    run(&mut runner, "key build.PLACE_WEST");
    let chest = block_at(&runner, WEST);
    let facing = runner
        .app
        .core
        .game
        .world
        .get::<&FactoryBlock>(chest)
        .unwrap()
        .facing;
    assert_eq!(facing, Facing::South);
}

#[test]
fn rotate_mode_turns_directional_blocks_and_explains_the_rest() {
    let mut runner = after(&format!("{READY_TO_PLACE}key build.PLACE_WEST\n"));
    let game = &mut runner.app.core.game;
    let east = Position { x: 1, y: 0, z: 0 };
    game.res
        .world_state
        .world
        .set_tile_cached(1, 0, 0, lithicrivers_core::TileKind::Grass);
    let arrow = factory::place(
        &mut game.world,
        &mut game.res,
        FactoryKind::TestArrow,
        east,
        Facing::North,
    )
    .unwrap();
    run(&mut runner, "key build.TOGGLE_BREAK_PLACE_MODE");
    assert_eq!(runner.app.panels.build.mode, BuildMode::Rotate);

    run(&mut runner, "key build.PLACE_EAST");
    let facing = runner
        .app
        .core
        .game
        .world
        .get::<&FactoryBlock>(arrow)
        .unwrap()
        .facing;
    assert_eq!(facing, Facing::East);
    assert!(last_message(&runner).contains("face east"));

    run(&mut runner, "key build.PLACE_WEST");
    assert!(last_message(&runner).contains("The chest has no direction to rotate"));
    run(&mut runner, "key build.PLACE_SOUTH");
    assert!(last_message(&runner).contains("Nothing to rotate there"));
}

#[test]
fn pass_time_advances_ten_game_seconds() {
    let mut runner = after("skip-splash\nkey ui.MENU_PREV\ncheat tp 0 0 0");
    let tick = runner.app.core.game.res.time.tick;
    let steps = runner.app.core.game.res.factory_clock.steps;
    run(&mut runner, "key action.PASS_TIME");
    let game = &runner.app.core.game;
    assert_eq!(game.res.time.tick, tick + 1000);
    assert_eq!(
        game.res.factory_clock.steps,
        steps + 1000 / factory::FACTORY_STEP_TICKS
    );
}

#[test]
fn a_dog_arriving_stops_passing_time_and_starts_combat() {
    // A dog five tiles away reaches the player partway through the wait
    let mut runner =
        after("skip-splash\nkey ui.MENU_PREV\ncheat tp 0 0 0\ncheat spawn_enemy feral_dog ~-5 ~ ~");
    let tick = runner.app.core.game.res.time.tick;
    run(
        &mut runner,
        "key action.PASS_TIME\nexpect-text \"Action Queue\"",
    );
    let passed = runner.app.core.game.res.time.tick - tick;
    assert!(passed < 1000, "stopped early after {passed} ticks");
    assert!(runner.app.combat.is_active());
    assert!(state_json(&runner.app).contains("\"active\": true"));
}

//! Headless tests for factory blocks in the client: placing and using chests, rotating,
//! and passing time.

use crate::app_state::BuildMode;
use crate::headless::{chain_commands, state_json, Runner};
use lithicrivers_core::components::{Inventory, ItemKind, Position};
use lithicrivers_core::factory::{self, Facing, FactoryBlock, FactoryKind};

/// From the World tab at (0, 0, 0): Place mode with a chest on hotbar slot F1.
const READY_TO_PLACE: &[&str] = &[
    "skip-splash",
    "key ui.MENU_PREV",
    "cheat tp 0 0 0",
    "cheat giveitem chest 1",
    "key build.TOGGLE_BREAK_PLACE_MODE 2",
    "key hotbar.HOTBAR_SLOT_1 2",
    // The picker lists Clear slot, then placeable items in inventory order: Acorn, Chest
    "key movement.MOVE_SOUTH 2",
    "key ui.MENU_ACTIVATE",
];
/// From Place mode: back to movement mode (through Rotate).
const TO_MOVEMENT: &[&str] = &["key build.TOGGLE_BREAK_PLACE_MODE 2"];
const WEST: Position = Position { x: -1, y: 0, z: 0 };

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
    let mut runner =
        chain_commands(&[READY_TO_PLACE, &["key build.PLACE_WEST"], TO_MOVEMENT].concat());
    let chest = block_at(&runner, WEST);
    let player = runner.app.core.game.get_player_entity().unwrap();
    assert_eq!(
        count(&runner, player, ItemKind::Chest),
        0,
        "the chest was used"
    );

    // Store 3 logs: the player's log stack is the third (Nail, Acorn, Log, ...)
    runner.chain(&[
        "key action.INTERACT",
        "expect-text \"Chest Items (0/16)\"",
        "key movement.MOVE_EAST",
        "key movement.MOVE_SOUTH 2",
        "key ui.MENU_ACTIVATE 3",
    ]);
    assert_eq!(count(&runner, chest, ItemKind::Log), 3);
    assert_eq!(count(&runner, player, ItemKind::Log), 0);

    // Take one back from the chest panel
    runner.chain(&["key movement.MOVE_WEST", "key ui.MENU_ACTIVATE"]);
    assert_eq!(count(&runner, chest, ItemKind::Log), 2);
    assert_eq!(count(&runner, player, ItemKind::Log), 1);
    assert!(last_message(&runner).contains("Took Log"));
}

#[test]
fn breaking_a_chest_gives_it_back_and_spills_its_contents() {
    let mut runner = chain_commands(&[READY_TO_PLACE, &["key build.PLACE_WEST"]].concat());
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
    runner.chain(&[
        "key build.TOGGLE_BREAK_PLACE_MODE 3",
        "key build.BREAK_WEST",
    ]);
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
    let mut runner = chain_commands(&[READY_TO_PLACE, &["key build.ROTATE_PLACEMENT 2"]].concat());
    assert_eq!(runner.app.panels.build.placement_facing, Facing::South);
    assert!(
        runner.screen_text().contains("facing v south"),
        "hotbar shows the facing"
    );
    runner.chain(&["key build.PLACE_WEST"]);
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
    let mut runner = chain_commands(&[READY_TO_PLACE, &["key build.PLACE_WEST"]].concat());
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
    runner.chain(&["key build.TOGGLE_BREAK_PLACE_MODE"]);
    assert_eq!(runner.app.panels.build.mode, BuildMode::Rotate);

    runner.chain(&["key build.PLACE_EAST"]);
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

    runner.chain(&["key build.PLACE_WEST"]);
    assert!(last_message(&runner).contains("The chest has no direction to rotate"));
    runner.chain(&["key build.PLACE_SOUTH"]);
    assert!(last_message(&runner).contains("Nothing to rotate there"));
}

#[test]
fn pass_time_advances_ten_game_seconds() {
    let mut runner = chain_commands(&["skip-splash", "key ui.MENU_PREV", "cheat tp 0 0 0"]);
    let tick = runner.app.core.game.res.time.tick;
    let steps = runner.app.core.game.res.factory_clock.steps;
    runner.chain(&["key action.PASS_TIME"]);
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
    let mut runner = chain_commands(&[
        "skip-splash",
        "key ui.MENU_PREV",
        "cheat tp 0 0 0",
        "cheat spawn_enemy feral_dog ~-5 ~ ~",
    ]);
    let tick = runner.app.core.game.res.time.tick;
    runner.chain(&["key action.PASS_TIME", "expect-text \"Action Queue\""]);
    let passed = runner.app.core.game.res.time.tick - tick;
    assert!(passed < 1000, "stopped early after {passed} ticks");
    assert!(runner.app.combat.is_active());
    assert!(state_json(&runner.app).contains("\"active\": true"));
}

/// Where the open Choose Interaction menu lists the choice at `pos`.
fn choice_index(runner: &Runner, pos: Position) -> usize {
    let crate::app_state::MultiActionSelectState::SelectingAction {
        available_actions, ..
    } = &runner.app.panels.multi_action_select
    else {
        panic!("the Choose Interaction menu is open");
    };
    available_actions
        .iter()
        .position(|a| a.position() == pos)
        .expect("a choice there")
}

/// Grass from (-3, -3) to (6, 3) around the origin, so a build site is always clear.
fn clear_site(runner: &mut Runner) {
    let world = &mut runner.app.core.game.res.world_state.world;
    for x in -3..=6 {
        for y in -3..=3 {
            world.set_tile_cached(x, y, 0, lithicrivers_core::TileKind::Grass);
        }
    }
}

#[test]
fn belt_line_carries_logs_into_a_chest() {
    let mut runner = chain_commands(&[
        "skip-splash",
        "key ui.MENU_PREV",
        "cheat tp 0 0 0",
        "cheat giveitem wooden_conveyor 4",
        "cheat giveitem chest",
    ]);
    clear_site(&mut runner);
    // Picker order follows the inventory: Clear slot, Acorn, Wooden Conveyor, Chest
    runner.chain(&[
        "key build.TOGGLE_BREAK_PLACE_MODE 2",
        "key hotbar.HOTBAR_SLOT_1 2",
        "key movement.MOVE_SOUTH 2",
        "key ui.MENU_ACTIVATE",
        "key hotbar.HOTBAR_SLOT_2 2",
        "key movement.MOVE_SOUTH 3",
        "key ui.MENU_ACTIVATE",
        // Belts face east; build y = -1 from x = -1 to 2, chest at x = 3
        "key build.ROTATE_PLACEMENT",
        "key hotbar.HOTBAR_SLOT_1",
        "key build.PLACE_NORTHWEST",
        "key build.PLACE_NORTH",
        "key build.PLACE_NORTHEAST",
        "key build.TOGGLE_BREAK_PLACE_MODE 2",
        "key movement.MOVE_EAST 2",
        "key build.TOGGLE_BREAK_PLACE_MODE 2",
        "key build.PLACE_NORTH",
        "key hotbar.HOTBAR_SLOT_2",
        "key build.PLACE_NORTHEAST",
        "key build.TOGGLE_BREAK_PLACE_MODE 2",
    ]);
    let game = &runner.app.core.game;
    for x in -1..=2 {
        let pos = Position { x, y: -1, z: 0 };
        let belt = block_at(&runner, pos);
        let block = *game.world.get::<&FactoryBlock>(belt).unwrap();
        assert_eq!(
            (block.kind, block.facing),
            (FactoryKind::Belt, Facing::East),
            "{pos:?}"
        );
    }
    let chest = block_at(&runner, Position { x: 3, y: -1, z: 0 });

    // Select logs (Nail, Acorn, Log, ...) in the inventory, then load the belt north of us
    runner.chain(&[
        "key movement.MOVE_WEST 2",
        "key ui.MENU_NEXT 5",
        "key movement.MOVE_SOUTH 2",
        "key ui.MENU_PREV 5",
        "key action.INTERACT",
        "expect-text \"Put Log on belt (N)\"",
    ]);
    // Three times: choose the belt to the north in the menu, let the log ride, interact again
    for _ in 0..3 {
        let north = choice_index(&runner, Position { x: 0, y: -1, z: 0 });
        if north > 0 {
            runner.chain(&[&format!("key movement.MOVE_SOUTH {north}")]);
        }
        runner.chain(&[
            "key ui.MENU_ACTIVATE",
            "key action.PASS_TIME",
            "key action.INTERACT",
        ]);
    }
    runner.chain(&["key ui.CLOSE_HELP_MENU", "key action.PASS_TIME"]);
    assert_eq!(
        count(&runner, chest, ItemKind::Log),
        3,
        "three logs arrived"
    );
}

/// From the World tab at (0, 0, 0): a power pole to the north-west and a burner generator to
/// the west, placed from the hotbar, with a stone in the inventory.
const SMALL_GRID: &[&str] = &[
    "skip-splash",
    "key ui.MENU_PREV",
    "cheat tp 0 0 0",
    "cheat giveitem power_pole",
    "cheat giveitem burner_generator",
    "cheat giveitem stone",
    "key build.TOGGLE_BREAK_PLACE_MODE 2",
    // Picker: Clear slot, Acorn, Power Pole, Burner Generator, Stone
    "key hotbar.HOTBAR_SLOT_1 2",
    "key movement.MOVE_SOUTH 2",
    "key ui.MENU_ACTIVATE",
    "key hotbar.HOTBAR_SLOT_2 2",
    "key movement.MOVE_SOUTH 3",
    "key ui.MENU_ACTIVATE",
    "key hotbar.HOTBAR_SLOT_1",
    "key build.PLACE_NORTHWEST",
    "key hotbar.HOTBAR_SLOT_2",
    "key build.PLACE_WEST",
    "key build.TOGGLE_BREAK_PLACE_MODE 2",
    "key movement.WAIT",
    "key action.INTERACT",
];

#[test]
fn generator_window_refuses_stone() {
    let mut runner = chain_commands(SMALL_GRID);
    let generator = choice_index(&runner, Position { x: -1, y: 0, z: 0 });
    // Your Items: Nail, Acorn, Log, Stick, Torch, String, Scrap Electronics, Stone
    runner.chain(&[
        &format!("key movement.MOVE_SOUTH {generator}"),
        "key ui.MENU_ACTIVATE",
        "expect-text \"Burner Generator - network\"",
        "key movement.MOVE_SOUTH 7",
        "key ui.MENU_ACTIVATE",
        "expect-text \"Stone isn't fuel\"",
    ]);
    let generator = block_at(&runner, Position { x: -1, y: 0, z: 0 });
    assert_eq!(count(&runner, generator, ItemKind::Stone), 0);
}

#[test]
fn power_pole_shows_its_network() {
    let mut runner = chain_commands(SMALL_GRID);
    let pole = choice_index(&runner, Position { x: -1, y: -1, z: 0 });
    runner.chain(&[
        &format!("key movement.MOVE_SOUTH {pole}"),
        "key ui.MENU_ACTIVATE",
        "expect-text \"Power Network\"",
        "expect-text \"Poles: 1   Generators: 1   Machines: 0\"",
        "key ui.CLOSE_HELP_MENU",
    ]);
    assert!(runner.app.panels.power_info.is_none());
    assert!(!runner.screen_text().contains("Power Network"));
}

/// The reference tree farm runs 30 game minutes unattended; see the script for its layout.
const TREE_FARM: &str = include_str!("../../../scripts/snapshots/factory-tree-farm.txt");

fn run_tree_farm() -> (Runner, String) {
    let lines: Vec<&str> = TREE_FARM.lines().collect();
    let mut runner = Runner::new(12345, (100, 30));
    let output = runner.chain(&lines);
    (runner, output)
}

#[test]
fn tree_farm_makes_more_wood_than_it_burns() {
    let (runner, output) = run_tree_farm();
    let chest = block_at(&runner, Position { x: -12, y: 0, z: 0 });
    let generator = block_at(
        &runner,
        Position {
            x: -17,
            y: -4,
            z: 0,
        },
    );
    let logs_made = count(&runner, chest, ItemKind::Log);
    // The generator started with 3 logs; one it has started counts as burned
    let logs_burned = 3 - count(&runner, generator, ItemKind::Log);
    assert!(logs_burned > 0, "the farm drew power");
    assert!(
        logs_made > logs_burned,
        "{logs_made} logs made, {logs_burned} burned"
    );
    assert!(count(&runner, chest, ItemKind::Acorn) > 0);
    assert_eq!(run_tree_farm().1, output, "deterministic");
}

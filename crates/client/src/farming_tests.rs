//! Headless tests for tree farming: planting an acorn from the hotbar, letting it grow,
//! and chopping the tree.

use crate::headless::chain_commands;
use lithicrivers_core::components::{DroppedItem, ItemKind, Position};
use lithicrivers_core::TileKind;

/// World tab at (0, 0, 0), Place mode, the starting acorn on hotbar slot F1.
const READY_TO_PLANT: &[&str] = &[
    "skip-splash",
    "key ui.MENU_PREV",
    "cheat tp 0 0 0",
    "key build.TOGGLE_BREAK_PLACE_MODE 2",
    "key hotbar.HOTBAR_SLOT_1 2",
    "key movement.MOVE_SOUTH",
    "key ui.MENU_ACTIVATE",
];

#[test]
fn plant_grow_and_chop_a_tree() {
    let mut runner = chain_commands(&[READY_TO_PLANT, &["key build.PLACE_WEST"]].concat());
    let tile = |runner: &mut crate::headless::Runner| {
        runner
            .app
            .core
            .game
            .res
            .world_state
            .world
            .get_tile_cached(-1, 0, 0)
    };
    assert_eq!(tile(&mut runner), TileKind::Sapling);
    let acorns = runner
        .app
        .core
        .game
        .get_player_component::<lithicrivers_core::components::Inventory>()
        .unwrap()
        .slots
        .iter()
        .filter(|s| s.kind == ItemKind::Acorn)
        .count();
    assert_eq!(acorns, 0, "the only acorn was planted");

    // 5 game minutes is the longest growth time; pass a little more, in Break mode
    runner.chain(&[
        "key build.TOGGLE_BREAK_PLACE_MODE 3",
        "key action.PASS_TIME 31",
    ]);
    assert_eq!(tile(&mut runner), TileKind::Tree);

    runner.chain(&["key build.BREAK_WEST", "expect-text \"You chop the tree\""]);
    let dropped: Vec<ItemKind> = runner
        .app
        .core
        .game
        .world
        .query::<(&Position, &DroppedItem)>()
        .iter()
        .filter(|(_, (p, _))| **p == Position { x: -1, y: 0, z: 0 })
        .map(|(_, (_, d))| d.kind)
        .collect();
    assert!(
        dropped.contains(&ItemKind::Log) && dropped.contains(&ItemKind::Acorn),
        "{dropped:?}"
    );
}

#[test]
fn acorns_need_grass_or_dirt() {
    let mut runner = chain_commands(READY_TO_PLANT);
    // Air is passable but isn't soil
    runner
        .app
        .core
        .game
        .res
        .world_state
        .world
        .set_tile_cached(0, -1, 0, TileKind::Air);
    runner.chain(&[
        "key build.PLACE_NORTH",
        "expect-text \"Acorns need grass or dirt\"",
    ]);
}

//! Every menu, list and window moves with both the numpad and the arrow keys, and open
//! windows keep Left and Right instead of letting them switch tabs behind them.

use crate::headless::Runner;
use ratatui::buffer::Buffer;

/// Steps that open a menu, from a fresh run (seed 12345, 100x30).
struct Menu {
    name: &'static str,
    setup: &'static [&'static str],
    /// A window over the tabs, which keeps Left and Right
    window: bool,
    /// Moves its selection sideways rather than up and down (the body repair window has
    /// one recipe, but cycles body parts with Left and Right)
    sideways: bool,
}

const PLACE_WEST_THEN_INTERACT: [&str; 7] = [
    "key build.TOGGLE_BREAK_PLACE_MODE 2",
    "key hotbar.HOTBAR_SLOT_1 2",
    "key movement.MOVE_SOUTH 2",
    "key ui.MENU_ACTIVATE",
    "key build.PLACE_WEST",
    "key build.TOGGLE_BREAK_PLACE_MODE 2",
    "key action.INTERACT",
];

fn menus() -> Vec<Menu> {
    let leak = |v: Vec<&'static str>| -> &'static [&'static str] { Vec::leak(v) };
    let placed = |item: &'static str, extra: &[&'static str]| {
        let mut steps = vec!["skip-splash", "key ui.MENU_PREV", "cheat tp 0 0 0", item];
        steps.extend(PLACE_WEST_THEN_INTERACT);
        steps.extend(extra);
        leak(steps)
    };
    vec![
        Menu {
            name: "body repair",
            setup: &["skip-splash", "key ui.MENU_NEXT 3", "rawkey r"],
            window: true,
            sideways: true,
        },
        Menu {
            name: "body parts",
            setup: &["skip-splash", "key ui.MENU_NEXT 3"],
            window: false,
            sideways: false,
        },
        Menu {
            name: "crafting list",
            setup: &["skip-splash", "key ui.MENU_NEXT 5"],
            window: false,
            sideways: false,
        },
        Menu {
            name: "inventory list",
            setup: &["skip-splash", "key ui.MENU_NEXT 4"],
            window: false,
            sideways: false,
        },
        Menu {
            name: "item dictionary",
            setup: &["skip-splash", "key ui.MENU_NEXT 6"],
            window: false,
            sideways: false,
        },
        Menu {
            name: "global map markers",
            setup: &["skip-splash", "key ui.MENU_NEXT 1"],
            window: false,
            sideways: false,
        },
        Menu {
            name: "chest window",
            setup: placed("cheat giveitem chest 1", &["key movement.MOVE_EAST"]),
            window: true,
            sideways: false,
        },
        Menu {
            name: "furnace window",
            setup: placed("cheat giveitem crude_furnace", &[]),
            window: true,
            sideways: false,
        },
        Menu {
            name: "filter picker",
            setup: leak({
                let mut s = vec![
                    "skip-splash",
                    "key ui.MENU_PREV",
                    "cheat tp 0 0 0",
                    "cheat giveitem filter_extractor",
                    "key build.TOGGLE_BREAK_PLACE_MODE 2",
                    "key hotbar.HOTBAR_SLOT_1 2",
                    "key movement.MOVE_SOUTH 2",
                    "key ui.MENU_ACTIVATE",
                    "key hotbar.HOTBAR_SLOT_1",
                    "key build.PLACE_WEST",
                    "key build.TOGGLE_BREAK_PLACE_MODE 2",
                ];
                s.push("key action.INTERACT");
                s
            }),
            window: true,
            sideways: false,
        },
        Menu {
            name: "hotbar block picker",
            setup: &[
                "skip-splash",
                "key ui.MENU_PREV",
                "key build.TOGGLE_BREAK_PLACE_MODE 2",
                "key hotbar.HOTBAR_SLOT_1 2",
            ],
            window: true,
            sideways: false,
        },
        Menu {
            name: "corpse loot",
            setup: &[
                "skip-splash",
                "key ui.MENU_PREV",
                "cheat tp 17 21 0",
                "key action.INTERACT",
            ],
            window: true,
            sideways: false,
        },
        Menu {
            name: "dialogue",
            setup: &[
                "skip-splash",
                "key ui.MENU_PREV",
                "key movement.MOVE_EAST",
                "key action.INTERACT",
            ],
            window: true,
            sideways: false,
        },
        Menu {
            name: "combat moves",
            setup: &[
                "skip-splash",
                "key ui.MENU_PREV",
                "cheat tp 0 0 0",
                "cheat spawn_enemy feral_dog",
                "key movement.WAIT",
            ],
            window: true,
            sideways: false,
        },
        Menu {
            name: "death screen",
            setup: &[
                "skip-splash",
                "key ui.MENU_PREV",
                "cheat tp 0 0 0",
                "cheat suicide",
            ],
            window: true,
            sideways: false,
        },
        Menu {
            name: "stairs prompt",
            setup: &[
                "world factory-showcase",
                "key ui.MENU_PREV",
                "cheat tp 0 5 0",
                "key movement.MOVE_DOWN",
                "cheat giveitem stairs 2",
                "key movement.MOVE_DOWN",
            ],
            window: true,
            sideways: false,
        },
    ]
}

/// A run after `steps`, on a screen big enough to show every menu's rows.
fn run(steps: &[&str]) -> Runner {
    let mut runner = Runner::new(12345, (160, 50));
    runner.chain(steps);
    runner
}

/// The screen, with styles (a selection is often only a highlight), after `keys` from
/// the menu's setup.
fn after(menu: &Menu, keys: &[&str]) -> Buffer {
    let mut runner = run(menu.setup);
    runner.chain(keys);
    runner.buffer().clone()
}

#[test]
fn numpad_and_arrow_keys_move_every_menu_the_same() {
    let mut failures = Vec::new();
    for menu in menus() {
        let before = run(menu.setup).buffer().clone();
        let (numpad_key, arrow_key) = if menu.sideways {
            ("key movement.MOVE_EAST", "key ui.MENU_NEXT")
        } else {
            ("key movement.MOVE_SOUTH", "key ui.MENU_DOWN")
        };
        let numpad = after(&menu, &[numpad_key]);
        let arrow = after(&menu, &[arrow_key]);
        if numpad == before {
            failures.push(format!("{}: {numpad_key} didn't move it", menu.name));
        }
        if arrow != numpad {
            failures.push(format!(
                "{}: {arrow_key} differs from {numpad_key}",
                menu.name
            ));
        }
        let numpad_up = after(
            &menu,
            &["key movement.MOVE_SOUTH", "key movement.MOVE_NORTH"],
        );
        let arrow_up = after(&menu, &["key ui.MENU_DOWN", "key ui.MENU_UP"]);
        if arrow_up != numpad_up {
            failures.push(format!("{}: the Up arrow differs from numpad 8", menu.name));
        }
    }
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}

#[test]
fn open_windows_keep_left_and_right() {
    let mut failures = Vec::new();
    for menu in menus().into_iter().filter(|m| m.window) {
        for key in ["key ui.MENU_PREV", "key ui.MENU_NEXT"] {
            let mut runner = run(menu.setup);
            let tab = runner.app.ui.current_tab;
            runner.chain(&[key]);
            if runner.app.ui.current_tab != tab {
                failures.push(format!("{}: {key} switched tabs behind it", menu.name));
            }
        }
        let numpad = after(&menu, &["key movement.MOVE_WEST"]);
        let arrow = after(&menu, &["key ui.MENU_PREV"]);
        if arrow != numpad {
            failures.push(format!(
                "{}: the Left arrow differs from numpad 4",
                menu.name
            ));
        }
    }
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}

#[test]
fn quests_move_with_both_key_sets() {
    let open = || {
        let mut runner = run(&["skip-splash", "key ui.MENU_NEXT 2"]);
        for quest in [
            lithicrivers_core::quests::replace_arm(None),
            lithicrivers_core::dialogue::ActiveQuest::new(
                lithicrivers_core::dialogue::QuestType::RepairBrokenAndroid,
                "Repair the Broken Android".to_string(),
                String::new(),
                vec![],
                None,
            ),
        ] {
            runner.app.core.game.res.start_quest(quest);
        }
        runner
    };
    for key in ["key movement.MOVE_SOUTH", "key ui.MENU_DOWN"] {
        let mut runner = open();
        runner.chain(&[key]);
        assert_eq!(runner.app.panels.quests.selected_quest_index, 1, "{key}");
    }
}

//! Input for the teleporter window: pick a destination and jump, or rename the teleporter.
//! Transit stations use the same window, without renaming.

use crate::App;
use crossterm::event::KeyCode;
use lithicrivers_core::teleporter::{self, Teleporter};

/// Handle a key while the teleporter window is open. Returns false if it isn't open.
pub fn handle_teleporter_window_input(app: &mut App, key: KeyCode) -> bool {
    let Some(mut window) = app.panels.teleporter_window.clone() else {
        return false;
    };
    let keys = &app.ui.keybinds;
    if let Some(name) = &mut window.renaming {
        if keys.matches("teleporter", "END_RENAME", &key) {
            let name = name.clone();
            window.renaming = None;
            if teleporter::rename(&mut app.core.game.world, window.entity, &name) {
                app.core
                    .game
                    .res
                    .log(format!("Teleporter renamed to {}", name.trim()));
            }
        } else if keys.matches("teleporter", "DELETE_CHAR", &key) {
            name.pop();
        } else if keys.matches("ui", "CLOSE_HELP_MENU", &key) {
            window.renaming = None;
        } else if let KeyCode::Char(c) = key {
            if name.chars().count() < teleporter::MAX_NAME_CHARS {
                name.push(c);
            }
        }
        app.panels.teleporter_window = Some(window);
        return true;
    }
    let destinations = teleporter::destinations(&app.core.game.world, window.entity);
    if keys.matches("ui", "CLOSE_HELP_MENU", &key) {
        app.panels.teleporter_window = None;
        return true;
    }
    if keys.menu_up(&key) {
        window.selected = window.selected.saturating_sub(1);
    } else if keys.menu_down(&key) {
        window.selected = (window.selected + 1).min(destinations.len().saturating_sub(1));
    } else if keys.matches("teleporter", "RENAME", &key)
        && app
            .core
            .game
            .world
            .get::<&Teleporter>(window.entity)
            .is_ok()
    {
        window.renaming = app
            .core
            .game
            .world
            .get::<&Teleporter>(window.entity)
            .ok()
            .map(|t| t.name.clone());
    } else if keys.matches("ui", "MENU_ACTIVATE", &key) || key == KeyCode::Enter {
        if let Some(dest) = destinations.get(window.selected) {
            jump(app, window.entity, dest.entity);
            return true;
        }
    }
    app.panels.teleporter_window = Some(window);
    true
}

/// Jump from `from` to `to`: on success close the window, follow the player with the view
/// and look cursor, and start the arrival rings; otherwise say why.
fn jump(app: &mut App, from: hecs::Entity, to: hecs::Entity) {
    match app.core.game.teleport(from, to) {
        Ok(landed) => {
            app.panels.teleporter_window = None;
            (app.ui.view_x, app.ui.view_y, app.ui.view_z) = (landed.x, landed.y, landed.z);
            app.panels.look.cursor = landed;
            app.ui.teleport_arrival = Some((landed, app.clock.now()));
        }
        Err(e) => app.core.game.res.log_red(e.message()),
    }
}

#[cfg(test)]
mod tests {
    use crate::headless::{chain_commands, Runner as HeadlessRunner};
    use lithicrivers_core::components::Position;
    use lithicrivers_core::factory::{self, Facing, FactoryKind};
    use lithicrivers_core::teleporter::{self, Teleporter, TransitStation};

    fn player(runner: &HeadlessRunner) -> Position {
        runner.app.core.game.get_player_position().unwrap()
    }

    /// Place a block beside nothing in particular, on a cleared tile.
    fn put(runner: &mut HeadlessRunner, kind: FactoryKind, pos: Position) -> hecs::Entity {
        let game = &mut runner.app.core.game;
        game.res.world_state.world.set_tile_cached(
            pos.x,
            pos.y,
            pos.z,
            lithicrivers_core::tiles::TileKind::Grass,
        );
        factory::place(&mut game.world, &mut game.res, kind, pos, Facing::North).unwrap()
    }

    #[test]
    fn a_jump_through_the_window_moves_the_player_and_the_view() {
        let mut runner = chain_commands(&["skip-splash", "key ui.MENU_PREV", "cheat tp 0 0 0"]);
        let start = player(&runner);
        let east = Position {
            x: start.x + 1,
            ..start
        };
        let far = Position {
            x: start.x + 200,
            y: start.y - 50,
            z: start.z,
        };
        let from = put(&mut runner, FactoryKind::Teleporter, east);
        put(&mut runner, FactoryKind::Teleporter, far);
        runner
            .app
            .core
            .game
            .world
            .get::<&mut Teleporter>(from)
            .unwrap()
            .charge_j = teleporter::CAPACITY_J;
        runner.chain(&["key action.INTERACT"]);
        assert!(runner.app.panels.teleporter_window.is_some());
        runner.chain(&["key ui.MENU_ACTIVATE"]);
        assert!(runner.app.panels.teleporter_window.is_none());
        let landed = player(&runner);
        assert_eq!(
            landed,
            Position {
                y: far.y - 1,
                ..far
            }
        );
        assert_eq!(
            (runner.app.ui.view_x, runner.app.ui.view_y),
            (landed.x, landed.y)
        );
        assert!(runner.app.ui.teleport_arrival.is_some());
        let left = runner
            .app
            .core
            .game
            .world
            .get::<&Teleporter>(from)
            .unwrap()
            .charge_j;
        assert_eq!(
            left,
            teleporter::CAPACITY_J - teleporter::jump_cost(east, far)
        );
    }

    #[test]
    fn an_empty_teleporter_says_how_much_more_it_needs() {
        let mut runner = chain_commands(&["skip-splash", "key ui.MENU_PREV", "cheat tp 0 0 0"]);
        let start = player(&runner);
        put(
            &mut runner,
            FactoryKind::Teleporter,
            Position {
                x: start.x + 1,
                ..start
            },
        );
        put(
            &mut runner,
            FactoryKind::Teleporter,
            Position {
                x: start.x + 9,
                ..start
            },
        );
        runner.chain(&["key action.INTERACT", "key ui.MENU_ACTIVATE"]);
        assert_eq!(player(&runner), start);
        runner.chain(&["expect-text \"needs 10.1 MJ more\""]);
        assert!(runner.app.panels.teleporter_window.is_some(), "stays open");
    }

    #[test]
    fn renaming_from_the_window_shows_in_other_lists() {
        let mut runner = chain_commands(&["skip-splash", "key ui.MENU_PREV", "cheat tp 0 0 0"]);
        let start = player(&runner);
        let a = put(
            &mut runner,
            FactoryKind::Teleporter,
            Position {
                x: start.x + 1,
                ..start
            },
        );
        let b = put(
            &mut runner,
            FactoryKind::Teleporter,
            Position {
                x: start.x + 9,
                ..start
            },
        );
        runner.chain(&[
            "key action.INTERACT",
            "key teleporter.RENAME",
            "key teleporter.DELETE_CHAR 12",
        ]);
        assert!(runner.app.typing(), "renaming takes typed keys");
        runner.chain(&["type \"Furnace Row\"", "key teleporter.END_RENAME"]);
        assert!(!runner.app.typing());
        let listed = teleporter::destinations(&runner.app.core.game.world, b);
        assert_eq!(listed[0].entity, a);
        assert_eq!(listed[0].name, "Furnace Row");
    }

    #[test]
    fn a_dormant_station_wakes_on_first_use_without_opening() {
        let mut runner = chain_commands(&["skip-splash", "key ui.MENU_PREV", "cheat tp 0 0 0"]);
        let start = player(&runner);
        let station = put(
            &mut runner,
            FactoryKind::TransitStation,
            Position {
                x: start.x + 1,
                ..start
            },
        );
        runner.chain(&["key action.INTERACT"]);
        assert!(runner.app.panels.teleporter_window.is_none());
        assert!(
            runner
                .app
                .core
                .game
                .world
                .get::<&TransitStation>(station)
                .unwrap()
                .activated
        );
        runner.chain(&[
            "expect-text \"Transit node registered\"",
            "key action.INTERACT",
        ]);
        assert!(runner.app.panels.teleporter_window.is_some());
    }

    #[test]
    fn the_device_jumps_from_the_inventory_tab_on_a_charged_cell() {
        use lithicrivers_core::components::{Inventory, ItemKind, ItemStack};
        let mut runner = chain_commands(&["skip-splash", "key ui.MENU_PREV", "cheat tp 0 0 0"]);
        let start = player(&runner);
        let far = Position {
            x: start.x + 60,
            ..start
        };
        let tp = put(&mut runner, FactoryKind::Teleporter, far);
        let me = runner.app.core.game.get_player_entity().unwrap();
        {
            let mut inv = runner
                .app
                .core
                .game
                .world
                .get::<&mut Inventory>(me)
                .unwrap();
            inv.slots.clear();
            inv.add(ItemKind::MobileTeleAccessDevice, 1);
            inv.add_stack(ItemStack {
                charge_j: Some(20_000_000),
                ..ItemStack::new(ItemKind::StandardPowerCell, 1)
            });
        }
        runner.chain(&["key ui.MENU_NEXT 5", "key inventory.USE_ITEM"]);
        assert_eq!(
            runner
                .app
                .panels
                .teleporter_window
                .as_ref()
                .map(|w| w.entity),
            Some(me)
        );
        runner.chain(&["key ui.MENU_ACTIVATE"]);
        assert_eq!(
            player(&runner),
            Position {
                y: far.y - 1,
                ..far
            }
        );
        let cell = runner
            .app
            .core
            .game
            .world
            .get::<&Inventory>(me)
            .unwrap()
            .slots[1];
        assert_eq!(
            cell.charge_j,
            Some(20_000_000 - teleporter::jump_cost(start, far))
        );
        let _ = tp;
    }
}

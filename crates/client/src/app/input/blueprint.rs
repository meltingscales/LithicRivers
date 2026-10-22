//! Blueprint mode: move a cursor with the place-direction keys (or the mouse), mark two
//! corners to copy an area, and paste the selected blueprint at the cursor. Also the
//! Blueprints window, where blueprints are selected, renamed and deleted.

use crate::app_state::{BlueprintWindow, BuildMode};
use crate::{App, MenuTab};
use crossterm::event::KeyCode;
use lithicrivers_core::blueprints::{self, PasteError};
use lithicrivers_core::components::Position;

const DIRECTIONS: [(&str, (i64, i64)); 8] = [
    ("PLACE_NORTHWEST", (-1, -1)),
    ("PLACE_NORTH", (0, -1)),
    ("PLACE_NORTHEAST", (1, -1)),
    ("PLACE_WEST", (-1, 0)),
    ("PLACE_EAST", (1, 0)),
    ("PLACE_SOUTHWEST", (-1, 1)),
    ("PLACE_SOUTH", (0, 1)),
    ("PLACE_SOUTHEAST", (1, 1)),
];

fn key(app: &App, category: &str, action: &str) -> String {
    app.core
        .config_manager
        .get_printable_key_for_keybind(category, action)
        .to_string()
}

/// Enter Blueprint mode, or leave it for Movement mode.
pub fn toggle_mode(app: &mut App) {
    let build = &mut app.panels.build;
    if build.mode == BuildMode::Blueprint {
        build.mode = BuildMode::Movement;
        app.core.game.res.log("Build mode: Movement");
        return;
    }
    build.mode = BuildMode::Blueprint;
    build.blueprint.cursor = (1, 0);
    build.blueprint.first_corner = None;
    build.blueprint.pending_replace = None;
    build.blueprint.copying = app.core.game.res.blueprints.blueprints.is_empty();
    let hint = if build.blueprint.copying {
        copy_hint(app)
    } else {
        paste_hint(app)
    };
    app.core
        .game
        .res
        .log(format!("Build mode: Blueprint. {hint}"));
}

fn copy_hint(app: &App) -> String {
    format!(
        "Move the cursor ({}/{}/{}/{}) and press {} on two corners to copy an area",
        key(app, "build", "PLACE_NORTH"),
        key(app, "build", "PLACE_WEST"),
        key(app, "build", "PLACE_SOUTH"),
        key(app, "build", "PLACE_EAST"),
        key(app, "build", "PLACE_CENTER"),
    )
}

fn paste_hint(app: &App) -> String {
    let name = app
        .core
        .game
        .res
        .blueprints
        .selected()
        .map_or("", |b| b.name.as_str())
        .to_string();
    format!(
        "Pasting {name}: {} pastes, {} turns, {} copies a new one, {} lists them",
        key(app, "build", "PLACE_CENTER"),
        key(app, "build", "ROTATE_PLACEMENT"),
        key(app, "build", "NEW_BLUEPRINT"),
        key(app, "build", "BLUEPRINT_LIBRARY"),
    )
}

/// The tile under the blueprint cursor.
pub fn cursor_tile(app: &App) -> Option<Position> {
    let player = app.core.game.get_player_position()?;
    let (dx, dy) = app.panels.build.blueprint.cursor;
    Some(Position {
        x: player.x + dx,
        y: player.y + dy,
        z: player.z,
    })
}

/// Put the cursor on `tile` (from the mouse).
pub fn set_cursor(app: &mut App, tile: Position) {
    if let Some(player) = app.core.game.get_player_position() {
        if tile.z == player.z {
            app.panels.build.blueprint.cursor = (tile.x - player.x, tile.y - player.y);
        }
    }
}

/// The selected blueprint, turned the way the player has turned it.
pub fn paste_preview(app: &App) -> Option<blueprints::Blueprint> {
    let tool = &app.panels.build.blueprint;
    if tool.copying {
        return None;
    }
    let library = &app.core.game.res.blueprints;
    library.selected().map(|b| b.rotated(tool.rotation))
}

/// Keys in Blueprint mode. Returns false for keys it leaves alone (moving the player).
pub fn handle_blueprint_input(app: &mut App, key_code: KeyCode) -> bool {
    if app.ui.current_tab != MenuTab::World || app.panels.build.mode != BuildMode::Blueprint {
        return false;
    }
    let keys = &app.ui.keybinds;
    if let Some(&(_, (dx, dy))) = DIRECTIONS
        .iter()
        .find(|(action, _)| keys.matches("build", action, &key_code))
    {
        let tool = &mut app.panels.build.blueprint;
        tool.cursor = (tool.cursor.0 + dx, tool.cursor.1 + dy);
        tool.pending_replace = None;
        return true;
    }
    if keys.matches("build", "PLACE_CENTER", &key_code) {
        if let Some(tile) = cursor_tile(app) {
            act_at(app, tile);
        }
    } else if keys.matches("build", "ROTATE_PLACEMENT", &key_code) {
        let tool = &mut app.panels.build.blueprint;
        if tool.copying {
            return true;
        }
        tool.rotation = (tool.rotation + 1) % 4;
        tool.pending_replace = None;
        app.core.game.res.log("Turned the blueprint clockwise");
    } else if keys.matches("build", "NEW_BLUEPRINT", &key_code) {
        let tool = &mut app.panels.build.blueprint;
        tool.copying = true;
        tool.first_corner = None;
        let hint = copy_hint(app);
        app.core.game.res.log(hint);
    } else if keys.matches("build", "BLUEPRINT_LIBRARY", &key_code) {
        app.panels.blueprint_window = Some(BlueprintWindow::default());
    } else if keys.matches("ui", "CLOSE_HELP_MENU", &key_code) && app.panels.build.blueprint.copying
    {
        let tool = &mut app.panels.build.blueprint;
        if tool.first_corner.take().is_some() || app.core.game.res.blueprints.blueprints.is_empty()
        {
            app.core.game.res.log("Cleared the first corner");
        } else {
            tool.copying = false;
            let hint = paste_hint(app);
            app.core.game.res.log(hint);
        }
    } else {
        return false;
    }
    true
}

/// Mark a corner, or paste, at `tile`: what the place key or a click does in Blueprint mode.
pub fn act_at(app: &mut App, tile: Position) {
    if app.panels.build.blueprint.copying {
        mark_corner(app, tile);
    } else {
        paste_at(app, tile);
    }
}

fn mark_corner(app: &mut App, tile: Position) {
    let Some(first) = app.panels.build.blueprint.first_corner else {
        app.panels.build.blueprint.first_corner = Some(tile);
        app.core.game.res.log(format!(
            "First corner at ({}, {}); press {} on the opposite corner",
            tile.x,
            tile.y,
            key(app, "build", "PLACE_CENTER")
        ));
        return;
    };
    let game = &mut app.core.game;
    let name = game.res.blueprints.next_name();
    let blueprint = blueprints::capture(&game.world, &mut game.res, first, tile, name);
    let tool = &mut app.panels.build.blueprint;
    tool.first_corner = None;
    if blueprint.blocks.is_empty() {
        game.res.log("Nothing you built is in that area");
        return;
    }
    let message = format!(
        "Saved {} ({}x{}, {} blocks)",
        blueprint.name,
        blueprint.width,
        blueprint.height,
        blueprint.blocks.len()
    );
    game.res.blueprints.add(blueprint);
    tool.copying = false;
    tool.rotation = 0;
    game.res.log(message);
}

fn paste_at(app: &mut App, tile: Position) {
    let Some(blueprint) = paste_preview(app) else {
        app.core
            .game
            .res
            .log("No blueprint selected: copy an area first");
        return;
    };
    let Some(player) = app.core.game.get_player_entity() else {
        return;
    };
    let rotation = app.panels.build.blueprint.rotation;
    let confirmed = app.panels.build.blueprint.pending_replace == Some((tile, rotation));
    let game = &mut app.core.game;
    match blueprints::paste(
        &mut game.world,
        &mut game.res,
        &blueprint,
        tile,
        player,
        confirmed,
    ) {
        Ok(n) => {
            app.panels.build.blueprint.pending_replace = None;
            game.res
                .log(format!("Pasted {} ({n} blocks)", blueprint.name));
        }
        Err(e) => {
            app.panels.build.blueprint.pending_replace =
                matches!(e, PasteError::NeedsConfirm(_)).then_some((tile, rotation));
            game.res.log(e.message());
        }
    }
}

/// Keys while the Blueprints window is open. Returns false if it isn't open.
pub fn handle_blueprint_window_input(app: &mut App, key_code: KeyCode) -> bool {
    let Some(mut window) = app.panels.blueprint_window.clone() else {
        return false;
    };
    let keys = &app.ui.keybinds;
    let library = &mut app.core.game.res.blueprints;
    if let Some(name) = &mut window.renaming {
        if keys.matches("blueprints", "END_RENAME", &key_code) {
            let name = name.trim().to_string();
            if let Some(b) = library.blueprints.get_mut(library.selected) {
                if !name.is_empty() {
                    b.name = name;
                }
            }
            window.renaming = None;
        } else if keys.matches("blueprints", "DELETE_CHAR", &key_code) {
            name.pop();
        } else if keys.matches("ui", "CLOSE_HELP_MENU", &key_code) {
            window.renaming = None;
        } else if let KeyCode::Char(c) = key_code {
            if name.chars().count() < MAX_NAME {
                name.push(c);
            }
        }
        app.panels.blueprint_window = Some(window);
        return true;
    }
    let count = library.blueprints.len();
    if keys.matches("ui", "CLOSE_HELP_MENU", &key_code) {
        app.panels.blueprint_window = None;
        return true;
    } else if keys.matches("movement", "MOVE_NORTH", &key_code)
        || keys.matches("ui", "CREDITS_SCROLL_UP", &key_code)
    {
        library.selected = library.selected.saturating_sub(1);
    } else if keys.matches("movement", "MOVE_SOUTH", &key_code)
        || keys.matches("ui", "CREDITS_SCROLL_DOWN", &key_code)
    {
        library.selected = (library.selected + 1).min(count.saturating_sub(1));
    } else if keys.matches("ui", "MENU_ACTIVATE", &key_code) {
        app.panels.blueprint_window = None;
        if count > 0 {
            let tool = &mut app.panels.build.blueprint;
            tool.copying = false;
            tool.rotation = 0;
            tool.pending_replace = None;
            let hint = paste_hint(app);
            app.core.game.res.log(hint);
        }
        return true;
    } else if keys.matches("blueprints", "RENAME", &key_code) {
        window.renaming = library.selected().map(|b| b.name.clone());
    } else if keys.matches("blueprints", "DELETE", &key_code) {
        if let Some(removed) = library.remove_selected() {
            app.core.game.res.log(format!("Deleted {}", removed.name));
        }
    }
    app.panels.blueprint_window = Some(window);
    true
}

/// The longest name a blueprint can have.
pub const MAX_NAME: usize = 24;

#[cfg(test)]
mod tests {
    use crate::headless::Runner;
    use lithicrivers_core::components::{Inventory, ItemKind, Position};
    use lithicrivers_core::factory::{Facing, FactoryBlock, FactoryKind};

    /// The snapshot script's build and copy: Blueprint 1 holds three east belts at
    /// (-1..=1, -1) and a chest at (1, 0); the cursor ends on the chest.
    fn copied() -> Runner {
        let script = include_str!("../../../../../scripts/snapshots/blueprint-paste-preview.txt");
        let steps: Vec<&str> = script
            .lines()
            .skip_while(|l| !l.starts_with("world"))
            .take_while(|l| !l.starts_with("expect-text \"Saved"))
            .filter(|l| !l.starts_with('#'))
            .collect();
        let mut runner = Runner::new(12345, (100, 30));
        runner.chain(&steps);
        assert_eq!(runner.app.core.game.res.blueprints.blueprints.len(), 1);
        runner
    }

    fn block(runner: &Runner, x: i64, y: i64) -> Option<FactoryBlock> {
        let game = &runner.app.core.game;
        let e = *game.res.factory_index.get(&Position { x, y, z: 0 })?;
        game.world.get::<&FactoryBlock>(e).ok().map(|b| *b)
    }

    fn player_has(runner: &Runner, kind: ItemKind) -> u32 {
        let game = &runner.app.core.game;
        let player = game.get_player_entity().unwrap();
        game.world
            .get::<&Inventory>(player)
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

    #[test]
    fn pasting_over_a_chest_asks_first_then_refunds_it() {
        let mut runner = copied();
        // Something in the chest, to see it come back
        let chest = runner.app.core.game.res.factory_index[&Position { x: 1, y: 0, z: 0 }];
        runner
            .app
            .core
            .game
            .world
            .get::<&mut Inventory>(chest)
            .unwrap()
            .add(ItemKind::Log, 2);
        let logs = player_has(&runner, ItemKind::Log);
        // Back over the original build: all four blocks are replaced
        runner.chain(&[
            "key build.PLACE_WEST 2",
            "key build.PLACE_NORTH",
            "key build.PLACE_CENTER",
        ]);
        assert!(last_message(&runner)
            .ends_with("Pasting here replaces 4 blocks you built; paste again to confirm"));
        // Moving the cursor away and back asks again
        runner.chain(&[
            "key build.PLACE_EAST",
            "key build.PLACE_WEST",
            "key build.PLACE_CENTER",
        ]);
        assert!(last_message(&runner).contains("paste again to confirm"));
        runner.chain(&["key build.PLACE_CENTER"]);
        assert!(
            last_message(&runner).ends_with("Pasted Blueprint 1 (4 blocks)"),
            "{}",
            last_message(&runner)
        );
        assert_eq!(
            player_has(&runner, ItemKind::Log),
            logs + 2,
            "the chest's contents come back"
        );
        assert_eq!(player_has(&runner, ItemKind::WoodenConveyor), 3);
        assert_eq!(player_has(&runner, ItemKind::Chest), 1);
        assert_eq!(
            block(&runner, 1, 0).map(|b| b.kind),
            Some(FactoryKind::Chest)
        );
    }

    #[test]
    fn a_turned_paste_runs_down_from_the_cursor() {
        let mut runner = copied();
        // Turned once, the blueprint is 2 wide and 3 tall: the belts run down its right
        // column and the chest sits bottom left. Its corner goes at (-2, 2)
        runner.chain(&[
            "key build.PLACE_WEST 3",
            "key build.PLACE_SOUTH 2",
            "key build.ROTATE_PLACEMENT",
            "key build.PLACE_CENTER",
        ]);
        assert!(
            last_message(&runner).ends_with("Pasted Blueprint 1 (4 blocks)"),
            "{}",
            last_message(&runner)
        );
        for y in 2..=4 {
            assert_eq!(
                block(&runner, -1, y).map(|b| (b.kind, b.facing)),
                Some((FactoryKind::Belt, Facing::South)),
                "y = {y}"
            );
        }
        assert_eq!(
            block(&runner, -2, 4).map(|b| b.kind),
            Some(FactoryKind::Chest)
        );
    }

    #[test]
    fn a_short_inventory_says_what_is_missing() {
        let mut runner = copied();
        runner.chain(&[
            "key build.PLACE_WEST 2",
            "key build.PLACE_SOUTH 2",
            "key build.PLACE_CENTER",
        ]);
        runner.chain(&["key build.PLACE_SOUTH 2", "key build.PLACE_CENTER"]);
        assert!(
            last_message(&runner)
                .ends_with("Not enough to paste: need 1 more chest, 3 more wooden conveyors"),
            "{}",
            last_message(&runner)
        );
    }

    #[test]
    fn the_window_renames_selects_and_deletes() {
        let mut runner = copied();
        runner.chain(&[
            "key build.BLUEPRINT_LIBRARY",
            "key blueprints.RENAME",
            "key blueprints.DELETE_CHAR 11",
            "type \"Belt run\"",
            "key blueprints.END_RENAME",
            "expect-text \"Belt run  3x2\"",
        ]);
        assert_eq!(
            runner.app.core.game.res.blueprints.blueprints[0].name,
            "Belt run"
        );
        runner.chain(&[
            "key blueprints.DELETE",
            "expect-text \"no blueprints yet\"",
            "key ui.CLOSE_HELP_MENU",
        ]);
        assert!(runner.app.panels.blueprint_window.is_none());
    }

    #[test]
    fn the_mouse_moves_the_cursor_and_clicks_paste() {
        let mut runner = copied();
        let _ = runner.screen_text();
        // The player's cell at 100x30; two tiles down is open ground
        let (col, row) = (38, 12);
        runner.chain(&[&format!("move {col} {row}")]);
        assert_eq!(runner.app.panels.build.blueprint.cursor, (0, 2));
        runner.chain(&[&format!("click {col} {row} left")]);
        assert!(
            last_message(&runner).ends_with("Pasted Blueprint 1 (4 blocks)"),
            "{}",
            last_message(&runner)
        );
        assert_eq!(
            block(&runner, 1, 2).map(|b| b.kind),
            Some(FactoryKind::Belt)
        );
    }

    #[test]
    fn the_library_survives_save_reload() {
        let mut runner = copied();
        runner.chain(&["save-reload"]);
        assert_eq!(runner.app.core.game.res.blueprints.blueprints.len(), 1);
    }
}

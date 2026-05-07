use crate::{App, MenuTab};
use crossterm::event::KeyCode;
use std::error::Error;

/// Handle look mode input when on World tab - returns true if input was handled
pub fn handle_look_mode_input(app: &mut App, key: KeyCode) -> Result<bool, Box<dyn Error>> {
    // Only handle input when on the World tab or Tutorial tab (for tutorial functionality)
    if app.ui.current_tab != MenuTab::World && app.ui.current_tab != MenuTab::Tutorial {
        return Ok(false);
    }

    // Toggle Look mode
    if app.ui.keybinds.matches("action", "LOOK_TOGGLE", &key) {
        app.panels.look.mode = !app.panels.look.mode;
        // Reset cursor to player on toggle on
        if app.panels.look.mode {
            if let Some(e) = app.core.game.get_player_entity() {
                if let Ok(pos) = app
                    .core
                    .game
                    .world
                    .get::<&lithicrivers_core::components::Position>(e)
                {
                    app.panels.look.cursor = *pos;
                    // Align all view coords to cursor
                    app.ui.view_x = app.panels.look.cursor.x;
                    app.ui.view_y = app.panels.look.cursor.y;
                    app.ui.view_z = app.panels.look.cursor.z;
                }
            }
            app.core.game.res.log("Look mode: ON");
        } else {
            app.core.game.res.log("Look mode: OFF");
        }
        return Ok(true);
    }

    // In Look mode, remap movement keys to move the look cursor without ticking
    if app.panels.look.mode {
        let mut moved = false;
        if app.ui.keybinds.matches("movement", "MOVE_NORTH", &key) {
            app.panels.look.cursor.y -= 1;
            app.ui.view_y = app.panels.look.cursor.y;
            moved = true;
        } else if app.ui.keybinds.matches("movement", "MOVE_SOUTH", &key) {
            app.panels.look.cursor.y += 1;
            app.ui.view_y = app.panels.look.cursor.y;
            moved = true;
        } else if app.ui.keybinds.matches("movement", "MOVE_WEST", &key) {
            app.panels.look.cursor.x -= 1;
            app.ui.view_x = app.panels.look.cursor.x;
            moved = true;
        } else if app.ui.keybinds.matches("movement", "MOVE_EAST", &key) {
            app.panels.look.cursor.x += 1;
            app.ui.view_x = app.panels.look.cursor.x;
            moved = true;
        } else if app.ui.keybinds.matches("movement", "MOVE_NORTHWEST", &key) {
            app.panels.look.cursor.x -= 1;
            app.panels.look.cursor.y -= 1;
            app.ui.view_x = app.panels.look.cursor.x;
            app.ui.view_y = app.panels.look.cursor.y;
            moved = true;
        } else if app.ui.keybinds.matches("movement", "MOVE_NORTHEAST", &key) {
            app.panels.look.cursor.x += 1;
            app.panels.look.cursor.y -= 1;
            app.ui.view_x = app.panels.look.cursor.x;
            app.ui.view_y = app.panels.look.cursor.y;
            moved = true;
        } else if app.ui.keybinds.matches("movement", "MOVE_SOUTHWEST", &key) {
            app.panels.look.cursor.x -= 1;
            app.panels.look.cursor.y += 1;
            app.ui.view_x = app.panels.look.cursor.x;
            app.ui.view_y = app.panels.look.cursor.y;
            moved = true;
        } else if app.ui.keybinds.matches("movement", "MOVE_SOUTHEAST", &key) {
            app.panels.look.cursor.x += 1;
            app.panels.look.cursor.y += 1;
            app.ui.view_x = app.panels.look.cursor.x;
            app.ui.view_y = app.panels.look.cursor.y;
            moved = true;
        } else if app.ui.keybinds.matches("movement", "WAIT", &key) {
            // no-op, but treat as handled to avoid player waiting
            moved = true;
        } else if app.ui.keybinds.matches("movement", "MOVE_UP", &key) {
            app.panels.look.cursor.z += 1;
            app.ui.view_z = app.panels.look.cursor.z;
            moved = true;
        } else if app.ui.keybinds.matches("movement", "MOVE_DOWN", &key) {
            app.panels.look.cursor.z -= 1;
            app.ui.view_z = app.panels.look.cursor.z;
            moved = true;
        } else if app.ui.keybinds.matches("viewport", "VIEW_Z_UP", &key) {
            app.panels.look.cursor.z = app.panels.look.cursor.z.saturating_add(1);
            app.ui.view_z = app.panels.look.cursor.z;
            moved = true;
        } else if app.ui.keybinds.matches("viewport", "VIEW_Z_DOWN", &key) {
            app.panels.look.cursor.z = app.panels.look.cursor.z.saturating_sub(1);
            app.ui.view_z = app.panels.look.cursor.z;
            moved = true;
        }

        if moved {
            // Prefetch around the new cursor position for smoother draw
            let radius = 20i64;
            let left = app.panels.look.cursor.x - radius;
            let top = app.panels.look.cursor.y - radius;
            let right = app.panels.look.cursor.x + radius;
            let bottom = app.panels.look.cursor.y + radius;
            app.core.game.res.world_state.world.prefetch_rect_at_z(
                left,
                top,
                right,
                bottom,
                app.panels.look.cursor.z,
            );
            return Ok(true);
        }
    }

    // No look mode input was handled
    Ok(false)
}

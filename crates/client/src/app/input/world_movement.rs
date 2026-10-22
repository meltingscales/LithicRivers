use crate::{App, MenuTab};
use crossterm::event::KeyCode;
use std::error::Error;

/// Handle world movement input when on World tab - returns true if input was handled
pub fn handle_world_movement_input(app: &mut App, key: KeyCode) -> Result<bool, Box<dyn Error>> {
    // Only process movement if we're currently on the world panel or tutorial tab (for tutorial functionality)
    if app.ui.current_tab != MenuTab::World && app.ui.current_tab != MenuTab::Tutorial {
        return Ok(false);
    }

    // Up or down with nothing to climb offers to build stairs instead
    for (action, dz) in [("MOVE_UP", 1), ("MOVE_DOWN", -1)] {
        if app.ui.keybinds.matches("movement", action, &key)
            && crate::app::input::stairs_prompt::before_climb(app, dz)
        {
            return Ok(true);
        }
    }

    if app.ui.keybinds.matches_movement(&key) {
        if app.ui.keybinds.matches("movement", "MOVE_NORTH", &key) {
            app.core.game.queue_player_move(0, -1);
        }
        if app.ui.keybinds.matches("movement", "MOVE_SOUTH", &key) {
            app.core.game.queue_player_move(0, 1);
        }
        if app.ui.keybinds.matches("movement", "MOVE_WEST", &key) {
            app.core.game.queue_player_move(-1, 0);
        }
        if app.ui.keybinds.matches("movement", "MOVE_EAST", &key) {
            app.core.game.queue_player_move(1, 0);
        }
        if app.ui.keybinds.matches("movement", "MOVE_NORTHWEST", &key) {
            app.core.game.queue_player_move(-1, -1);
        }
        if app.ui.keybinds.matches("movement", "MOVE_NORTHEAST", &key) {
            app.core.game.queue_player_move(1, -1);
        }
        if app.ui.keybinds.matches("movement", "MOVE_SOUTHWEST", &key) {
            app.core.game.queue_player_move(-1, 1);
        }
        if app.ui.keybinds.matches("movement", "MOVE_SOUTHEAST", &key) {
            app.core.game.queue_player_move(1, 1);
        }
        if app.ui.keybinds.matches("movement", "WAIT", &key) {
            app.core.game.queue_player_move(0, 0);
        }
        if app.ui.keybinds.matches("movement", "MOVE_UP", &key) {
            app.core.game.queue_player_move_z(1);
        }
        if app.ui.keybinds.matches("movement", "MOVE_DOWN", &key) {
            app.core.game.queue_player_move_z(-1);
        }

        let tick_result = app.core.game.tick();

        // Check tutorial progression for any inventory changes after movement/pickup
        if let Some(player_entity) = app.core.game.get_player_entity() {
            if let Ok(inventory) = app
                .core
                .game
                .world
                .get::<&lithicrivers_core::components::Inventory>(player_entity)
            {
                tracing::info!(target: "tutorial", "Movement tick completed, checking tutorial with {} inventory stacks", inventory.slots.len());
                let tutorial_advanced = app.ui.tutorial_system.handle_inventory_change(&inventory);
                tracing::info!(target: "tutorial", "Tutorial check after movement returned: {}", tutorial_advanced);
            }
        }

        // Movement also works on the Tutorial tab; this switches to World if combat starts
        app.apply_tick_result(&tick_result);
        app.snap_view_to_player_z();
        return Ok(true);
    }

    // No world movement input was handled
    Ok(false)
}

use crate::{App, CombatUiState, MenuTab};
use crossterm::event::KeyCode;
use lithicrivers_core::game::GameTickResult;
use std::error::Error;

/// Handle world movement input when on World tab - returns true if input was handled
pub fn handle_world_movement_input(app: &mut App, key: KeyCode) -> Result<bool, Box<dyn Error>> {
    // Only process movement if we're currently on the world panel or tutorial tab (for tutorial functionality)
    if app.ui.current_tab != MenuTab::World && app.ui.current_tab != MenuTab::Tutorial {
        return Ok(false);
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
        if tick_result.contains(GameTickResult::CombatTriggered) {
            app.combat = CombatUiState::Active {
                current_move: 0,
                current_enemy: 0,
                enemy_timers: vec![], // Deprecated - using ActionQueue now
                move_scroll_offset: 0,
            };
        }
        if tick_result.contains(GameTickResult::CombatEnded) {
            app.combat = CombatUiState::None;
        }
        app.snap_view_to_player_z();
        return Ok(true);
    }

    // No world movement input was handled
    Ok(false)
}

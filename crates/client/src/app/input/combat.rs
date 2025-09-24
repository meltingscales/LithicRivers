use crate::{App, CombatUiState};
use crossterm::event::KeyCode;
use lithicrivers_core::{game::GameTickResult, moves::get_available_moves};
use std::error::Error;

/// Handle combat-specific input when combat is active - returns true if input was handled
pub fn handle_combat_input(app: &mut App, key: KeyCode) -> Result<bool, Box<dyn Error>> {
    // Only handle input when combat is active
    let CombatUiState::Active { .. } = &app.combat else {
        return Ok(false);
    };

    // Extract combat state values to avoid borrow conflicts
    let (mut current_move, mut current_enemy, mut move_scroll_offset) = match &app.combat {
        CombatUiState::Active {
            current_move,
            current_enemy,
            move_scroll_offset,
            ..
        } => (*current_move, *current_enemy, *move_scroll_offset),
        _ => unreachable!(),
    };

    // Move selection (Up/Down) - with scrolling support and cursor skipping for disabled moves
    let max_moves = get_available_moves().len();
    const VISIBLE_MOVES: usize = 4; // Number of moves visible at once

    if app.ui.keybinds.matches("movement", "MOVE_NORTH", &key)
        || app.ui.keybinds.matches("movement", "MOVE_UP", &key)
    {
        let new_move = find_previous_usable_move(app, current_move, max_moves);
        current_move = new_move;
        // Scroll up if needed
        if current_move < move_scroll_offset {
            move_scroll_offset = current_move;
        }
        // Scroll to show the last move if we wrapped around
        if new_move == max_moves - 1 && current_move == 0 {
            move_scroll_offset = max_moves.saturating_sub(VISIBLE_MOVES);
        }

        // Update combat state
        if let CombatUiState::Active {
            current_move: ref mut state_move,
            move_scroll_offset: ref mut state_offset,
            ..
        } = app.combat
        {
            *state_move = current_move;
            *state_offset = move_scroll_offset;
        }
        return Ok(true);
    }
    if app.ui.keybinds.matches("movement", "MOVE_SOUTH", &key)
        || app.ui.keybinds.matches("movement", "MOVE_DOWN", &key)
    {
        let new_move = find_next_usable_move(app, current_move, max_moves);
        current_move = new_move;
        // Scroll down if needed
        if current_move >= move_scroll_offset + VISIBLE_MOVES {
            move_scroll_offset = (current_move + 1).saturating_sub(VISIBLE_MOVES);
        }
        // Handle wrap-around to beginning
        if new_move == 0 && current_move == max_moves - 1 {
            move_scroll_offset = 0;
        }

        // Update combat state
        if let CombatUiState::Active {
            current_move: ref mut state_move,
            move_scroll_offset: ref mut state_offset,
            ..
        } = app.combat
        {
            *state_move = current_move;
            *state_offset = move_scroll_offset;
        }
        return Ok(true);
    }

    // Enemy/target selection (Left/Right) - get actual enemy count
    let enemy_count = get_combat_enemy_count(&mut app.core.game);
    if enemy_count > 0 {
        if app.ui.keybinds.matches("movement", "MOVE_WEST", &key) {
            current_enemy = if current_enemy == 0 {
                enemy_count - 1
            } else {
                current_enemy - 1
            };

            // Update combat state
            if let CombatUiState::Active {
                current_enemy: ref mut state_enemy,
                ..
            } = app.combat
            {
                *state_enemy = current_enemy;
            }
            return Ok(true);
        }
        if app.ui.keybinds.matches("movement", "MOVE_EAST", &key) {
            current_enemy = (current_enemy + 1) % enemy_count;

            // Update combat state
            if let CombatUiState::Active {
                current_enemy: ref mut state_enemy,
                ..
            } = app.combat
            {
                *state_enemy = current_enemy;
            }
            return Ok(true);
        }
    }

    // Use selected move (Space/Enter) - queue the action instead of immediate execution
    if app.ui.keybinds.matches("ui", "MENU_ACTIVATE", &key) {
        // Check if combat is actually active before allowing move queuing
        if !app.core.game.res.player_state.combat_active {
            app.core
                .game
                .res
                .log("Cannot use moves - not in combat!".to_string());
            return Ok(true);
        }

        // Get the player entity
        if let Some(player_entity) = app.core.game.get_player_entity() {
            // Get available moves and selected move
            let available_moves = lithicrivers_core::moves::get_available_moves();
            if let Some(selected_move) = available_moves.get(current_move) {
                // Check if player can use this move considering pending energy consumption
                let can_use = if let Ok(energy) =
                    app.core
                        .game
                        .world
                        .get::<&lithicrivers_core::components::Energy>(player_entity)
                {
                    let pending_energy = calculate_pending_energy_consumption(app);
                    let available_energy = energy.current.saturating_sub(pending_energy);
                    available_energy >= selected_move.energy_cost
                } else {
                    false
                };

                if !can_use {
                    app.core
                        .game
                        .res
                        .log("Not enough energy (including queued moves)!".to_string());
                    return Ok(true);
                }

                // Find target entity if needed
                let target_entity =
                    if selected_move.move_type != lithicrivers_core::moves::MoveType::Escape {
                        // Not escape move - need a target
                        // Find the actual entity for the selected enemy
                        let mut enemy_count = 0;
                        let mut target = None;
                        for (entity, (_, combat, _)) in app
                            .core
                            .game
                            .world
                            .query::<(
                                &lithicrivers_core::components::Position,
                                &lithicrivers_core::components::Combat,
                                &lithicrivers_core::components::GameEntity,
                            )>()
                            .iter()
                        {
                            if entity == player_entity || !combat.triggered {
                                continue;
                            }
                            // Skip dead enemies
                            if app
                                .core
                                .game
                                .world
                                .get::<&lithicrivers_core::components::Dead>(entity)
                                .is_ok()
                            {
                                continue;
                            }
                            if enemy_count == current_enemy {
                                target = Some(entity);
                                break;
                            }
                            enemy_count += 1;
                        }

                        // Validate we have a target for moves that need one
                        if target.is_none() {
                            app.core
                                .game
                                .res
                                .log("No valid target for this move!".to_string());
                            return Ok(true);
                        }
                        target
                    } else {
                        None
                    };

                // Create queued action
                let execution_time = selected_move.execution_time_ticks;
                let action = lithicrivers_core::moves::QueuedAction {
                    entity: player_entity,
                    action: lithicrivers_core::moves::CombatAction::PlayerMove {
                        move_data: selected_move.clone(),
                        target_entity,
                        target_position: None, // Not used for current moves
                    },
                    execution_time_ticks: execution_time,
                    remaining_time_ticks: execution_time,
                };

                // Ensure player has an action queue component
                if app
                    .core
                    .game
                    .world
                    .get::<&lithicrivers_core::moves::ActionQueue>(player_entity)
                    .is_err()
                {
                    app.core
                        .game
                        .world
                        .insert_one(player_entity, lithicrivers_core::moves::ActionQueue::new())
                        .ok();
                }

                // Queue the action
                if let Ok(mut queue) = app
                    .core
                    .game
                    .world
                    .get::<&mut lithicrivers_core::moves::ActionQueue>(player_entity)
                {
                    queue.queue_action(action);

                    // If no current action, start this one
                    if queue.current_action.is_none() {
                        queue.start_next_action();
                    }

                    app.core.game.res.log(format!(
                        "Queued {} against target {}",
                        selected_move.name, current_enemy
                    ));
                }
            } else {
                app.core.game.res.log("Invalid move selected!".to_string());
            }

            // Process game ticks to allow combat actions to execute and check for combat end
            // We may need multiple ticks for actions to complete
            for _ in 0..10 {
                // Max 10 ticks to prevent infinite loops
                let tick_result = app.core.game.tick();
                if tick_result.contains(GameTickResult::CombatEnded) {
                    app.combat = CombatUiState::None;
                    break;
                }
                // If nothing significant happened, stop ticking
                if tick_result.contains(GameTickResult::NoAction) {
                    break;
                }
            }
        }
        return Ok(true);
    }

    // Clear move queue (C key)
    if app.ui.keybinds.matches("combat", "CLEAR_MOVE_QUEUE", &key) {
        if let Some(player_entity) = app.core.game.get_player_entity() {
            if let Ok(mut queue) = app
                .core
                .game
                .world
                .get::<&mut lithicrivers_core::moves::ActionQueue>(player_entity)
            {
                queue.clear();
                app.core.game.res.log("Cleared move queue");
            }
        }
        return Ok(true);
    }

    // Manual combat tick (NUMPAD_5 / Wait key)
    if app.ui.keybinds.matches("movement", "WAIT", &key) {
        // Process a single game tick to advance combat
        let tick_result = app.core.game.tick();
        if tick_result.contains(GameTickResult::CombatEnded) {
            app.combat = CombatUiState::None;
        }
        app.core.game.res.log("Advanced combat timing");
        return Ok(true);
    }

    // Exit combat (Escape)
    if app.ui.keybinds.matches("ui", "CLOSE_HELP_MENU", &key) {
        // Clear the player's action queue when manually exiting combat
        if let Some(player_entity) = app.core.game.get_player_entity() {
            if let Ok(mut queue) = app
                .core
                .game
                .world
                .get::<&mut lithicrivers_core::moves::ActionQueue>(player_entity)
            {
                queue.clear();
                app.core.game.res.log("Cleared action queue on combat exit");
            }
        }

        // End combat for all nearby enemies (this was missing!)
        app.core.game.end_combat_around_player();

        app.combat = CombatUiState::None;
        app.core.game.res.log("Exited combat (cheat mode)");
        return Ok(true);
    }

    // If we reach here, input was handled but no specific action taken
    // Block all other input during combat
    Ok(true)
}

/// Get the current count of living combat enemies adjacent to the player
fn get_combat_enemy_count(game: &mut lithicrivers_core::Game) -> usize {
    use lithicrivers_core::components::{Combat, GameEntity, Position};

    let mut count = 0;

    if let Some(player_entity) = game.get_player_entity() {
        if let Ok(player_pos) = game.world.get::<&Position>(player_entity) {
            let player_pos = *player_pos;

            // Define the 8 adjacent positions around the player (same as combat_trigger_system)
            let adjacent_positions = [
                (player_pos.x - 1, player_pos.y - 1), // NW
                (player_pos.x, player_pos.y - 1),     // N
                (player_pos.x + 1, player_pos.y - 1), // NE
                (player_pos.x - 1, player_pos.y),     // W
                (player_pos.x + 1, player_pos.y),     // E
                (player_pos.x - 1, player_pos.y + 1), // SW
                (player_pos.x, player_pos.y + 1),     // S
                (player_pos.x + 1, player_pos.y + 1), // SE
            ];

            // Look for adjacent combat entities that are alive and in combat
            for (entity, (pos, combat, _)) in game
                .world
                .query::<(&Position, &Combat, &GameEntity)>()
                .iter()
            {
                if entity == player_entity {
                    continue; // Skip player
                }

                // Check if enemy is in any of the 8 adjacent positions
                let is_adjacent = adjacent_positions.iter().any(|&(adj_x, adj_y)| {
                    pos.x == adj_x && pos.y == adj_y && pos.z == player_pos.z
                });

                if is_adjacent && combat.triggered {
                    // Skip dead enemies
                    if game
                        .world
                        .get::<&lithicrivers_core::components::Dead>(entity)
                        .is_ok()
                    {
                        continue;
                    }

                    count += 1;
                }
            }
        }
    }

    count
}

/// Calculate total pending energy consumption from queued moves
fn calculate_pending_energy_consumption(app: &App) -> u32 {
    let mut total_pending = 0;

    if let Some(player_entity) = app.core.game.get_player_entity() {
        if let Ok(queue) = app
            .core
            .game
            .world
            .get::<&lithicrivers_core::moves::ActionQueue>(player_entity)
        {
            // Check current action
            if let Some(current_action) = &queue.current_action {
                if let lithicrivers_core::moves::CombatAction::PlayerMove { move_data, .. } =
                    &current_action.action
                {
                    total_pending += move_data.energy_cost;
                }
            }

            // Check queued actions
            for action in queue.get_queued_actions() {
                if let lithicrivers_core::moves::CombatAction::PlayerMove { move_data, .. } =
                    &action.action
                {
                    total_pending += move_data.energy_cost;
                }
            }
        }
    }

    total_pending
}

/// Check if a move is usable based on player's current energy minus pending consumption
fn is_move_usable(app: &App, move_index: usize) -> bool {
    // Get the player's current energy
    if let Some(player_entity) = app.core.game.get_player_entity() {
        if let Ok(energy) = app
            .core
            .game
            .world
            .get::<&lithicrivers_core::components::Energy>(player_entity)
        {
            // Get the available moves
            let available_moves = lithicrivers_core::moves::get_available_moves();
            if let Some(selected_move) = available_moves.get(move_index) {
                // Calculate available energy after pending consumption
                let pending_energy = calculate_pending_energy_consumption(app);
                let available_energy = energy.current.saturating_sub(pending_energy);
                return available_energy >= selected_move.energy_cost;
            }
        }
    }
    false
}

/// Find the next usable move when navigating down
fn find_next_usable_move(app: &App, current_move: usize, max_moves: usize) -> usize {
    let start_move = (current_move + 1) % max_moves;
    let mut check_move = start_move;

    // Try to find a usable move within max_moves attempts
    for _ in 0..max_moves {
        if is_move_usable(app, check_move) {
            return check_move;
        }
        check_move = (check_move + 1) % max_moves;
    }

    // If no usable move found, return the next move anyway (fallback)
    start_move
}

/// Find the previous usable move when navigating up
fn find_previous_usable_move(app: &App, current_move: usize, max_moves: usize) -> usize {
    let start_move = if current_move == 0 {
        max_moves - 1
    } else {
        current_move - 1
    };
    let mut check_move = start_move;

    // Try to find a usable move within max_moves attempts
    for _ in 0..max_moves {
        if is_move_usable(app, check_move) {
            return check_move;
        }
        check_move = if check_move == 0 {
            max_moves - 1
        } else {
            check_move - 1
        };
    }

    // If no usable move found, return the previous move anyway (fallback)
    start_move
}

//! Input for the Choose Interaction grid: a movement key picks the tile in that direction (the
//! wait key the player's own tile), and the menu up and down keys walk the whole list.

use crate::app::input_handler::execute_interaction_action;
use crate::app_state::MultiActionSelectState;
use crate::App;
use crossterm::event::KeyCode;
use lithicrivers_core::components::Position;
use std::error::Error;

/// The grid's cells in reading order: each tile's (dx, dy) from the player and the keybind
/// that picks it.
pub const GRID: [((i64, i64), &str, &str); 9] = [
    ((-1, -1), "movement", "MOVE_NORTHWEST"),
    ((0, -1), "movement", "MOVE_NORTH"),
    ((1, -1), "movement", "MOVE_NORTHEAST"),
    ((-1, 0), "movement", "MOVE_WEST"),
    ((0, 0), "movement", "WAIT"),
    ((1, 0), "movement", "MOVE_EAST"),
    ((-1, 1), "movement", "MOVE_SOUTHWEST"),
    ((0, 1), "movement", "MOVE_SOUTH"),
    ((1, 1), "movement", "MOVE_SOUTHEAST"),
];

/// The keybind that picks the tile at `offset`.
pub fn keybind_for(offset: (i64, i64)) -> (&'static str, &'static str) {
    GRID.iter()
        .find(|(o, ..)| *o == offset)
        .map_or(("movement", "WAIT"), |&(_, category, action)| {
            (category, action)
        })
}

pub fn player_position(app: &App) -> Option<Position> {
    let player = app.core.game.get_player_entity()?;
    app.core
        .game
        .world
        .get::<&Position>(player)
        .ok()
        .map(|p| *p)
}

/// Handle a key while the Choose Interaction menu is open. Returns true if it was.
pub fn handle_multi_action_input(app: &mut App, key: KeyCode) -> Result<bool, Box<dyn Error>> {
    let MultiActionSelectState::SelectingAction {
        available_actions,
        selected_action,
        ..
    } = &app.panels.multi_action_select
    else {
        return Ok(false);
    };
    let count = available_actions.len();
    let mut selected = *selected_action;
    let keys = &app.ui.keybinds;

    if keys.matches("ui", "CLOSE_HELP_MENU", &key) {
        app.panels.multi_action_select = MultiActionSelectState::None;
        app.core.game.res.log("Cancelled interaction");
        return Ok(true);
    }
    if let Some(&(offset, ..)) = GRID
        .iter()
        .find(|(_, category, action)| keys.matches(category, action, &key))
    {
        choose_direction(app, offset);
        return Ok(true);
    }
    if keys.matches("ui", "MENU_UP", &key) {
        selected = selected.saturating_sub(1);
    } else if keys.matches("ui", "MENU_DOWN", &key) {
        selected = (selected + 1).min(count.saturating_sub(1));
    } else if keys.matches("ui", "MENU_ACTIVATE", &key) || key == KeyCode::Enter {
        run(app, selected);
        return Ok(true);
    }
    if let MultiActionSelectState::SelectingAction {
        selected_action,
        picked,
        ..
    } = &mut app.panels.multi_action_select
    {
        *selected_action = selected;
        *picked = None;
    }
    Ok(true)
}

/// Pick the tile at `offset` from the player: its only choice runs at once; with several,
/// the first pick selects the first and each further pick the next; with none, nothing
/// happens but a message.
pub fn choose_direction(app: &mut App, offset: (i64, i64)) {
    let MultiActionSelectState::SelectingAction {
        available_actions,
        selected_action,
        picked,
    } = &app.panels.multi_action_select
    else {
        return;
    };
    let Some(player) = player_position(app) else {
        return;
    };
    let there: Vec<usize> = available_actions
        .iter()
        .enumerate()
        .filter(|(_, a)| a.offset_from(player) == offset)
        .map(|(i, _)| i)
        .collect();
    match there.as_slice() {
        [] => app.core.game.res.log("Nothing there"),
        [only] => run(app, *only),
        _ => {
            let next = there
                .iter()
                .position(|&i| i == *selected_action)
                .filter(|_| *picked == Some(offset))
                .map_or(there[0], |p| there[(p + 1) % there.len()]);
            if let MultiActionSelectState::SelectingAction {
                selected_action,
                picked,
                ..
            } = &mut app.panels.multi_action_select
            {
                *selected_action = next;
                *picked = Some(offset);
            }
        }
    }
}

/// Close the menu and run choice `index`.
fn run(app: &mut App, index: usize) {
    let MultiActionSelectState::SelectingAction {
        available_actions, ..
    } = std::mem::take(&mut app.panels.multi_action_select)
    else {
        return;
    };
    if let Some(action) = available_actions.get(index) {
        execute_interaction_action(app, action);
    }
}

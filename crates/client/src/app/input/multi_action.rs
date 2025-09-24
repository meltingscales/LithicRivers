use crate::app::input_handler::execute_interaction_action;
use crate::App;
use crossterm::event::KeyCode;
use std::error::Error;

/// Handle multi-action selection input when active - returns true if input was handled
pub fn handle_multi_action_input(app: &mut App, key: KeyCode) -> Result<bool, Box<dyn Error>> {
    if let crate::app_state::MultiActionSelectState::SelectingAction {
        available_actions,
        selected_action,
    } = &app.panels.multi_action_select
    {
        let actions = available_actions.clone();
        let mut selected = *selected_action;

        if app.ui.keybinds.matches("ui", "CLOSE_HELP_MENU", &key) {
            app.panels.multi_action_select = crate::app_state::MultiActionSelectState::None;
            app.core.game.res.log("Cancelled interaction");
            return Ok(true);
        }

        if app.ui.keybinds.matches("movement", "MOVE_NORTH", &key) {
            selected = selected.saturating_sub(1);
        }

        if app.ui.keybinds.matches("movement", "MOVE_SOUTH", &key) {
            if selected < actions.len().saturating_sub(1) {
                selected += 1;
            }
        }

        if app.ui.keybinds.matches("ui", "MENU_ACTIVATE", &key) || key == KeyCode::Enter {
            if selected < actions.len() {
                let chosen_action = &actions[selected];
                app.panels.multi_action_select = crate::app_state::MultiActionSelectState::None;

                // Execute the chosen interaction
                execute_interaction_action(app, chosen_action);
            }
            return Ok(true);
        }

        app.panels.multi_action_select =
            crate::app_state::MultiActionSelectState::SelectingAction {
                available_actions: actions,
                selected_action: selected,
            };
        return Ok(true);
    }

    Ok(false)
}

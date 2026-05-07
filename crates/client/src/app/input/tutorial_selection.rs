use crate::{App, TutorialSelectionState};
use crossterm::event::KeyCode;
use std::error::Error;

/// Handle tutorial selection modal input - returns true if input was handled
pub fn handle_tutorial_selection_input(
    app: &mut App,
    key: KeyCode,
) -> Result<bool, Box<dyn Error>> {
    // Open tutorial selection modal
    if app.ui.keybinds.matches("ui", "TUTORIAL_SELECT", &key) {
        let available_tutorials =
            crate::tutorialsystem::TutorialDatabase::get_available_tutorials();
        if !available_tutorials.is_empty() {
            app.panels.tutorial_selection = TutorialSelectionState::SelectingTutorial {
                selected_tutorial: 0,
            };
        }
        return Ok(true);
    }

    // Handle modal navigation when active
    if let TutorialSelectionState::SelectingTutorial { selected_tutorial } =
        &mut app.panels.tutorial_selection
    {
        let available_tutorials =
            crate::tutorialsystem::TutorialDatabase::get_available_tutorials();

        if app.ui.keybinds.matches("ui", "CLOSE_HELP_MENU", &key) {
            // ESC - Close modal
            app.panels.tutorial_selection = TutorialSelectionState::None;
            return Ok(true);
        }

        if app.ui.keybinds.matches("ui", "CREDITS_SCROLL_UP", &key) {
            // UP - Previous tutorial
            if *selected_tutorial > 0 {
                *selected_tutorial -= 1;
            } else {
                *selected_tutorial = available_tutorials.len().saturating_sub(1);
            }
            return Ok(true);
        }

        if app.ui.keybinds.matches("ui", "CREDITS_SCROLL_DOWN", &key) {
            // DOWN - Next tutorial
            *selected_tutorial = (*selected_tutorial + 1) % available_tutorials.len();
            return Ok(true);
        }

        if app.ui.keybinds.matches("ui", "MENU_ACTIVATE", &key) {
            // ENTER/SPACE - Start selected tutorial
            if let Some(tutorial) = available_tutorials.get(*selected_tutorial) {
                match tutorial.id.as_str() {
                    "basics" => {
                        let steps = crate::tutorialsystem::TutorialDatabase::get_basics_tutorial();
                        app.ui.tutorial_system.start_tutorial(&tutorial.id, steps);
                    }
                    _ => {
                        // For now, just show a message for unimplemented tutorials
                        app.core.game.res.log(format!(
                            "Tutorial '{}' is not yet implemented",
                            tutorial.title
                        ));
                    }
                }
            }
            app.panels.tutorial_selection = TutorialSelectionState::None;
            return Ok(true);
        }

        return Ok(true); // Consume all input when modal is active
    }

    Ok(false) // Input not handled
}

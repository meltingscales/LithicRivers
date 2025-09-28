use crossterm::event::KeyCode;
use std::error::Error;

use crate::App;

pub fn handle_tutorial_skip_input(app: &mut App, key: KeyCode) -> Result<bool, Box<dyn Error>> {
    // Only handle tutorial skip when there's an active tutorial
    if app.ui.tutorial_system.current_step().is_none() {
        return Ok(false);
    }

    // Check for SHIFT+Y (tutorial skip)
    if app.ui.keybinds.matches("ui", "TUTORIAL_SKIP", &key.into()) {
        // Mark current step as completed and advance
        app.ui.tutorial_system.advance_step();
        return Ok(true);
    }

    Ok(false)
}

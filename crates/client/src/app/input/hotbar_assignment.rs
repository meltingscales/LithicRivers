use crate::app_state::HotbarAssignmentState;
use crate::App;
use crossterm::event::KeyCode;
use std::error::Error;

/// Handle hotbar assignment modal input - returns true if input was handled
pub fn handle_hotbar_assignment_input(app: &mut App, key: KeyCode) -> Result<bool, Box<dyn Error>> {
    if let HotbarAssignmentState::ChoosingBlock {
        hotbar_slot,
        available_blocks,
        selected_block,
    } = &app.panels.hotbar_assignment
    {
        let slot = *hotbar_slot;
        let blocks = available_blocks.clone();
        let mut selected = *selected_block;

        if app.ui.keybinds.matches("ui", "CLOSE_HELP_MENU", &key) {
            app.panels.hotbar_assignment = HotbarAssignmentState::None;
            app.core.game.res.log("Cancelled block assignment");
            return Ok(true);
        }
        
        // NEW: Clear hotbar slot functionality - press 'c' to clear
        if app.ui.keybinds.matches("combat", "CLEAR_MOVE_QUEUE", &key) {
            // Clear the hotbar slot
            app.panels.build.hotbar_assignments[slot] = None;
            app.panels.hotbar_assignment = HotbarAssignmentState::None;
            app.core.game.res.log(format!("Cleared slot F{}", slot + 1));
            return Ok(true);
        }
        
        if app.ui.keybinds.matches("movement", "MOVE_NORTH", &key) {
            selected = selected.saturating_sub(1);
        }
        if app.ui.keybinds.matches("movement", "MOVE_SOUTH", &key) {
            if selected < blocks.len().saturating_sub(1) {
                selected += 1;
            }
        }
        if app.ui.keybinds.matches("ui", "MENU_ACTIVATE", &key) || key == KeyCode::Enter {
            if selected < blocks.len() {
                let chosen_block = blocks[selected];
                app.panels.build.hotbar_assignments[slot] = Some(chosen_block);
                app.panels.hotbar_assignment = HotbarAssignmentState::None;
                app.core.game.res.log(format!(
                    "Assigned {} to slot F{}",
                    lithicrivers_core::components::itemkind_name(chosen_block),
                    slot + 1
                ));
            }
            return Ok(true);
        }

        app.panels.hotbar_assignment = HotbarAssignmentState::ChoosingBlock {
            hotbar_slot: slot,
            available_blocks: blocks,
            selected_block: selected,
        };
        return Ok(true);
    }

    Ok(false)
}
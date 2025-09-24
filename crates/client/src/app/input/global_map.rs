use crate::{App, MenuTab};
use crossterm::event::KeyCode;
use std::error::Error;

pub fn handle_input(app: &mut App, key: KeyCode) -> Result<bool, Box<dyn Error>> {
    // Only handle input when Global Map tab is active
    if app.ui.current_tab != MenuTab::GlobalMap {
        return Ok(false);
    }

    match key {
        // Up arrow or numpad 8 - previous marker
        KeyCode::Up | KeyCode::Char('8') => {
            let marker_count = app.core.game.res.quest_markers.len();
            // Total list items = marker_count + 1 (for unselect option at index 0)
            let total_list_items = marker_count + 1;
            if total_list_items > 0 {
                if app.panels.global_map.selected_marker_index > 0 {
                    app.panels.global_map.selected_marker_index -= 1;
                } else {
                    app.panels.global_map.selected_marker_index = total_list_items - 1;
                }
            }
            return Ok(true);
        }
        // Down arrow or numpad 2 - next marker
        KeyCode::Down | KeyCode::Char('2') => {
            let marker_count = app.core.game.res.quest_markers.len();
            // Total list items = marker_count + 1 (for unselect option at index 0)
            let total_list_items = marker_count + 1;
            if total_list_items > 0 {
                app.panels.global_map.selected_marker_index =
                    (app.panels.global_map.selected_marker_index + 1) % total_list_items;
            }
            return Ok(true);
        }
        _ => return Ok(false),
    }
}

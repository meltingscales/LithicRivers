use crate::ui::panels::global_map::selected_marker;
use crate::{App, MenuTab};
use crossterm::event::KeyCode;
use lithicrivers_core::resources::QuestMarkerType;
use std::error::Error;

pub fn handle_input(app: &mut App, key: KeyCode) -> Result<bool, Box<dyn Error>> {
    // Only handle input when Global Map tab is active
    if app.ui.current_tab != MenuTab::GlobalMap {
        return Ok(false);
    }

    // The delete prompt takes every key until it's answered
    if let Some(index) = app.panels.global_map.confirm_delete {
        let keys = &app.ui.keybinds;
        if keys.matches("global_map", "CONFIRM_DELETE", &key) {
            app.panels.global_map.confirm_delete = None;
            if index < app.core.game.res.quest_markers.len() {
                let marker = app.core.game.res.quest_markers.remove(index);
                app.core
                    .game
                    .res
                    .log(format!("Deleted the marker for {}.", marker.name));
                app.panels.global_map.selected_marker_index = 0;
            }
        } else if keys.matches("global_map", "CANCEL_DELETE", &key) {
            app.panels.global_map.confirm_delete = None;
        }
        return Ok(true);
    }

    if app.ui.keybinds.matches("global_map", "DELETE_MARKER", &key) {
        if let Some(index) = selected_marker(app) {
            if app.core.game.res.quest_markers[index].marker_type == QuestMarkerType::Death {
                app.panels.global_map.confirm_delete = Some(index);
            } else {
                app.core
                    .game
                    .res
                    .log("Only wreck markers can be deleted.".to_string());
            }
            return Ok(true);
        }
        return Ok(false);
    }

    match key {
        // Up - previous marker
        k if app.ui.keybinds.menu_up(&k) => {
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
            Ok(true)
        }
        // Down - next marker
        k if app.ui.keybinds.menu_down(&k) => {
            let marker_count = app.core.game.res.quest_markers.len();
            // Total list items = marker_count + 1 (for unselect option at index 0)
            let total_list_items = marker_count + 1;
            if total_list_items > 0 {
                app.panels.global_map.selected_marker_index =
                    (app.panels.global_map.selected_marker_index + 1) % total_list_items;
            }
            Ok(true)
        }
        _ => Ok(false),
    }
}

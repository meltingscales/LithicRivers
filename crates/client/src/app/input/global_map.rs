use crate::ui::panels::global_map::{markers_by_distance, selected_marker};
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

    if app.panels.global_map.renaming.is_some() {
        rename_input(app, key);
        return Ok(true);
    }

    let keys = &app.ui.keybinds;
    let selected = selected_marker(app);
    let selected_waypoint = selected
        .filter(|&i| app.core.game.res.quest_markers[i].marker_type == QuestMarkerType::Waypoint);
    if keys.matches("global_map", "DELETE_MARKER", &key) {
        let Some(index) = selected else {
            return Ok(false);
        };
        if app.core.game.res.quest_markers[index]
            .marker_type
            .deletable()
        {
            app.panels.global_map.confirm_delete = Some(index);
        } else {
            app.core
                .game
                .res
                .log("Quest markers can't be deleted.".to_string());
        }
        return Ok(true);
    }
    if keys.matches("global_map", "ADD_WAYPOINT", &key) {
        let Some(pos) = app.core.game.get_player_position() else {
            return Ok(false);
        };
        let index = app.core.game.res.add_waypoint(pos);
        let name = app.core.game.res.quest_markers[index].name.clone();
        select(app, index);
        app.core.game.res.log(format!(
            "Added {name} at ({}, {}, {}).",
            pos.x, pos.y, pos.z
        ));
        return Ok(true);
    }
    if keys.matches("global_map", "RENAME_WAYPOINT", &key) {
        let Some(index) = selected_waypoint else {
            return Ok(false);
        };
        app.panels.global_map.renaming = Some(app.core.game.res.quest_markers[index].name.clone());
        return Ok(true);
    }
    if keys.matches("global_map", "MOVE_WAYPOINT", &key) {
        let (Some(index), Some(pos)) = (selected_waypoint, app.core.game.get_player_position())
        else {
            return Ok(false);
        };
        let marker = &mut app.core.game.res.quest_markers[index];
        (marker.x, marker.y, marker.z) = (pos.x, pos.y, pos.z);
        let name = marker.name.clone();
        // It is now the nearest marker, so its row moves
        select(app, index);
        app.core.game.res.log(format!(
            "Moved {name} to ({}, {}, {}).",
            pos.x, pos.y, pos.z
        ));
        return Ok(true);
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

/// The longest name a waypoint can have.
pub const MAX_WAYPOINT_NAME: usize = 24;

/// Select the marker at `index` in `quest_markers`, wherever it sorts in the list.
fn select(app: &mut App, index: usize) {
    if let Some(row) = markers_by_distance(app)
        .iter()
        .position(|&(i, _)| i == index)
    {
        // Row 0 is "unselect"
        app.panels.global_map.selected_marker_index = row + 1;
    }
}

/// Keys while renaming the selected waypoint: typed characters, erase, keep or drop.
fn rename_input(app: &mut App, key: KeyCode) {
    let keys = &app.ui.keybinds;
    let Some(name) = app.panels.global_map.renaming.as_mut() else {
        return;
    };
    if keys.matches("global_map", "END_RENAME", &key) {
        let name = name.trim().to_string();
        app.panels.global_map.renaming = None;
        if let (Some(index), false) = (selected_marker(app), name.is_empty()) {
            let marker = &mut app.core.game.res.quest_markers[index];
            let old = std::mem::replace(&mut marker.name, name.clone());
            app.core.game.res.log(format!("Renamed {old} to {name}."));
        }
    } else if keys.matches("global_map", "DELETE_CHAR", &key) {
        name.pop();
    } else if keys.matches("ui", "CLOSE_HELP_MENU", &key) {
        app.panels.global_map.renaming = None;
    } else if let KeyCode::Char(c) = key {
        if name.chars().count() < MAX_WAYPOINT_NAME {
            name.push(c);
        }
    }
}

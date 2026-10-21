//! Input for the Item Dictionary tab: move through the list, and type a search.

use crate::{App, MenuTab};
use crossterm::event::KeyCode;

/// Handle a key on the Item Dictionary tab. Returns false on other tabs, and for keys the
/// tab doesn't use (so tab switching and the rest still work).
pub fn handle_item_dictionary_input(app: &mut App, key: KeyCode) -> bool {
    if app.ui.current_tab != MenuTab::ItemDictionary {
        return false;
    }
    let count = crate::ui::panels::item_dictionary::filtered_items(app).len();
    let keys = &app.ui.keybinds;
    let state = &mut app.panels.item_dictionary;
    if state.searching {
        if keys.matches("dictionary", "END_SEARCH", &key) {
            state.searching = false;
        } else if keys.matches("ui", "CLOSE_HELP_MENU", &key) {
            state.searching = false;
            state.search.clear();
        } else if keys.matches("dictionary", "DELETE_CHAR", &key) {
            state.search.pop();
        } else if let KeyCode::Char(c) = key {
            state.search.push(c);
        } else {
            return false;
        }
        state.selected = 0;
        return true;
    }
    if keys.matches("dictionary", "SEARCH", &key) {
        state.searching = true;
    } else if keys.matches("ui", "CLOSE_HELP_MENU", &key) && !state.search.is_empty() {
        state.search.clear();
        state.selected = 0;
    } else if keys.matches("ui", "CREDITS_SCROLL_UP", &key)
        || keys.matches("movement", "MOVE_NORTH", &key)
    {
        state.selected = state.selected.saturating_sub(1);
    } else if keys.matches("ui", "CREDITS_SCROLL_DOWN", &key)
        || keys.matches("movement", "MOVE_SOUTH", &key)
    {
        state.selected = (state.selected + 1).min(count.saturating_sub(1));
    } else {
        return false;
    }
    true
}

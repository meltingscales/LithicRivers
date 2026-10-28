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
    if let Some(reset) = state.search.handle(keys, &key) {
        if reset {
            state.selected = 0;
        }
        return true;
    }
    if keys.menu_up(&key) {
        state.selected = state.selected.saturating_sub(1);
    } else if keys.menu_down(&key) {
        state.selected = (state.selected + 1).min(count.saturating_sub(1));
    } else {
        return false;
    }
    true
}

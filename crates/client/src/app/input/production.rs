//! Input for the Production tab: move through the item list, and switch the time range.

use crate::{App, MenuTab};
use crossterm::event::KeyCode;

/// Handle a key on the Production tab. Returns false on other tabs, and for keys the tab
/// doesn't use.
pub fn handle_production_input(app: &mut App, key: KeyCode) -> bool {
    if app.ui.current_tab != MenuTab::Production {
        return false;
    }
    let ranges = crate::ui::panels::production::RANGES.len();
    let count = crate::ui::panels::production::rows(app).len();
    let keys = &app.ui.keybinds;
    let state = &mut app.panels.production;
    if keys.menu_up(&key) {
        state.selected = state.selected.saturating_sub(1);
    } else if keys.menu_down(&key) {
        state.selected = (state.selected + 1).min(count.saturating_sub(1));
    } else if keys.matches("stats", "CYCLE_RANGE", &key) {
        state.range = (state.range + 1) % ranges;
        let count = crate::ui::panels::production::rows(app).len();
        let state = &mut app.panels.production;
        state.selected = state.selected.min(count.saturating_sub(1));
    } else {
        return false;
    }
    true
}

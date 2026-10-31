//! Input for the filter picker: choose "no filter" or an item for a filter extractor or
//! belt router.

use crate::app::input::search::SearchBox;
use crate::App;
use crossterm::event::KeyCode;
use lithicrivers_core::components::{itemkind_name, ItemKind};
use strum::IntoEnumIterator;

/// The picker's rows: no filter, then every item whose name the search matches.
pub fn rows(search: &SearchBox) -> Vec<Option<ItemKind>> {
    std::iter::once(None)
        .chain(
            ItemKind::iter()
                .filter(|&item| search.matches([itemkind_name(item)]))
                .map(Some),
        )
        .collect()
}

/// Handle a key while the filter picker is open. Returns false if it isn't open.
pub fn handle_filter_picker_input(app: &mut App, key: KeyCode) -> bool {
    let Some(mut picker) = app.panels.filter_picker.clone() else {
        return false;
    };
    let keys = &app.ui.keybinds;
    // The search takes typed keys first; Escape clears a search before it closes the picker
    if let Some(reset) = picker.search.handle(keys, &key) {
        if reset {
            picker.selected = 0;
        }
        app.panels.filter_picker = Some(picker);
        return true;
    }
    let rows = rows(&picker.search);
    if keys.matches("ui", "CLOSE_HELP_MENU", &key) {
        app.panels.filter_picker = None;
        return true;
    }
    if keys.menu_up(&key) {
        picker.selected = picker.selected.saturating_sub(1);
    } else if keys.menu_down(&key) {
        picker.selected = (picker.selected + 1).min(rows.len() - 1);
    } else if keys.matches("ui", "MENU_ACTIVATE", &key) || key == KeyCode::Enter {
        let filter = rows[picker.selected.min(rows.len() - 1)];
        lithicrivers_core::logistics::set_filter(&mut app.core.game.world, picker.entity, filter);
        app.panels.filter_picker = None;
        app.core.game.res.log(match filter {
            Some(item) => format!("Filter set to {}", itemkind_name(item)),
            None => "Filter cleared".to_string(),
        });
        return true;
    }
    app.panels.filter_picker = Some(picker);
    true
}

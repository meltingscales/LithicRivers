//! Input for the filter picker: choose "no filter" or an item for a filter extractor or
//! belt router.

use crate::App;
use crossterm::event::KeyCode;
use lithicrivers_core::components::{itemkind_name, ItemKind};
use strum::IntoEnumIterator;

/// The picker's rows: no filter, then every item.
pub fn rows() -> Vec<Option<ItemKind>> {
    std::iter::once(None)
        .chain(ItemKind::iter().map(Some))
        .collect()
}

/// Handle a key while the filter picker is open. Returns false if it isn't open.
pub fn handle_filter_picker_input(app: &mut App, key: KeyCode) -> bool {
    let Some(mut picker) = app.panels.filter_picker else {
        return false;
    };
    let keys = &app.ui.keybinds;
    let rows = rows();
    if keys.matches("ui", "CLOSE_HELP_MENU", &key) {
        app.panels.filter_picker = None;
        return true;
    }
    if keys.matches("movement", "MOVE_NORTH", &key) || keys.matches("ui", "CREDITS_SCROLL_UP", &key)
    {
        picker.selected = picker.selected.saturating_sub(1);
    } else if keys.matches("movement", "MOVE_SOUTH", &key)
        || keys.matches("ui", "CREDITS_SCROLL_DOWN", &key)
    {
        picker.selected = (picker.selected + 1).min(rows.len() - 1);
    } else if keys.matches("ui", "MENU_ACTIVATE", &key) || key == KeyCode::Enter {
        let filter = rows[picker.selected];
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

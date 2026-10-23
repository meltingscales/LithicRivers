//! Input for the furnace window: move between its slots and the player's items, and move
//! one item at a time either way.

use crate::app_state::FurnaceWindow;
use crate::App;
use crossterm::event::KeyCode;
use lithicrivers_core::components::{itemkind_name, Inventory};
use lithicrivers_core::furnace::{self, Slot, TransferError};

/// Handle a key while the furnace window is open. Returns false if it isn't open.
pub fn handle_furnace_window_input(app: &mut App, key: KeyCode) -> bool {
    let Some(mut window) = app.panels.furnace_window else {
        return false;
    };
    let keys = &app.ui.keybinds;
    if keys.matches("ui", "CLOSE_HELP_MENU", &key) {
        app.panels.furnace_window = None;
        app.core
            .game
            .res
            .log(format!("Closed {}", window.kind.name()));
        return true;
    }
    let Some(player) = app.core.game.get_player_entity() else {
        return true;
    };
    let player_items = crate::app::item_filter::PlayerRows::for_furnace_slot(
        &app.core.game.world,
        player,
        window.slot,
    )
    .len();
    let slots = Slot::for_kind(window.kind);
    let slot_index = slots
        .iter()
        .position(|&s| s == window.slot)
        .unwrap_or(0);
    if keys.matches("movement", "MOVE_WEST", &key) || keys.matches("movement", "MOVE_EAST", &key) {
        window.player_focus = !window.player_focus;
    } else if keys.matches("movement", "MOVE_NORTH", &key) {
        if window.player_focus {
            window.player_item = window.player_item.saturating_sub(1);
        } else {
            window.slot = slots[slot_index.saturating_sub(1)];
        }
    } else if keys.matches("movement", "MOVE_SOUTH", &key) {
        if window.player_focus {
            window.player_item = (window.player_item + 1).min(player_items.saturating_sub(1));
        } else {
            window.slot = slots[(slot_index + 1).min(slots.len() - 1)];
        }
    } else if keys.matches("ui", "MENU_ACTIVATE", &key) || key == KeyCode::Enter {
        transfer(app, &window, player);
    }
    // Keep the player selection on a shown stack (the list follows the selected slot)
    let player_items = crate::app::item_filter::PlayerRows::for_furnace_slot(
        &app.core.game.world,
        player,
        window.slot,
    )
    .len();
    window.player_item = window.player_item.min(player_items.saturating_sub(1));
    app.panels.furnace_window = Some(window);
    true
}

/// Store the selected player item into the selected slot, or take one item out of it.
fn transfer(app: &mut App, window: &FurnaceWindow, player: hecs::Entity) {
    let world = &mut app.core.game.world;
    let rows = crate::app::item_filter::PlayerRows::for_furnace_slot(world, player, window.slot);
    let player_slot = rows.slot(window.player_item).unwrap_or(usize::MAX);
    let item = world
        .get::<&Inventory>(player)
        .ok()
        .and_then(|inv| inv.slots.get(player_slot).map(|s| s.kind));
    let (result, verb) = if window.player_focus {
        let result = furnace::store(world, player, window.entity, player_slot, window.slot);
        (result, "Stored")
    } else {
        (
            furnace::take(world, window.entity, window.slot, player),
            "Took",
        )
    };
    let message = match result {
        Ok(kind) => format!("{verb} {}", itemkind_name(kind)),
        Err(TransferError::Refused(refusal)) => match item {
            Some(item) => refusal.message(item),
            None => return,
        },
        Err(TransferError::InventoryFull) => "Your inventory is full".to_string(),
        Err(TransferError::Empty) => return,
    };
    app.core.game.res.log(message);
}

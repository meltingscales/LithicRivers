use crate::App;
use crate::MenuTab;
use crossterm::event::KeyCode;
use lithicrivers_core::components::{itemkind_name, itemkind_sprite_name, ItemKind};
use lithicrivers_core::components::{DroppedItem, Inventory as InvComp, Position, SpriteRef};
use std::error::Error;

/// Handle inventory tab input - returns true if input was handled
pub fn handle_inventory_input(app: &mut App, key: KeyCode) -> Result<bool, Box<dyn Error>> {
    // Only handle input when on the inventory tab
    if app.ui.current_tab != MenuTab::Inventory {
        return Ok(false);
    }

    // Auto-pickup toggle
    if app
        .ui
        .keybinds
        .matches("inventory", "TOGGLE_ITEM_AUTO_PICKUP_KEY", &key)
    {
        if let Some(e) = app.core.game.get_player_entity() {
            if let Ok(mut inv) = app.core.game.world.get::<&mut InvComp>(e) {
                inv.auto_pickup = !inv.auto_pickup;
                let state = if inv.auto_pickup { "ON" } else { "OFF" };
                app.core
                    .game
                    .res
                    .log(format!("Item auto-pickup: {}", state));
            }
        }
        return Ok(true);
    }

    // Navigation: Up/Down or North/South to move selection
    if app.ui.keybinds.matches("ui", "CREDITS_SCROLL_UP", &key)
        || app.ui.keybinds.matches("movement", "MOVE_NORTH", &key)
    {
        if let Some(e) = app.core.game.get_player_entity() {
            if let Ok(inv) = app.core.game.world.get::<&InvComp>(e) {
                if !inv.slots.is_empty() {
                    if app.panels.inventory.selected == 0 {
                        app.panels.inventory.selected = inv.slots.len() - 1;
                    } else {
                        app.panels.inventory.selected -= 1;
                    }
                }
            }
        }
        return Ok(true);
    }

    if app.ui.keybinds.matches("ui", "CREDITS_SCROLL_DOWN", &key)
        || app.ui.keybinds.matches("movement", "MOVE_SOUTH", &key)
    {
        if let Some(e) = app.core.game.get_player_entity() {
            if let Ok(inv) = app.core.game.world.get::<&InvComp>(e) {
                if !inv.slots.is_empty() {
                    app.panels.inventory.selected =
                        (app.panels.inventory.selected + 1) % inv.slots.len();
                }
            }
        }
        return Ok(true);
    }

    // Helper to get current selection
    let mut selected: Option<(ItemKind, u32)> = None;
    if let Some(e) = app.core.game.get_player_entity() {
        if let Ok(inv) = app.core.game.world.get::<&InvComp>(e) {
            if !inv.slots.is_empty() {
                let idx = app.panels.inventory.selected.min(inv.slots.len() - 1);
                selected = Some((inv.slots[idx].kind, inv.slots[idx].qty));
            }
        }
    }

    // Drop selected item (quantity 1 for now)
    if app.ui.keybinds.matches("inventory", "DROP_ITEM", &key) {
        if let Some((kind, qty)) = selected {
            if qty == 0 {
                return Ok(true);
            }
            if let Some(e) = app.core.game.get_player_entity() {
                // Copy player position, then drop immutable borrow before mutating world
                let (px, py, pz) = {
                    let Ok(ppos) = app
                        .core
                        .game
                        .world
                        .get::<&lithicrivers_core::components::Position>(e)
                    else {
                        return Ok(true);
                    };
                    (ppos.x, ppos.y, ppos.z)
                };

                let drop_qty = 1u32;
                // Decrement inventory (mutable borrow scope ends before spawn)
                if let Ok(mut inv) = app.core.game.world.get::<&mut InvComp>(e) {
                    if app.panels.inventory.selected < inv.slots.len() {
                        let slot = &mut inv.slots[app.panels.inventory.selected];
                        if slot.qty >= drop_qty {
                            slot.qty -= drop_qty;
                            if slot.qty == 0 {
                                inv.slots.remove(app.panels.inventory.selected);
                                if app.panels.inventory.selected > 0 {
                                    app.panels.inventory.selected -= 1;
                                }
                            }
                        }
                    }
                }
                // Spawn DroppedItem entity with SpriteRef
                let _ = app.core.game.world.spawn((
                    Position {
                        x: px,
                        y: py,
                        z: pz,
                    },
                    DroppedItem {
                        kind,
                        qty: drop_qty,
                    },
                    SpriteRef::new("items", itemkind_sprite_name(kind)),
                ));
                app.core
                    .game
                    .res
                    .log(format!("Dropped 1 {}", itemkind_name(kind)));
            }
        }
        return Ok(true);
    }

    // Duplicate selected item (cheat)
    if app
        .ui
        .keybinds
        .matches("inventory", "CHEAT_DUPLICATE_ITEM", &key)
    {
        if let Some(e) = app.core.game.get_player_entity() {
            if let Ok(mut inv) = app.core.game.world.get::<&mut InvComp>(e) {
                if !inv.slots.is_empty() {
                    let idx = app.panels.inventory.selected.min(inv.slots.len() - 1);
                    let kind = inv.slots[idx].kind;
                    inv.slots[idx].qty = inv.slots[idx].qty.saturating_add(1);
                    app.core
                        .game
                        .res
                        .log(format!("Duplicated 1 {}", itemkind_name(kind)));
                }
            }
        }
        return Ok(true);
    }

    // Destroy selected item (remove 1)
    if app.ui.keybinds.matches("inventory", "DESTROY_ITEM", &key) {
        if let Some(e) = app.core.game.get_player_entity() {
            if let Ok(mut inv) = app.core.game.world.get::<&mut InvComp>(e) {
                if !inv.slots.is_empty() {
                    let idx = app.panels.inventory.selected.min(inv.slots.len() - 1);
                    let kind = inv.slots[idx].kind;
                    if inv.slots[idx].qty > 0 {
                        inv.slots[idx].qty -= 1;
                        if inv.slots[idx].qty == 0 {
                            inv.slots.remove(idx);
                            if app.panels.inventory.selected > 0 {
                                app.panels.inventory.selected -= 1;
                            }
                        }
                        app.core
                            .game
                            .res
                            .log(format!("Destroyed 1 {}", itemkind_name(kind)));
                    }
                }
            }
        }
        return Ok(true);
    }

    Ok(false)
}

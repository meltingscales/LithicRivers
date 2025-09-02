use crossterm::event::KeyCode;
use std::{error::Error, time::Instant};

use lithicrivers_core::{
    components::{
        itemkind_name, itemkind_sprite_name, DroppedItem, Inventory as InvComp, ItemKind,
        ItemStack, Position, SpriteRef,
    },
    game::GameTickResult,
};

use crate::{ui::panels::get_player_inventory, App, MenuTab, Scale, SplashState};

pub fn handle_input(app: &mut App, key: KeyCode) -> Result<(), Box<dyn Error>> {
    // Handle splash screen skipping first
    if let Some(_start_time) = app.splash_start_time {
        match app.splash_state {
            SplashState::Logo => {
                // Any key skips to next screen
                app.splash_state = SplashState::GameTitle;
                app.splash_start_time = Some(Instant::now());
                return Ok(());
            }
            SplashState::GameTitle => {
                // Any key skips to boot message
                app.splash_state = SplashState::BootMessage;
                app.splash_start_time = Some(Instant::now());
                app.boot_display_text.clear();
                app.boot_line_index = 0;
                app.last_line_time = Instant::now();
                app.boot_complete = false;
                return Ok(());
            }
            SplashState::BootMessage => {
                if !app.boot_complete {
                    // Skip to end of text
                    let full_text = app.boot_message_lines.join("\n");
                    app.boot_display_text = full_text.clone();
                    app.boot_line_index = full_text.len();
                    app.boot_complete = true;
                    app.splash_start_time = Some(Instant::now());
                } else {
                    // Move to main UI if already complete
                    app.splash_state = SplashState::MainUI;
                }
                return Ok(());
            }
            _ => {}
        }
    }

    // log key to log
    // tracing::info!(target: "game", "key pressed: {:?}", key);

    // First, handle configurable keybind actions
    // Toggle Look mode
    if app.keybinds.matches("action", "LOOK_TOGGLE", &key) {
        app.look_mode = !app.look_mode;
        // Reset cursor to player on toggle on
        if app.look_mode {
            if let Some(e) = app.game.res.player_entity {
                if let Ok(pos) = app
                    .game
                    .world
                    .get::<&lithicrivers_core::components::Position>(e)
                {
                    app.look_cursor = *pos;
                    // Align all view coords to cursor
                    app.game.res.view_x = app.look_cursor.x;
                    app.game.res.view_y = app.look_cursor.y;
                    app.game.res.view_z = app.look_cursor.z;
                }
            }
            app.game.res.log("Look mode: ON");
        } else {
            app.game.res.log("Look mode: OFF");
        }
        return Ok(());
    }

    // In Look mode, remap movement keys to move the look cursor without ticking
    if app.look_mode {
        let mut moved = false;
        if app.keybinds.matches("movement", "MOVE_NORTH", &key) {
            app.look_cursor.y -= 1;
            app.game.res.view_y = app.look_cursor.y;
            moved = true;
        } else if app.keybinds.matches("movement", "MOVE_SOUTH", &key) {
            app.look_cursor.y += 1;
            app.game.res.view_y = app.look_cursor.y;
            moved = true;
        } else if app.keybinds.matches("movement", "MOVE_WEST", &key) {
            app.look_cursor.x -= 1;
            app.game.res.view_x = app.look_cursor.x;
            moved = true;
        } else if app.keybinds.matches("movement", "MOVE_EAST", &key) {
            app.look_cursor.x += 1;
            app.game.res.view_x = app.look_cursor.x;
            moved = true;
        } else if app.keybinds.matches("movement", "MOVE_NORTHWEST", &key) {
            app.look_cursor.x -= 1;
            app.look_cursor.y -= 1;
            app.game.res.view_x = app.look_cursor.x;
            app.game.res.view_y = app.look_cursor.y;
            moved = true;
        } else if app.keybinds.matches("movement", "MOVE_NORTHEAST", &key) {
            app.look_cursor.x += 1;
            app.look_cursor.y -= 1;
            app.game.res.view_x = app.look_cursor.x;
            app.game.res.view_y = app.look_cursor.y;
            moved = true;
        } else if app.keybinds.matches("movement", "MOVE_SOUTHWEST", &key) {
            app.look_cursor.x -= 1;
            app.look_cursor.y += 1;
            app.game.res.view_x = app.look_cursor.x;
            app.game.res.view_y = app.look_cursor.y;
            moved = true;
        } else if app.keybinds.matches("movement", "MOVE_SOUTHEAST", &key) {
            app.look_cursor.x += 1;
            app.look_cursor.y += 1;
            app.game.res.view_x = app.look_cursor.x;
            app.game.res.view_y = app.look_cursor.y;
            moved = true;
        } else if app.keybinds.matches("movement", "WAIT", &key) {
            // no-op, but treat as handled to avoid player waiting
            moved = true;
        } else if app.keybinds.matches("movement", "MOVE_UP", &key) {
            app.look_cursor.z += 1;
            app.game.res.view_z = app.look_cursor.z;
            moved = true;
        } else if app.keybinds.matches("movement", "MOVE_DOWN", &key) {
            app.look_cursor.z -= 1;
            app.game.res.view_z = app.look_cursor.z;
            moved = true;
        } else if app.keybinds.matches("viewport", "VIEW_Z_UP", &key) {
            app.look_cursor.z = app.look_cursor.z.saturating_add(1);
            app.game.res.view_z = app.look_cursor.z;
            moved = true;
        } else if app.keybinds.matches("viewport", "VIEW_Z_DOWN", &key) {
            app.look_cursor.z = app.look_cursor.z.saturating_sub(1);
            app.game.res.view_z = app.look_cursor.z;
            moved = true;
        }

        if moved {
            // Prefetch around the new cursor position for smoother draw
            let radius = 20i32;
            let left = app.look_cursor.x - radius;
            let top = app.look_cursor.y - radius;
            let right = app.look_cursor.x + radius;
            let bottom = app.look_cursor.y + radius;
            app.game
                .res
                .world
                .prefetch_rect(left, top, right, bottom, app.look_cursor.z);
            return Ok(());
        }
    }
    // Inventory: toggle item auto-pickup
    if app.current_tab == MenuTab::Inventory
        && app
            .keybinds
            .matches("inventory", "TOGGLE_ITEM_AUTO_PICKUP_KEY", &key)
    {
        if let Some(e) = app.game.res.player_entity {
            if let Ok(mut inv) = app.game.world.get::<&mut InvComp>(e) {
                inv.auto_pickup = !inv.auto_pickup;
                let state = if inv.auto_pickup { "ON" } else { "OFF" };
                app.game.res.log(format!("Item auto-pickup: {}", state));
            }
        }
        return Ok(());
    }

    // Crafting panel-specific navigation and actions
    if app.current_tab == MenuTab::Crafting {
        // Get player inventory for crafting checks
        let inventory = get_player_inventory(app);
        let recipe_count = app.recipe_handler.get_recipes().len();

        // Navigation: Up/Down or North/South to move selection
        if app.keybinds.matches("ui", "CREDITS_SCROLL_UP", &key)
            || app.keybinds.matches("movement", "MOVE_NORTH", &key)
        {
            if recipe_count > 0 {
                if app.craft_selected == 0 {
                    app.craft_selected = recipe_count - 1;
                } else {
                    app.craft_selected = app.craft_selected.saturating_sub(1);
                }
            }
            return Ok(());
        }
        if app.keybinds.matches("ui", "CREDITS_SCROLL_DOWN", &key)
            || app.keybinds.matches("movement", "MOVE_SOUTH", &key)
        {
            if recipe_count > 0 {
                app.craft_selected = (app.craft_selected + 1) % recipe_count;
            }
            return Ok(());
        }

        // Craft item on Enter or Activate key
        if app.keybinds.matches("ui", "ACTIVATE", &key) || key == KeyCode::Enter {
            if let Some(recipe) = app.recipe_handler.get_recipes().get(app.craft_selected) {
                if app.recipe_handler.can_craft(app.craft_selected, &inventory) {
                    if let Some(e) = app.game.res.player_entity {
                        if let Ok(mut inv) = app.game.world.get::<&mut InvComp>(e) {
                            // Consume ingredients
                            for &(item, qty) in recipe.ingredients {
                                let mut remaining = qty;
                                for slot in inv.slots.iter_mut() {
                                    if slot.kind == item && remaining > 0 {
                                        let consume = slot.qty.min(remaining);
                                        slot.qty -= consume;
                                        remaining -= consume;

                                        if slot.qty == 0 {
                                            // Remove empty slots - the inventory will be compacted later
                                            // by the game's inventory management system
                                        }
                                    }
                                }
                            }

                            // Add crafted item to inventory
                            let mut added = false;
                            for slot in inv.slots.iter_mut() {
                                if slot.kind == recipe.result && slot.qty < 1000 {
                                    // Arbitrary max stack size
                                    slot.qty = slot.qty.saturating_add(recipe.quantity);
                                    added = true;
                                    break;
                                }
                            }

                            if !added {
                                inv.slots.push(ItemStack {
                                    kind: recipe.result,
                                    qty: recipe.quantity,
                                });
                            }

                            app.craft_message = Some((
                                format!(
                                    "Crafted {}x {}",
                                    recipe.quantity,
                                    itemkind_name(recipe.result)
                                ),
                                30, // Display for 30 frames (~0.5 seconds at 60 FPS)
                            ));

                            app.game.res.log(format!(
                                "Crafted {}x {}",
                                recipe.quantity,
                                itemkind_name(recipe.result)
                            ));
                        }
                    }
                } else {
                    app.craft_message = Some((
                        "Not enough resources to craft this item".to_string(),
                        60, // Display for 1 second at 60 FPS
                    ));
                }
            }
            return Ok(());
        }

        // Don't process movement keys in crafting panel
        if app.keybinds.matches("movement", "MOVE_NORTH", &key)
            || app.keybinds.matches("movement", "MOVE_SOUTH", &key)
            || app.keybinds.matches("movement", "MOVE_WEST", &key)
            || app.keybinds.matches("movement", "MOVE_EAST", &key)
            || app.keybinds.matches("movement", "MOVE_NORTHWEST", &key)
            || app.keybinds.matches("movement", "MOVE_NORTHEAST", &key)
            || app.keybinds.matches("movement", "MOVE_SOUTHWEST", &key)
            || app.keybinds.matches("movement", "MOVE_SOUTHEAST", &key)
            || app.keybinds.matches("movement", "WAIT", &key)
            || app.keybinds.matches("movement", "MOVE_UP", &key)
            || app.keybinds.matches("movement", "MOVE_DOWN", &key)
        {
            return Ok(());
        }
    }

    // Inventory panel-specific navigation and actions
    if app.current_tab == MenuTab::Inventory {
        // Move selection: support Up/Down keys and numpad 8/2 (MOVE_NORTH/SOUTH)
        if app.keybinds.matches("ui", "CREDITS_SCROLL_UP", &key)
            || app.keybinds.matches("movement", "MOVE_NORTH", &key)
        {
            if let Some(e) = app.game.res.player_entity {
                if let Ok(inv) = app.game.world.get::<&InvComp>(e) {
                    if !inv.slots.is_empty() {
                        if app.inv_selected == 0 {
                            app.inv_selected = inv.slots.len() - 1;
                        } else {
                            app.inv_selected -= 1;
                        }
                    }
                }
            }
            return Ok(());
        }
        if app.keybinds.matches("ui", "CREDITS_SCROLL_DOWN", &key)
            || app.keybinds.matches("movement", "MOVE_SOUTH", &key)
        {
            if let Some(e) = app.game.res.player_entity {
                if let Ok(inv) = app.game.world.get::<&InvComp>(e) {
                    if !inv.slots.is_empty() {
                        app.inv_selected = (app.inv_selected + 1) % inv.slots.len();
                    }
                }
            }
            return Ok(());
        }

        // Helper to get current selection
        let mut selected: Option<(ItemKind, u32)> = None;
        if let Some(e) = app.game.res.player_entity {
            if let Ok(inv) = app.game.world.get::<&InvComp>(e) {
                if !inv.slots.is_empty() {
                    let idx = app.inv_selected.min(inv.slots.len() - 1);
                    selected = Some((inv.slots[idx].kind, inv.slots[idx].qty));
                }
            }
        }

        // Drop selected item (quantity 1 for now)
        if app.keybinds.matches("inventory", "DROP_ITEM", &key) {
            if let Some((kind, qty)) = selected {
                if qty == 0 {
                    return Ok(());
                }
                if let Some(e) = app.game.res.player_entity {
                    // Copy player position, then drop immutable borrow before mutating world
                    let (px, py, pz) = {
                        let Ok(ppos) = app
                            .game
                            .world
                            .get::<&lithicrivers_core::components::Position>(e)
                        else {
                            return Ok(());
                        };
                        (ppos.x, ppos.y, ppos.z)
                    };

                    let drop_qty = 1u32;
                    // Decrement inventory (mutable borrow scope ends before spawn)
                    if let Ok(mut inv) = app.game.world.get::<&mut InvComp>(e) {
                        if app.inv_selected < inv.slots.len() {
                            let slot = &mut inv.slots[app.inv_selected];
                            if slot.qty >= drop_qty {
                                slot.qty -= drop_qty;
                                if slot.qty == 0 {
                                    inv.slots.remove(app.inv_selected);
                                    if app.inv_selected > 0 {
                                        app.inv_selected -= 1;
                                    }
                                }
                            }
                        }
                    }
                    // Spawn DroppedItem entity with SpriteRef
                    let _ = app.game.world.spawn((
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
                    app.game
                        .res
                        .log(format!("Dropped 1 {}", itemkind_name(kind)));
                }
            }
            return Ok(());
        }

        // Duplicate selected item (cheat)
        if app
            .keybinds
            .matches("inventory", "CHEAT_DUPLICATE_ITEM", &key)
        {
            if let Some(e) = app.game.res.player_entity {
                if let Ok(mut inv) = app.game.world.get::<&mut InvComp>(e) {
                    if !inv.slots.is_empty() {
                        let idx = app.inv_selected.min(inv.slots.len() - 1);
                        let kind = inv.slots[idx].kind;
                        inv.slots[idx].qty = inv.slots[idx].qty.saturating_add(1);
                        app.game
                            .res
                            .log(format!("Duplicated 1 {}", itemkind_name(kind)));
                    }
                }
            }
            return Ok(());
        }

        // Destroy selected item (remove 1)
        if app.keybinds.matches("inventory", "DESTROY_ITEM", &key) {
            if let Some(e) = app.game.res.player_entity {
                if let Ok(mut inv) = app.game.world.get::<&mut InvComp>(e) {
                    if !inv.slots.is_empty() {
                        let idx = app.inv_selected.min(inv.slots.len() - 1);
                        let kind = inv.slots[idx].kind;
                        if inv.slots[idx].qty > 0 {
                            inv.slots[idx].qty -= 1;
                            if inv.slots[idx].qty == 0 {
                                inv.slots.remove(idx);
                                if app.inv_selected > 0 {
                                    app.inv_selected -= 1;
                                }
                            }
                            app.game
                                .res
                                .log(format!("Destroyed 1 {}", itemkind_name(kind)));
                        }
                    }
                }
            }
            return Ok(());
        }
    }

    if app.keybinds.matches_movement(&key) {
        if app.keybinds.matches("movement", "MOVE_NORTH", &key) {
            app.game.queue_player_move(0, -1);
        }
        if app.keybinds.matches("movement", "MOVE_SOUTH", &key) {
            app.game.queue_player_move(0, 1);
        }
        if app.keybinds.matches("movement", "MOVE_WEST", &key) {
            app.game.queue_player_move(-1, 0);
        }
        if app.keybinds.matches("movement", "MOVE_EAST", &key) {
            app.game.queue_player_move(1, 0);
        }
        if app.keybinds.matches("movement", "MOVE_NORTHWEST", &key) {
            app.game.queue_player_move(-1, -1);
        }
        if app.keybinds.matches("movement", "MOVE_NORTHEAST", &key) {
            app.game.queue_player_move(1, -1);
        }
        if app.keybinds.matches("movement", "MOVE_SOUTHWEST", &key) {
            app.game.queue_player_move(-1, 1);
        }
        if app.keybinds.matches("movement", "MOVE_SOUTHEAST", &key) {
            app.game.queue_player_move(1, 1);
        }
        if app.keybinds.matches("movement", "WAIT", &key) {
            app.game.queue_player_move(0, 0);
        }
        if app.keybinds.matches("movement", "MOVE_UP", &key) {
            app.game.queue_player_move_z(1);
        }
        if app.keybinds.matches("movement", "MOVE_DOWN", &key) {
            app.game.queue_player_move_z(-1);
        }

        let tick_result = app.game.tick();
        if tick_result.contains(GameTickResult::CombatTriggered) {
            app.combat_happening = true;
            panic!("todo show combat panel...");
        }
        app.snap_view_to_player_z();
        return Ok(());
    }

    if app.keybinds.matches("action", "MINE", &key) {
        app.game.queue_mine();
        let tick_result = app.game.tick();
        if tick_result.contains(GameTickResult::MiningSuccess) {
            app.snap_view_to_player_z();
        }
        return Ok(());
    }

    if app.keybinds.matches("scale", "SCALE_UP", &key) {
        app.scale = match app.scale {
            Scale::Small => Scale::Medium,
            Scale::Medium => Scale::Large,
            Scale::Large => Scale::Large,
        };
        return Ok(());
    }
    if app.keybinds.matches("scale", "SCALE_DOWN", &key) {
        app.scale = match app.scale {
            Scale::Large => Scale::Medium,
            Scale::Medium => Scale::Small,
            Scale::Small => Scale::Small,
        };
        return Ok(());
    }
    if app.keybinds.matches("scale", "SCALE_RESET", &key) {
        app.scale = Scale::Small;
        return Ok(());
    }
    // UI: Quit
    if app.keybinds.matches("ui", "QUIT", &key) {
        app.game.res.log("Quit requested (keybind)");
        tracing::info!(target: "game", "quit_requested tick={}", app.game.res.gametick);
        app.should_quit = true;
        return Ok(());
    }
    // UI: Menu activation and paging
    if app.keybinds.matches("ui", "MENU_ACTIVATE", &key) {
        app.activate_menu();
        return Ok(());
    }
    if app.keybinds.matches("ui", "MENU_PREV", &key) {
        app.current_tab = app.current_tab.prev();
        return Ok(());
    }
    if app.keybinds.matches("ui", "MENU_NEXT", &key) {
        app.current_tab = app.current_tab.next();
        return Ok(());
    }
    // Credits scroll
    if app.current_tab == MenuTab::Credits {
        if app.keybinds.matches("ui", "CREDITS_SCROLL_UP", &key) {
            app.credits_scroll = app.credits_scroll.saturating_sub(1);
            return Ok(());
        }
        if app.keybinds.matches("ui", "CREDITS_SCROLL_DOWN", &key) {
            app.credits_scroll = app.credits_scroll.saturating_add(1);
            return Ok(());
        }
    }
    // Help scroll
    if app.current_tab == MenuTab::Help {
        match key {
            KeyCode::Up => {
                app.help_scroll = app.help_scroll.saturating_sub(1);
                return Ok(());
            }
            KeyCode::Down => {
                app.help_scroll = app.help_scroll.saturating_add(1);
                return Ok(());
            }
            _ => {}
        }
    }
    // View Z slice up/down
    if app.keybinds.matches("viewport", "VIEW_Z_UP", &key) {
        if app.current_tab == MenuTab::Credits {
            app.credits_scroll = app.credits_scroll.saturating_sub(10);
        } else {
            app.game.res.view_z = app.game.res.view_z.saturating_add(1);
        }
        return Ok(());
    }
    if app.keybinds.matches("viewport", "VIEW_Z_DOWN", &key) {
        if app.current_tab == MenuTab::Credits {
            app.credits_scroll = app.credits_scroll.saturating_add(10);
        } else {
            app.game.res.view_z = app.game.res.view_z.saturating_sub(1);
        }
        return Ok(());
    }
    // Save/Load via config
    if app.keybinds.matches("ui", "SAVE_JSON", &key) {
        tracing::info!(target: "game", "save_begin path=save.json tick={}", app.game.res.gametick);
        app.game
            .save_json("save.json")
            .map_err(|e| format!("save_json error: {:?}", e))?;
        app.game.res.log("Saved to save.json");
        tracing::info!(target: "game", "save_end path=save.json tick={}", app.game.res.gametick);
        return Ok(());
    }
    if app.keybinds.matches("ui", "LOAD_JSON", &key) {
        tracing::info!(target: "game", "load_begin path=save.json tick={}", app.game.res.gametick);
        app.game
            .load_json("save.json")
            .map_err(|e| format!("load_json error: {:?}", e))?;
        app.game.res.log("Loaded from save.json");
        tracing::info!(target: "game", "load_end path=save.json tick={}", app.game.res.gametick);
        return Ok(());
    }

    Ok(())
}

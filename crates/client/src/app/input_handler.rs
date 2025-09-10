use crossterm::event::KeyCode;
use std::{error::Error, time::Instant};

use lithicrivers_core::{
    components::{
        itemkind_name, itemkind_sprite_name, DroppedItem, Inventory as InvComp, ItemKind,
        ItemStack, Position, SpriteRef,
    },
    game::GameTickResult,
    moves::get_available_moves,
};

use crate::{ui::panels::get_player_inventory, App, CombatUiState, MenuTab, Scale, SplashState};

pub fn handle_input(app: &mut App, key: KeyCode) -> Result<(), Box<dyn Error>> {
    // Handle splash screen skipping first
    if let Some(_start_time) = app.splash.start_time {
        match app.splash.state {
            SplashState::Logo => {
                // Any key skips to next screen
                app.splash.state = SplashState::GameTitle;
                app.splash.start_time = Some(Instant::now());
                return Ok(());
            }
            SplashState::GameTitle => {
                // Any key skips to boot message
                app.splash.state = SplashState::BootMessage;
                app.splash.start_time = Some(Instant::now());
                app.splash.boot_display_text.clear();
                app.splash.boot_line_index = 0;
                app.splash.last_line_time = Instant::now();
                app.splash.boot_complete = false;
                return Ok(());
            }
            SplashState::BootMessage => {
                if !app.splash.boot_complete {
                    // Skip to end of text
                    let full_text = app.splash.boot_message_lines.join("\n");
                    app.splash.boot_display_text = full_text.clone();
                    app.splash.boot_line_index = full_text.len();
                    app.splash.boot_complete = true;
                    app.splash.start_time = Some(Instant::now());
                } else {
                    // Move to main UI if already complete
                    app.splash.state = SplashState::MainUI;
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
    if app.ui.current_tab == MenuTab::World {
        if app.ui.keybinds.matches("action", "LOOK_TOGGLE", &key) {
            app.panels.look.mode = !app.panels.look.mode;
            // Reset cursor to player on toggle on
            if app.panels.look.mode {
                if let Some(e) = app.core.game.get_player_entity() {
                    if let Ok(pos) = app
                        .core
                        .game
                        .world
                        .get::<&lithicrivers_core::components::Position>(e)
                    {
                        app.panels.look.cursor = *pos;
                        // Align all view coords to cursor
                        app.ui.view_x = app.panels.look.cursor.x;
                        app.ui.view_y = app.panels.look.cursor.y;
                        app.ui.view_z = app.panels.look.cursor.z;
                    }
                }
                app.core.game.res.log("Look mode: ON");
            } else {
                app.core.game.res.log("Look mode: OFF");
            }
            return Ok(());
        }

        // In Look mode, remap movement keys to move the look cursor without ticking
        if app.panels.look.mode {
            let mut moved = false;
            if app.ui.keybinds.matches("movement", "MOVE_NORTH", &key) {
                app.panels.look.cursor.y -= 1;
                app.ui.view_y = app.panels.look.cursor.y;
                moved = true;
            } else if app.ui.keybinds.matches("movement", "MOVE_SOUTH", &key) {
                app.panels.look.cursor.y += 1;
                app.ui.view_y = app.panels.look.cursor.y;
                moved = true;
            } else if app.ui.keybinds.matches("movement", "MOVE_WEST", &key) {
                app.panels.look.cursor.x -= 1;
                app.ui.view_x = app.panels.look.cursor.x;
                moved = true;
            } else if app.ui.keybinds.matches("movement", "MOVE_EAST", &key) {
                app.panels.look.cursor.x += 1;
                app.ui.view_x = app.panels.look.cursor.x;
                moved = true;
            } else if app.ui.keybinds.matches("movement", "MOVE_NORTHWEST", &key) {
                app.panels.look.cursor.x -= 1;
                app.panels.look.cursor.y -= 1;
                app.ui.view_x = app.panels.look.cursor.x;
                app.ui.view_y = app.panels.look.cursor.y;
                moved = true;
            } else if app.ui.keybinds.matches("movement", "MOVE_NORTHEAST", &key) {
                app.panels.look.cursor.x += 1;
                app.panels.look.cursor.y -= 1;
                app.ui.view_x = app.panels.look.cursor.x;
                app.ui.view_y = app.panels.look.cursor.y;
                moved = true;
            } else if app.ui.keybinds.matches("movement", "MOVE_SOUTHWEST", &key) {
                app.panels.look.cursor.x -= 1;
                app.panels.look.cursor.y += 1;
                app.ui.view_x = app.panels.look.cursor.x;
                app.ui.view_y = app.panels.look.cursor.y;
                moved = true;
            } else if app.ui.keybinds.matches("movement", "MOVE_SOUTHEAST", &key) {
                app.panels.look.cursor.x += 1;
                app.panels.look.cursor.y += 1;
                app.ui.view_x = app.panels.look.cursor.x;
                app.ui.view_y = app.panels.look.cursor.y;
                moved = true;
            } else if app.ui.keybinds.matches("movement", "WAIT", &key) {
                // no-op, but treat as handled to avoid player waiting
                moved = true;
            } else if app.ui.keybinds.matches("movement", "MOVE_UP", &key) {
                app.panels.look.cursor.z += 1;
                app.ui.view_z = app.panels.look.cursor.z;
                moved = true;
            } else if app.ui.keybinds.matches("movement", "MOVE_DOWN", &key) {
                app.panels.look.cursor.z -= 1;
                app.ui.view_z = app.panels.look.cursor.z;
                moved = true;
            } else if app.ui.keybinds.matches("viewport", "VIEW_Z_UP", &key) {
                app.panels.look.cursor.z = app.panels.look.cursor.z.saturating_add(1);
                app.ui.view_z = app.panels.look.cursor.z;
                moved = true;
            } else if app.ui.keybinds.matches("viewport", "VIEW_Z_DOWN", &key) {
                app.panels.look.cursor.z = app.panels.look.cursor.z.saturating_sub(1);
                app.ui.view_z = app.panels.look.cursor.z;
                moved = true;
            }

            if moved {
                // Prefetch around the new cursor position for smoother draw
                let radius = 20i32;
                let left = app.panels.look.cursor.x - radius;
                let top = app.panels.look.cursor.y - radius;
                let right = app.panels.look.cursor.x + radius;
                let bottom = app.panels.look.cursor.y + radius;
                app.core.game.res.world_state.world.prefetch_rect_at_z(
                    left,
                    top,
                    right,
                    bottom,
                    app.panels.look.cursor.z,
                );
                return Ok(());
            }
        }
    }
    // Inventory: toggle item auto-pickup
    if app.ui.current_tab == MenuTab::Inventory
        && app
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
        return Ok(());
    }

    // Crafting panel-specific navigation and actions
    if app.ui.current_tab == MenuTab::Crafting {
        // Get player inventory for crafting checks
        let inventory = get_player_inventory(app);
        let recipe_count = app.panels.crafting.recipe_handler.get_recipes().len();

        // Navigation: Up/Down or North/South to move selection
        if app.ui.keybinds.matches("ui", "CREDITS_SCROLL_UP", &key)
            || app.ui.keybinds.matches("movement", "MOVE_NORTH", &key)
        {
            if recipe_count > 0 {
                if app.panels.crafting.selected == 0 {
                    app.panels.crafting.selected = recipe_count - 1;
                } else {
                    app.panels.crafting.selected = app.panels.crafting.selected.saturating_sub(1);
                }
            }
            return Ok(());
        }
        if app.ui.keybinds.matches("ui", "CREDITS_SCROLL_DOWN", &key)
            || app.ui.keybinds.matches("movement", "MOVE_SOUTH", &key)
        {
            if recipe_count > 0 {
                app.panels.crafting.selected = (app.panels.crafting.selected + 1) % recipe_count;
            }
            return Ok(());
        }

        // Craft item on Enter or Activate key
        if app.ui.keybinds.matches("ui", "ACTIVATE", &key) || key == KeyCode::Enter {
            if let Some(recipe) = app
                .panels
                .crafting
                .recipe_handler
                .get_recipes()
                .get(app.panels.crafting.selected)
            {
                if app
                    .panels
                    .crafting
                    .recipe_handler
                    .can_craft(app.panels.crafting.selected, &inventory)
                {
                    if let Some(e) = app.core.game.get_player_entity() {
                        if let Ok(mut inv) = app.core.game.world.get::<&mut InvComp>(e) {
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

                            app.panels.crafting.message = Some((
                                format!(
                                    "Crafted {}x {}",
                                    recipe.quantity,
                                    itemkind_name(recipe.result)
                                ),
                                30, // Display for 30 frames (~0.5 seconds at 60 FPS)
                            ));

                            app.core.game.res.log(format!(
                                "Crafted {}x {}",
                                recipe.quantity,
                                itemkind_name(recipe.result)
                            ));
                        }
                    }
                } else {
                    app.panels.crafting.message = Some((
                        "Not enough resources to craft this item".to_string(),
                        60, // Display for 1 second at 60 FPS
                    ));
                }
            }
            return Ok(());
        }

        // Don't process movement keys in crafting panel
        if app.ui.keybinds.matches("movement", "MOVE_NORTH", &key)
            || app.ui.keybinds.matches("movement", "MOVE_SOUTH", &key)
            || app.ui.keybinds.matches("movement", "MOVE_WEST", &key)
            || app.ui.keybinds.matches("movement", "MOVE_EAST", &key)
            || app.ui.keybinds.matches("movement", "MOVE_NORTHWEST", &key)
            || app.ui.keybinds.matches("movement", "MOVE_NORTHEAST", &key)
            || app.ui.keybinds.matches("movement", "MOVE_SOUTHWEST", &key)
            || app.ui.keybinds.matches("movement", "MOVE_SOUTHEAST", &key)
            || app.ui.keybinds.matches("movement", "WAIT", &key)
            || app.ui.keybinds.matches("movement", "MOVE_UP", &key)
            || app.ui.keybinds.matches("movement", "MOVE_DOWN", &key)
        {
            return Ok(());
        }
    }

    // Inventory panel-specific navigation and actions
    if app.ui.current_tab == MenuTab::Inventory {
        // Move selection: support Up/Down keys and numpad 8/2 (MOVE_NORTH/SOUTH)
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
            return Ok(());
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
            return Ok(());
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
                    return Ok(());
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
                            return Ok(());
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
            return Ok(());
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
            return Ok(());
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
            return Ok(());
        }
    }

    // Handle corpse looting input when active
    if let crate::app_state::CorpseLootingState::SelectingCorpse {
        adjacent_entities,
        selected_corpse,
    } = &app.panels.corpse_looting
    {
        let entities_clone = adjacent_entities.clone();
        let selected = *selected_corpse;

        if app.ui.keybinds.matches("ui", "CLOSE_HELP_MENU", &key) {
            app.panels.corpse_looting = crate::app_state::CorpseLootingState::None;
            app.core.game.res.log("Cancelled corpse interaction");
            return Ok(());
        }
        if app.ui.keybinds.matches("movement", "MOVE_NORTH", &key) {
            app.panels.corpse_looting = crate::app_state::CorpseLootingState::SelectingCorpse {
                adjacent_entities: entities_clone.clone(),
                selected_corpse: selected.saturating_sub(1),
            };
            return Ok(());
        }
        if app.ui.keybinds.matches("movement", "MOVE_SOUTH", &key) {
            app.panels.corpse_looting = crate::app_state::CorpseLootingState::SelectingCorpse {
                adjacent_entities: entities_clone.clone(),
                selected_corpse: (selected + 1).min(entities_clone.len().saturating_sub(1)),
            };
            return Ok(());
        }
        if app.ui.keybinds.matches("ui", "MENU_ACTIVATE", &key) || key == KeyCode::Enter {
            if selected < entities_clone.len() {
                let entity = entities_clone[selected];
                app.panels.corpse_looting = crate::app_state::CorpseLootingState::LootingCorpse {
                    entity,
                    selected_loot_item: 0,
                    selected_player_item: 0,
                    loot_panel_focus: true,
                };
                app.core.game.res.log("Started looting corpse");
            }
            return Ok(());
        }
        return Ok(());
    }

    if let crate::app_state::CorpseLootingState::LootingCorpse {
        entity,
        selected_loot_item,
        selected_player_item,
        loot_panel_focus,
    } = &app.panels.corpse_looting
    {
        let corpse_entity = *entity;
        let mut loot_item = *selected_loot_item;
        let mut player_item = *selected_player_item;
        let mut panel_focus = *loot_panel_focus;

        if app.ui.keybinds.matches("ui", "CLOSE_HELP_MENU", &key) {
            app.panels.corpse_looting = crate::app_state::CorpseLootingState::None;
            app.core.game.res.log("Finished looting");
            return Ok(());
        }
        if app.ui.keybinds.matches("movement", "MOVE_WEST", &key)
            || app.ui.keybinds.matches("movement", "MOVE_EAST", &key)
        {
            panel_focus = !panel_focus;
        }
        if app.ui.keybinds.matches("movement", "MOVE_NORTH", &key) {
            if panel_focus {
                loot_item = loot_item.saturating_sub(1);
            } else {
                player_item = player_item.saturating_sub(1);
            }
        }
        if app.ui.keybinds.matches("movement", "MOVE_SOUTH", &key) {
            if panel_focus {
                if let Ok(inv) = app
                    .core
                    .game
                    .world
                    .get::<&lithicrivers_core::components::Inventory>(corpse_entity)
                {
                    if !inv.slots.is_empty() {
                        loot_item = (loot_item + 1).min(inv.slots.len() - 1);
                    }
                }
            } else {
                if let Some(player_entity) = app.core.game.get_player_entity() {
                    if let Ok(inv) = app
                        .core
                        .game
                        .world
                        .get::<&lithicrivers_core::components::Inventory>(player_entity)
                    {
                        if !inv.slots.is_empty() {
                            player_item = (player_item + 1).min(inv.slots.len() - 1);
                        }
                    }
                }
            }
        }
        if (app.ui.keybinds.matches("ui", "MENU_ACTIVATE", &key) || key == KeyCode::Enter)
            && panel_focus
        {
            take_item_from_corpse(app, corpse_entity, loot_item);
            // Adjust selection if needed after taking item
            if let Ok(inv) = app
                .core
                .game
                .world
                .get::<&lithicrivers_core::components::Inventory>(corpse_entity)
            {
                if loot_item >= inv.slots.len() && !inv.slots.is_empty() {
                    loot_item = inv.slots.len() - 1;
                }
            }
        }

        app.panels.corpse_looting = crate::app_state::CorpseLootingState::LootingCorpse {
            entity: corpse_entity,
            selected_loot_item: loot_item,
            selected_player_item: player_item,
            loot_panel_focus: panel_focus,
        };
        return Ok(());
    }

    // Handle combat-specific input when combat is active
    if let CombatUiState::Active {
        current_move,
        current_enemy,
        enemy_timers,
        move_scroll_offset,
    } = &mut app.combat
    {
        // Move selection (Up/Down) - with scrolling support
        let max_moves = get_available_moves().len();
        const VISIBLE_MOVES: usize = 4; // Number of moves visible at once

        if app.ui.keybinds.matches("movement", "MOVE_NORTH", &key)
            || app.ui.keybinds.matches("movement", "MOVE_UP", &key)
        {
            if *current_move == 0 {
                *current_move = max_moves - 1;
                // Scroll to show the last move
                *move_scroll_offset = max_moves.saturating_sub(VISIBLE_MOVES);
            } else {
                *current_move -= 1;
                // Scroll up if needed
                if *current_move < *move_scroll_offset {
                    *move_scroll_offset = *current_move;
                }
            }
            return Ok(());
        }
        if app.ui.keybinds.matches("movement", "MOVE_SOUTH", &key)
            || app.ui.keybinds.matches("movement", "MOVE_DOWN", &key)
        {
            *current_move = (*current_move + 1) % max_moves;
            // Scroll down if needed
            if *current_move >= *move_scroll_offset + VISIBLE_MOVES {
                *move_scroll_offset = (*current_move + 1).saturating_sub(VISIBLE_MOVES);
            }
            // Handle wrap-around to beginning
            if *current_move == 0 {
                *move_scroll_offset = 0;
            }
            return Ok(());
        }

        // Enemy/target selection (Left/Right) - get actual enemy count
        let enemy_count = get_combat_enemy_count(&mut app.core.game);
        if enemy_count > 0 {
            if app.ui.keybinds.matches("movement", "MOVE_WEST", &key) {
                *current_enemy = if *current_enemy == 0 {
                    enemy_count - 1
                } else {
                    *current_enemy - 1
                };
                return Ok(());
            }
            if app.ui.keybinds.matches("movement", "MOVE_EAST", &key) {
                *current_enemy = (*current_enemy + 1) % enemy_count;
                return Ok(());
            }
        }

        // Use selected move (Space/Enter) - queue the action instead of immediate execution
        if app.ui.keybinds.matches("ui", "MENU_ACTIVATE", &key) {
            // Check if combat is actually active before allowing move queuing
            if !app.core.game.res.player_state.combat_active {
                app.core.game.res.log("Cannot use moves - not in combat!".to_string());
                return Ok(());
            }

            // Get the player entity
            if let Some(player_entity) = app.core.game.get_player_entity() {
                // Get available moves and selected move
                let available_moves = lithicrivers_core::moves::get_available_moves();
                if let Some(selected_move) = available_moves.get(*current_move) {
                    // Check if player can use this move (only energy, timing handled by action queue)
                    let can_use = if let Ok(energy) =
                        app.core
                            .game
                            .world
                            .get::<&lithicrivers_core::components::Energy>(player_entity)
                    {
                        energy.current >= selected_move.energy_cost
                    } else {
                        false
                    };

                    if !can_use {
                        app.core.game.res.log("Cannot use this move!".to_string());
                        return Ok(());
                    }

                    // Find target entity if needed
                    let target_entity = if *current_move != 3 {
                        // Not escape move - need a target
                        // Find the actual entity for the selected enemy
                        let mut enemy_count = 0;
                        let mut target = None;
                        for (entity, (_, combat, _)) in app
                            .core
                            .game
                            .world
                            .query::<(
                                &lithicrivers_core::components::Position,
                                &lithicrivers_core::components::Combat,
                                &lithicrivers_core::components::GameEntity,
                            )>()
                            .iter()
                        {
                            if entity == player_entity || !combat.triggered {
                                continue;
                            }
                            // Skip dead enemies
                            if app.core.game.world.get::<&lithicrivers_core::components::Dead>(entity).is_ok() {
                                continue;
                            }
                            if enemy_count == *current_enemy {
                                target = Some(entity);
                                break;
                            }
                            enemy_count += 1;
                        }
                        
                        // Validate we have a target for moves that need one
                        if target.is_none() {
                            app.core.game.res.log("No valid target for this move!".to_string());
                            return Ok(());
                        }
                        target
                    } else {
                        None
                    };

                    // Create queued action
                    let execution_time = selected_move.execution_time_ticks;
                    let action = lithicrivers_core::moves::QueuedAction {
                        entity: player_entity,
                        action: lithicrivers_core::moves::CombatAction::PlayerMove {
                            move_data: selected_move.clone(),
                            target_entity,
                            target_position: None, // Not used for current moves
                        },
                        execution_time_ticks: execution_time,
                        remaining_time_ticks: execution_time,
                    };

                    // Ensure player has an action queue component
                    if app
                        .core
                        .game
                        .world
                        .get::<&lithicrivers_core::moves::ActionQueue>(player_entity)
                        .is_err()
                    {
                        app.core
                            .game
                            .world
                            .insert_one(player_entity, lithicrivers_core::moves::ActionQueue::new())
                            .ok();
                    }

                    // Queue the action
                    if let Ok(mut queue) =
                        app.core
                            .game
                            .world
                            .get::<&mut lithicrivers_core::moves::ActionQueue>(player_entity)
                    {
                        queue.queue_action(action);

                        // If no current action, start this one
                        if queue.current_action.is_none() {
                            queue.start_next_action();
                        }

                        app.core.game.res.log(format!(
                            "Queued {} against target {}",
                            selected_move.name, current_enemy
                        ));
                    }
                } else {
                    app.core.game.res.log("Invalid move selected!".to_string());
                }

                // Process game ticks to allow combat actions to execute and check for combat end
                // We may need multiple ticks for actions to complete
                for _ in 0..10 {
                    // Max 10 ticks to prevent infinite loops
                    let tick_result = app.core.game.tick();
                    if tick_result.contains(GameTickResult::CombatEnded) {
                        app.combat = CombatUiState::None;
                        break;
                    }
                    // If nothing significant happened, stop ticking
                    if tick_result.contains(GameTickResult::NoAction) {
                        break;
                    }
                }
            }
            return Ok(());
        }

        // Exit combat (Escape)
        if app.ui.keybinds.matches("ui", "CLOSE_HELP_MENU", &key) {
            // Clear the player's action queue when manually exiting combat
            if let Some(player_entity) = app.core.game.get_player_entity() {
                if let Ok(mut queue) = app
                    .core
                    .game
                    .world
                    .get::<&mut lithicrivers_core::moves::ActionQueue>(player_entity)
                {
                    queue.clear();
                    app.core.game.res.log("Cleared action queue on combat exit");
                }
            }

            // End combat for all nearby enemies (this was missing!)
            app.core.game.end_combat_around_player();

            app.combat = CombatUiState::None;
            app.core.game.res.log("Exited combat (cheat mode)");
            return Ok(());
        }

        // Block all other input during combat
        return Ok(());
    }

    // only process movement if we're currely on the world panel
    if app.ui.current_tab == MenuTab::World {
        if app.ui.keybinds.matches_movement(&key) {
            if app.ui.keybinds.matches("movement", "MOVE_NORTH", &key) {
                app.core.game.queue_player_move(0, -1);
            }
            if app.ui.keybinds.matches("movement", "MOVE_SOUTH", &key) {
                app.core.game.queue_player_move(0, 1);
            }
            if app.ui.keybinds.matches("movement", "MOVE_WEST", &key) {
                app.core.game.queue_player_move(-1, 0);
            }
            if app.ui.keybinds.matches("movement", "MOVE_EAST", &key) {
                app.core.game.queue_player_move(1, 0);
            }
            if app.ui.keybinds.matches("movement", "MOVE_NORTHWEST", &key) {
                app.core.game.queue_player_move(-1, -1);
            }
            if app.ui.keybinds.matches("movement", "MOVE_NORTHEAST", &key) {
                app.core.game.queue_player_move(1, -1);
            }
            if app.ui.keybinds.matches("movement", "MOVE_SOUTHWEST", &key) {
                app.core.game.queue_player_move(-1, 1);
            }
            if app.ui.keybinds.matches("movement", "MOVE_SOUTHEAST", &key) {
                app.core.game.queue_player_move(1, 1);
            }
            if app.ui.keybinds.matches("movement", "WAIT", &key) {
                app.core.game.queue_player_move(0, 0);
            }
            if app.ui.keybinds.matches("movement", "MOVE_UP", &key) {
                app.core.game.queue_player_move_z(1);
            }
            if app.ui.keybinds.matches("movement", "MOVE_DOWN", &key) {
                app.core.game.queue_player_move_z(-1);
            }

            let tick_result = app.core.game.tick();
            if tick_result.contains(GameTickResult::CombatTriggered) {
                app.combat = CombatUiState::Active {
                    current_move: 0,
                    current_enemy: 0,
                    enemy_timers: vec![], // Deprecated - using ActionQueue now
                    move_scroll_offset: 0,
                };
            }
            if tick_result.contains(GameTickResult::CombatEnded) {
                app.combat = CombatUiState::None;
            }
            app.snap_view_to_player_z();
            return Ok(());
        }

        if app.ui.keybinds.matches("action", "MINE", &key) {
            app.core.game.queue_mine();
            let tick_result = app.core.game.tick();
            if tick_result.contains(GameTickResult::MiningSuccess) {
                app.snap_view_to_player_z();
            }
            if tick_result.contains(GameTickResult::CombatEnded) {
                app.combat = CombatUiState::None;
            }
            return Ok(());
        }

        if app.ui.keybinds.matches("action", "INTERACT", &key) {
            handle_corpse_interaction(app);
            return Ok(());
        }

        if app.ui.keybinds.matches("scale", "SCALE_UP", &key) {
            app.ui.scale = match app.ui.scale {
                Scale::Small => Scale::Medium,
                Scale::Medium => Scale::Large,
                Scale::Large => Scale::Large,
            };
            return Ok(());
        }
        if app.ui.keybinds.matches("scale", "SCALE_DOWN", &key) {
            app.ui.scale = match app.ui.scale {
                Scale::Large => Scale::Medium,
                Scale::Medium => Scale::Small,
                Scale::Small => Scale::Small,
            };
            return Ok(());
        }
        if app.ui.keybinds.matches("scale", "SCALE_RESET", &key) {
            app.ui.scale = Scale::Small;
            return Ok(());
        }
    }
    // UI: Quit
    if app.ui.keybinds.matches("ui", "QUIT", &key) {
        app.core.game.res.log("Quit requested (keybind)");
        tracing::info!(target: "game", "quit_requested tick={}", app.core.game.res.time.tick);
        app.core.should_quit = true;
        return Ok(());
    }
    // UI: Menu activation and paging
    if app.ui.keybinds.matches("ui", "MENU_ACTIVATE", &key) {
        app.activate_menu();
        return Ok(());
    }
    // Block menu navigation during combat
    if !app.combat.is_active() {
        if app.ui.keybinds.matches("ui", "MENU_PREV", &key) {
            app.ui.current_tab = app.ui.current_tab.prev();
            return Ok(());
        }
        if app.ui.keybinds.matches("ui", "MENU_NEXT", &key) {
            app.ui.current_tab = app.ui.current_tab.next();
            return Ok(());
        }
    }
    // Credits scroll
    if app.ui.current_tab == MenuTab::Credits {
        if app.ui.keybinds.matches("ui", "CREDITS_SCROLL_UP", &key) {
            app.panels.credits.scroll = app.panels.credits.scroll.saturating_sub(1);
            return Ok(());
        }
        if app.ui.keybinds.matches("ui", "CREDITS_SCROLL_DOWN", &key) {
            app.panels.credits.scroll = app.panels.credits.scroll.saturating_add(1);
            return Ok(());
        }
    }
    // Help scroll
    if app.ui.current_tab == MenuTab::Help {
        match key {
            KeyCode::Up => {
                app.panels.help.scroll = app.panels.help.scroll.saturating_sub(1);
                return Ok(());
            }
            KeyCode::Down => {
                app.panels.help.scroll = app.panels.help.scroll.saturating_add(1);
                return Ok(());
            }
            _ => {}
        }
    }
    // View Z slice up/down
    if app.ui.keybinds.matches("viewport", "VIEW_Z_UP", &key) {
        if app.ui.current_tab == MenuTab::Credits {
            app.panels.credits.scroll = app.panels.credits.scroll.saturating_sub(10);
        } else {
            app.ui.view_z = app.ui.view_z.saturating_add(1);
        }
        return Ok(());
    }
    if app.ui.keybinds.matches("viewport", "VIEW_Z_DOWN", &key) {
        if app.ui.current_tab == MenuTab::Credits {
            app.panels.credits.scroll = app.panels.credits.scroll.saturating_add(10);
        } else {
            app.ui.view_z = app.ui.view_z.saturating_sub(1);
        }
        return Ok(());
    }
    // Save/Load via config
    if app.ui.keybinds.matches("ui", "SAVE_JSON", &key) {
        tracing::info!(target: "game", "save_begin path=save.json tick={}", app.core.game.res.time.tick);
        app.core
            .game
            .save_json("save.json")
            .map_err(|e| format!("save_json error: {:?}", e))?;
        app.core.game.res.log("Saved to save.json");
        tracing::info!(target: "game", "save_end path=save.json tick={}", app.core.game.res.time.tick);
        return Ok(());
    }
    if app.ui.keybinds.matches("ui", "LOAD_JSON", &key) {
        tracing::info!(target: "game", "load_begin path=save.json tick={}", app.core.game.res.time.tick);
        app.core
            .game
            .load_json("save.json")
            .map_err(|e| format!("load_json error: {:?}", e))?;
        app.core.game.res.log("Loaded from save.json");
        tracing::info!(target: "game", "load_end path=save.json tick={}", app.core.game.res.time.tick);
        return Ok(());
    }

    Ok(())
}

/// Get the current count of living combat enemies adjacent to the player
fn get_combat_enemy_count(game: &mut lithicrivers_core::Game) -> usize {
    use lithicrivers_core::components::{Combat, GameEntity, Position};

    let mut count = 0;

    if let Some(player_entity) = game.get_player_entity() {
        if let Ok(player_pos) = game.world.get::<&Position>(player_entity) {
            let player_pos = *player_pos;

            // Define the 8 adjacent positions around the player (same as combat_trigger_system)
            let adjacent_positions = [
                (player_pos.x - 1, player_pos.y - 1), // NW
                (player_pos.x, player_pos.y - 1),     // N
                (player_pos.x + 1, player_pos.y - 1), // NE
                (player_pos.x - 1, player_pos.y),     // W
                (player_pos.x + 1, player_pos.y),     // E
                (player_pos.x - 1, player_pos.y + 1), // SW
                (player_pos.x, player_pos.y + 1),     // S
                (player_pos.x + 1, player_pos.y + 1), // SE
            ];

            // Look for adjacent combat entities that are alive and in combat
            for (entity, (pos, combat, _)) in game
                .world
                .query::<(&Position, &Combat, &GameEntity)>()
                .iter()
            {
                if entity == player_entity {
                    continue; // Skip player
                }

                // Check if enemy is in any of the 8 adjacent positions
                let is_adjacent = adjacent_positions.iter().any(|&(adj_x, adj_y)| {
                    pos.x == adj_x && pos.y == adj_y && pos.z == player_pos.z
                });

                if is_adjacent && combat.triggered {
                    // Skip dead enemies
                    if game
                        .world
                        .get::<&lithicrivers_core::components::Dead>(entity)
                        .is_ok()
                    {
                        continue;
                    }

                    count += 1;
                }
            }
        }
    }

    count
}

/// Handle corpse interaction - find adjacent corpses and initiate looting
fn handle_corpse_interaction(app: &mut App) {
    use crate::app_state::CorpseLootingState;
    use lithicrivers_core::components::{EntityKind, Inventory as InvComp, Position};

    // Get player position
    let player_pos = if let Some(player_entity) = app.core.game.get_player_entity() {
        if let Ok(pos) = app.core.game.world.get::<&Position>(player_entity) {
            *pos
        } else {
            app.core.game.res.log("Cannot find player position");
            return;
        }
    } else {
        app.core.game.res.log("Cannot find player entity");
        return;
    };

    // Find all adjacent corpses (3x3 grid centered on player)
    let mut adjacent_corpses = Vec::new();
    for (entity, (pos, entity_kind, _inv)) in app
        .core
        .game
        .world
        .query::<(&Position, &EntityKind, &InvComp)>()
        .iter()
    {
        if *entity_kind == EntityKind::Corpse {
            let dx = (pos.x - player_pos.x).abs();
            let dy = (pos.y - player_pos.y).abs();
            let dz = (pos.z - player_pos.z).abs();

            // Check if within 3x3 grid on same Z level
            if dx <= 1 && dy <= 1 && dz == 0 {
                adjacent_corpses.push(entity);
            }
        }
    }

    if adjacent_corpses.is_empty() {
        app.core.game.res.log("No corpses nearby to loot");
        return;
    }

    if adjacent_corpses.len() == 1 {
        // Single corpse - go directly to looting
        let entity = adjacent_corpses[0];
        app.panels.corpse_looting = CorpseLootingState::LootingCorpse {
            entity,
            selected_loot_item: 0,
            selected_player_item: 0,
            loot_panel_focus: true,
        };
        app.core.game.res.log("Started looting corpse");
    } else {
        // Multiple corpses - show selection modal
        app.panels.corpse_looting = CorpseLootingState::SelectingCorpse {
            adjacent_entities: adjacent_corpses,
            selected_corpse: 0,
        };
        app.core.game.res.log("Choose which corpse to loot");
    }
}

/// Take an item from a corpse and add it to the player's inventory
fn take_item_from_corpse(app: &mut App, corpse_entity: hecs::Entity, item_idx: usize) {
    use lithicrivers_core::components::{itemkind_name, Inventory as InvComp, ItemStack};

    // First, get the item info we need before mutable borrows
    let item_info = if let Ok(inv) = app.core.game.world.get::<&InvComp>(corpse_entity) {
        if let Some(stack) = inv.slots.get(item_idx) {
            if stack.qty > 0 {
                Some((stack.kind, itemkind_name(stack.kind).to_string()))
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    if let Some((item_kind, item_name)) = item_info {
        // Take one item from corpse
        if let Ok(mut corpse_inv) = app.core.game.world.get::<&mut InvComp>(corpse_entity) {
            if let Some(stack) = corpse_inv.slots.get_mut(item_idx) {
                if stack.qty > 0 {
                    stack.qty -= 1;

                    // Remove empty stacks
                    if stack.qty == 0 {
                        corpse_inv.slots.remove(item_idx);
                    }
                }
            }
        }

        // Add to player inventory
        if let Some(player_entity) = app.core.game.get_player_entity() {
            if let Ok(mut player_inv) = app.core.game.world.get::<&mut InvComp>(player_entity) {
                // Try to stack with existing item
                let mut added = false;
                for stack in player_inv.slots.iter_mut() {
                    if stack.kind == item_kind && stack.qty < 1000 {
                        stack.qty += 1;
                        added = true;
                        break;
                    }
                }

                // Create new stack if couldn't add to existing
                if !added {
                    player_inv.slots.push(ItemStack {
                        kind: item_kind,
                        qty: 1,
                    });
                }

                app.core.game.res.log(format!("Took {}", item_name));
            }
        }
    }
}

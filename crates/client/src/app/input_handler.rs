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

use crate::app_state::HotbarAssignmentState;
use crate::{ui::panels::get_player_inventory, App, CombatUiState, MenuTab, Scale, SplashState};

/// Calculate the maximum scroll value for the credits panel
fn get_max_credits_scroll(app: &App) -> u16 {
    // Count lines in credits text and subtract visible area height
    let line_count = app.panels.credits.text.lines().count() as u16;
    // Assume panel height is around 20 lines (terminal height minus UI elements)
    // This is a conservative estimate - in practice the panel might be larger
    let visible_lines = 20;
    line_count.saturating_sub(visible_lines)
}

/// Calculate the maximum scroll value for the help panel
fn get_max_help_scroll(app: &App) -> u16 {
    // For help panel, we need to count the dynamically generated lines
    // This is an approximation based on the keybinds structure
    let base_lines = 15; // Movement diagram and basic text
    let keybind_categories = 5; // viewport, scale, action, ui, inventory
    let avg_keybinds_per_category = 8;
    let total_lines = base_lines + (keybind_categories * (avg_keybinds_per_category + 2)) as u16; // +2 for category header and spacing
    let visible_lines = 20;
    total_lines.saturating_sub(visible_lines)
}

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
    tracing::info!(target: "game", "key pressed: {:?}", key);

    // Special debug for F key
    if key == KeyCode::Char('f') || key == KeyCode::Char('F') {
        tracing::info!(target: "game", "F key detected! Current tab: {:?}, splash state: {:?}", 
            app.ui.current_tab, app.splash.state);
    }

    // First, handle configurable keybind actions
    if app.ui.current_tab == MenuTab::World {
        // In Break/Place mode, handle 9-directional actions FIRST (before look toggle)
        use crate::app_state::BuildMode;
        if app.panels.build.mode != BuildMode::Movement {
            let mut handled = false;

            // Map build keys to 9 directions using keybind configuration
            let direction_offset = match app.panels.build.mode {
                BuildMode::Break => {
                    if app.ui.keybinds.matches("build", "BREAK_NORTHWEST", &key) {
                        Some((-1, -1))
                    } else if app.ui.keybinds.matches("build", "BREAK_NORTH", &key) {
                        Some((0, -1))
                    } else if app.ui.keybinds.matches("build", "BREAK_NORTHEAST", &key) {
                        Some((1, -1))
                    } else if app.ui.keybinds.matches("build", "BREAK_WEST", &key) {
                        Some((-1, 0))
                    } else if app.ui.keybinds.matches("build", "BREAK_CENTER", &key) {
                        Some((0, 0))
                    } else if app.ui.keybinds.matches("build", "BREAK_EAST", &key) {
                        Some((1, 0))
                    } else if app.ui.keybinds.matches("build", "BREAK_SOUTHWEST", &key) {
                        Some((-1, 1))
                    } else if app.ui.keybinds.matches("build", "BREAK_SOUTH", &key) {
                        Some((0, 1))
                    } else if app.ui.keybinds.matches("build", "BREAK_SOUTHEAST", &key) {
                        Some((1, 1))
                    } else {
                        None
                    }
                }
                BuildMode::Place => {
                    if app.ui.keybinds.matches("build", "PLACE_NORTHWEST", &key) {
                        Some((-1, -1))
                    } else if app.ui.keybinds.matches("build", "PLACE_NORTH", &key) {
                        Some((0, -1))
                    } else if app.ui.keybinds.matches("build", "PLACE_NORTHEAST", &key) {
                        Some((1, -1))
                    } else if app.ui.keybinds.matches("build", "PLACE_WEST", &key) {
                        Some((-1, 0))
                    } else if app.ui.keybinds.matches("build", "PLACE_CENTER", &key) {
                        Some((0, 0))
                    } else if app.ui.keybinds.matches("build", "PLACE_EAST", &key) {
                        Some((1, 0))
                    } else if app.ui.keybinds.matches("build", "PLACE_SOUTHWEST", &key) {
                        Some((-1, 1))
                    } else if app.ui.keybinds.matches("build", "PLACE_SOUTH", &key) {
                        Some((0, 1))
                    } else if app.ui.keybinds.matches("build", "PLACE_SOUTHEAST", &key) {
                        Some((1, 1))
                    } else {
                        None
                    }
                }
                BuildMode::Movement => None, // This shouldn't happen due to outer condition
            };

            if let Some((dx, dy)) = direction_offset {
                if let Some(player_entity) = app.core.game.get_player_entity() {
                    // Extract coordinates first to avoid borrow conflicts
                    let (target_x, target_y, target_z) = if let Ok(player_pos) = app
                        .core
                        .game
                        .world
                        .get::<&lithicrivers_core::components::Position>(player_entity)
                    {
                        (player_pos.x + dx, player_pos.y + dy, player_pos.z)
                    } else {
                        return Ok(()); // No position component
                    };

                    match app.panels.build.mode {
                        BuildMode::Break => {
                            // Queue mining at target position
                            app.core.game.queue_mine_at(target_x, target_y, target_z);
                            let tick_result = app.core.game.tick();
                            if tick_result
                                .contains(lithicrivers_core::game::GameTickResult::MiningSuccess)
                            {
                                app.core.game.res.log(format!(
                                    "Broke block at ({}, {}, {})",
                                    target_x, target_y, target_z
                                ));
                            }
                            handled = true;
                        }
                        BuildMode::Place => {
                            // Check if player has a block assigned to the selected hotbar slot
                            let selected_slot = app.panels.build.selected_hotbar_slot;
                            if let Some(block_item) =
                                app.panels.build.hotbar_assignments[selected_slot]
                            {
                                // Check if this item can be placed as a block
                                if let Some(tile_kind) =
                                    lithicrivers_core::tiles::TileKind::from_item_kind(block_item)
                                {
                                    // Check if player has this item in inventory
                                    if has_item_in_inventory(app, block_item) {
                                        // Check if placement position is valid
                                        if is_valid_placement_position(
                                            app, target_x, target_y, target_z,
                                        ) {
                                            // Place the block in the world
                                            app.core.game.res.world_state.world.set_tile_cached(
                                                target_x, target_y, target_z, tile_kind,
                                            );

                                            // Remove 1 from player inventory
                                            remove_item_from_inventory(app, block_item, 1);

                                            let item_name =
                                                lithicrivers_core::components::itemkind_name(
                                                    block_item,
                                                );
                                            app.core.game.res.log(format!(
                                                "Placed {} at ({}, {}, {})",
                                                item_name, target_x, target_y, target_z
                                            ));
                                        } else {
                                            app.core.game.res.log("Cannot place block here");
                                        }
                                    } else {
                                        let item_name =
                                            lithicrivers_core::components::itemkind_name(
                                                block_item,
                                            );
                                        app.core
                                            .game
                                            .res
                                            .log(format!("No {} in inventory", item_name));
                                    }
                                } else {
                                    let item_name =
                                        lithicrivers_core::components::itemkind_name(block_item);
                                    app.core
                                        .game
                                        .res
                                        .log(format!("{} cannot be placed as a block", item_name));
                                }
                            } else {
                                app.core.game.res.log(format!(
                                    "No block assigned to slot F{}",
                                    selected_slot + 1
                                ));
                            }
                            handled = true;
                        }
                        BuildMode::Movement => {
                            // This case shouldn't happen due to the outer condition
                        }
                    }
                }
            }

            if handled {
                return Ok(());
            }
        }

        // Toggle Look mode (only if not handled by build mode above)
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

        // Toggle Build mode (cycles between Movement -> Break -> Place)
        if app
            .ui
            .keybinds
            .matches("build", "TOGGLE_BREAK_PLACE_MODE", &key)
        {
            // Prevent mode switching during combat
            if lithicrivers_core::combat_state_manager::CombatStateManager::is_in_combat(
                &app.core.game.res,
            ) {
                app.core.game.res.log("Cannot change modes during combat!");
                return Ok(());
            }

            use crate::app_state::BuildMode;
            app.panels.build.mode = app.panels.build.mode.next();
            app.core
                .game
                .res
                .log(format!("Build mode: {}", app.panels.build.mode.name()));
            return Ok(());
        }

        // Handle hotbar selection using configured keys
        let hotbar_slot = if app.ui.keybinds.matches("hotbar", "HOTBAR_SLOT_1", &key) {
            Some(0)
        } else if app.ui.keybinds.matches("hotbar", "HOTBAR_SLOT_2", &key) {
            Some(1)
        } else if app.ui.keybinds.matches("hotbar", "HOTBAR_SLOT_3", &key) {
            Some(2)
        } else if app.ui.keybinds.matches("hotbar", "HOTBAR_SLOT_4", &key) {
            Some(3)
        } else if app.ui.keybinds.matches("hotbar", "HOTBAR_SLOT_5", &key) {
            Some(4)
        } else if app.ui.keybinds.matches("hotbar", "HOTBAR_SLOT_6", &key) {
            Some(5)
        } else if app.ui.keybinds.matches("hotbar", "HOTBAR_SLOT_7", &key) {
            Some(6)
        } else if app.ui.keybinds.matches("hotbar", "HOTBAR_SLOT_8", &key) {
            Some(7)
        } else if app.ui.keybinds.matches("hotbar", "HOTBAR_SLOT_9", &key) {
            Some(8)
        } else if app.ui.keybinds.matches("hotbar", "HOTBAR_SLOT_10", &key) {
            Some(9)
        } else if app.ui.keybinds.matches("hotbar", "HOTBAR_SLOT_11", &key) {
            Some(10)
        } else if app.ui.keybinds.matches("hotbar", "HOTBAR_SLOT_12", &key) {
            Some(11)
        } else {
            None
        };

        if let Some(slot) = hotbar_slot {
            let now = std::time::Instant::now();
            const DOUBLE_TAP_WINDOW_MS: u128 = 500; // 500ms window for double-tap

            // Check for double-tap
            let is_double_tap =
                if let Some((last_slot, last_time)) = app.panels.build.last_fkey_press {
                    last_slot == slot
                        && now.duration_since(last_time).as_millis() <= DOUBLE_TAP_WINDOW_MS
                } else {
                    false
                };

            if is_double_tap {
                // Double-tap detected - open block picker
                if app.ui.current_tab == MenuTab::World {
                    // Get available blocks from player inventory
                    let available_blocks = get_blocks_from_inventory(app);
                    if !available_blocks.is_empty() {
                        app.panels.hotbar_assignment = HotbarAssignmentState::ChoosingBlock {
                            hotbar_slot: slot,
                            available_blocks,
                            selected_block: 0,
                        };
                        app.core
                            .game
                            .res
                            .log(format!("Choose block for slot F{}", slot + 1));
                    } else {
                        app.core.game.res.log("No blocks available in inventory");
                    }
                }
                // Reset the double-tap tracker
                app.panels.build.last_fkey_press = None;
            } else {
                // Single tap - select hotbar slot
                app.panels.build.selected_hotbar_slot = slot;
                app.core
                    .game
                    .res
                    .log(format!("Selected hotbar slot: {}", slot + 1));
                // Update double-tap tracker
                app.panels.build.last_fkey_press = Some((slot, now));
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

    // Handle hotbar assignment input when active
    if let HotbarAssignmentState::ChoosingBlock {
        hotbar_slot,
        available_blocks,
        selected_block,
    } = &app.panels.hotbar_assignment
    {
        let slot = *hotbar_slot;
        let blocks = available_blocks.clone();
        let mut selected = *selected_block;

        if app.ui.keybinds.matches("ui", "CLOSE_HELP_MENU", &key) {
            app.panels.hotbar_assignment = HotbarAssignmentState::None;
            app.core.game.res.log("Cancelled block assignment");
            return Ok(());
        }
        if app.ui.keybinds.matches("movement", "MOVE_NORTH", &key) {
            selected = selected.saturating_sub(1);
        }
        if app.ui.keybinds.matches("movement", "MOVE_SOUTH", &key) {
            if selected < blocks.len().saturating_sub(1) {
                selected += 1;
            }
        }
        if app.ui.keybinds.matches("ui", "MENU_ACTIVATE", &key) || key == KeyCode::Enter {
            if selected < blocks.len() {
                let chosen_block = blocks[selected];
                app.panels.build.hotbar_assignments[slot] = Some(chosen_block);
                app.panels.hotbar_assignment = HotbarAssignmentState::None;
                app.core.game.res.log(format!(
                    "Assigned {} to slot F{}",
                    lithicrivers_core::components::itemkind_name(chosen_block),
                    slot + 1
                ));
            }
            return Ok(());
        }

        app.panels.hotbar_assignment = HotbarAssignmentState::ChoosingBlock {
            hotbar_slot: slot,
            available_blocks: blocks,
            selected_block: selected,
        };
        return Ok(());
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

    // Handle NPC interaction input when active
    if let crate::app_state::NPCInteractionState::SelectingNPC {
        adjacent_npcs,
        selected_npc,
    } = &app.panels.npc_interaction
    {
        let npcs_clone = adjacent_npcs.clone();
        let selected = *selected_npc;
        if app.ui.keybinds.matches("ui", "CLOSE_HELP_MENU", &key) {
            app.panels.npc_interaction = crate::app_state::NPCInteractionState::None;
            app.core.game.res.log("Cancelled NPC interaction");
            return Ok(());
        }
        if app.ui.keybinds.matches("movement", "MOVE_NORTH", &key) {
            app.panels.npc_interaction = crate::app_state::NPCInteractionState::SelectingNPC {
                adjacent_npcs: npcs_clone.clone(),
                selected_npc: selected.saturating_sub(1),
            };
            return Ok(());
        }
        if app.ui.keybinds.matches("movement", "MOVE_SOUTH", &key) {
            app.panels.npc_interaction = crate::app_state::NPCInteractionState::SelectingNPC {
                adjacent_npcs: npcs_clone.clone(),
                selected_npc: (selected + 1).min(npcs_clone.len().saturating_sub(1)),
            };
            return Ok(());
        }
        if app.ui.keybinds.matches("ui", "MENU_ACTIVATE", &key) || key == KeyCode::Enter {
            if selected < npcs_clone.len() {
                let (npc_entity, npc_name) = npcs_clone[selected].clone();
                // Get real NPC data from dialogue system or use the first available NPC
                let npc_data = if let Some(npc) = app.panels.dialogue_system.npcs.first() {
                    npc.clone()
                } else {
                    // Fallback dummy NPC data if no NPCs in dialogue system
                    crate::app_state::NPCData {
                        name: npc_name.clone(),
                        portrait: "".to_string(),
                        dialogue_type: crate::app_state::DialogueType::Linear,
                        current_mood: crate::app_state::NPCMood::Neutral,
                        initial_dialogue: 0,
                        met_before: false,
                        has_quest: false,
                        shop_inventory: vec![],
                    }
                };

                app.panels.npc_interaction = crate::app_state::NPCInteractionState::InDialogue {
                    npc_entity,
                    npc_data,
                    current_dialogue_node: Some(0), // Start with first dialogue node
                    selected_choice: 0,
                    conversation_log: vec![],
                };
                app.core
                    .game
                    .res
                    .log(format!("Started conversation with {}", npc_name));
            }
            return Ok(());
        }
        return Ok(());
    }

    if let crate::app_state::NPCInteractionState::InDialogue {
        npc_entity,
        current_dialogue_node: _,
        selected_choice,
        ..
    } = &app.panels.npc_interaction
    {
        let entity = *npc_entity;
        let mut choice = *selected_choice;

        if app.ui.keybinds.matches("ui", "CLOSE_HELP_MENU", &key) {
            app.panels.npc_interaction = crate::app_state::NPCInteractionState::None;
            app.core.game.res.log("Ended conversation");
            return Ok(());
        }

        // For now, just handle basic navigation - we'll implement full dialogue later
        if app.ui.keybinds.matches("movement", "MOVE_NORTH", &key) {
            choice = choice.saturating_sub(1);
        }
        if app.ui.keybinds.matches("movement", "MOVE_SOUTH", &key) {
            choice = (choice + 1).min(2); // Limit to reasonable number of choices for now
        }
        if app.ui.keybinds.matches("ui", "MENU_ACTIVATE", &key) || key == KeyCode::Enter {
            // For now, just end the conversation
            app.panels.npc_interaction = crate::app_state::NPCInteractionState::None;
            app.core.game.res.log("Conversation ended");
            return Ok(());
        }

        // Update state
        // Get real NPC data from dialogue system or use the first available NPC
        let npc_data = if let Some(npc) = app.panels.dialogue_system.npcs.first() {
            npc.clone()
        } else {
            // Fallback dummy NPC data if no NPCs in dialogue system
            crate::app_state::NPCData {
                name: "Unknown NPC".to_string(),
                portrait: "".to_string(),
                dialogue_type: crate::app_state::DialogueType::Linear,
                current_mood: crate::app_state::NPCMood::Neutral,
                initial_dialogue: 0,
                met_before: false,
                has_quest: false,
                shop_inventory: vec![],
            }
        };

        app.panels.npc_interaction = crate::app_state::NPCInteractionState::InDialogue {
            npc_entity: entity,
            npc_data,
            current_dialogue_node: Some(0),
            selected_choice: choice,
            conversation_log: vec![],
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
                app.core
                    .game
                    .res
                    .log("Cannot use moves - not in combat!".to_string());
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
                            if app
                                .core
                                .game
                                .world
                                .get::<&lithicrivers_core::components::Dead>(entity)
                                .is_ok()
                            {
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
                            app.core
                                .game
                                .res
                                .log("No valid target for this move!".to_string());
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

        // Debug F key specifically
        if key == KeyCode::Char('f') || key == KeyCode::Char('F') {
            tracing::info!(target: "game", "Checking F key against INTERACT keybind");
            let matches = app.ui.keybinds.matches("action", "INTERACT", &key);
            tracing::info!(target: "game", "F key matches INTERACT: {}", matches);
        }

        if app.ui.keybinds.matches("action", "INTERACT", &key) {
            tracing::info!(target: "game", "F key matched INTERACT action, calling handle_interaction");
            handle_interaction(app);
            tracing::info!(target: "game", "handle_interaction completed");
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
        let max_scroll = get_max_credits_scroll(app);
        if app.ui.keybinds.matches("ui", "CREDITS_SCROLL_UP", &key) {
            app.panels.credits.scroll = app.panels.credits.scroll.saturating_sub(1);
            return Ok(());
        }
        if app.ui.keybinds.matches("ui", "CREDITS_SCROLL_DOWN", &key) {
            app.panels.credits.scroll =
                (app.panels.credits.scroll.saturating_add(1)).min(max_scroll);
            return Ok(());
        }
    }
    // Help scroll
    if app.ui.current_tab == MenuTab::Help {
        let max_scroll = get_max_help_scroll(app);
        if app.ui.keybinds.matches("ui", "HELP_SCROLL_UP", &key) {
            app.panels.help.scroll = app.panels.help.scroll.saturating_sub(1);
            return Ok(());
        }
        if app.ui.keybinds.matches("ui", "HELP_SCROLL_DOWN", &key) {
            app.panels.help.scroll = (app.panels.help.scroll.saturating_add(1)).min(max_scroll);
            return Ok(());
        }
        // Fallback to arrow keys for help scrolling
        match key {
            KeyCode::Up => {
                app.panels.help.scroll = app.panels.help.scroll.saturating_sub(1);
                return Ok(());
            }
            KeyCode::Down => {
                app.panels.help.scroll = (app.panels.help.scroll.saturating_add(1)).min(max_scroll);
                return Ok(());
            }
            _ => {}
        }
    }
    // View Z slice up/down
    if app.ui.keybinds.matches("viewport", "VIEW_Z_UP", &key) {
        if app.ui.current_tab == MenuTab::Credits {
            app.panels.credits.scroll = app.panels.credits.scroll.saturating_sub(10);
        } else if app.ui.current_tab == MenuTab::Help {
            app.panels.help.scroll = app.panels.help.scroll.saturating_sub(10);
        } else {
            app.ui.view_z = app.ui.view_z.saturating_add(1);
        }
        return Ok(());
    }
    if app.ui.keybinds.matches("viewport", "VIEW_Z_DOWN", &key) {
        if app.ui.current_tab == MenuTab::Credits {
            let max_scroll = get_max_credits_scroll(app);
            app.panels.credits.scroll =
                (app.panels.credits.scroll.saturating_add(10)).min(max_scroll);
        } else if app.ui.current_tab == MenuTab::Help {
            let max_scroll = get_max_help_scroll(app);
            app.panels.help.scroll = (app.panels.help.scroll.saturating_add(10)).min(max_scroll);
        } else {
            app.ui.view_z = app.ui.view_z.saturating_sub(1);
        }
        return Ok(());
    }
    // Save/Load via config (only in Menu tab)
    if app.ui.keybinds.matches("ui", "SAVE_JSON", &key) {
        if app.ui.current_tab != crate::MenuTab::Menu {
            app.core.game.res.log("Save only available in Menu tab");
            return Ok(());
        }
        tracing::info!(target: "game", "save_begin path=save.json tick={}", app.core.game.res.time.tick);
        let viewport = lithicrivers_core::save_load::ViewportSave {
            view_x: app.ui.view_x,
            view_y: app.ui.view_y,
            view_z: app.ui.view_z,
        };
        app.core
            .game
            .save_json_with_viewport("save.json", viewport)
            .map_err(|e| format!("save_json_with_viewport error: {:?}", e))?;
        app.core.game.res.log("Saved to save.json");
        tracing::info!(target: "game", "save_end path=save.json tick={}", app.core.game.res.time.tick);
        return Ok(());
    }
    if app.ui.keybinds.matches("ui", "LOAD_JSON", &key) {
        if app.ui.current_tab != crate::MenuTab::Menu {
            app.core.game.res.log("Load only available in Menu tab");
            return Ok(());
        }
        tracing::info!(target: "game", "load_begin path=save.json tick={}", app.core.game.res.time.tick);
        let viewport = app
            .core
            .game
            .load_json_with_viewport("save.json")
            .map_err(|e| format!("load_json_with_viewport error: {:?}", e))?;

        // Restore viewport
        app.ui.view_x = viewport.view_x;
        app.ui.view_y = viewport.view_y;
        app.ui.view_z = viewport.view_z;

        app.core.game.res.log("Loaded from save.json");
        tracing::info!(target: "game", "load_end path=save.json tick={}", app.core.game.res.time.tick);
        return Ok(());
    }

    // Debug: if we reach the end without handling F key, log it
    if key == KeyCode::Char('f') || key == KeyCode::Char('F') {
        tracing::info!(target: "game", "F key reached end of input handler without being handled!");
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

/// Handle general interaction - items, corpses, NPCs
fn handle_interaction(app: &mut App) {
    tracing::info!(target: "game", "handle_interaction called, checking for nearby interactables");

    // Instead of using core auto-interaction, use client-side selection UI
    // First, check what's nearby
    let player_pos = if let Some(player_entity) = app.core.game.get_player_entity() {
        if let Ok(pos) = app
            .core
            .game
            .world
            .get::<&lithicrivers_core::components::Position>(player_entity)
        {
            *pos
        } else {
            app.core.game.res.log("Cannot find player position");
            return;
        }
    } else {
        app.core.game.res.log("Cannot find player entity");
        return;
    };

    // Find nearby items
    let mut nearby_items = Vec::new();
    for (entity, (item, pos)) in app
        .core
        .game
        .world
        .query::<(
            &lithicrivers_core::components::DroppedItem,
            &lithicrivers_core::components::Position,
        )>()
        .iter()
    {
        if (pos.x - player_pos.x).abs() <= 1
            && (pos.y - player_pos.y).abs() <= 1
            && pos.z == player_pos.z
        {
            nearby_items.push((entity, *item, *pos));
        }
    }

    // Find nearby corpses using existing function
    let mut nearby_corpses = Vec::new();
    for (entity, (pos, entity_kind, _inv)) in app
        .core
        .game
        .world
        .query::<(
            &lithicrivers_core::components::Position,
            &lithicrivers_core::components::EntityKind,
            &lithicrivers_core::components::Inventory,
        )>()
        .iter()
    {
        if *entity_kind == lithicrivers_core::components::EntityKind::Corpse {
            let dx = (pos.x - player_pos.x).abs();
            let dy = (pos.y - player_pos.y).abs();
            let dz = (pos.z - player_pos.z).abs();

            if dx <= 1 && dy <= 1 && dz == 0 {
                nearby_corpses.push(entity);
            }
        }
    }

    // Find nearby NPCs
    let mut nearby_npcs = Vec::new();
    for (entity, (dialogue, pos)) in app
        .core
        .game
        .world
        .query::<(
            &lithicrivers_core::components::Dialogue,
            &lithicrivers_core::components::Position,
        )>()
        .iter()
    {
        if (pos.x - player_pos.x).abs() <= 1
            && (pos.y - player_pos.y).abs() <= 1
            && pos.z == player_pos.z
        {
            nearby_npcs.push((entity, dialogue.name.clone(), *pos));
        }
    }

    let total_interactables = nearby_items.len() + nearby_corpses.len() + nearby_npcs.len();
    tracing::info!(target: "game", "Found {} items, {} corpses, {} NPCs nearby", 
        nearby_items.len(), nearby_corpses.len(), nearby_npcs.len());

    if total_interactables == 0 {
        app.core.game.res.log("Nothing to interact with nearby.");
    } else if total_interactables == 1 && nearby_corpses.len() == 1 {
        // Single corpse - use the existing corpse interaction system
        tracing::info!(target: "game", "Single corpse found, using corpse interaction system");
        handle_corpse_interaction(app);
    } else if total_interactables == 1 && nearby_items.len() == 1 {
        // Single item - pick it up directly via core system
        tracing::info!(target: "game", "Single item found, picking up via core system");
        app.core.game.res.player_state.intent =
            lithicrivers_core::intent::PlayerIntent::interact(100);
        let tick_result = app.core.game.tick();

        // Handle combat state changes
        if tick_result.contains(lithicrivers_core::game::GameTickResult::CombatTriggered) {
            app.combat = CombatUiState::Active {
                current_move: 0,
                current_enemy: 0,
                enemy_timers: vec![],
                move_scroll_offset: 0,
            };
        }
        if tick_result.contains(lithicrivers_core::game::GameTickResult::CombatEnded) {
            app.combat = CombatUiState::None;
        }
    } else if total_interactables == 1 && nearby_npcs.len() == 1 {
        // Single NPC - use the existing NPC interaction system
        tracing::info!(target: "game", "Single NPC found, using NPC interaction system");
        handle_npc_interaction(app);
    } else {
        // Multiple targets - prioritize based on what's available
        if nearby_corpses.len() > 0 {
            tracing::info!(target: "game", "Multiple targets with corpses, using corpse interaction system");
            handle_corpse_interaction(app);
        } else if nearby_npcs.len() > 0 {
            tracing::info!(target: "game", "Multiple targets with NPCs, using NPC interaction system");
            handle_npc_interaction(app);
        } else {
            tracing::info!(target: "game", "Multiple item targets, falling back to core system");
            app.core
                .game
                .res
                .log("Multiple items to interact with (selection UI coming soon)");

            // Fall back to core system for items
            app.core.game.res.player_state.intent =
                lithicrivers_core::intent::PlayerIntent::interact(100);
            let tick_result = app.core.game.tick();

            // Handle combat state changes
            if tick_result.contains(lithicrivers_core::game::GameTickResult::CombatTriggered) {
                app.combat = CombatUiState::Active {
                    current_move: 0,
                    current_enemy: 0,
                    enemy_timers: vec![],
                    move_scroll_offset: 0,
                };
            }
            if tick_result.contains(lithicrivers_core::game::GameTickResult::CombatEnded) {
                app.combat = CombatUiState::None;
            }
        }
    }
}

/// Handle NPC interaction - find adjacent NPCs and initiate dialogue
fn handle_npc_interaction(app: &mut App) {
    use crate::app_state::NPCInteractionState;
    use lithicrivers_core::components::{Dialogue, Position};

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

    // Find all adjacent NPCs (3x3 grid centered on player)
    let mut adjacent_npcs = Vec::new();
    for (entity, (pos, dialogue)) in app.core.game.world.query::<(&Position, &Dialogue)>().iter() {
        let dx = (pos.x - player_pos.x).abs();
        let dy = (pos.y - player_pos.y).abs();
        let dz = (pos.z - player_pos.z).abs();

        if dx <= 1 && dy <= 1 && dz == 0 {
            adjacent_npcs.push((entity, dialogue.name.clone()));
            app.core.game.res.log(format!(
                "Found NPC '{}' at ({}, {}, {})",
                dialogue.name, pos.x, pos.y, pos.z
            ));
        }
    }

    if adjacent_npcs.is_empty() {
        app.core
            .game
            .res
            .log("No NPCs nearby to talk to.".to_string());
        return;
    } else if adjacent_npcs.len() == 1 {
        // Single NPC - start dialogue directly
        let (npc_entity, npc_name) = adjacent_npcs[0].clone();
        // Get real NPC data from dialogue system or use the first available NPC
        let npc_data = if let Some(npc) = app.panels.dialogue_system.npcs.first() {
            npc.clone()
        } else {
            // Fallback dummy NPC data if no NPCs in dialogue system
            crate::app_state::NPCData {
                name: npc_name.clone(),
                portrait: "".to_string(),
                dialogue_type: crate::app_state::DialogueType::Linear,
                current_mood: crate::app_state::NPCMood::Neutral,
                initial_dialogue: 0,
                met_before: false,
                has_quest: false,
                shop_inventory: vec![],
            }
        };

        app.panels.npc_interaction = NPCInteractionState::InDialogue {
            npc_entity,
            npc_data,
            current_dialogue_node: Some(0), // Start with first dialogue node
            selected_choice: 0,
            conversation_log: vec![],
        };
        app.core
            .game
            .res
            .log(format!("Started conversation with {}", npc_name));
    } else {
        // Multiple NPCs - show selection modal
        app.panels.npc_interaction = NPCInteractionState::SelectingNPC {
            adjacent_npcs,
            selected_npc: 0,
        };
        app.core.game.res.log("Choose which NPC to talk to");
    }
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

/// Get blocks available for hotbar assignment from player inventory
fn get_blocks_from_inventory(app: &App) -> Vec<lithicrivers_core::components::ItemKind> {
    use lithicrivers_core::components::{Inventory as InvComp, ItemKind};

    let mut blocks = Vec::new();

    if let Some(player_entity) = app.core.game.get_player_entity() {
        if let Ok(inv) = app.core.game.world.get::<&InvComp>(player_entity) {
            for stack in &inv.slots {
                // Only include items that are placeable blocks
                // For now, let's include all ItemKind variants that could be blocks
                match stack.kind {
                    ItemKind::Stone | ItemKind::PlankBlock => {
                        if !blocks.contains(&stack.kind) {
                            blocks.push(stack.kind);
                        }
                    }
                    _ => {} // Skip non-block items
                }
            }
        }
    }

    blocks
}

/// Check if the player has a specific item in their inventory
fn has_item_in_inventory(app: &App, item_kind: lithicrivers_core::components::ItemKind) -> bool {
    use lithicrivers_core::components::Inventory as InvComp;

    if let Some(player_entity) = app.core.game.get_player_entity() {
        if let Ok(inv) = app.core.game.world.get::<&InvComp>(player_entity) {
            for stack in &inv.slots {
                if stack.kind == item_kind && stack.qty > 0 {
                    return true;
                }
            }
        }
    }
    false
}

/// Remove a quantity of an item from the player's inventory
fn remove_item_from_inventory(
    app: &mut App,
    item_kind: lithicrivers_core::components::ItemKind,
    quantity: u32,
) {
    use lithicrivers_core::components::Inventory as InvComp;

    if let Some(player_entity) = app.core.game.get_player_entity() {
        if let Ok(mut inv) = app.core.game.world.get::<&mut InvComp>(player_entity) {
            // Find the first stack with this item kind
            for i in 0..inv.slots.len() {
                if inv.slots[i].kind == item_kind {
                    if inv.slots[i].qty >= quantity {
                        inv.slots[i].qty -= quantity;
                        // Remove the stack if quantity reaches 0
                        if inv.slots[i].qty == 0 {
                            inv.slots.remove(i);
                        }
                        return;
                    }
                }
            }
        }
    }
}

/// Check if a position is valid for placing a block
fn is_valid_placement_position(app: &mut App, x: i32, y: i32, z: i32) -> bool {
    // Check if there's already a solid block at this position
    let current_tile = app.core.game.res.world_state.world.get_tile_cached(x, y, z);

    // Can only place on passable tiles (not on solid blocks)
    if !current_tile.is_passable() {
        return false;
    }

    // Check if there's an entity (like the player) at this position
    for (_, (pos, _)) in app
        .core
        .game
        .world
        .query::<(
            &lithicrivers_core::components::Position,
            &lithicrivers_core::components::Player,
        )>()
        .iter()
    {
        if pos.x == x && pos.y == y && pos.z == z {
            return false; // Can't place on player
        }
    }

    // Additional check: don't place on other entities either
    for (_, pos) in app
        .core
        .game
        .world
        .query::<&lithicrivers_core::components::Position>()
        .iter()
    {
        if pos.x == x && pos.y == y && pos.z == z {
            // There's some entity at this position, check if it blocks placement
            // For now, let's be conservative and not place blocks where entities are
            return false;
        }
    }

    true
}

use crate::app_state::{BuildMode, HotbarAssignmentState};
use crate::{App, MenuTab};
use crossterm::event::KeyCode;
use lithicrivers_core::components::{Inventory as InvComp, ItemKind};
use std::{error::Error, time::Instant};

/// Handle world/build mode input when on World tab - returns true if input was handled
pub fn handle_world_build_input(app: &mut App, key: KeyCode) -> Result<bool, Box<dyn Error>> {
    // Only handle input when on the World tab
    if app.ui.current_tab != MenuTab::World {
        return Ok(false);
    }

    // In Break/Place mode, handle 9-directional actions FIRST (before look toggle)
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
                let (target_x, target_y, target_z) = if let Ok(player_pos) =
                    app.core
                        .game
                        .world
                        .get::<&lithicrivers_core::components::Position>(player_entity)
                {
                    (player_pos.x + dx, player_pos.y + dy, player_pos.z)
                } else {
                    return Ok(true); // No position component
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
                        if let Some(block_item) = app.panels.build.hotbar_assignments[selected_slot]
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
                                        lithicrivers_core::components::itemkind_name(block_item);
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
                            app.core
                                .game
                                .res
                                .log(format!("No block assigned to slot F{}", selected_slot + 1));
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
            return Ok(true);
        }
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
            return Ok(true);
        }

        app.panels.build.mode = app.panels.build.mode.next();
        app.core
            .game
            .res
            .log(format!("Build mode: {}", app.panels.build.mode.name()));
        return Ok(true);
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
        let now = Instant::now();
        const DOUBLE_TAP_WINDOW_MS: u128 = 500; // 500ms window for double-tap

        // Check for double-tap
        let is_double_tap = if let Some((last_slot, last_time)) = app.panels.build.last_fkey_press {
            last_slot == slot && now.duration_since(last_time).as_millis() <= DOUBLE_TAP_WINDOW_MS
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
        return Ok(true);
    }

    // No world/build input was handled
    Ok(false)
}

/// Get blocks available for hotbar assignment from player inventory
fn get_blocks_from_inventory(app: &App) -> Vec<lithicrivers_core::components::ItemKind> {
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
fn is_valid_placement_position(app: &mut App, x: i64, y: i64, z: i64) -> bool {
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

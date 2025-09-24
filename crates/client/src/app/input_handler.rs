use crossterm::event::KeyCode;
use std::error::Error;

use lithicrivers_core::game::GameTickResult;

use crate::app_state::HotbarAssignmentState;
use crate::{App, CombatUiState, MenuTab, Scale};

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
fn get_max_help_scroll(_app: &App) -> u16 {
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
    // Handle splash screen input when active - delegated to input module
    if crate::app::input::splash::handle_splash_input(app, key)? {
        return Ok(());
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
                let radius = 20i64;
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
    // Handle inventory tab input when active - delegated to input module
    if crate::app::input::inventory::handle_inventory_input(app, key)? {
        return Ok(());
    }

    // Handle crafting tab input when active - delegated to input module
    if crate::app::input::crafting::handle_crafting_input(app, key)? {
        return Ok(());
    }

    // Handle body tab input when active - delegated to input module
    if crate::app::input::body::handle_body_input(app, key)? {
        return Ok(());
    }

    // Handle hotbar assignment input when active - delegated to input module
    if crate::app::input::hotbar_assignment::handle_hotbar_assignment_input(app, key)? {
        return Ok(());
    }

    // Handle multi-action selection input when active - delegated to input module
    if crate::app::input::multi_action::handle_multi_action_input(app, key)? {
        return Ok(());
    }

    // Handle corpse looting input when active - delegated to input module
    if crate::app::input::corpse_looting::handle_corpse_looting_input(app, key)? {
        return Ok(());
    }

    // Handle NPC interaction input when active - delegated to input module
    if crate::app::input::npc_interaction::handle_npc_interaction_input(app, key)? {
        return Ok(());
    }

    // Handle combat-specific input when combat is active - delegated to input module
    if crate::app::input::combat::handle_combat_input(app, key)? {
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

    // Global Cheats: Toggle noclip mode
    if app
        .ui
        .keybinds
        .matches("inventory", "CHEAT_NOCLIP_TOGGLE", &key)
    {
        app.core.game.res.player_state.noclip_enabled =
            !app.core.game.res.player_state.noclip_enabled;
        let state = if app.core.game.res.player_state.noclip_enabled {
            "ON"
        } else {
            "OFF"
        };
        app.core.game.res.log(format!("Noclip mode: {}", state));
        tracing::info!(target: "game", "Noclip mode toggled: {}", state);
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

/// Handle general interaction - items, corpses, NPCs
fn handle_interaction(app: &mut App) {
    tracing::info!(target: "game", "handle_interaction called, checking for nearby interactables");

    // Get player position
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

    // Build a list of all available interactions
    use crate::app_state::{InteractionType, MultiActionSelectState};
    use lithicrivers_core::components::{
        itemkind_name, Dialogue, DroppedItem, EntityKind, Inventory, Position,
    };

    let mut available_actions = Vec::new();

    // Find nearby items
    for (entity, (item, pos)) in app
        .core
        .game
        .world
        .query::<(&DroppedItem, &Position)>()
        .iter()
    {
        if (pos.x - player_pos.x).abs() <= 1
            && (pos.y - player_pos.y).abs() <= 1
            && pos.z == player_pos.z
        {
            available_actions.push(InteractionType::PickupItem {
                entity,
                item_name: itemkind_name(item.kind).to_string(),
                position: *pos,
            });
        }
    }

    // Find nearby corpses
    for (entity, (pos, entity_kind, _inv)) in app
        .core
        .game
        .world
        .query::<(&Position, &EntityKind, &Inventory)>()
        .iter()
    {
        if *entity_kind == EntityKind::Corpse {
            let dx = (pos.x - player_pos.x).abs();
            let dy = (pos.y - player_pos.y).abs();
            let dz = (pos.z - player_pos.z).abs();

            if dx <= 1 && dy <= 1 && dz == 0 {
                available_actions.push(InteractionType::LootCorpse {
                    entity,
                    position: *pos,
                });
            }
        }
    }

    // Find nearby NPCs
    for (entity, (dialogue, pos)) in app.core.game.world.query::<(&Dialogue, &Position)>().iter() {
        if (pos.x - player_pos.x).abs() <= 1
            && (pos.y - player_pos.y).abs() <= 1
            && pos.z == player_pos.z
        {
            available_actions.push(InteractionType::TalkToNPC {
                entity,
                npc_name: dialogue.name.clone(),
                position: *pos,
            });
        }
    }

    // Find nearby doors (check adjacent tiles in all 8 directions)
    use lithicrivers_core::tile_registry::TileKind;
    let adjacent_positions = [
        // Cardinal directions
        Position {
            x: player_pos.x,
            y: player_pos.y - 1, // North
            z: player_pos.z,
        },
        Position {
            x: player_pos.x + 1,
            y: player_pos.y - 1, // Northeast
            z: player_pos.z,
        },
        Position {
            x: player_pos.x + 1,
            y: player_pos.y, // East
            z: player_pos.z,
        },
        Position {
            x: player_pos.x + 1,
            y: player_pos.y + 1, // Southeast
            z: player_pos.z,
        },
        Position {
            x: player_pos.x,
            y: player_pos.y + 1, // South
            z: player_pos.z,
        },
        Position {
            x: player_pos.x - 1,
            y: player_pos.y + 1, // Southwest
            z: player_pos.z,
        },
        Position {
            x: player_pos.x - 1,
            y: player_pos.y, // West
            z: player_pos.z,
        },
        Position {
            x: player_pos.x - 1,
            y: player_pos.y - 1, // Northwest
            z: player_pos.z,
        },
    ];

    for pos in adjacent_positions {
        let tile = app
            .core
            .game
            .res
            .world_state
            .world
            .get_tile_cached(pos.x, pos.y, pos.z);
        match tile {
            TileKind::Door => {
                available_actions.push(InteractionType::OpenCloseDoor {
                    position: pos,
                    is_open: false,
                });
            }
            TileKind::DoorOpen => {
                available_actions.push(InteractionType::OpenCloseDoor {
                    position: pos,
                    is_open: true,
                });
            }
            _ => {}
        }
    }

    tracing::info!(target: "game", "Found {} total interactions nearby", available_actions.len());

    match available_actions.len() {
        0 => {
            app.core.game.res.log("Nothing to interact with nearby.");
        }
        1 => {
            // Single interaction - execute directly
            let action = &available_actions[0];
            execute_interaction_action(app, action);
        }
        _ => {
            // Multiple interactions - show selection modal
            tracing::info!(target: "game", "Multiple interactions found, showing selection modal");
            app.panels.multi_action_select = MultiActionSelectState::SelectingAction {
                available_actions,
                selected_action: 0,
            };
            app.core.game.res.log("Choose interaction:");
        }
    }
}

/// Execute a specific interaction action
pub fn execute_interaction_action(app: &mut App, action: &crate::app_state::InteractionType) {
    use crate::app_state::{CorpseLootingState, InteractionType, NPCInteractionState};

    match action {
        InteractionType::PickupItem { .. } => {
            // Use core system for item pickup
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
        InteractionType::LootCorpse { entity, .. } => {
            // Start corpse looting directly
            app.panels.corpse_looting = CorpseLootingState::LootingCorpse {
                entity: *entity,
                selected_loot_item: 0,
                selected_player_item: 0,
                loot_panel_focus: true,
            };
            app.core.game.res.log("Started looting corpse");
        }
        InteractionType::TalkToNPC {
            entity, npc_name, ..
        } => {
            // Start NPC dialogue directly
            if let Some(conversation) = app.panels.dialogue_engine.start_conversation(0) {
                app.panels.npc_interaction = NPCInteractionState::InDialogue {
                    npc_entity: *entity,
                    conversation,
                    selected_choice: 0,
                };
            } else {
                app.core
                    .game
                    .res
                    .log("Failed to start conversation - no NPC available");
            }
            app.core
                .game
                .res
                .log(format!("Started conversation with {}", npc_name));
        }
        InteractionType::OpenCloseDoor { position, is_open } => {
            use lithicrivers_core::tile_registry::TileKind;

            // Toggle the door state
            let new_tile = if *is_open {
                TileKind::Door // Close the door
            } else {
                TileKind::DoorOpen // Open the door
            };

            // Set the tile in the world map
            app.core
                .game
                .res
                .world_state
                .world
                .set_tile_cached(position.x, position.y, position.z, new_tile);

            // Log the action
            let action_name = if *is_open { "closed" } else { "opened" };
            app.core
                .game
                .res
                .log(format!("You {} the door", action_name));
        }
    }
}

/// Take an item from a corpse and add it to the player's inventory
pub fn take_item_from_corpse(app: &mut App, corpse_entity: hecs::Entity, item_idx: usize) {
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

use crossterm::event::KeyCode;
use std::error::Error;

use lithicrivers_core::game::GameTickResult;

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

    // Handle world/build mode input when on World tab - delegated to input module
    if crate::app::input::world_build::handle_world_build_input(app, key)? {
        return Ok(());
    }

    // Handle look mode input when on World tab - delegated to input module
    if crate::app::input::look_mode::handle_look_mode_input(app, key)? {
        return Ok(());
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

    // Handle world movement input when on World tab - delegated to input module
    if crate::app::input::world_movement::handle_world_movement_input(app, key)? {
        return Ok(());
    }

    // Handle world-specific actions (mining, interaction, scale) when on World tab
    if app.ui.current_tab == MenuTab::World {
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

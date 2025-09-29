use crate::{App, CombatUiState, MenuTab, Scale};
use crossterm::event::KeyCode;
use lithicrivers_core::game::GameTickResult;
use std::error::Error;

/// Handle world action input when on World tab - returns true if input was handled
pub fn handle_world_action_input(app: &mut App, key: KeyCode) -> Result<bool, Box<dyn Error>> {
    // Only handle input when on the World tab or Tutorial tab (for tutorial functionality)
    if app.ui.current_tab != MenuTab::World && app.ui.current_tab != MenuTab::Tutorial {
        return Ok(false);
    }

    // Mining action
    if app.ui.keybinds.matches("action", "MINE", &key) {
        app.core.game.queue_mine();
        let tick_result = app.core.game.tick();
        if tick_result.contains(GameTickResult::MiningSuccess) {
            app.snap_view_to_player_z();
        }
        if tick_result.contains(GameTickResult::CombatEnded) {
            app.combat = CombatUiState::None;
        }
        return Ok(true);
    }

    // Debug F key specifically
    if key == KeyCode::Char('f') || key == KeyCode::Char('F') {
        tracing::info!(target: "game", "Checking F key against INTERACT keybind");
        let matches = app.ui.keybinds.matches("action", "INTERACT", &key);
        tracing::info!(target: "game", "F key matches INTERACT: {}", matches);
    }

    // Interaction handling
    if app.ui.keybinds.matches("action", "INTERACT", &key) {
        tracing::info!(target: "game", "F key matched INTERACT action, calling handle_interaction");
        handle_interaction(app);
        tracing::info!(target: "game", "handle_interaction completed");
        return Ok(true);
    }

    // Torch toggle
    if app.ui.keybinds.matches("action", "TOGGLE_TORCH", &key) {
        handle_torch_toggle(app);
        return Ok(true);
    }

    // Scale controls
    if app.ui.keybinds.matches("scale", "SCALE_UP", &key) {
        app.ui.scale = match app.ui.scale {
            Scale::Small => Scale::Medium,
            Scale::Medium => Scale::Large,
            Scale::Large => Scale::Large,
        };
        return Ok(true);
    }
    if app.ui.keybinds.matches("scale", "SCALE_DOWN", &key) {
        app.ui.scale = match app.ui.scale {
            Scale::Large => Scale::Medium,
            Scale::Medium => Scale::Small,
            Scale::Small => Scale::Small,
        };
        return Ok(true);
    }
    if app.ui.keybinds.matches("scale", "SCALE_RESET", &key) {
        app.ui.scale = Scale::Small;
        return Ok(true);
    }

    // No world action input was handled
    Ok(false)
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

            // Check tutorial progression for inventory changes after pickup
            if let Some(player_entity) = app.core.game.get_player_entity() {
                if let Ok(inventory) = app
                    .core
                    .game
                    .world
                    .get::<&lithicrivers_core::components::Inventory>(player_entity)
                {
                    tracing::info!(target: "tutorial", "Item picked up, calling tutorial system with inventory containing {} stacks",
                        inventory.slots.len());
                    let tutorial_advanced =
                        app.ui.tutorial_system.handle_inventory_change(&inventory);
                    tracing::info!(target: "tutorial", "Tutorial system returned: {}", tutorial_advanced);
                }
            }

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
            let conversation = crate::app_state::ConversationState {
                current_node_id: Some(lithicrivers_core::dialogue::DialogueNodeID::Start),
                player_mood: crate::app_state::NPCMood::Neutral,
            };
            app.panels.npc_interaction = NPCInteractionState::InDialogue {
                npc_entity: *entity,
                conversation,
                selected_choice: 0,
            };
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

/// Handle torch toggle - check for torch in inventory and toggle light source
fn handle_torch_toggle(app: &mut App) {
    use lithicrivers_core::components::{Inventory, ItemKind, LightSource};

    // Get player entity
    let player_entity = if let Some(entity) = app.core.game.get_player_entity() {
        entity
    } else {
        app.core.game.res.log("Cannot find player entity");
        return;
    };

    // Check if player has torch in inventory
    let has_torch = if let Ok(inventory) = app.core.game.world.get::<&Inventory>(player_entity) {
        inventory
            .slots
            .iter()
            .any(|stack| stack.kind == ItemKind::Torch && stack.qty > 0)
    } else {
        false
    };

    // Toggle torch if available
    if let Ok(mut light_source) = app.core.game.world.get::<&mut LightSource>(player_entity) {
        if has_torch {
            let was_equipped = light_source.torch_equipped;
            light_source.set_torch_equipped(!was_equipped);

            if light_source.torch_equipped {
                app.core
                    .game
                    .res
                    .log("Torch lit! Light radius increased to 8.");
            } else {
                app.core
                    .game
                    .res
                    .log("Torch extinguished. Light radius reduced to 2.");
            }
        } else {
            app.core.game.res.log("No torch available in inventory.");
        }
    } else {
        app.core
            .game
            .res
            .log("Cannot access light source component");
    }
}

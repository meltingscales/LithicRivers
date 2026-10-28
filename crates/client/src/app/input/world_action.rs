use crate::{App, MenuTab, Scale};
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
        app.apply_tick_result(&tick_result);
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

    // Let 10 game seconds pass (stops early if combat starts)
    if app.ui.keybinds.matches("action", "PASS_TIME", &key) {
        let tick_result = app.core.game.pass_time();
        app.core.game.res.log("Time passes...");
        app.apply_tick_result(&tick_result);
        return Ok(true);
    }

    // Context help panel on the World page
    if app.ui.keybinds.matches("ui", "TOGGLE_CONTEXT_HELP", &key) {
        app.ui.show_context_help = !app.ui.show_context_help;
        app.core.config_manager.data.settings["ui"]["SHOW_CONTEXT_HELP"] =
            serde_json::json!(app.ui.show_context_help);
        if app.persist_config {
            if let Err(e) = app.core.config_manager.save() {
                tracing::warn!(target: "game", "could not save config: {e:#}");
            }
        }
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

/// What pressing interact would do now: every interaction adjacent to the player.
pub struct NearbyInteractions {
    pub actions: Vec<crate::app_state::InteractionType>,
}

/// Find the interactions adjacent to the player. Used by the interact key and the context
/// help panel, so both always agree. `None` if there is no player.
pub fn nearby_interactions(app: &App) -> Option<NearbyInteractions> {
    let player_pos = app.core.game.get_player_position()?;

    // Build a list of all available interactions
    use crate::app_state::InteractionType;
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

    // Find nearby corpses: loot the full ones, clear away the empty ones
    for (entity, (pos, entity_kind, inv, wreck)) in app
        .core
        .game
        .world
        .query::<(
            &Position,
            &EntityKind,
            &Inventory,
            Option<&lithicrivers_core::components::Wreck>,
        )>()
        .iter()
    {
        let adjacent = (pos.x - player_pos.x).abs() <= 1
            && (pos.y - player_pos.y).abs() <= 1
            && pos.z == player_pos.z;
        if *entity_kind != EntityKind::Corpse || !adjacent {
            continue;
        }
        if inv.slots.is_empty() {
            available_actions.push(InteractionType::ClearCorpse {
                entity,
                position: *pos,
                wreck: wreck.is_some(),
            });
        } else {
            available_actions.push(InteractionType::LootCorpse {
                entity,
                position: *pos,
            });
        }
    }

    // Find nearby chests
    for (entity, (pos, block)) in app
        .core
        .game
        .world
        .query::<(&Position, &lithicrivers_core::factory::FactoryBlock)>()
        .iter()
    {
        use lithicrivers_core::factory::FactoryKind;
        let adjacent = (pos.x - player_pos.x).abs() <= 1
            && (pos.y - player_pos.y).abs() <= 1
            && pos.z == player_pos.z;
        if !adjacent {
            continue;
        }
        let position = *pos;
        match block.kind {
            FactoryKind::Chest => {
                available_actions.push(InteractionType::OpenChest { entity, position })
            }
            FactoryKind::BurnerGenerator => {
                available_actions.push(InteractionType::OpenGenerator { entity, position })
            }
            FactoryKind::PowerPole => {
                available_actions.push(InteractionType::InspectPole { entity, position })
            }
            FactoryKind::CrudeFurnace | FactoryKind::ElectricFurnace => {
                available_actions.push(InteractionType::OpenFurnace {
                    entity,
                    position,
                    kind: block.kind,
                })
            }
            FactoryKind::FilterExtractor | FactoryKind::BeltRouter => {
                available_actions.push(InteractionType::SetFilter {
                    entity,
                    position,
                    kind: block.kind,
                })
            }
            FactoryKind::AutoPlanter | FactoryKind::AutoHarvester => {
                available_actions.push(InteractionType::OpenMachine {
                    entity,
                    position,
                    kind: block.kind,
                })
            }
            _ => {}
        }
    }

    // Find nearby empty belts, which can take one of the selected inventory item
    let selected_item = app
        .core
        .game
        .get_player_component::<Inventory>()
        .and_then(|inv| {
            inv.slots
                .get(
                    app.panels
                        .inventory
                        .selected
                        .min(inv.slots.len().saturating_sub(1)),
                )
                .map(|s| s.kind)
        });
    if let Some(item) = selected_item {
        for (entity, (pos, belt)) in app
            .core
            .game
            .world
            .query::<(&Position, &lithicrivers_core::factory::BeltSlot)>()
            .iter()
        {
            if belt.item.is_none()
                && (pos.x - player_pos.x).abs() <= 1
                && (pos.y - player_pos.y).abs() <= 1
                && pos.z == player_pos.z
            {
                available_actions.push(InteractionType::LoadBelt {
                    entity,
                    position: *pos,
                    item,
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
            .get_tile(pos.x, pos.y, pos.z);
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

    Some(NearbyInteractions {
        actions: available_actions,
    })
}

/// Handle general interaction - items, corpses, NPCs
fn handle_interaction(app: &mut App) {
    use crate::app_state::MultiActionSelectState;
    tracing::info!(target: "game", "handle_interaction called, checking for nearby interactables");

    let Some(NearbyInteractions {
        actions: available_actions,
    }) = nearby_interactions(app)
    else {
        app.core.game.res.log("Cannot find player position");
        return;
    };

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
                picked: None,
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
            tracing::info!(target: "tutorial", "=== PICKUP ITEM ACTION TRIGGERED ===");
            // Use core system for item pickup
            app.core.game.res.player_state.intent =
                lithicrivers_core::intent::PlayerIntent::interact(100);
            let tick_result = app.core.game.tick();
            tracing::info!(target: "tutorial", "Item pickup completed");

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

            app.apply_tick_result(&tick_result);
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
        InteractionType::ClearCorpse { entity, wreck, .. } => {
            let _ = app.core.game.world.despawn(*entity);
            let what = if *wreck { "wreck" } else { "corpse" };
            app.core
                .game
                .res
                .log(format!("You clear away the empty {what}."));
        }
        InteractionType::LoadBelt { entity, item, .. } => {
            let Some(player) = app.core.game.get_player_entity() else {
                return;
            };
            let slot = app
                .core
                .game
                .world
                .get::<&lithicrivers_core::components::Inventory>(player)
                .ok()
                .and_then(|inv| inv.slots.iter().position(|s| s.kind == *item));
            let Some(slot) = slot else {
                return;
            };
            match lithicrivers_core::factory::move_one_item_to_block(
                &mut app.core.game.world,
                player,
                *entity,
                slot,
            ) {
                Ok(kind) => app.core.game.res.log(format!(
                    "Put {} on the belt",
                    lithicrivers_core::components::itemkind_name(kind)
                )),
                Err(_) => app.core.game.res.log("The belt is full"),
            }
        }
        InteractionType::InspectPole { entity, .. } => {
            app.panels.power_info = Some(*entity);
        }
        InteractionType::SetFilter { entity, .. } => {
            // Start on the current filter
            let current = lithicrivers_core::logistics::filter(&app.core.game.world, *entity);
            let selected = current.map_or(0, |item| {
                use strum::IntoEnumIterator;
                1 + lithicrivers_core::components::ItemKind::iter()
                    .position(|i| i == item)
                    .unwrap_or(0)
            });
            app.panels.filter_picker = Some(crate::app_state::FilterPicker {
                entity: *entity,
                selected,
            });
        }
        InteractionType::OpenFurnace { entity, kind, .. } => {
            app.panels.furnace_window = Some(crate::app_state::FurnaceWindow {
                entity: *entity,
                kind: *kind,
                slot: lithicrivers_core::furnace::Slot::Input,
                player_item: 0,
                player_focus: true,
            });
            app.core.game.res.log(format!("Opened {}", kind.name()));
        }
        InteractionType::OpenMachine { entity, kind, .. } => {
            app.panels.corpse_looting = CorpseLootingState::LootingCorpse {
                entity: *entity,
                selected_loot_item: 0,
                selected_player_item: 0,
                loot_panel_focus: false,
            };
            app.core.game.res.log(format!("Opened {}", kind.name()));
        }
        InteractionType::OpenGenerator { entity, .. } => {
            app.panels.corpse_looting = CorpseLootingState::LootingCorpse {
                entity: *entity,
                selected_loot_item: 0,
                selected_player_item: 0,
                loot_panel_focus: false,
            };
            app.core.game.res.log("Opened generator");
        }
        InteractionType::OpenChest { entity, .. } => {
            // Chests use the same two-panel transfer window as corpses
            app.panels.corpse_looting = CorpseLootingState::LootingCorpse {
                entity: *entity,
                selected_loot_item: 0,
                selected_player_item: 0,
                loot_panel_focus: true,
            };
            app.core.game.res.log("Opened chest");
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

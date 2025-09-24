use crate::App;
use crossterm::event::KeyCode;
use std::error::Error;

/// Handle NPC interaction input when active - returns true if input was handled
pub fn handle_npc_interaction_input(app: &mut App, key: KeyCode) -> Result<bool, Box<dyn Error>> {
    // Handle NPC selection phase
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
            return Ok(true);
        }

        if app.ui.keybinds.matches("movement", "MOVE_NORTH", &key) {
            app.panels.npc_interaction = crate::app_state::NPCInteractionState::SelectingNPC {
                adjacent_npcs: npcs_clone.clone(),
                selected_npc: selected.saturating_sub(1),
            };
            return Ok(true);
        }

        if app.ui.keybinds.matches("movement", "MOVE_SOUTH", &key) {
            app.panels.npc_interaction = crate::app_state::NPCInteractionState::SelectingNPC {
                adjacent_npcs: npcs_clone.clone(),
                selected_npc: (selected + 1).min(npcs_clone.len().saturating_sub(1)),
            };
            return Ok(true);
        }

        if app.ui.keybinds.matches("ui", "MENU_ACTIVATE", &key) || key == KeyCode::Enter {
            if selected < npcs_clone.len() {
                let (npc_entity, npc_name) = npcs_clone[selected].clone();
                // Start conversation using the new dialogue engine

                if let Some(conversation) = app.panels.dialogue_engine.start_conversation(0) {
                    app.panels.npc_interaction =
                        crate::app_state::NPCInteractionState::InDialogue {
                            npc_entity,
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
            return Ok(true);
        }
        return Ok(true);
    }

    // Handle dialogue phase
    if let crate::app_state::NPCInteractionState::InDialogue {
        npc_entity,
        conversation,
        selected_choice,
    } = &app.panels.npc_interaction
    {
        let entity = *npc_entity;
        let mut choice = *selected_choice;

        if app.ui.keybinds.matches("ui", "CLOSE_HELP_MENU", &key) {
            app.panels.npc_interaction = crate::app_state::NPCInteractionState::None;
            app.core.game.res.log("Ended conversation");
            return Ok(true);
        }

        // Handle dialogue navigation using the new engine
        if app.ui.keybinds.matches("movement", "MOVE_NORTH", &key) {
            choice = choice.saturating_sub(1);
        }

        if app.ui.keybinds.matches("movement", "MOVE_SOUTH", &key) {
            // Get the current dialogue node to check how many choices are available
            if let Some(node) = app.panels.dialogue_engine.get_current_node(conversation) {
                choice = (choice + 1).min(node.choices.len().saturating_sub(1));
            }
        }

        if app.ui.keybinds.matches("ui", "MENU_ACTIVATE", &key) || key == KeyCode::Enter {
            // Process the choice using the dialogue engine
            if let Some(result) = app
                .panels
                .dialogue_engine
                .process_choice(conversation, choice)
            {
                if result.conversation_ended {
                    app.panels.npc_interaction = crate::app_state::NPCInteractionState::None;
                    app.core.game.res.log("Conversation ended");
                } else if let Some(next_node_id) = result.next_node_id {
                    // Continue conversation with next node
                    let mut new_conversation = conversation.clone();
                    new_conversation.current_node_id = Some(next_node_id);
                    new_conversation.selected_choice = 0; // Reset choice selection

                    app.panels.npc_interaction =
                        crate::app_state::NPCInteractionState::InDialogue {
                            npc_entity: entity,
                            conversation: new_conversation,
                            selected_choice: 0,
                        };

                    if result.unlocked_quest {
                        app.core.game.res.log("New quest unlocked!");
                    }
                    if let Some(_shop_item) = result.shop_transaction {
                        app.core.game.res.log("Shop transaction available");
                    }
                }
            }
            return Ok(true);
        }

        // Update the selected choice
        app.panels.npc_interaction = crate::app_state::NPCInteractionState::InDialogue {
            npc_entity: entity,
            conversation: conversation.clone(),
            selected_choice: choice,
        };
        return Ok(true);
    }

    Ok(false)
}

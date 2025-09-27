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

                // Start conversation at the beginning of the dialogue tree
                let conversation = crate::app_state::ConversationState {
                    current_node_id: Some("start".to_string()),
                };
                app.panels.npc_interaction = crate::app_state::NPCInteractionState::InDialogue {
                    npc_entity,
                    conversation,
                    selected_choice: 0,
                };
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
            if let Some(ref current_node_id) = conversation.current_node_id {
                if let Some(node) = app.panels.dialogue_tree.get_node(current_node_id) {
                    choice = (choice + 1).min(node.choices.len().saturating_sub(1));
                }
            }
        }

        if app.ui.keybinds.matches("ui", "MENU_ACTIVATE", &key) || key == KeyCode::Enter {
            // Process the choice using the core dialogue tree
            if let Some(ref current_node_id) = conversation.current_node_id {
                if let Some(node) = app.panels.dialogue_tree.get_node(current_node_id) {
                    if choice < node.choices.len() {
                        let selected_choice = &node.choices[choice];

                        if let Some(ref next_node_id) = selected_choice.leads_to {
                            // Continue conversation with next node
                            let new_conversation = crate::app_state::ConversationState {
                                current_node_id: Some(next_node_id.clone()),
                            };

                            app.panels.npc_interaction =
                                crate::app_state::NPCInteractionState::InDialogue {
                                    npc_entity: entity,
                                    conversation: new_conversation,
                                    selected_choice: 0,
                                };
                        } else {
                            // Conversation ended
                            app.panels.npc_interaction =
                                crate::app_state::NPCInteractionState::None;
                            app.core.game.res.log("Conversation ended");
                        }
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

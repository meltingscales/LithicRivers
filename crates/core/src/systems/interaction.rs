use crate::components::{Dialogue, DroppedItem, EntityKind, Inventory, ItemKind, Player, Position};
use crate::resources::Resources;
use hecs::World;

/// Handle interaction with nearby objects (items, corpses, NPCs)
pub fn interaction_system(world: &mut World, res: &mut Resources, player_entity: hecs::Entity) {
    // Get player position
    let player_pos = if let Ok(pos) = world.get::<&Position>(player_entity) {
        *pos
    } else {
        res.log("Player has no position!".to_string());
        return;
    };

    res.log(format!(
        "Player interaction at ({}, {}, {})",
        player_pos.x, player_pos.y, player_pos.z
    ));

    // Find all nearby interactable objects in a 3x3 area around player
    let mut nearby_items = Vec::new();
    let mut nearby_corpses = Vec::new();
    let mut nearby_npcs = Vec::new();

    // Check for dropped items
    for (entity, (item, pos)) in world.query::<(&DroppedItem, &Position)>().iter() {
        if (pos.x - player_pos.x).abs() <= 1
            && (pos.y - player_pos.y).abs() <= 1
            && pos.z == player_pos.z
        {
            nearby_items.push((entity, *item, *pos));
            res.log(format!(
                "Found item {} (qty: {}) at ({}, {}, {})",
                crate::components::itemkind_name(item.kind),
                item.qty,
                pos.x,
                pos.y,
                pos.z
            ));
        }
    }

    // Check for corpses with inventory - collect entities first to avoid borrow conflicts
    let corpse_candidates: Vec<(hecs::Entity, Position)> = world
        .query::<(&Position, &EntityKind)>()
        .iter()
        .filter_map(|(entity, (pos, kind))| {
            if entity != player_entity
                && *kind == EntityKind::Corpse
                && (pos.x - player_pos.x).abs() <= 1
                && (pos.y - player_pos.y).abs() <= 1
                && pos.z == player_pos.z
            {
                Some((entity, *pos))
            } else {
                None
            }
        })
        .collect();

    // Now check each corpse's inventory separately
    for (entity, pos) in corpse_candidates {
        if let Ok(inv) = world.get::<&Inventory>(entity) {
            if !inv.slots.is_empty() {
                nearby_corpses.push((entity, pos));
                res.log(format!(
                    "Found corpse with {} items at ({}, {}, {})",
                    inv.slots.len(),
                    pos.x,
                    pos.y,
                    pos.z
                ));
            }
        }
    }

    // Check for NPCs with dialogue
    for (entity, (dialogue, pos)) in world.query::<(&Dialogue, &Position)>().iter() {
        if (pos.x - player_pos.x).abs() <= 1
            && (pos.y - player_pos.y).abs() <= 1
            && pos.z == player_pos.z
        {
            nearby_npcs.push((entity, dialogue.name.clone(), *pos));
            res.log(format!(
                "Found NPC '{}' at ({}, {}, {})",
                dialogue.name, pos.x, pos.y, pos.z
            ));
        }
    }

    // Count total interactables
    let total_interactables = nearby_items.len() + nearby_corpses.len() + nearby_npcs.len();

    if total_interactables == 0 {
        res.log("Nothing to interact with nearby.".to_string());
        return;
    } else if total_interactables == 1 {
        // Single target - interact directly
        if !nearby_items.is_empty() {
            let (entity, item, _) = nearby_items[0];
            pickup_item(world, res, player_entity, entity, item);
        } else if !nearby_corpses.is_empty() {
            let (entity, _) = nearby_corpses[0];
            start_corpse_looting(world, res, entity);
        } else if !nearby_npcs.is_empty() {
            let (entity, name, _) = &nearby_npcs[0];
            start_npc_dialogue(world, res, *entity, name.clone());
        }
    } else {
        // Multiple targets - need selection UI
        let mut options = Vec::new();

        for (_, item, pos) in &nearby_items {
            options.push(format!(
                "Pick up {} at ({}, {})",
                crate::components::itemkind_name(item.kind),
                pos.x,
                pos.y
            ));
        }

        for (_, pos) in &nearby_corpses {
            options.push(format!("Loot corpse at ({}, {})", pos.x, pos.y));
        }

        for (_, name, pos) in &nearby_npcs {
            options.push(format!("Talk to {} at ({}, {})", name, pos.x, pos.y));
        }

        // For now, just log the options. Later we'll add UI selection
        res.log("Multiple things to interact with:".to_string());
        for (i, option) in options.iter().enumerate() {
            res.log(format!("  {}: {}", i + 1, option));
        }
        res.log("(Selection UI not implemented yet)".to_string());
    }
}

/// Handle interaction with a specific target by ID
pub fn interaction_with_target_system(
    _world: &mut World,
    res: &mut Resources,
    _player_entity: hecs::Entity,
    _target_id: Option<usize>,
) {
    // TODO: Implement specific target interaction
    // This would be used when the player selects from the interaction menu
    res.log("Specific target interaction not yet implemented.".to_string());
}

/// Pick up a dropped item
pub fn pickup_item(
    world: &mut World,
    res: &mut Resources,
    player_entity: hecs::Entity,
    item_entity: hecs::Entity,
    item: DroppedItem,
) {
    // Add item to player inventory
    let pickup_successful = if let Ok(mut inventory) = world.get::<&mut Inventory>(player_entity) {
        inventory.add(item.kind, item.qty);
        res.log(format!(
            "Picked up {} {}.",
            item.qty,
            crate::components::itemkind_name(item.kind)
        ));
        true
    } else {
        res.log("Player has no inventory!".to_string());
        false
    };

    // Remove the dropped item from world (separate borrow)
    if pickup_successful {
        if let Err(e) = world.despawn(item_entity) {
            res.log(format!("Warning: Failed to despawn item: {:?}", e));
        }
    }
}

/// Start corpse looting interaction
pub fn start_corpse_looting(world: &mut World, res: &mut Resources, corpse_entity: hecs::Entity) {
    // For now, just transfer all items automatically
    // Later this will open the looting UI
    // First, collect the items from the corpse
    let items_to_loot: Vec<(ItemKind, u32)> =
        if let Ok(corpse_inv) = world.get::<&Inventory>(corpse_entity) {
            corpse_inv
                .slots
                .iter()
                .map(|stack| (stack.kind, stack.qty))
                .collect()
        } else {
            Vec::new()
        };

    if items_to_loot.is_empty() {
        res.log("Nothing to loot from this corpse.".to_string());
        return;
    }

    // Find player and add items to their inventory
    if let Some((player_entity, _)) = world.query::<&Player>().iter().next() {
        if let Ok(mut player_inv) = world.get::<&mut Inventory>(player_entity) {
            for (item_kind, qty) in items_to_loot {
                player_inv.add(item_kind, qty);
                res.log(format!(
                    "Looted {} {}.",
                    qty,
                    crate::components::itemkind_name(item_kind)
                ));
            }
        }
    }

    // Clear corpse inventory
    if let Ok(mut corpse_inv) = world.get::<&mut Inventory>(corpse_entity) {
        corpse_inv.slots.clear();
        res.log("Finished looting corpse.".to_string());
    } else {
        res.log("Failed to access corpse inventory for clearing.".to_string());
    }
}

/// Start dialogue with an NPC
fn start_npc_dialogue(
    world: &mut World,
    res: &mut Resources,
    npc_entity: hecs::Entity,
    npc_name: String,
) {
    res.log(format!("Started conversation with {}.", npc_name));

    // Mark NPC as met and set to greeting dialogue
    let dialogue_initialized = if let Ok(mut dialogue) = world.get::<&mut Dialogue>(npc_entity) {
        if !dialogue.met_before {
            dialogue.met_before = true;
            res.log(format!("First time meeting {}!", npc_name));
        }

        dialogue.current_dialogue_id = Some("start".to_string()); // Start with greeting
        true
    } else {
        res.log(format!("Warning: {} has no dialogue component.", npc_name));
        false
    };

    // Simple dialogue simulation for testing (separate borrow)
    if dialogue_initialized {
        simulate_dialogue_interaction(world, res, npc_entity, &npc_name);
    }
}

/// Simulate a dialogue interaction for testing (will be replaced by UI)
fn simulate_dialogue_interaction(
    world: &mut World,
    res: &mut Resources,
    npc_entity: hecs::Entity,
    _npc_name: &str,
) {
    use crate::dialogue::DialogueTree;

    let dialogue_tree = DialogueTree::create_quest_tutorial_broken_android_tree();
    let player_entity = world.query::<&Player>().iter().next().unwrap().0;

    // Get current dialogue state
    let current_dialogue_id = if let Ok(dialogue) = world.get::<&Dialogue>(npc_entity) {
        dialogue
            .current_dialogue_id
            .clone()
            .unwrap_or("start".to_string())
    } else {
        "start".to_string()
    };

    if let Some(dialogue_node) = dialogue_tree.get_node(&current_dialogue_id) {
        res.log(format!(
            "{}: \"{}\"",
            dialogue_node.speaker, dialogue_node.text
        ));

        // Show available choices
        for (i, choice) in dialogue_node.choices.iter().enumerate() {
            let mut choice_text = format!("{}. {}", i + 1, choice.text);

            // Check if player has required items
            if let Some(required_item) = &choice.requires_item {
                let has_item = if let Ok(inventory) = world.get::<&Inventory>(player_entity) {
                    inventory.slots.iter().any(|stack| {
                        crate::components::itemkind_name(stack.kind) == required_item
                            && stack.qty > 0
                    })
                } else {
                    false
                };

                if !has_item {
                    choice_text = format!("{} [Need: {}]", choice_text, required_item);
                } else {
                    choice_text = format!("{} [✓]", choice_text);
                }
            }

            res.log(choice_text);
        }

        // Auto-select first valid choice for testing
        if let Some(choice) = dialogue_node.choices.first() {
            execute_dialogue_choice(
                world,
                res,
                npc_entity,
                player_entity,
                choice,
                &dialogue_tree,
            );
        }
    }
}

/// Execute a dialogue choice (handle trading, mood changes, etc.)
fn execute_dialogue_choice(
    world: &mut World,
    res: &mut Resources,
    npc_entity: hecs::Entity,
    player_entity: hecs::Entity,
    choice: &crate::dialogue::DialogueChoice,
    dialogue_tree: &crate::dialogue::DialogueTree,
) {
    res.log(format!("You: \"{}\"", choice.text));

    // Handle item requirements and trading
    if let Some(required_item) = &choice.requires_item {
        let mut can_trade = false;

        // Check if player has the required item and remove it
        if let Ok(mut inventory) = world.get::<&mut Inventory>(player_entity) {
            for stack in inventory.slots.iter_mut() {
                if crate::components::itemkind_name(stack.kind) == required_item && stack.qty > 0 {
                    stack.qty -= 1;
                    res.log(format!("Used 1 {}.", required_item));
                    can_trade = true;
                    break;
                }
            }
            // Remove empty stacks
            inventory.slots.retain(|stack| stack.qty > 0);
        }

        if !can_trade {
            res.log(format!("You don't have any {} to trade!", required_item));
            return;
        }
    }

    // Handle shop items (trading rewards)
    if let Some(ref next_dialogue_id) = choice.leads_to {
        if let Some(next_node) = dialogue_tree.get_node(next_dialogue_id) {
            if let Some(shop_item) = &next_node.shop_item {
                // Give player the traded item
                if let Ok(mut inventory) = world.get::<&mut Inventory>(player_entity) {
                    let item_kind = match shop_item.as_str() {
                        "Log" => ItemKind::Log,
                        "Acorn" => ItemKind::Acorn,
                        _ => ItemKind::Log, // Default fallback
                    };
                    inventory.add(item_kind, 1);
                    res.log(format!("Received 1 {}!", shop_item));
                }
            }
        }
    }

    // Handle NPC mood changes
    if let Some(new_mood) = choice.npc_mood_change {
        if let Ok(mut dialogue) = world.get::<&mut Dialogue>(npc_entity) {
            dialogue.current_mood = new_mood;
            res.log(format!("NPC mood changed to {:?}.", new_mood));
        }
    }

    // Handle player mood changes
    if let Some(player_mood) = choice.player_mood_change {
        // For now, just log the player mood change
        // In the future, this could affect player portrait or dialogue options
        res.log(format!("You feel {:?}.", player_mood));
    }

    // Handle quest unlocking
    if choice.unlocks_quest {
        res.log("New quest unlocked!".to_string());
    }

    // Move to next dialogue or end conversation
    if let Some(ref next_dialogue_id) = choice.leads_to {
        if let Ok(mut dialogue) = world.get::<&mut Dialogue>(npc_entity) {
            dialogue.current_dialogue_id = Some(next_dialogue_id.clone());
        }
        res.log("Conversation continues...".to_string());
    } else {
        res.log("Conversation ended.".to_string());
    }
}

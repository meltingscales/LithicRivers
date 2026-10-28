use crate::App;
use crossterm::event::KeyCode;
use std::error::Error;

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

/// Handle corpse looting input when active - returns true if input was handled
pub fn handle_corpse_looting_input(app: &mut App, key: KeyCode) -> Result<bool, Box<dyn Error>> {
    // Handle corpse selection phase
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
            return Ok(true);
        }

        if app.ui.keybinds.menu_up(&key) {
            app.panels.corpse_looting = crate::app_state::CorpseLootingState::SelectingCorpse {
                adjacent_entities: entities_clone.clone(),
                selected_corpse: selected.saturating_sub(1),
            };
            return Ok(true);
        }

        if app.ui.keybinds.menu_down(&key) {
            app.panels.corpse_looting = crate::app_state::CorpseLootingState::SelectingCorpse {
                adjacent_entities: entities_clone.clone(),
                selected_corpse: (selected + 1).min(entities_clone.len().saturating_sub(1)),
            };
            return Ok(true);
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
            return Ok(true);
        }
        return Ok(true);
    }

    // Handle actual looting phase
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
            return Ok(true);
        }

        if app.ui.keybinds.menu_left(&key) || app.ui.keybinds.menu_right(&key) {
            panel_focus = !panel_focus;
        }

        if app.ui.keybinds.menu_up(&key) {
            if panel_focus {
                loot_item = loot_item.saturating_sub(1);
            } else {
                player_item = player_item.saturating_sub(1);
            }
        }

        if app.ui.keybinds.menu_down(&key) {
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
            } else if let Some(player_entity) = app.core.game.get_player_entity() {
                let rows = crate::app::item_filter::PlayerRows::for_container(
                    &app.core.game.world,
                    player_entity,
                    corpse_entity,
                );
                player_item = (player_item + 1).min(rows.len().saturating_sub(1));
            }
        }

        if app.ui.keybinds.matches("ui", "TAKE_ALL", &key) {
            if let Some(player) = app.core.game.get_player_entity() {
                let world = &mut app.core.game.world;
                let moved =
                    lithicrivers_core::factory::move_all_items(world, corpse_entity, player);
                let left = world
                    .get::<&lithicrivers_core::components::Inventory>(corpse_entity)
                    .map_or(0, |inv| inv.slots.len());
                let message = match (moved, left) {
                    (0, 0) => "Nothing to take".to_string(),
                    (0, _) => "Your inventory is full".to_string(),
                    (n, 0) => format!("Took all {n} items"),
                    (n, _) => format!("Took {n} items; your inventory is full"),
                };
                app.core.game.res.log(message);
            }
            app.panels.corpse_looting = crate::app_state::CorpseLootingState::LootingCorpse {
                entity: corpse_entity,
                selected_loot_item: 0,
                selected_player_item: player_item,
                loot_panel_focus: panel_focus,
            };
            return Ok(true);
        }

        let activate =
            app.ui.keybinds.matches("ui", "MENU_ACTIVATE", &key) || key == KeyCode::Enter;
        let is_chest = app
            .core
            .game
            .world
            .get::<&lithicrivers_core::factory::FactoryBlock>(corpse_entity)
            .is_ok();
        if activate && is_chest {
            // Chests move items both ways: from the focused panel to the other one
            if let Some(player) = app.core.game.get_player_entity() {
                use lithicrivers_core::factory::{
                    move_one_item, move_one_item_to_block, TransferError,
                };
                let world = &mut app.core.game.world;
                // The selected row of the (maybe filtered) list, as an inventory stack
                let rows = crate::app::item_filter::PlayerRows::for_container(
                    world,
                    player,
                    corpse_entity,
                );
                let player_slot = rows.slot(player_item).unwrap_or(usize::MAX);
                // Storing goes through the block's own rules (a generator only takes fuel)
                let (result, verb) = if panel_focus {
                    (
                        move_one_item(world, corpse_entity, player, loot_item),
                        "Took",
                    )
                } else {
                    let result = move_one_item_to_block(world, player, corpse_entity, player_slot);
                    (result, "Stored")
                };
                use lithicrivers_core::factory::FactoryKind;
                let block_kind = world
                    .get::<&lithicrivers_core::factory::FactoryBlock>(corpse_entity)
                    .map(|b| b.kind)
                    .unwrap_or(FactoryKind::Chest);
                let block_name = block_kind.name();
                let stored_item = world
                    .get::<&lithicrivers_core::components::Inventory>(player)
                    .ok()
                    .and_then(|inv| inv.slots.get(player_slot).map(|s| s.kind));
                // Why the block refuses this item outright, rather than being full
                let refusal = match (block_kind, stored_item) {
                    (FactoryKind::BurnerGenerator, Some(item))
                        if lithicrivers_core::power::fuel_energy(item).is_none() =>
                    {
                        Some(format!(
                            "{} isn't fuel",
                            lithicrivers_core::components::itemkind_name(item)
                        ))
                    }
                    (FactoryKind::AutoPlanter, Some(item))
                        if item != lithicrivers_core::components::ItemKind::Acorn =>
                    {
                        Some("The auto-planter only takes acorns".to_string())
                    }
                    (FactoryKind::AutoHarvester, Some(_)) => {
                        Some("The auto-harvester only gives items out".to_string())
                    }
                    _ => None,
                };
                match result {
                    Ok(kind) => app.core.game.res.log(format!(
                        "{verb} {}",
                        lithicrivers_core::components::itemkind_name(kind)
                    )),
                    Err(TransferError::Full) if panel_focus => {
                        app.core.game.res.log("Your inventory is full")
                    }
                    Err(TransferError::Full) if refusal.is_some() => {
                        app.core.game.res.log(refusal.unwrap_or_default())
                    }
                    Err(TransferError::Full) => {
                        app.core.game.res.log(format!("The {block_name} is full"))
                    }
                    Err(TransferError::Empty) => {}
                }
                // Keep both selections on a stack that still exists
                let len = |e| {
                    app.core
                        .game
                        .world
                        .get::<&lithicrivers_core::components::Inventory>(e)
                        .map(|inv| inv.slots.len())
                        .unwrap_or(0)
                };
                loot_item = loot_item.min(len(corpse_entity).saturating_sub(1));
                let rows = crate::app::item_filter::PlayerRows::for_container(
                    &app.core.game.world,
                    player,
                    corpse_entity,
                );
                player_item = player_item.min(rows.len().saturating_sub(1));
            }
        } else if activate && panel_focus {
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
        return Ok(true);
    }

    Ok(false)
}

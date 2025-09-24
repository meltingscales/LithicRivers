use crate::app::input_handler::take_item_from_corpse;
use crate::App;
use crossterm::event::KeyCode;
use std::error::Error;

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

        if app.ui.keybinds.matches("movement", "MOVE_NORTH", &key) {
            app.panels.corpse_looting = crate::app_state::CorpseLootingState::SelectingCorpse {
                adjacent_entities: entities_clone.clone(),
                selected_corpse: selected.saturating_sub(1),
            };
            return Ok(true);
        }

        if app.ui.keybinds.matches("movement", "MOVE_SOUTH", &key) {
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
        return Ok(true);
    }

    Ok(false)
}

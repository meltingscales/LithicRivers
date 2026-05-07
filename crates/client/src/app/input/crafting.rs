use crate::ui::panels::get_player_inventory;
use crate::{App, MenuTab};
use crossterm::event::KeyCode;
use lithicrivers_core::components::{itemkind_name, Inventory as InvComp, ItemStack};
use std::error::Error;

/// Handle crafting tab input - returns true if input was handled
pub fn handle_crafting_input(app: &mut App, key: KeyCode) -> Result<bool, Box<dyn Error>> {
    // Only handle input when on the crafting tab
    if app.ui.current_tab != MenuTab::Crafting {
        return Ok(false);
    }

    // Get player inventory for crafting checks
    let inventory = get_player_inventory(app);
    let recipe_count = app.panels.crafting.recipe_handler.get_recipes().len();

    // Navigation: Up/Down or North/South to move selection
    if app.ui.keybinds.matches("ui", "CREDITS_SCROLL_UP", &key)
        || app.ui.keybinds.matches("movement", "MOVE_NORTH", &key)
    {
        if recipe_count > 0 {
            if app.panels.crafting.selected == 0 {
                app.panels.crafting.selected = recipe_count - 1;
            } else {
                app.panels.crafting.selected = app.panels.crafting.selected.saturating_sub(1);
            }
        }
        return Ok(true);
    }

    if app.ui.keybinds.matches("ui", "CREDITS_SCROLL_DOWN", &key)
        || app.ui.keybinds.matches("movement", "MOVE_SOUTH", &key)
    {
        if recipe_count > 0 {
            app.panels.crafting.selected = (app.panels.crafting.selected + 1) % recipe_count;
        }
        return Ok(true);
    }

    // Craft item on Enter or Activate key
    if app.ui.keybinds.matches("ui", "ACTIVATE", &key) || key == KeyCode::Enter {
        if let Some(recipe) = app
            .panels
            .crafting
            .recipe_handler
            .get_recipes()
            .get(app.panels.crafting.selected)
        {
            if app
                .panels
                .crafting
                .recipe_handler
                .can_craft(app.panels.crafting.selected, &inventory)
            {
                if let Some(e) = app.core.game.get_player_entity() {
                    if let Ok(mut inv) = app.core.game.world.get::<&mut InvComp>(e) {
                        // Consume ingredients
                        for &(item, qty) in recipe.ingredients {
                            let mut remaining = qty;
                            for slot in inv.slots.iter_mut() {
                                if slot.kind == item && remaining > 0 {
                                    let consume = slot.qty.min(remaining);
                                    slot.qty -= consume;
                                    remaining -= consume;

                                    if slot.qty == 0 {
                                        // Remove empty slots - the inventory will be compacted later
                                        // by the game's inventory management system
                                    }
                                }
                            }
                        }

                        // Add crafted item to inventory
                        let mut added = false;
                        for slot in inv.slots.iter_mut() {
                            if slot.kind == recipe.result && slot.qty < 1000 {
                                // Arbitrary max stack size
                                slot.qty = slot.qty.saturating_add(recipe.quantity);
                                added = true;
                                break;
                            }
                        }

                        if !added {
                            inv.slots.push(ItemStack {
                                kind: recipe.result,
                                qty: recipe.quantity,
                            });
                        }

                        app.panels.crafting.message = Some((
                            format!(
                                "Crafted {}x {}",
                                recipe.quantity,
                                itemkind_name(recipe.result)
                            ),
                            30, // Display for 30 frames (~0.5 seconds at 60 FPS)
                        ));

                        app.core.game.res.log(format!(
                            "Crafted {}x {}",
                            recipe.quantity,
                            itemkind_name(recipe.result)
                        ));
                    }
                }
            } else {
                app.panels.crafting.message = Some((
                    "Not enough resources to craft this item".to_string(),
                    60, // Display for 1 second at 60 FPS
                ));
            }
        }
        return Ok(true);
    }

    // Don't process movement keys in crafting panel
    if app.ui.keybinds.matches("movement", "MOVE_NORTH", &key)
        || app.ui.keybinds.matches("movement", "MOVE_SOUTH", &key)
        || app.ui.keybinds.matches("movement", "MOVE_WEST", &key)
        || app.ui.keybinds.matches("movement", "MOVE_EAST", &key)
        || app.ui.keybinds.matches("movement", "MOVE_NORTHWEST", &key)
        || app.ui.keybinds.matches("movement", "MOVE_NORTHEAST", &key)
        || app.ui.keybinds.matches("movement", "MOVE_SOUTHWEST", &key)
        || app.ui.keybinds.matches("movement", "MOVE_SOUTHEAST", &key)
        || app.ui.keybinds.matches("movement", "WAIT", &key)
        || app.ui.keybinds.matches("movement", "MOVE_UP", &key)
        || app.ui.keybinds.matches("movement", "MOVE_DOWN", &key)
    {
        return Ok(true);
    }

    Ok(false)
}

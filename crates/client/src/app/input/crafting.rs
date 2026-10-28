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

    // The search box gets the key first
    if let Some(reset) = app.panels.crafting.search.handle(&app.ui.keybinds, &key) {
        if reset {
            app.panels.crafting.selected = 0;
        }
        return Ok(true);
    }

    // Get player inventory for crafting checks
    let inventory = get_player_inventory(app);
    let shown = crate::ui::panels::crafting::filtered_recipes(app);
    let recipe_count = shown.len();

    // Navigation: Up/Down or North/South to move selection
    if app.ui.keybinds.menu_up(&key) {
        if recipe_count > 0 {
            if app.panels.crafting.selected == 0 {
                app.panels.crafting.selected = recipe_count - 1;
            } else {
                app.panels.crafting.selected = app.panels.crafting.selected.saturating_sub(1);
            }
        }
        return Ok(true);
    }

    if app.ui.keybinds.menu_down(&key) {
        if recipe_count > 0 {
            app.panels.crafting.selected = (app.panels.crafting.selected + 1) % recipe_count;
        }
        return Ok(true);
    }

    // Craft item on Enter or Activate key
    if app.ui.keybinds.matches("ui", "MENU_ACTIVATE", &key) || key == KeyCode::Enter {
        let index = shown.get(app.panels.crafting.selected).copied();
        if let Some((index, recipe)) = index.and_then(|i| {
            app.panels
                .crafting
                .recipe_handler
                .get_recipes()
                .get(i)
                .map(|r| (i, r))
        }) {
            if app
                .panels
                .crafting
                .recipe_handler
                .can_craft(index, &inventory)
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

    // Don't process movement keys in crafting panel (the arrow keys still switch tabs)
    Ok(app.ui.keybinds.menu_up(&key)
        || app.ui.keybinds.menu_down(&key)
        || app.ui.keybinds.matches("movement", "MOVE_WEST", &key)
        || app.ui.keybinds.matches("movement", "MOVE_EAST", &key)
        || app.ui.keybinds.matches("movement", "MOVE_NORTHWEST", &key)
        || app.ui.keybinds.matches("movement", "MOVE_NORTHEAST", &key)
        || app.ui.keybinds.matches("movement", "MOVE_SOUTHWEST", &key)
        || app.ui.keybinds.matches("movement", "MOVE_SOUTHEAST", &key)
        || app.ui.keybinds.matches("movement", "WAIT", &key)
        || app.ui.keybinds.matches("movement", "MOVE_UP", &key)
        || app.ui.keybinds.matches("movement", "MOVE_DOWN", &key))
}

#[cfg(test)]
mod search_tests {
    use crate::headless::{chain_commands, Runner};
    use crate::ui::panels::crafting::filtered_recipes;
    use lithicrivers_core::components::{Inventory, ItemKind};

    fn crafting_tab(steps: &[&str]) -> Runner {
        let mut runner = chain_commands(&["skip-splash", "key ui.MENU_NEXT 5"]);
        runner.chain(steps);
        runner
    }

    fn shown_results(runner: &mut Runner) -> Vec<ItemKind> {
        let recipes = runner
            .app
            .panels
            .crafting
            .recipe_handler
            .get_recipes()
            .to_vec();
        filtered_recipes(&mut runner.app)
            .into_iter()
            .map(|i| recipes[i].result)
            .collect()
    }

    fn count(runner: &Runner, item: ItemKind) -> u32 {
        let game = &runner.app.core.game;
        let player = game.get_player_entity().unwrap();
        let inv = game.world.get::<&Inventory>(player).unwrap();
        inv.slots
            .iter()
            .filter(|s| s.kind == item)
            .map(|s| s.qty)
            .sum()
    }

    #[test]
    fn searching_plank_lists_the_plank_recipe_and_its_uses() {
        let mut runner = crafting_tab(&[
            "key search.SEARCH",
            "type \"plank\"",
            "key search.END_SEARCH",
        ]);
        let recipes = runner
            .app
            .panels
            .crafting
            .recipe_handler
            .get_recipes()
            .to_vec();
        let expected: Vec<ItemKind> = recipes
            .iter()
            .filter(|r| {
                r.result == ItemKind::WoodenPlank
                    || r.result == ItemKind::PlankBlock
                    || r.ingredients
                        .iter()
                        .any(|&(i, _)| i == ItemKind::WoodenPlank)
            })
            .map(|r| r.result)
            .collect();
        let shown = shown_results(&mut runner);
        assert!(shown.contains(&ItemKind::WoodenPlank));
        assert!(shown.contains(&ItemKind::Chest), "chests use planks");
        assert!(
            !shown.contains(&ItemKind::Nail),
            "nails have nothing to do with planks"
        );
        for item in &expected {
            assert!(shown.contains(item), "{item:?}");
        }
        assert!(runner.screen_text().contains("Search: plank"));
        assert!(runner.screen_text().contains(&format!(
            "showing {} of {}",
            shown.len(),
            recipes.len()
        )));
    }

    #[test]
    fn crafting_acts_on_the_selected_shown_recipe_and_escape_clears() {
        let mut runner = crafting_tab(&[
            "key search.SEARCH",
            "type \"plank\"",
            "key search.END_SEARCH",
            "key movement.MOVE_SOUTH",
        ]);
        let second = shown_results(&mut runner)[1];
        let name = lithicrivers_core::components::itemkind_sprite_name(second);
        runner.chain(&[&format!("cheat givecraftingcomponents {name}")]);
        let before = count(&runner, second);
        runner.chain(&["key ui.MENU_ACTIVATE"]);
        assert!(count(&runner, second) > before, "crafted {second:?}");

        runner.chain(&["rawkey ESCAPE"]);
        let all = runner
            .app
            .panels
            .crafting
            .recipe_handler
            .get_recipes()
            .len();
        assert_eq!(shown_results(&mut runner).len(), all);
        assert!(runner.app.panels.crafting.search.text.is_empty());
    }
}

use crate::ui::panels::get_player_inventory;
use crate::{App, MenuTab};
use crossterm::event::KeyCode;
use std::error::Error;

use lithicrivers_core::model::body::PART_ORDER;

/// The selected part and what fits it.
fn selection(
    app: &App,
) -> (
    lithicrivers_core::model::body::BodyPartType,
    Vec<(lithicrivers_core::components::ItemKind, u32)>,
) {
    let part = PART_ORDER[app.panels.body.selected_part.min(PART_ORDER.len() - 1)];
    (part, app.core.game.fitting_items(part))
}

/// Install the `index`-th fitting item into the selected part (swapping out what's there).
pub(crate) fn install_fitting(app: &mut App, index: usize) {
    let (part, fitting) = selection(app);
    app.panels.body.choosing_item = false;
    if let Some(&(item, _)) = fitting.get(index) {
        let _ = app.core.game.install_part_into(part, item);
    }
}

/// The Body tab's own keys: move between parts (or fitting items), install and remove.
fn handle_part_keys(app: &mut App, key: &KeyCode) -> bool {
    let keys = &app.ui.keybinds;
    let (part, fitting) = selection(app);
    let body = &mut app.panels.body;
    if body.choosing_item {
        if keys.menu_up(key) {
            body.selected_item = body.selected_item.saturating_sub(1);
        } else if keys.menu_down(key) {
            body.selected_item = (body.selected_item + 1).min(fitting.len().saturating_sub(1));
        } else if keys.matches("ui", "MENU_ACTIVATE", key)
            || keys.matches("body", "INSTALL_PART", key)
        {
            let index = body.selected_item;
            install_fitting(app, index);
        } else if keys.matches("ui", "CLOSE_HELP_MENU", key) {
            body.choosing_item = false;
        } else {
            return false;
        }
        return true;
    }
    if keys.menu_up(key) {
        body.selected_part = body.selected_part.saturating_sub(1);
    } else if keys.menu_down(key) {
        body.selected_part = (body.selected_part + 1).min(PART_ORDER.len() - 1);
    } else if keys.matches("body", "INSTALL_PART", key) || keys.matches("ui", "MENU_ACTIVATE", key)
    {
        match fitting.len() {
            0 => {
                let name = app
                    .core
                    .game
                    .get_player_component::<lithicrivers_core::model::body::Body>()
                    .and_then(|b| b.parts.get(&part).map(|p| p.name.clone()))
                    .unwrap_or_default();
                app.core
                    .game
                    .res
                    .log_yellow(format!("Nothing in the inventory fits the {name}"));
            }
            1 => install_fitting(app, 0),
            _ => {
                body.choosing_item = true;
                body.selected_item = 0;
            }
        }
    } else if keys.matches("body", "REMOVE_PART", key) {
        let _ = app.core.game.remove_part(part);
    } else {
        return false;
    }
    true
}

/// Handle body tab input - returns true if input was handled
pub fn handle_body_input(app: &mut App, key: KeyCode) -> Result<bool, Box<dyn Error>> {
    // Only handle input when on the body tab
    if app.ui.current_tab != MenuTab::Body {
        return Ok(false);
    }

    // Choose a part, then install, swap or remove it (not while the repair window is open)
    if matches!(
        app.panels.body_repair,
        crate::app_state::BodyRepairState::None
    ) && handle_part_keys(app, &key)
    {
        return Ok(true);
    }

    // Handle R key to open repair modal
    if key == KeyCode::Char('r') || key == KeyCode::Char('R') {
        // Initialize repair state with first recipe and first available body part selected
        let available_repairs = (0..app.core.repair_handler.get_recipes().len()).collect();

        // The selected part if it can be repaired, else the first that can
        let selected = PART_ORDER[app.panels.body.selected_part.min(PART_ORDER.len() - 1)];
        let initial_body_part = if let Some(player_entity) = app.core.game.get_player_entity() {
            if let Ok(body) = app
                .core
                .game
                .world
                .get::<&lithicrivers_core::model::body::Body>(player_entity)
            {
                // Repairable: not missing and not at max integrity
                std::iter::once(&selected)
                    .chain(PART_ORDER.iter())
                    .find(|&&part_type| {
                        body.parts
                            .get(&part_type)
                            .map(|part| {
                                part.state != lithicrivers_core::model::body::BodyPartState::Missing
                                    && part.integrity < 100
                            })
                            .unwrap_or(false)
                    })
                    .copied()
            } else {
                None
            }
        } else {
            None
        };

        app.panels.body_repair = crate::app_state::BodyRepairState::SelectingRepairAndPart {
            available_repairs,
            selected_repair: 0,
            selected_body_part: initial_body_part,
        };
        app.core.game.res.log("Opened repair interface");
        return Ok(true);
    }

    // Handle repair modal input
    if let crate::app_state::BodyRepairState::SelectingRepairAndPart {
        selected_repair,
        selected_body_part,
        available_repairs,
    } = &app.panels.body_repair
    {
        let mut repair_idx = *selected_repair;
        let mut part_selection = *selected_body_part;

        // Handle ESC to close repair modal
        if app.ui.keybinds.matches("ui", "CLOSE_HELP_MENU", &key) {
            app.panels.body_repair = crate::app_state::BodyRepairState::None;
            app.core.game.res.log("Closed repair interface");
            return Ok(true);
        }

        // Handle navigation
        if app.ui.keybinds.menu_up(&key) {
            let recipe_count = app.core.repair_handler.get_recipes().len();
            if recipe_count > 0 {
                repair_idx = if repair_idx == 0 {
                    recipe_count - 1
                } else {
                    repair_idx - 1
                };
            }
        } else if app.ui.keybinds.menu_down(&key) {
            let recipe_count = app.core.repair_handler.get_recipes().len();
            if recipe_count > 0 {
                repair_idx = (repair_idx + 1) % recipe_count;
            }
        } else if app.ui.keybinds.menu_left(&key) {
            // Previous body part
            use lithicrivers_core::model::body::BodyPartType;
            let parts = [
                BodyPartType::Head,
                BodyPartType::Torso,
                BodyPartType::PowerSource,
                BodyPartType::LeftArm,
                BodyPartType::RightArm,
                BodyPartType::LeftLeg,
                BodyPartType::RightLeg,
            ];

            if let Some(current_part) = part_selection {
                if let Some(current_idx) = parts.iter().position(|&p| p == current_part) {
                    let new_idx = if current_idx == 0 {
                        parts.len() - 1
                    } else {
                        current_idx - 1
                    };
                    part_selection = Some(parts[new_idx]);
                }
            } else {
                part_selection = Some(parts[parts.len() - 1]);
            }
        } else if app.ui.keybinds.menu_right(&key) {
            // Next body part
            use lithicrivers_core::model::body::BodyPartType;
            let parts = [
                BodyPartType::Head,
                BodyPartType::Torso,
                BodyPartType::PowerSource,
                BodyPartType::LeftArm,
                BodyPartType::RightArm,
                BodyPartType::LeftLeg,
                BodyPartType::RightLeg,
            ];

            if let Some(current_part) = part_selection {
                if let Some(current_idx) = parts.iter().position(|&p| p == current_part) {
                    let new_idx = (current_idx + 1) % parts.len();
                    part_selection = Some(parts[new_idx]);
                }
            } else {
                part_selection = Some(parts[0]);
            }
        } else if app.ui.keybinds.matches("ui", "MENU_ACTIVATE", &key) || key == KeyCode::Enter {
            // Execute repair
            if let Some(body_part) = part_selection {
                let mut inventory = get_player_inventory(app);
                if app.core.repair_handler.can_repair(repair_idx, &inventory) {
                    if let Some(player_entity) = app.core.game.get_player_entity() {
                        // Get the current body and check if we can repair this part
                        if let Ok(mut body) =
                            app.core
                                .game
                                .world
                                .get::<&mut lithicrivers_core::model::body::Body>(player_entity)
                        {
                            if let Some(part) = body.parts.get_mut(&body_part) {
                                if part.integrity >= 100 {
                                    app.core
                                        .game
                                        .res
                                        .log("Body part is already at maximum integrity");
                                } else if app
                                    .core
                                    .repair_handler
                                    .can_repair_part(repair_idx, body_part, part.state)
                                {
                                    if let Some(new_integrity) = app
                                        .core
                                        .repair_handler
                                        .apply_repair(repair_idx, &mut inventory, part.integrity)
                                    {
                                        part.integrity = new_integrity;

                                        // Update state if integrity is now above thresholds
                                        if part.integrity > 0 && part.state == lithicrivers_core::model::body::BodyPartState::Missing {
                                            part.state = lithicrivers_core::model::body::BodyPartState::Damaged;
                                        } else if part.integrity >= 75 && part.state == lithicrivers_core::model::body::BodyPartState::Damaged {
                                            part.state = lithicrivers_core::model::body::BodyPartState::Functional;
                                        }

                                        // Update player inventory
                                        if let Ok(mut inv) = app.core.game.world.get::<&mut lithicrivers_core::components::Inventory>(player_entity) {
                                            inv.slots.clear();
                                            for (item_kind, quantity) in inventory {
                                                if quantity > 0 {
                                                    inv.slots.push(lithicrivers_core::components::ItemStack {
                                                        kind: item_kind,
                                                        qty: quantity,
                                                    });
                                                }
                                            }
                                        }

                                        app.core.game.res.log(format!(
                                            "Repaired {} for +{} integrity",
                                            part.name,
                                            app.core.repair_handler.get_recipes()[repair_idx]
                                                .durability_restored
                                        ));
                                    } else {
                                        app.core
                                            .game
                                            .res
                                            .log("Repair failed - insufficient materials");
                                    }
                                } else {
                                    app.core
                                        .game
                                        .res
                                        .log("Cannot repair this body part with selected recipe");
                                }
                            } else {
                                app.core.game.res.log("Body part not found");
                            }
                        } else {
                            app.core.game.res.log("Cannot access player body");
                        }
                    }
                } else {
                    app.core.game.res.log("Insufficient materials for repair");
                }
            } else {
                app.core.game.res.log("Please select a body part to repair");
            }
            return Ok(true);
        }

        // Update repair state
        app.panels.body_repair = crate::app_state::BodyRepairState::SelectingRepairAndPart {
            available_repairs: available_repairs.clone(),
            selected_repair: repair_idx,
            selected_body_part: part_selection,
        };
        return Ok(true);
    }

    Ok(false)
}

#[cfg(test)]
mod tests {
    use crate::app_state::BodyRepairState;
    use crate::headless::chain_commands;
    use lithicrivers_core::model::body::BodyPartType;

    #[test]
    fn repair_opens_on_the_selected_part() {
        // The damaged right leg, seventh in the list
        let runner = chain_commands(&[
            "skip-splash",
            "key ui.MENU_NEXT 3",
            "key movement.MOVE_SOUTH 6",
            "rawkey r",
        ]);
        assert!(matches!(
            runner.app.panels.body_repair,
            BodyRepairState::SelectingRepairAndPart {
                selected_body_part: Some(BodyPartType::RightLeg),
                ..
            }
        ));
    }
}

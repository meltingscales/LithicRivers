use crossterm::event::KeyCode;
use std::error::Error;

use crate::App;

pub fn handle_input(app: &mut App, key: KeyCode) -> Result<(), Box<dyn Error>> {
    // Handle splash screen input when active - delegated to input module
    if crate::app::input::splash::handle_splash_input(app, key)? {
        return Ok(());
    }

    // log key to log
    tracing::info!(target: "game", "key pressed: {:?}", key);

    // Special debug for F key
    if key == KeyCode::Char('f') || key == KeyCode::Char('F') {
        tracing::info!(target: "game", "F key detected! Current tab: {:?}, splash state: {:?}",
            app.ui.current_tab, app.splash.state);
    }

    // Handle world/build mode input when on World tab - delegated to input module
    if crate::app::input::world_build::handle_world_build_input(app, key)? {
        return Ok(());
    }

    // Handle look mode input when on World tab - delegated to input module
    if crate::app::input::look_mode::handle_look_mode_input(app, key)? {
        return Ok(());
    }
    // Handle inventory tab input when active - delegated to input module
    if crate::app::input::inventory::handle_inventory_input(app, key)? {
        return Ok(());
    }

    // Handle crafting tab input when active - delegated to input module
    if crate::app::input::crafting::handle_crafting_input(app, key)? {
        return Ok(());
    }

    // Handle body tab input when active - delegated to input module
    if crate::app::input::body::handle_body_input(app, key)? {
        return Ok(());
    }

    // Handle hotbar assignment input when active - delegated to input module
    if crate::app::input::hotbar_assignment::handle_hotbar_assignment_input(app, key)? {
        return Ok(());
    }

    // Handle multi-action selection input when active - delegated to input module
    if crate::app::input::multi_action::handle_multi_action_input(app, key)? {
        return Ok(());
    }

    // Handle corpse looting input when active - delegated to input module
    if crate::app::input::corpse_looting::handle_corpse_looting_input(app, key)? {
        return Ok(());
    }

    // Handle NPC interaction input when active - delegated to input module
    if crate::app::input::npc_interaction::handle_npc_interaction_input(app, key)? {
        return Ok(());
    }

    // Handle combat-specific input when combat is active - delegated to input module
    if crate::app::input::combat::handle_combat_input(app, key)? {
        return Ok(());
    }

    // Handle world movement input when on World tab - delegated to input module
    if crate::app::input::world_movement::handle_world_movement_input(app, key)? {
        return Ok(());
    }

    // Handle world action input when on World tab - delegated to input module
    if crate::app::input::world_action::handle_world_action_input(app, key)? {
        return Ok(());
    }

    // Handle global UI input (quit, menu navigation, scrolling, save/load) - delegated to input module
    if crate::app::input::global_ui::handle_global_ui_input(app, key)? {
        return Ok(());
    }

    // Debug: if we reach the end without handling F key, log it
    if key == KeyCode::Char('f') || key == KeyCode::Char('F') {
        tracing::info!(target: "game", "F key reached end of input handler without being handled!");
    }

    Ok(())
}

/// Execute a specific interaction action (delegated to world_action module)
pub fn execute_interaction_action(app: &mut App, action: &crate::app_state::InteractionType) {
    crate::app::input::world_action::execute_interaction_action(app, action);
}

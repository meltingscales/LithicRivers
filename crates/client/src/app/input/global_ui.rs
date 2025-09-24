use crate::{App, MenuTab};
use crossterm::event::KeyCode;
use std::error::Error;

/// Handle global UI input (quit, noclip, menu navigation, scrolling, save/load) - returns true if input was handled
pub fn handle_global_ui_input(app: &mut App, key: KeyCode) -> Result<bool, Box<dyn Error>> {
    // UI: Quit
    if app.ui.keybinds.matches("ui", "QUIT", &key) {
        app.core.game.res.log("Quit requested (keybind)");
        tracing::info!(target: "game", "quit_requested tick={}", app.core.game.res.time.tick);
        app.core.should_quit = true;
        return Ok(true);
    }

    // Global Cheats: Toggle noclip mode
    if app
        .ui
        .keybinds
        .matches("inventory", "CHEAT_NOCLIP_TOGGLE", &key)
    {
        app.core.game.res.player_state.noclip_enabled =
            !app.core.game.res.player_state.noclip_enabled;
        let state = if app.core.game.res.player_state.noclip_enabled {
            "ON"
        } else {
            "OFF"
        };
        app.core.game.res.log(format!("Noclip mode: {}", state));
        tracing::info!(target: "game", "Noclip mode toggled: {}", state);
        return Ok(true);
    }

    // UI: Menu activation and paging
    if app.ui.keybinds.matches("ui", "MENU_ACTIVATE", &key) {
        app.activate_menu();
        return Ok(true);
    }
    // Block menu navigation during combat
    if !app.combat.is_active() {
        if app.ui.keybinds.matches("ui", "MENU_PREV", &key) {
            app.ui.current_tab = app.ui.current_tab.prev();
            return Ok(true);
        }
        if app.ui.keybinds.matches("ui", "MENU_NEXT", &key) {
            app.ui.current_tab = app.ui.current_tab.next();
            return Ok(true);
        }
    }

    // Credits scroll
    if app.ui.current_tab == MenuTab::Credits {
        let max_scroll = get_max_credits_scroll(app);
        if app.ui.keybinds.matches("ui", "CREDITS_SCROLL_UP", &key) {
            app.panels.credits.scroll = app.panels.credits.scroll.saturating_sub(1);
            return Ok(true);
        }
        if app.ui.keybinds.matches("ui", "CREDITS_SCROLL_DOWN", &key) {
            app.panels.credits.scroll =
                (app.panels.credits.scroll.saturating_add(1)).min(max_scroll);
            return Ok(true);
        }
    }

    // Help scroll
    if app.ui.current_tab == MenuTab::Help {
        let max_scroll = get_max_help_scroll(app);
        if app.ui.keybinds.matches("ui", "HELP_SCROLL_UP", &key) {
            app.panels.help.scroll = app.panels.help.scroll.saturating_sub(1);
            return Ok(true);
        }
        if app.ui.keybinds.matches("ui", "HELP_SCROLL_DOWN", &key) {
            app.panels.help.scroll = (app.panels.help.scroll.saturating_add(1)).min(max_scroll);
            return Ok(true);
        }
        // Fallback to arrow keys for help scrolling
        match key {
            KeyCode::Up => {
                app.panels.help.scroll = app.panels.help.scroll.saturating_sub(1);
                return Ok(true);
            }
            KeyCode::Down => {
                app.panels.help.scroll = (app.panels.help.scroll.saturating_add(1)).min(max_scroll);
                return Ok(true);
            }
            _ => {}
        }
    }

    // View Z slice up/down
    if app.ui.keybinds.matches("viewport", "VIEW_Z_UP", &key) {
        if app.ui.current_tab == MenuTab::Credits {
            app.panels.credits.scroll = app.panels.credits.scroll.saturating_sub(10);
        } else if app.ui.current_tab == MenuTab::Help {
            app.panels.help.scroll = app.panels.help.scroll.saturating_sub(10);
        } else {
            app.ui.view_z = app.ui.view_z.saturating_add(1);
        }
        return Ok(true);
    }
    if app.ui.keybinds.matches("viewport", "VIEW_Z_DOWN", &key) {
        if app.ui.current_tab == MenuTab::Credits {
            let max_scroll = get_max_credits_scroll(app);
            app.panels.credits.scroll =
                (app.panels.credits.scroll.saturating_add(10)).min(max_scroll);
        } else if app.ui.current_tab == MenuTab::Help {
            let max_scroll = get_max_help_scroll(app);
            app.panels.help.scroll = (app.panels.help.scroll.saturating_add(10)).min(max_scroll);
        } else {
            app.ui.view_z = app.ui.view_z.saturating_sub(1);
        }
        return Ok(true);
    }

    // Save/Load via config (only in Menu tab)
    if app.ui.keybinds.matches("ui", "SAVE_JSON", &key) {
        if app.ui.current_tab != crate::MenuTab::Menu {
            app.core.game.res.log("Save only available in Menu tab");
            return Ok(true);
        }
        tracing::info!(target: "game", "save_begin path=save.json tick={}", app.core.game.res.time.tick);
        let viewport = lithicrivers_core::save_load::ViewportSave {
            view_x: app.ui.view_x,
            view_y: app.ui.view_y,
            view_z: app.ui.view_z,
        };
        app.core
            .game
            .save_json_with_viewport("save.json", viewport)
            .map_err(|e| format!("save_json_with_viewport error: {:?}", e))?;
        app.core.game.res.log("Saved to save.json");
        tracing::info!(target: "game", "save_end path=save.json tick={}", app.core.game.res.time.tick);
        return Ok(true);
    }
    if app.ui.keybinds.matches("ui", "LOAD_JSON", &key) {
        if app.ui.current_tab != crate::MenuTab::Menu {
            app.core.game.res.log("Load only available in Menu tab");
            return Ok(true);
        }
        tracing::info!(target: "game", "load_begin path=save.json tick={}", app.core.game.res.time.tick);
        let viewport = app
            .core
            .game
            .load_json_with_viewport("save.json")
            .map_err(|e| format!("load_json_with_viewport error: {:?}", e))?;

        // Restore viewport
        app.ui.view_x = viewport.view_x;
        app.ui.view_y = viewport.view_y;
        app.ui.view_z = viewport.view_z;

        app.core.game.res.log("Loaded from save.json");
        tracing::info!(target: "game", "load_end path=save.json tick={}", app.core.game.res.time.tick);
        return Ok(true);
    }

    // No global UI input was handled
    Ok(false)
}

/// Calculate the maximum scroll value for the credits panel
fn get_max_credits_scroll(app: &App) -> u16 {
    // Count lines in credits text and subtract visible area height
    let line_count = app.panels.credits.text.lines().count() as u16;
    // Assume panel height is around 20 lines (terminal height minus UI elements)
    // This is a conservative estimate - in practice the panel might be larger
    let visible_lines = 20;
    line_count.saturating_sub(visible_lines)
}

/// Calculate the maximum scroll value for the help panel
fn get_max_help_scroll(_app: &App) -> u16 {
    // For help panel, we need to count the dynamically generated lines
    // This is an approximation based on the keybinds structure
    let base_lines = 15; // Movement diagram and basic text
    let keybind_categories = 5; // viewport, scale, action, ui, inventory
    let avg_keybinds_per_category = 8;
    let total_lines = base_lines + (keybind_categories * (avg_keybinds_per_category + 2)) as u16; // +2 for category header and spacing
    let visible_lines = 20;
    total_lines.saturating_sub(visible_lines)
}

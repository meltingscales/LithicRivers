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

    // UI: Menu activation and paging
    if app.ui.keybinds.matches("ui", "MENU_ACTIVATE", &key) {
        app.activate_menu();
        return Ok(true);
    }
    // Block menu navigation during combat
    if !app.combat.is_active() {
        if app.ui.keybinds.matches("ui", "MENU_PREV", &key) {
            app.ui.current_tab = app.ui.current_tab.prev(app.ui.tutorial_visible);
            return Ok(true);
        }
        if app.ui.keybinds.matches("ui", "MENU_NEXT", &key) {
            app.ui.current_tab = app.ui.current_tab.next(app.ui.tutorial_visible);
            return Ok(true);
        }
    }

    // Credits scroll
    if app.ui.current_tab == MenuTab::Credits {
        let max_scroll = get_max_credits_scroll(app);
        if app.ui.keybinds.matches("ui", "CREDITS_SCROLL_UP", &key) || app.ui.keybinds.menu_up(&key)
        {
            app.panels.credits.scroll = app.panels.credits.scroll.saturating_sub(1);
            return Ok(true);
        }
        if app.ui.keybinds.matches("ui", "CREDITS_SCROLL_DOWN", &key)
            || app.ui.keybinds.menu_down(&key)
        {
            app.panels.credits.scroll =
                (app.panels.credits.scroll.saturating_add(1)).min(max_scroll);
            return Ok(true);
        }
    }

    // Help scroll
    if app.ui.current_tab == MenuTab::Help {
        let max_scroll = get_max_help_scroll(app);
        if app.ui.keybinds.menu_up(&key) {
            app.panels.help.scroll = app.panels.help.scroll.saturating_sub(1);
            return Ok(true);
        }
        if app.ui.keybinds.menu_down(&key) {
            app.panels.help.scroll = (app.panels.help.scroll.saturating_add(1)).min(max_scroll);
            return Ok(true);
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

    // Save/load the current world (only in Menu tab)
    let save = app.ui.keybinds.matches("ui", "SAVE_JSON", &key);
    let load = app.ui.keybinds.matches("ui", "LOAD_JSON", &key);
    let quit_to_title = app
        .ui
        .keybinds
        .matches("ui", "SAVE_AND_QUIT_TO_TITLE", &key);
    if (save || load || quit_to_title) && app.ui.current_tab != crate::MenuTab::Menu {
        if save {
            app.core.game.res.log("Save only available in Menu tab");
            return Ok(true);
        }
        if load {
            app.core.game.res.log("Load only available in Menu tab");
            return Ok(true);
        }
        return Ok(false);
    }
    if save {
        match app.save_world() {
            Ok(()) => app.core.game.res.log_green("World saved"),
            Err(e) => app.core.game.res.log_red(format!("Save failed: {e:#}")),
        }
        return Ok(true);
    }
    if load {
        let folder = app.current_world.as_ref().and_then(|w| w.folder.clone());
        match folder {
            None => app.core.game.res.log("This world hasn't been saved yet"),
            Some(folder) => match app.load_world(folder) {
                Ok(()) => app.core.game.res.log_green("World reloaded from last save"),
                Err(e) => app.core.game.res.log_red(format!("Load failed: {e:#}")),
            },
        }
        return Ok(true);
    }
    if quit_to_title {
        if let Err(e) = app.save_and_quit_to_title() {
            app.core.game.res.log_red(format!("Save failed: {e:#}"));
        }
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

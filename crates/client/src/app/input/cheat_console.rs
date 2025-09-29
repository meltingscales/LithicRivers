use crate::app_state::CheatConsoleState;
use crate::App;
use crossterm::event::KeyCode;
use lithicrivers_core::components::Position;
use std::collections::HashMap;
use std::error::Error;

/// Command descriptor with name, description, and handler
pub struct Command {
    pub name: &'static str,
    pub description: &'static str,
    pub handler: fn(&mut App, &[&str]) -> Result<(), Box<dyn Error>>,
}

/// Registry of all available commands
pub struct CommandRegistry {
    commands: HashMap<&'static str, Command>,
}

impl CommandRegistry {
    /// Create a new command registry with all available commands
    pub fn new() -> Self {
        let mut commands = HashMap::new();

        // Register all commands
        let cmd_list = [
            Command {
                name: "tp",
                description: "tp x y z - Teleport to coordinates",
                handler: handle_tp_command,
            },
            Command {
                name: "noclip_toggle",
                description: "noclip_toggle - Toggle noclip mode",
                handler: handle_noclip_toggle_command,
            },
            Command {
                name: "toggle_fogofwar",
                description: "toggle_fogofwar - Toggle fog of war rendering",
                handler: handle_toggle_fogofwar_command,
            },
            Command {
                name: "toggle_tutorial",
                description: "toggle_tutorial - Toggle tutorial panel visibility",
                handler: handle_toggle_tutorial_command,
            },
            Command {
                name: "giveitem",
                description:
                    "giveitem <item_sprite_name> <quantity> - Give items to player inventory",
                handler: handle_giveitem_command,
            },
        ];

        for cmd in cmd_list.iter() {
            commands.insert(
                cmd.name,
                Command {
                    name: cmd.name,
                    description: cmd.description,
                    handler: cmd.handler,
                },
            );
        }

        Self { commands }
    }

    /// Get commands that match a prefix for autocomplete
    pub fn get_matching_commands(&self, prefix: &str) -> Vec<&'static str> {
        let mut matches: Vec<_> = self
            .commands
            .keys()
            .filter(|name| name.starts_with(prefix))
            .copied()
            .collect();
        matches.sort();
        matches
    }

    /// Get command descriptions for help
    pub fn get_command_descriptions(&self) -> Vec<&'static str> {
        let mut descriptions: Vec<_> = self.commands.values().map(|cmd| cmd.description).collect();
        descriptions.sort();
        descriptions
    }

    /// Execute a command
    pub fn execute(&self, app: &mut App, command: &str) -> Result<(), Box<dyn Error>> {
        let parts: Vec<&str> = command.trim().split_whitespace().collect();
        if parts.is_empty() {
            return Ok(());
        }

        if let Some(cmd) = self.commands.get(parts[0]) {
            (cmd.handler)(app, &parts)
        } else {
            // Unknown command - could add error message system later
            Ok(())
        }
    }
}

/// Get the global command registry
pub fn get_command_registry() -> CommandRegistry {
    CommandRegistry::new()
}

/// Find the longest common prefix among a list of strings
fn find_common_prefix(strings: &[String]) -> String {
    if strings.is_empty() {
        return String::new();
    }

    if strings.len() == 1 {
        return strings[0].clone();
    }

    let first = &strings[0];
    let mut common_len = 0;

    for (i, ch) in first.chars().enumerate() {
        if strings.iter().all(|s| s.chars().nth(i) == Some(ch)) {
            common_len = i + 1;
        } else {
            break;
        }
    }

    first.chars().take(common_len).collect()
}

/// Update autocomplete suggestions based on current input
fn update_autocomplete(input: &str) -> (Vec<String>, Option<usize>) {
    if input.is_empty() {
        return (Vec::new(), None);
    }

    let registry = get_command_registry();
    let matches: Vec<String> = registry
        .get_matching_commands(input)
        .into_iter()
        .map(|s| s.to_string())
        .collect();

    (matches, None)
}

// Individual command handlers
fn handle_tp_command(app: &mut App, parts: &[&str]) -> Result<(), Box<dyn Error>> {
    if parts.len() == 4 {
        if let (Ok(x), Ok(y), Ok(z)) = (
            parts[1].parse::<i64>(),
            parts[2].parse::<i64>(),
            parts[3].parse::<i64>(),
        ) {
            let new_position = Position { x, y, z };
            if let Some(player_entity) = app.core.game.get_player_entity() {
                if let Ok(mut pos) = app.core.game.world.get::<&mut Position>(player_entity) {
                    *pos = new_position;
                    // Update UI viewport to follow player
                    app.ui.view_x = x;
                    app.ui.view_y = y;
                    app.ui.view_z = z;
                    // Update look cursor to new position
                    app.panels.look.cursor = new_position;
                }
            }
        }
    }
    Ok(())
}

fn handle_noclip_toggle_command(app: &mut App, _parts: &[&str]) -> Result<(), Box<dyn Error>> {
    app.core.game.res.player_state.noclip_enabled = !app.core.game.res.player_state.noclip_enabled;
    let state = if app.core.game.res.player_state.noclip_enabled {
        "enabled"
    } else {
        "disabled"
    };
    app.core.game.res.log(format!("Noclip mode: {}", state));
    Ok(())
}

fn handle_toggle_fogofwar_command(app: &mut App, _parts: &[&str]) -> Result<(), Box<dyn Error>> {
    app.core.game.res.player_state.fog_of_war_enabled =
        !app.core.game.res.player_state.fog_of_war_enabled;
    let state = if app.core.game.res.player_state.fog_of_war_enabled {
        "enabled"
    } else {
        "disabled"
    };
    app.core.game.res.log(format!("Fog of war: {}", state));
    Ok(())
}

fn handle_toggle_tutorial_command(app: &mut App, _parts: &[&str]) -> Result<(), Box<dyn Error>> {
    use crate::MenuTab;

    // Toggle tutorial visibility
    app.ui.tutorial_visible = !app.ui.tutorial_visible;

    if app.ui.tutorial_visible {
        // Tutorial is now visible - switch to Tutorial tab
        app.ui.current_tab = MenuTab::Tutorial;
        app.core.game.res.log("Tutorial system enabled");
    } else {
        // Tutorial is now hidden - switch away from Tutorial tab if currently on it
        if app.ui.current_tab == MenuTab::Tutorial {
            app.ui.current_tab = MenuTab::World;
        }
        app.core.game.res.log("Tutorial system disabled");
    }
    Ok(())
}

fn handle_giveitem_command(app: &mut App, parts: &[&str]) -> Result<(), Box<dyn Error>> {
    if parts.len() < 3 {
        app.core
            .game
            .res
            .log("Usage: giveitem <item_sprite_name> <quantity>".to_string());
        let available_items = lithicrivers_core::components::get_all_item_sprite_names().join(", ");
        app.core
            .game
            .res
            .log(format!("Available items: {}", available_items));
        return Ok(());
    }

    // Parse quantity
    let quantity = match parts[2].parse::<u32>() {
        Ok(q) if q > 0 => q,
        _ => {
            app.core
                .game
                .res
                .log("Error: Quantity must be a positive number".to_string());
            return Ok(());
        }
    };

    // Parse item using sprite name
    let item_kind = match lithicrivers_core::components::parse_itemkind_from_sprite_name(parts[1]) {
        Some(kind) => kind,
        None => {
            app.core
                .game
                .res
                .log(format!("Error: Unknown item '{}'", parts[1]));
            let available_items =
                lithicrivers_core::components::get_all_item_sprite_names().join(", ");
            app.core
                .game
                .res
                .log(format!("Available items: {}", available_items));
            return Ok(());
        }
    };

    // Get player entity and add item to inventory
    if let Some(player_entity) = app.core.game.get_player_entity() {
        // First, add the item to inventory
        let add_success = if let Ok(mut inventory) =
            app.core
                .game
                .world
                .get::<&mut lithicrivers_core::components::Inventory>(player_entity)
        {
            inventory.add(item_kind, quantity);
            let item_display_name = lithicrivers_core::components::itemkind_name(item_kind);
            app.core.game.res.log(format!(
                "Added {} {} to inventory",
                quantity, item_display_name
            ));
            true
        } else {
            app.core
                .game
                .res
                .log("Error: Player has no inventory!".to_string());
            false
        };

        // Then, if successful, update quest objectives (separate borrow)
        if add_success {
            if let Ok(player_inventory) =
                app.core
                    .game
                    .world
                    .get::<&lithicrivers_core::components::Inventory>(player_entity)
            {
                app.core
                    .game
                    .res
                    .update_quest_objectives_from_inventory(&player_inventory);
            }
        }
    } else {
        app.core
            .game
            .res
            .log("Error: Player not found!".to_string());
    }

    Ok(())
}

pub fn handle_input(app: &mut App, key: KeyCode) -> Result<bool, Box<dyn Error>> {
    match &app.panels.cheat_console {
        CheatConsoleState::None => {
            // Check if we should open the console with OPEN_COMMAND_MENU keybind
            if app.ui.keybinds.matches("ui", "OPEN_COMMAND_MENU", &key) {
                app.panels.cheat_console = CheatConsoleState::Open {
                    input: String::new(),
                    cursor_position: 0,
                    autocomplete_suggestions: Vec::new(),
                    autocomplete_index: None,
                    scroll_offset: 0,
                };
                return Ok(true);
            }
        }
        CheatConsoleState::Open {
            input,
            cursor_position,
            autocomplete_suggestions,
            autocomplete_index,
            scroll_offset,
        } => {
            match key {
                // Close console with Escape
                KeyCode::Esc => {
                    app.panels.cheat_console = CheatConsoleState::None;
                    return Ok(true);
                }
                // Execute command with Enter
                KeyCode::Enter => {
                    let command = input.clone();
                    app.panels.cheat_console = CheatConsoleState::None;
                    execute_command(app, &command)?;
                    return Ok(true);
                }
                // Handle text input
                KeyCode::Char(c) => {
                    let mut new_input = input.clone();
                    new_input.insert(*cursor_position, c);
                    let new_cursor = cursor_position + 1;
                    let (suggestions, index) = update_autocomplete(&new_input);
                    app.panels.cheat_console = CheatConsoleState::Open {
                        input: new_input,
                        cursor_position: new_cursor,
                        autocomplete_suggestions: suggestions,
                        autocomplete_index: index,
                        scroll_offset: *scroll_offset,
                    };
                    return Ok(true);
                }
                // Handle backspace
                KeyCode::Backspace => {
                    if *cursor_position > 0 {
                        let mut new_input = input.clone();
                        new_input.remove(*cursor_position - 1);
                        let new_cursor = cursor_position - 1;
                        let (suggestions, index) = update_autocomplete(&new_input);
                        app.panels.cheat_console = CheatConsoleState::Open {
                            input: new_input,
                            cursor_position: new_cursor,
                            autocomplete_suggestions: suggestions,
                            autocomplete_index: index,
                            scroll_offset: *scroll_offset,
                        };
                    }
                    return Ok(true);
                }
                // Handle cursor movement
                KeyCode::Left => {
                    if *cursor_position > 0 {
                        app.panels.cheat_console = CheatConsoleState::Open {
                            input: input.clone(),
                            cursor_position: cursor_position - 1,
                            autocomplete_suggestions: autocomplete_suggestions.clone(),
                            autocomplete_index: *autocomplete_index,
                            scroll_offset: *scroll_offset,
                        };
                    }
                    return Ok(true);
                }
                KeyCode::Right => {
                    if *cursor_position < input.len() {
                        app.panels.cheat_console = CheatConsoleState::Open {
                            input: input.clone(),
                            cursor_position: cursor_position + 1,
                            autocomplete_suggestions: autocomplete_suggestions.clone(),
                            autocomplete_index: *autocomplete_index,
                            scroll_offset: *scroll_offset,
                        };
                    }
                    return Ok(true);
                }
                // Home/End keys
                KeyCode::Home => {
                    app.panels.cheat_console = CheatConsoleState::Open {
                        input: input.clone(),
                        cursor_position: 0,
                        autocomplete_suggestions: autocomplete_suggestions.clone(),
                        autocomplete_index: *autocomplete_index,
                        scroll_offset: *scroll_offset,
                    };
                    return Ok(true);
                }
                KeyCode::End => {
                    app.panels.cheat_console = CheatConsoleState::Open {
                        input: input.clone(),
                        cursor_position: input.len(),
                        autocomplete_suggestions: autocomplete_suggestions.clone(),
                        autocomplete_index: *autocomplete_index,
                        scroll_offset: *scroll_offset,
                    };
                    return Ok(true);
                }
                // Delete key
                KeyCode::Delete => {
                    if *cursor_position < input.len() {
                        let mut new_input = input.clone();
                        new_input.remove(*cursor_position);
                        let (suggestions, index) = update_autocomplete(&new_input);
                        app.panels.cheat_console = CheatConsoleState::Open {
                            input: new_input,
                            cursor_position: *cursor_position,
                            autocomplete_suggestions: suggestions,
                            autocomplete_index: index,
                            scroll_offset: *scroll_offset,
                        };
                    }
                    return Ok(true);
                }
                // Tab for autocomplete - bash-like completion
                KeyCode::Tab => {
                    if !autocomplete_suggestions.is_empty() {
                        if autocomplete_suggestions.len() == 1 {
                            // Single match - complete it fully
                            let completed = autocomplete_suggestions[0].clone();
                            let (new_suggestions, new_index) = update_autocomplete(&completed);
                            app.panels.cheat_console = CheatConsoleState::Open {
                                input: completed.clone(),
                                cursor_position: completed.len(),
                                autocomplete_suggestions: new_suggestions,
                                autocomplete_index: new_index,
                                scroll_offset: *scroll_offset,
                            };
                        } else {
                            // Multiple matches - bash-like behavior
                            let common_prefix = find_common_prefix(autocomplete_suggestions);

                            // If we can extend the input with the common prefix, do that first
                            if common_prefix.len() > input.len() {
                                let (new_suggestions, new_index) =
                                    update_autocomplete(&common_prefix);
                                app.panels.cheat_console = CheatConsoleState::Open {
                                    input: common_prefix.clone(),
                                    cursor_position: common_prefix.len(),
                                    autocomplete_suggestions: new_suggestions,
                                    autocomplete_index: new_index,
                                    scroll_offset: *scroll_offset,
                                };
                            } else {
                                // No more common prefix to complete - cycle through suggestions
                                let new_index = match autocomplete_index {
                                    Some(idx) => Some((idx + 1) % autocomplete_suggestions.len()),
                                    None => Some(0),
                                };
                                app.panels.cheat_console = CheatConsoleState::Open {
                                    input: input.clone(),
                                    cursor_position: *cursor_position,
                                    autocomplete_suggestions: autocomplete_suggestions.clone(),
                                    autocomplete_index: new_index,
                                    scroll_offset: *scroll_offset,
                                };
                            }
                        }
                    }
                    return Ok(true);
                }
                // Handle scrolling through command list
                key if app
                    .ui
                    .keybinds
                    .matches("ui", "CHEAT_CONSOLE_SCROLL_UP", &key) =>
                {
                    if input.is_empty() && autocomplete_suggestions.is_empty() {
                        // Only scroll when showing all commands (no input/autocomplete)
                        let new_scroll_offset = scroll_offset.saturating_sub(1);
                        app.panels.cheat_console = CheatConsoleState::Open {
                            input: input.clone(),
                            cursor_position: *cursor_position,
                            autocomplete_suggestions: autocomplete_suggestions.clone(),
                            autocomplete_index: *autocomplete_index,
                            scroll_offset: new_scroll_offset,
                        };
                    }
                    return Ok(true);
                }
                key if app
                    .ui
                    .keybinds
                    .matches("ui", "CHEAT_CONSOLE_SCROLL_DOWN", &key) =>
                {
                    if input.is_empty() && autocomplete_suggestions.is_empty() {
                        // Only scroll when showing all commands (no input/autocomplete)
                        let registry = get_command_registry();
                        let commands = registry.get_command_descriptions();
                        let max_scroll = commands.len().saturating_sub(5); // Show max 5 commands at a time
                        let new_scroll_offset = (*scroll_offset + 1).min(max_scroll);
                        app.panels.cheat_console = CheatConsoleState::Open {
                            input: input.clone(),
                            cursor_position: *cursor_position,
                            autocomplete_suggestions: autocomplete_suggestions.clone(),
                            autocomplete_index: *autocomplete_index,
                            scroll_offset: new_scroll_offset,
                        };
                    }
                    return Ok(true);
                }
                _ => return Ok(true), // Consume all other input while console is open
            }
        }
    }

    Ok(false) // Didn't handle input
}

fn execute_command(app: &mut App, command: &str) -> Result<(), Box<dyn Error>> {
    let registry = get_command_registry();
    registry.execute(app, command)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_common_prefix() {
        // Test empty list
        assert_eq!(find_common_prefix(&[]), "");

        // Test single item
        assert_eq!(find_common_prefix(&["test".to_string()]), "test");

        // Test multiple items with common prefix
        let items = vec!["toggle_fogofwar".to_string(), "toggle_tutorial".to_string()];
        assert_eq!(find_common_prefix(&items), "toggle_");

        // Test multiple items with no common prefix
        let items = vec!["tp".to_string(), "noclip_toggle".to_string()];
        assert_eq!(find_common_prefix(&items), "");

        // Test items where one is a prefix of another
        let items = vec!["test".to_string(), "testing".to_string()];
        assert_eq!(find_common_prefix(&items), "test");
    }
}

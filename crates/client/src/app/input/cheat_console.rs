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
                name: "fogofwar_toggle",
                description: "fogofwar_toggle - Toggle fog of war rendering",
                handler: handle_fogofwar_toggle_command,
            },
            Command {
                name: "toggle_tutorial",
                description: "toggle_tutorial - Toggle tutorial panel visibility",
                handler: handle_toggle_tutorial_command,
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

fn handle_fogofwar_toggle_command(app: &mut App, _parts: &[&str]) -> Result<(), Box<dyn Error>> {
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
                };
                return Ok(true);
            }
        }
        CheatConsoleState::Open {
            input,
            cursor_position,
            autocomplete_suggestions,
            autocomplete_index,
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
                    };
                    return Ok(true);
                }
                KeyCode::End => {
                    app.panels.cheat_console = CheatConsoleState::Open {
                        input: input.clone(),
                        cursor_position: input.len(),
                        autocomplete_suggestions: autocomplete_suggestions.clone(),
                        autocomplete_index: *autocomplete_index,
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
                        };
                    }
                    return Ok(true);
                }
                // Tab for autocomplete
                KeyCode::Tab => {
                    if !autocomplete_suggestions.is_empty() {
                        // If there's only one suggestion, complete it
                        if autocomplete_suggestions.len() == 1 {
                            let completed = autocomplete_suggestions[0].clone();
                            let (new_suggestions, new_index) = update_autocomplete(&completed);
                            app.panels.cheat_console = CheatConsoleState::Open {
                                input: completed.clone(),
                                cursor_position: completed.len(),
                                autocomplete_suggestions: new_suggestions,
                                autocomplete_index: new_index,
                            };
                        } else {
                            // Cycle through suggestions
                            let new_index = match autocomplete_index {
                                Some(idx) => Some((idx + 1) % autocomplete_suggestions.len()),
                                None => Some(0),
                            };
                            app.panels.cheat_console = CheatConsoleState::Open {
                                input: input.clone(),
                                cursor_position: *cursor_position,
                                autocomplete_suggestions: autocomplete_suggestions.clone(),
                                autocomplete_index: new_index,
                            };
                        }
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

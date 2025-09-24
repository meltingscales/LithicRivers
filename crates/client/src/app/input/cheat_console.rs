use crate::app_state::CheatConsoleState;
use crate::App;
use crossterm::event::KeyCode;
use lithicrivers_core::components::Position;
use std::error::Error;

pub fn handle_input(app: &mut App, key: KeyCode) -> Result<bool, Box<dyn Error>> {
    match &app.panels.cheat_console {
        CheatConsoleState::None => {
            // Check if we should open the console with "/"
            if key == KeyCode::Char('/') {
                app.panels.cheat_console = CheatConsoleState::Open {
                    input: String::new(),
                    cursor_position: 0,
                };
                return Ok(true);
            }
        }
        CheatConsoleState::Open {
            input,
            cursor_position,
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
                    app.panels.cheat_console = CheatConsoleState::Open {
                        input: new_input,
                        cursor_position: new_cursor,
                    };
                    return Ok(true);
                }
                // Handle backspace
                KeyCode::Backspace => {
                    if *cursor_position > 0 {
                        let mut new_input = input.clone();
                        new_input.remove(*cursor_position - 1);
                        let new_cursor = cursor_position - 1;
                        app.panels.cheat_console = CheatConsoleState::Open {
                            input: new_input,
                            cursor_position: new_cursor,
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
                        };
                    }
                    return Ok(true);
                }
                KeyCode::Right => {
                    if *cursor_position < input.len() {
                        app.panels.cheat_console = CheatConsoleState::Open {
                            input: input.clone(),
                            cursor_position: cursor_position + 1,
                        };
                    }
                    return Ok(true);
                }
                // Home/End keys
                KeyCode::Home => {
                    app.panels.cheat_console = CheatConsoleState::Open {
                        input: input.clone(),
                        cursor_position: 0,
                    };
                    return Ok(true);
                }
                KeyCode::End => {
                    app.panels.cheat_console = CheatConsoleState::Open {
                        input: input.clone(),
                        cursor_position: input.len(),
                    };
                    return Ok(true);
                }
                // Delete key
                KeyCode::Delete => {
                    if *cursor_position < input.len() {
                        let mut new_input = input.clone();
                        new_input.remove(*cursor_position);
                        app.panels.cheat_console = CheatConsoleState::Open {
                            input: new_input,
                            cursor_position: *cursor_position,
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
    let parts: Vec<&str> = command.trim().split_whitespace().collect();
    if parts.is_empty() {
        return Ok(());
    }

    match parts[0] {
        "tp" => {
            if parts.len() == 4 {
                if let (Ok(x), Ok(y), Ok(z)) = (
                    parts[1].parse::<i64>(),
                    parts[2].parse::<i64>(),
                    parts[3].parse::<i64>(),
                ) {
                    let new_position = Position { x, y, z };
                    if let Some(player_entity) = app.core.game.get_player_entity() {
                        if let Ok(mut pos) = app.core.game.world.get::<&mut Position>(player_entity)
                        {
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
        }
        "noclip_toggle" => {
            // Toggle noclip mode
            app.core.game.res.player_state.noclip_enabled =
                !app.core.game.res.player_state.noclip_enabled;
            let state = if app.core.game.res.player_state.noclip_enabled {
                "enabled"
            } else {
                "disabled"
            };
            app.core.game.res.log(format!("Noclip mode: {}", state));
        }
        _ => {
            // Unknown command - could add error message system later
        }
    }

    Ok(())
}

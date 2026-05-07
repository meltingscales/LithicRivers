use crate::{App, SplashState};
use crossterm::event::KeyCode;
use std::{error::Error, time::Instant};

/// Handle splash screen input - returns true if input was handled
pub fn handle_splash_input(app: &mut App, _key: KeyCode) -> Result<bool, Box<dyn Error>> {
    // Handle splash screen skipping first
    if let Some(_start_time) = app.splash.start_time {
        match app.splash.state {
            SplashState::Logo => {
                // Any key skips to next screen
                app.splash.state = SplashState::GameTitle;
                app.splash.start_time = Some(Instant::now());
                return Ok(true);
            }
            SplashState::GameTitle => {
                // Any key skips to boot message
                app.splash.state = SplashState::BootMessage;
                app.splash.start_time = Some(Instant::now());
                app.splash.boot_display_text.clear();
                app.splash.boot_line_index = 0;
                app.splash.last_line_time = Instant::now();
                app.splash.boot_complete = false;
                return Ok(true);
            }
            SplashState::BootMessage => {
                if !app.splash.boot_complete {
                    // Skip to end of text
                    let full_text = app.splash.boot_message_lines.join("\n");
                    app.splash.boot_display_text = full_text.clone();
                    app.splash.boot_line_index = full_text.len();
                    app.splash.boot_complete = true;
                    app.splash.start_time = Some(Instant::now());
                } else {
                    // Move to main UI if already complete
                    app.splash.state = SplashState::MainUI;
                }
                return Ok(true);
            }
            _ => {
                // Not handling other splash states
                return Ok(false);
            }
        }
    }

    // No splash screen active
    Ok(false)
}

//! Input handling and key bindings

use crossterm::event::KeyCode;

/// Format a KeyCode as a human-readable string
pub fn format_keycode(kc: &KeyCode) -> String {
    match kc {
        KeyCode::Char(c) => c.to_string(),
        KeyCode::BackTab => "Shift+Tab".to_string(),
        KeyCode::Backspace => "Backspace".to_string(),
        KeyCode::Tab => "Tab".to_string(),
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Esc => "Esc".to_string(),
        _ => format!("{:?}", kc),
    }
}

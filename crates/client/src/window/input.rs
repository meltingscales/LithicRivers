//! Turns window events into the input the game already takes from the terminal:
//! crossterm key codes, and mouse events at screen cells.

use crate::app::input::mouse::{MouseButton, MouseInput, MouseKind};
use crossterm::event::KeyCode;
use winit::keyboard::{Key, NamedKey};

/// The key code a terminal would report for a key press, or `None` for keys the game
/// never sees (modifiers alone, media keys).
pub fn key_code(key: &Key, text: Option<&str>, shift: bool) -> Option<KeyCode> {
    let code = match key {
        Key::Named(named) => match named {
            NamedKey::Enter => KeyCode::Enter,
            NamedKey::Escape => KeyCode::Esc,
            NamedKey::Tab if shift => KeyCode::BackTab,
            NamedKey::Tab => KeyCode::Tab,
            NamedKey::Backspace => KeyCode::Backspace,
            NamedKey::Delete => KeyCode::Delete,
            NamedKey::Insert => KeyCode::Insert,
            NamedKey::Home => KeyCode::Home,
            NamedKey::End => KeyCode::End,
            NamedKey::PageUp => KeyCode::PageUp,
            NamedKey::PageDown => KeyCode::PageDown,
            NamedKey::ArrowUp => KeyCode::Up,
            NamedKey::ArrowDown => KeyCode::Down,
            NamedKey::ArrowLeft => KeyCode::Left,
            NamedKey::ArrowRight => KeyCode::Right,
            NamedKey::Space => KeyCode::Char(' '),
            NamedKey::F1 => KeyCode::F(1),
            NamedKey::F2 => KeyCode::F(2),
            NamedKey::F3 => KeyCode::F(3),
            NamedKey::F4 => KeyCode::F(4),
            NamedKey::F5 => KeyCode::F(5),
            NamedKey::F6 => KeyCode::F(6),
            NamedKey::F7 => KeyCode::F(7),
            NamedKey::F8 => KeyCode::F(8),
            NamedKey::F9 => KeyCode::F(9),
            NamedKey::F10 => KeyCode::F(10),
            NamedKey::F11 => KeyCode::F(11),
            NamedKey::F12 => KeyCode::F(12),
            _ => return None,
        },
        // The typed text already has Shift applied, as a terminal's would
        Key::Character(chars) => {
            let typed = text.unwrap_or(chars.as_str());
            let mut it = typed.chars();
            match (it.next(), it.next()) {
                (Some(c), None) if !c.is_control() => KeyCode::Char(c),
                _ => return None,
            }
        }
        _ => return None,
    };
    Some(code)
}

/// The screen cell under a pointer position in pixels.
pub fn cell_at(x: f64, y: f64, cell: (u32, u32)) -> (u16, u16) {
    let col = (x.max(0.0) / cell.0 as f64) as u64;
    let row = (y.max(0.0) / cell.1 as f64) as u64;
    (
        col.min(u16::MAX as u64) as u16,
        row.min(u16::MAX as u64) as u16,
    )
}

/// A mouse button press at a cell.
pub fn click(
    button: winit::event::MouseButton,
    cell: (u16, u16),
    shift: bool,
) -> Option<MouseInput> {
    let button = match button {
        winit::event::MouseButton::Left => MouseButton::Left,
        winit::event::MouseButton::Right => MouseButton::Right,
        winit::event::MouseButton::Middle => MouseButton::Middle,
        _ => return None,
    };
    Some(at(MouseKind::Click(button), cell, shift))
}

/// Wheel steps (positive is away from the user, up) as scroll events at a cell, one per
/// line, like a terminal reports them.
pub fn scrolls(lines: f32, cell: (u16, u16), shift: bool) -> Vec<MouseInput> {
    let kind = if lines > 0.0 {
        MouseKind::ScrollUp
    } else {
        MouseKind::ScrollDown
    };
    let steps = lines
        .abs()
        .round()
        .max(if lines == 0.0 { 0.0 } else { 1.0 }) as usize;
    vec![at(kind, cell, shift); steps]
}

/// The pointer moving onto a cell.
pub fn moved(cell: (u16, u16), shift: bool) -> MouseInput {
    at(MouseKind::Move, cell, shift)
}

fn at(kind: MouseKind, (col, row): (u16, u16), shift: bool) -> MouseInput {
    MouseInput {
        kind,
        col,
        row,
        shift,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chr(s: &str) -> Key {
        Key::Character(s.into())
    }

    #[test]
    fn named_keys_match_the_terminal() {
        let cases = [
            (NamedKey::Enter, KeyCode::Enter),
            (NamedKey::Escape, KeyCode::Esc),
            (NamedKey::Tab, KeyCode::Tab),
            (NamedKey::Backspace, KeyCode::Backspace),
            (NamedKey::ArrowLeft, KeyCode::Left),
            (NamedKey::PageDown, KeyCode::PageDown),
            (NamedKey::Space, KeyCode::Char(' ')),
            (NamedKey::F1, KeyCode::F(1)),
            (NamedKey::F12, KeyCode::F(12)),
        ];
        for (named, code) in cases {
            assert_eq!(key_code(&Key::Named(named), None, false), Some(code));
        }
        assert_eq!(
            key_code(&Key::Named(NamedKey::Tab), None, true),
            Some(KeyCode::BackTab)
        );
        assert_eq!(key_code(&Key::Named(NamedKey::Shift), None, true), None);
    }

    #[test]
    fn characters_use_the_typed_text() {
        assert_eq!(
            key_code(&chr("v"), Some("v"), false),
            Some(KeyCode::Char('v'))
        );
        // Shift is already applied to the typed text
        assert_eq!(
            key_code(&chr("v"), Some("V"), true),
            Some(KeyCode::Char('V'))
        );
        assert_eq!(
            key_code(&chr(","), Some("<"), true),
            Some(KeyCode::Char('<'))
        );
        // Numpad digits with NumLock on, as the terminal reports them
        assert_eq!(
            key_code(&chr("8"), Some("8"), false),
            Some(KeyCode::Char('8'))
        );
        // No text (Ctrl held): fall back to the key itself
        assert_eq!(key_code(&chr("q"), None, false), Some(KeyCode::Char('q')));
        assert_eq!(key_code(&chr("ab"), Some("ab"), false), None);
    }

    #[test]
    fn pixels_to_cells() {
        assert_eq!(cell_at(0.0, 0.0, (10, 20)), (0, 0));
        assert_eq!(cell_at(19.9, 39.9, (10, 20)), (1, 1));
        assert_eq!(cell_at(25.0, 61.0, (10, 20)), (2, 3));
        assert_eq!(cell_at(-5.0, -1.0, (10, 20)), (0, 0));
    }

    #[test]
    fn mouse_events_at_cells() {
        let left = click(winit::event::MouseButton::Left, (3, 4), true).unwrap();
        assert_eq!(left.kind, MouseKind::Click(MouseButton::Left));
        assert_eq!((left.col, left.row, left.shift), (3, 4, true));
        assert!(click(winit::event::MouseButton::Back, (0, 0), false).is_none());

        let up = scrolls(2.0, (1, 1), false);
        assert_eq!(up.len(), 2);
        assert!(up.iter().all(|m| m.kind == MouseKind::ScrollUp));
        let down = scrolls(-0.4, (1, 1), true);
        assert_eq!(down.len(), 1, "a small wheel step still scrolls once");
        assert_eq!(down[0].kind, MouseKind::ScrollDown);
        assert!(scrolls(0.0, (1, 1), false).is_empty());

        assert_eq!(moved((5, 6), false).kind, MouseKind::Move);
    }
}

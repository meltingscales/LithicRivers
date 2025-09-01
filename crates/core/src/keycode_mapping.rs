use crossterm::event::KeyCode;

pub fn parse_keycode(s: &str) -> Option<KeyCode> {
    match s.to_uppercase().as_str() {
        "UP" => Some(KeyCode::Up),
        "DOWN" => Some(KeyCode::Down),
        "LEFT" => Some(KeyCode::Left),
        "RIGHT" => Some(KeyCode::Right),
        "ESCAPE" => Some(KeyCode::Esc),
        "ENTER" => Some(KeyCode::Enter),
        "SPACE" => Some(KeyCode::Char(' ')),
        "TAB" => Some(KeyCode::Tab),
        "BACKSPACE" => Some(KeyCode::Backspace),
        "DELETE" => Some(KeyCode::Delete),
        "INSERT" => Some(KeyCode::Insert),
        "PAGEUP" => Some(KeyCode::PageUp),
        "PAGEDOWN" => Some(KeyCode::PageDown),
        "HOME" => Some(KeyCode::Home),
        "END" => Some(KeyCode::End),
        "NUMLOCK" => Some(KeyCode::NumLock),
        "SCROLLLOCK" => Some(KeyCode::ScrollLock),
        "CAPSLOCK" => Some(KeyCode::CapsLock),
        "PAUSE" => Some(KeyCode::Pause),
        "F1" => Some(KeyCode::F(1)),
        "F2" => Some(KeyCode::F(2)),
        "F3" => Some(KeyCode::F(3)),
        "F4" => Some(KeyCode::F(4)),
        "F5" => Some(KeyCode::F(5)),
        "F6" => Some(KeyCode::F(6)),
        "F7" => Some(KeyCode::F(7)),
        "F8" => Some(KeyCode::F(8)),
        "F9" => Some(KeyCode::F(9)),
        "F10" => Some(KeyCode::F(10)),
        "F11" => Some(KeyCode::F(11)),
        "F12" => Some(KeyCode::F(12)),
        "NUMPAD_0" => Some(KeyCode::Char('0')),
        "NUMPAD_1" => Some(KeyCode::Char('1')),
        "NUMPAD_2" => Some(KeyCode::Char('2')),
        "NUMPAD_3" => Some(KeyCode::Char('3')),
        "NUMPAD_4" => Some(KeyCode::Char('4')),
        "NUMPAD_5" => Some(KeyCode::Char('5')),
        "NUMPAD_6" => Some(KeyCode::Char('6')),
        "NUMPAD_7" => Some(KeyCode::Char('7')),
        "NUMPAD_8" => Some(KeyCode::Char('8')),
        "NUMPAD_9" => Some(KeyCode::Char('9')),
        "MULTIPLY" => Some(KeyCode::Char('*')),
        "ADD" => Some(KeyCode::Char('+')),
        "SEPARATOR" => Some(KeyCode::Char('|')),
        "SUBTRACT" => Some(KeyCode::Char('-')),
        "DECIMAL" => Some(KeyCode::Char('.')),
        "DIVIDE" => Some(KeyCode::Char('/')),
        _ => {
            if s.len() == 1 {
                Some(KeyCode::Char(s.chars().next().unwrap()))
            } else {
                panic!("Invalid keycode: {s:?}");
            }
        }
    }
}

pub fn keycode_to_printable_name(keycode: KeyCode) -> &'static str {
    match keycode {
        KeyCode::Up => "UP",
        KeyCode::Down => "DOWN",
        KeyCode::Left => "LEFT",
        KeyCode::Right => "RIGHT",
        KeyCode::Esc => "ESCAPE",
        KeyCode::Enter => "ENTER",
        KeyCode::Char('0') => "0",
        KeyCode::Char('1') => "1",
        KeyCode::Char('2') => "2",
        KeyCode::Char('3') => "3",
        KeyCode::Char('4') => "4",
        KeyCode::Char('5') => "5",
        KeyCode::Char('6') => "6",
        KeyCode::Char('7') => "7",
        KeyCode::Char('8') => "8",
        KeyCode::Char('9') => "9",
        KeyCode::Char(c) => match c {
            'a'..='z'
            | 'A'..='Z'
            | '0'..='9'
            | ' '
            | '!'
            | '@'
            | '#'
            | '$'
            | '%'
            | '^'
            | '&'
            | '*'
            | '('
            | ')'
            | '-'
            | '_'
            | '='
            | '+'
            | '['
            | ']'
            | '{'
            | '}'
            | ';'
            | ':'
            | '\''
            | '"'
            | ','
            | '<'
            | '.'
            | '>'
            | '/'
            | '?'
            | '`'
            | '~'
            | '|'
            | '\\' => {
                // For printable ASCII characters, return a static string slice
                // This is safe because we're only matching on a small set of characters
                let s = c.to_string();
                Box::leak(s.into_boxed_str())
            }
            _ => panic!("Invalid character {c:?}"),
        },
        _ => panic!("Invalid keycode {keycode:?}"),
    }
}

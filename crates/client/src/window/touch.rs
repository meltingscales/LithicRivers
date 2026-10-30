//! On-screen controls and touch input for phones and tablets (`touch-controls` spec).
//! Desktop builds never use any of this: a keyboard is assumed there.
//!
//! The pad floats over the bottom of the game, see-through: movement on the left, a few
//! action buttons on the right. Each button sends the key bound to an action, so rebinding
//! works and every screen behaves as with a keyboard. Touches elsewhere are taps (a left
//! click) or drags (scroll steps).

use super::input;
use crate::app::client_app::App;
use crate::app::input::mouse::MouseInput;
use crossterm::event::KeyCode;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Paragraph, Widget};

/// One button's size in cells: big enough for a thumb at the default text size.
pub const BUTTON_COLS: u16 = 7;
pub const BUTTON_ROWS: u16 = 3;
/// The pad's height: three rows of buttons.
pub const PAD_ROWS: u16 = BUTTON_ROWS * 3;
/// The shortest screen that shows the pad: it never covers more than half.
const MIN_SCREEN_ROWS: u16 = PAD_ROWS * 2;
/// The narrowest pad: movement and actions, each three buttons wide, and a gap.
const MIN_PAD_COLS: u16 = BUTTON_COLS * 6 + 2;

/// What a button sends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Press {
    /// The first key bound to `category.ACTION`
    Keybind(&'static str, &'static str),
    Enter,
    Escape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PadButton {
    pub rect: Rect,
    /// What the button shows
    pub name: &'static str,
    pub press: Press,
}

const MOVEMENT: [[(&str, &str); 3]; 3] = [
    [
        ("↖", "MOVE_NORTHWEST"),
        ("↑", "MOVE_NORTH"),
        ("↗", "MOVE_NORTHEAST"),
    ],
    [("←", "MOVE_WEST"), ("·", "WAIT"), ("→", "MOVE_EAST")],
    [
        ("↙", "MOVE_SOUTHWEST"),
        ("↓", "MOVE_SOUTH"),
        ("↘", "MOVE_SOUTHEAST"),
    ],
];

const ACTIONS: [[(&str, Press); 3]; 2] = [
    [
        ("Use", Press::Keybind("action", "INTERACT")),
        ("Mine", Press::Keybind("action", "MINE")),
        ("Build", Press::Keybind("build", "TOGGLE_BREAK_PLACE_MODE")),
    ],
    [
        ("<Tab", Press::Keybind("ui", "MENU_PREV")),
        ("Tab>", Press::Keybind("ui", "MENU_NEXT")),
        ("Esc", Press::Escape),
    ],
];

/// Where the pad floats over a screen: its bottom rows, or `None` if the screen is too
/// small to keep most of the game visible.
pub fn pad_area(area: Rect) -> Option<Rect> {
    (area.height >= MIN_SCREEN_ROWS && area.width >= MIN_PAD_COLS)
        .then(|| Rect::new(area.x, area.bottom() - PAD_ROWS, area.width, PAD_ROWS))
}

/// The buttons in a pad area: the movement grid at the left edge, the actions at the
/// right edge in the bottom two rows.
pub fn layout(pad: Rect) -> Vec<PadButton> {
    let button = |x: u16, y: u16| Rect::new(x, y, BUTTON_COLS, BUTTON_ROWS);
    let mut buttons = Vec::new();
    for (r, row) in MOVEMENT.iter().enumerate() {
        for (c, &(name, action)) in row.iter().enumerate() {
            buttons.push(PadButton {
                rect: button(
                    pad.x + c as u16 * BUTTON_COLS,
                    pad.y + r as u16 * BUTTON_ROWS,
                ),
                name,
                press: Press::Keybind("movement", action),
            });
        }
    }
    let actions_x = pad.right() - BUTTON_COLS * 3;
    for (r, row) in ACTIONS.iter().enumerate() {
        for (c, &(name, press)) in row.iter().enumerate() {
            buttons.push(PadButton {
                rect: button(
                    actions_x + c as u16 * BUTTON_COLS,
                    pad.y + (r as u16 + 1) * BUTTON_ROWS,
                ),
                name,
                press,
            });
        }
    }
    // Accept the selected row, in any menu
    buttons.push(PadButton {
        rect: button(actions_x + BUTTON_COLS * 2, pad.y),
        name: "Enter",
        press: Press::Enter,
    });
    buttons
}

/// The button at a screen cell, if any.
pub fn button_at(buttons: &[PadButton], (col, row): (u16, u16)) -> Option<Press> {
    buttons
        .iter()
        .find(|b| b.rect.contains((col, row).into()))
        .map(|b| b.press)
}

/// The key a button sends, as the keybinds stand now.
pub fn key_code(app: &App, press: Press) -> Option<KeyCode> {
    match press {
        Press::Enter => Some(KeyCode::Enter),
        Press::Escape => Some(KeyCode::Esc),
        Press::Keybind(category, action) => app
            .ui
            .keybinds
            .get_map()
            .get(&format!("{category}:{action}"))
            .and_then(|codes| codes.first().copied()),
    }
}

/// A key as a button shows it, at most five characters.
fn key_label(code: KeyCode) -> String {
    let label = match code {
        KeyCode::Char(' ') => "Spc".to_string(),
        KeyCode::Char(c) => c.to_string(),
        KeyCode::Left => "←".to_string(),
        KeyCode::Right => "→".to_string(),
        KeyCode::Up => "↑".to_string(),
        KeyCode::Down => "↓".to_string(),
        KeyCode::Enter => "Ent".to_string(),
        KeyCode::Esc => "Esc".to_string(),
        KeyCode::Tab => "Tab".to_string(),
        KeyCode::F(n) => format!("F{n}"),
        other => format!("{other:?}"),
    };
    label.chars().take(5).collect()
}

/// Draw the pad's buttons over the game: each a tinted patch, a cell smaller than its
/// touch area so neighbors stay apart, with the name on top and the bound key in the
/// corner. Everywhere else the game shows through, dimmed.
pub fn render(buttons: &[PadButton], buf: &mut Buffer, app: &App) {
    let tint = Color::Rgb(45, 45, 45);
    let name = Style::default()
        .fg(Color::Rgb(255, 210, 120))
        .add_modifier(Modifier::BOLD)
        .remove_modifier(Modifier::DIM);
    let key = Style::default()
        .fg(Color::Rgb(150, 150, 150))
        .remove_modifier(Modifier::DIM);
    for button in buttons {
        let patch = Rect {
            width: button.rect.width - 1,
            height: button.rect.height - 1,
            ..button.rect
        };
        for y in patch.top()..patch.bottom() {
            for x in patch.left()..patch.right() {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.bg = tint;
                    cell.modifier.insert(Modifier::DIM);
                }
            }
        }
        let row = |offset: u16| Rect {
            y: patch.y + offset,
            height: 1,
            ..patch
        };
        Paragraph::new(Span::styled(button.name, name))
            .alignment(Alignment::Center)
            .render(row(0), buf);
        if let (Press::Keybind(..), Some(code)) = (button.press, key_code(app, button.press)) {
            Paragraph::new(Span::styled(key_label(code), key))
                .alignment(Alignment::Right)
                .render(row(1), buf);
        }
    }
}

/// One finger on the screen outside the pad.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Finger {
    id: u64,
    start: (u16, u16),
    /// The row last turned into scroll steps
    last_row: u16,
    dragged: bool,
}

/// Turns touches outside the pad into mouse input: a touch that ends where it started is
/// a left click there, and dragging scrolls one step per row, like the mouse wheel.
/// Only the first finger counts; others are ignored until it lifts.
#[derive(Debug, Default)]
pub struct Touches {
    finger: Option<Finger>,
}

impl Touches {
    pub fn started(&mut self, id: u64, cell: (u16, u16)) {
        if self.finger.is_none() {
            self.finger = Some(Finger {
                id,
                start: cell,
                last_row: cell.1,
                dragged: false,
            });
        }
    }

    /// Scroll steps for a finger moving: dragging up scrolls down, as content follows
    /// the finger.
    pub fn moved(&mut self, id: u64, cell: (u16, u16)) -> Vec<MouseInput> {
        let Some(finger) = self.finger.as_mut().filter(|f| f.id == id) else {
            return Vec::new();
        };
        if cell != finger.start {
            finger.dragged = true;
        }
        let rows = cell.1 as f32 - finger.last_row as f32;
        finger.last_row = cell.1;
        if rows == 0.0 {
            return Vec::new();
        }
        input::scrolls(rows, cell, false)
    }

    /// A tap's click (the pointer moved onto the cell, then a left click), or nothing
    /// after a drag.
    pub fn ended(&mut self, id: u64) -> Vec<MouseInput> {
        match self.finger.take_if(|f| f.id == id) {
            Some(finger) if !finger.dragged => {
                let click = input::click(winit::event::MouseButton::Left, finger.start, false);
                std::iter::once(input::moved(finger.start, false))
                    .chain(click)
                    .collect()
            }
            _ => Vec::new(),
        }
    }

    pub fn cancelled(&mut self, id: u64) {
        self.finger.take_if(|f| f.id == id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::input::mouse::{MouseButton, MouseKind};

    #[test]
    fn the_pad_floats_over_the_bottom_rows_or_not_at_all() {
        let pad = pad_area(Rect::new(0, 0, 120, 40)).unwrap();
        assert_eq!(pad, Rect::new(0, 40 - PAD_ROWS, 120, PAD_ROWS));
        assert!(pad_area(Rect::new(0, 0, 120, MIN_SCREEN_ROWS - 1)).is_none());
        assert!(pad_area(Rect::new(0, 0, MIN_PAD_COLS - 1, 40)).is_none());
    }

    #[test]
    fn the_game_shows_through_the_buttons_dimmed() {
        let runner = crate::headless::chain_commands(&["skip-splash"]);
        let area = Rect::new(0, 0, 80, PAD_ROWS);
        let mut buf = Buffer::empty(area);
        for y in 0..area.height {
            for x in 0..area.width {
                buf[(x, y)].set_symbol("v");
            }
        }
        let buttons = layout(area);
        render(&buttons, &mut buf, &runner.app);
        let east = buttons.iter().find(|b| b.name == "→").unwrap().rect;
        // Beside the name and the key: the game, dimmed, on the tint
        let beside = &buf[(east.x, east.y + 1)];
        assert_eq!(beside.symbol(), "v");
        assert!(beside.modifier.contains(Modifier::DIM));
        assert_eq!(beside.fg, Color::Reset, "keeps the game's color");
        let name_row: String = (east.x..east.right())
            .map(|x| buf[(x, east.y)].symbol().to_string())
            .collect();
        assert_eq!(
            name_row.matches('v').count(),
            6,
            "only the arrow covers the game: {name_row:?}"
        );
        let arrow = (east.x..east.right())
            .find(|&x| buf[(x, east.y)].symbol() == "→")
            .unwrap();
        assert!(!buf[(arrow, east.y)].modifier.contains(Modifier::DIM));
        // The gap around each patch, and between the groups: untouched
        for cell in [
            &buf[(east.right() - 1, east.y)],
            &buf[(east.x, east.bottom() - 1)],
            &buf[(40, 4)],
        ] {
            assert_eq!(cell.symbol(), "v");
            assert!(!cell.modifier.contains(Modifier::DIM));
        }
    }

    #[test]
    fn buttons_stay_inside_the_pad_and_never_overlap() {
        for (cols, rows) in [(MIN_PAD_COLS, PAD_ROWS), (80, PAD_ROWS), (200, PAD_ROWS)] {
            let pad = Rect::new(0, 30, cols, rows);
            let buttons = layout(pad);
            assert_eq!(buttons.len(), 16);
            for (i, a) in buttons.iter().enumerate() {
                assert_eq!(a.rect.intersection(pad), a.rect, "{} at {cols}", a.name);
                for b in &buttons[i + 1..] {
                    assert!(!a.rect.intersects(b.rect), "{} and {}", a.name, b.name);
                }
            }
        }
    }

    #[test]
    fn hits_find_the_button_under_the_finger() {
        let pad = Rect::new(0, 30, 100, PAD_ROWS);
        let buttons = layout(pad);
        let east = Press::Keybind("movement", "MOVE_EAST");
        // The middle row's right button, every cell of it
        for col in BUTTON_COLS * 2..BUTTON_COLS * 3 {
            for row in 30 + BUTTON_ROWS..30 + BUTTON_ROWS * 2 {
                assert_eq!(button_at(&buttons, (col, row)), Some(east));
            }
        }
        assert_eq!(
            button_at(&buttons, (99, 30 + PAD_ROWS - 1)),
            Some(Press::Escape)
        );
        assert_eq!(button_at(&buttons, (99, 30)), Some(Press::Enter));
        // Between the two groups, and above Use
        assert_eq!(button_at(&buttons, (50, 35)), None);
        assert_eq!(button_at(&buttons, (100 - BUTTON_COLS * 3, 30)), None);
    }

    #[test]
    fn buttons_send_the_bound_key() {
        let mut runner = crate::headless::chain_commands(&["skip-splash"]);
        let app = &mut runner.app;
        let interact = Press::Keybind("action", "INTERACT");
        assert_eq!(key_code(app, interact), Some(KeyCode::Char('f')));
        assert_eq!(key_code(app, Press::Escape), Some(KeyCode::Esc));
        app.core.config_manager.data.keybinds["action"]["INTERACT"] = serde_json::json!(["j"]);
        app.ui.keybinds = crate::app_state::Keybinds::from_config(&app.core.config_manager);
        assert_eq!(key_code(app, interact), Some(KeyCode::Char('j')));
        // The label follows the binding
        let buttons = layout(Rect::new(0, 0, 80, PAD_ROWS));
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, PAD_ROWS));
        render(&buttons, &mut buf, app);
        let use_button = buttons.iter().find(|b| b.name == "Use").unwrap().rect;
        let key_row: String = (use_button.x..use_button.right())
            .map(|x| buf[(x, use_button.y + 1)].symbol().to_string())
            .collect();
        assert!(key_row.contains('j'), "{key_row:?}");
        assert_eq!(key_code(app, Press::Enter), Some(KeyCode::Enter));
    }

    #[test]
    fn a_tap_clicks_where_it_started() {
        let mut touches = Touches::default();
        touches.started(1, (10, 5));
        assert!(touches.moved(1, (10, 5)).is_empty());
        let events = touches.ended(1);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].kind, MouseKind::Move);
        assert_eq!(events[1].kind, MouseKind::Click(MouseButton::Left));
        assert_eq!((events[1].col, events[1].row), (10, 5));
    }

    #[test]
    fn a_drag_scrolls_a_step_per_row_and_never_clicks() {
        let mut touches = Touches::default();
        touches.started(1, (10, 10));
        // A second finger is ignored
        touches.started(2, (40, 10));
        assert!(touches.moved(2, (40, 2)).is_empty());
        let up = touches.moved(1, (10, 7));
        assert_eq!(up.len(), 3);
        assert!(up.iter().all(|e| e.kind == MouseKind::ScrollDown));
        let down = touches.moved(1, (11, 9));
        assert_eq!(down.len(), 2);
        assert!(down.iter().all(|e| e.kind == MouseKind::ScrollUp));
        assert!(touches.ended(1).is_empty(), "a drag is not a tap");
        // Sideways only: no scroll, and no click either
        touches.started(3, (10, 10));
        assert!(touches.moved(3, (14, 10)).is_empty());
        assert!(touches.ended(3).is_empty());
    }
}

//! Reading real gamepads for the game window (`gamepad-input` spec), with `gilrs`. Its
//! events become `crate::gamepad::PadEvent`s, which the shared mapper turns into game input.
//! Desktop only: on Android the poller never opens, and the touch pad is used instead.
#![cfg_attr(target_os = "android", allow(dead_code))]

use crate::gamepad::{quantize, Dir8, PadEvent};
#[cfg(not(target_os = "android"))]
use crate::gamepad::{PadButton, Stick};

/// A `gilrs` button as ours (`gilrs` calls the bumpers triggers, and the triggers
/// "trigger 2").
#[cfg(not(target_os = "android"))]
pub fn button(b: gilrs::Button) -> Option<PadButton> {
    use gilrs::Button as G;
    Some(match b {
        G::South => PadButton::South,
        G::East => PadButton::East,
        G::West => PadButton::West,
        G::North => PadButton::North,
        G::LeftTrigger => PadButton::LeftBumper,
        G::RightTrigger => PadButton::RightBumper,
        G::LeftTrigger2 => PadButton::LeftTrigger,
        G::RightTrigger2 => PadButton::RightTrigger,
        G::Select => PadButton::Select,
        G::Start => PadButton::Start,
        G::LeftThumb => PadButton::LeftThumb,
        G::RightThumb => PadButton::RightThumb,
        G::DPadUp => PadButton::DPadUp,
        G::DPadDown => PadButton::DPadDown,
        G::DPadLeft => PadButton::DPadLeft,
        G::DPadRight => PadButton::DPadRight,
        _ => return None,
    })
}

/// Where each stick points, so a direction is sent only when it changes.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct StickState {
    pub x: f32,
    pub y: f32,
    pub dir: Option<Dir8>,
}

impl StickState {
    /// Take one axis value (`y` up-positive, as pads report it) and return the stick's new
    /// direction if it changed.
    pub fn axis(&mut self, is_x: bool, value: f32) -> Option<Option<Dir8>> {
        if is_x {
            self.x = value;
        } else {
            // Pads report up as positive; the game's y grows downward
            self.y = -value;
        }
        let dir = quantize(self.x, self.y);
        (dir != self.dir).then(|| {
            self.dir = dir;
            dir
        })
    }
}

/// The open gamepad library and the sticks' state.
#[derive(Default)]
#[cfg_attr(target_os = "android", allow(dead_code))]
pub struct Poller {
    #[cfg(not(target_os = "android"))]
    gilrs: Option<gilrs::Gilrs>,
    left: StickState,
    right: StickState,
    tried: bool,
}

impl Poller {
    /// Every pad event since the last call. Opens the library on first use; if it can't
    /// (no input devices permission, unsupported platform), gamepads are simply absent.
    pub fn poll(&mut self) -> Vec<PadEvent> {
        #[cfg(target_os = "android")]
        {
            Vec::new()
        }
        #[cfg(not(target_os = "android"))]
        {
            use gilrs::{Axis, EventType};
            if !self.tried {
                self.tried = true;
                match gilrs::Gilrs::new() {
                    Ok(g) => self.gilrs = Some(g),
                    Err(e) => tracing::warn!(target: "game", "no gamepad support: {e}"),
                }
            }
            let Some(gilrs) = self.gilrs.as_mut() else {
                return Vec::new();
            };
            let mut events = Vec::new();
            while let Some(gilrs::Event { event, .. }) = gilrs.next_event() {
                match event {
                    EventType::ButtonPressed(b, _) => {
                        if let Some(button) = button(b) {
                            events.push(PadEvent::Button {
                                button,
                                pressed: true,
                            });
                        }
                    }
                    EventType::ButtonReleased(b, _) => {
                        if let Some(button) = button(b) {
                            events.push(PadEvent::Button {
                                button,
                                pressed: false,
                            });
                        }
                    }
                    EventType::AxisChanged(axis, value, _) => {
                        let (stick, state, is_x) = match axis {
                            Axis::LeftStickX => (Stick::Left, &mut self.left, true),
                            Axis::LeftStickY => (Stick::Left, &mut self.left, false),
                            Axis::RightStickX => (Stick::Right, &mut self.right, true),
                            Axis::RightStickY => (Stick::Right, &mut self.right, false),
                            _ => continue,
                        };
                        if let Some(dir) = state.axis(is_x, value) {
                            events.push(PadEvent::Stick { stick, dir });
                        }
                    }
                    EventType::Disconnected => {
                        self.left = StickState::default();
                        self.right = StickState::default();
                        events.push(PadEvent::Stick {
                            stick: Stick::Left,
                            dir: None,
                        });
                        events.push(PadEvent::Stick {
                            stick: Stick::Right,
                            dir: None,
                        });
                    }
                    _ => {}
                }
            }
            events
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bumpers_and_triggers_keep_their_names() {
        assert_eq!(
            button(gilrs::Button::LeftTrigger),
            Some(PadButton::LeftBumper)
        );
        assert_eq!(
            button(gilrs::Button::RightTrigger2),
            Some(PadButton::RightTrigger)
        );
        assert_eq!(button(gilrs::Button::South), Some(PadButton::South));
        assert_eq!(button(gilrs::Button::DPadLeft), Some(PadButton::DPadLeft));
        assert_eq!(button(gilrs::Button::Mode), None);
    }

    #[test]
    fn sticks_report_a_direction_once_per_change() {
        let mut s = StickState::default();
        assert_eq!(s.axis(true, 0.2), None, "dead zone");
        assert_eq!(s.axis(true, 0.9), Some(Some(Dir8::E)));
        assert_eq!(s.axis(true, 0.95), None, "still east");
        // Up is positive on the pad, north in the game
        assert_eq!(s.axis(false, 0.9), Some(Some(Dir8::NE)));
        assert_eq!(s.axis(true, 0.0), Some(Some(Dir8::N)));
        assert_eq!(s.axis(false, 0.1), Some(None), "back to centre");
    }
}

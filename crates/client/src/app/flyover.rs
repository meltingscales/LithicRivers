//! The respawn flyover: after the player picks where to wake, the view zooms out over the
//! wreck, pans across the world to the destination, and zooms in to the largest scale.
//!
//! It only moves the view. The game has already woken the robot when it starts, so
//! skipping it (any key) or letting it finish leaves the game in the same state.

use crate::sprite_loader::Scale;
use crate::App;
use lithicrivers_core::components::Position;
use std::time::{Duration, Instant};

/// How long the whole flyover takes.
pub const FLYOVER_DURATION: Duration = Duration::from_millis(3000);

/// The flyover in progress: from the wreck to where the robot woke.
#[derive(Debug, Clone, Copy)]
pub struct Flyover {
    pub from: Position,
    pub to: Position,
    pub start: Instant,
}

/// Where the view is at a point in the flyover, `t` from 0 to 1: the first quarter zooms
/// out over `from`, the middle half pans to `to` with an ease in and out, and the last
/// quarter zooms in over `to`.
pub fn view_at(from: Position, to: Position, t: f64) -> (Position, Scale) {
    let t = t.clamp(0.0, 1.0);
    if t < 0.25 {
        let scale = if t < 0.125 {
            Scale::Medium
        } else {
            Scale::Small
        };
        return (from, scale);
    }
    if t < 0.75 {
        let p = (t - 0.25) / 0.5;
        let eased = p * p * (3.0 - 2.0 * p);
        let lerp = |a: i64, b: i64| a + ((b - a) as f64 * eased).round() as i64;
        let z = if p < 0.5 { from.z } else { to.z };
        let at = Position {
            x: lerp(from.x, to.x),
            y: lerp(from.y, to.y),
            z,
        };
        return (at, Scale::Small);
    }
    let scale = if t < 0.875 {
        Scale::Medium
    } else {
        Scale::Large
    };
    (to, scale)
}

impl App {
    /// Start the flyover from `from` to `to`, showing the first frame at once.
    pub(crate) fn start_flyover(&mut self, from: Position, to: Position) {
        self.flyover = Some(Flyover {
            from,
            to,
            start: self.clock.now(),
        });
        self.update_flyover();
    }

    /// Move the view along the flyover; at its end, leave the view on the destination at
    /// the largest scale.
    pub(crate) fn update_flyover(&mut self) {
        let Some(flyover) = self.flyover else {
            return;
        };
        let elapsed = self.clock.since(flyover.start);
        if elapsed >= FLYOVER_DURATION {
            self.finish_flyover();
            return;
        }
        let t = elapsed.as_secs_f64() / FLYOVER_DURATION.as_secs_f64();
        let (at, scale) = view_at(flyover.from, flyover.to, t);
        (self.ui.view_x, self.ui.view_y, self.ui.view_z) = (at.x, at.y, at.z);
        self.ui.scale = scale;
    }

    /// End the flyover now, as if it had played to the end.
    pub(crate) fn finish_flyover(&mut self) {
        if let Some(flyover) = self.flyover.take() {
            let to = flyover.to;
            (self.ui.view_x, self.ui.view_y, self.ui.view_z) = (to.x, to.y, to.z);
            self.ui.scale = Scale::Large;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(x: i64, y: i64, z: i64) -> Position {
        Position { x, y, z }
    }

    #[test]
    fn the_view_zooms_out_pans_and_zooms_in() {
        let (from, to) = (at(0, 0, 0), at(100, -40, -2));
        assert_eq!(view_at(from, to, 0.0), (from, Scale::Medium));
        assert_eq!(view_at(from, to, 0.2), (from, Scale::Small));
        assert_eq!(view_at(from, to, 0.5), (at(50, -20, -2), Scale::Small));
        assert_eq!(view_at(from, to, 0.8), (to, Scale::Medium));
        assert_eq!(view_at(from, to, 1.0), (to, Scale::Large));
    }
}

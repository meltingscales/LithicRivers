//! The world's tick and the time of day that follows from it. A day is 15 game minutes and
//! tick 0 is 08:00, so the time of day saves and replays with the tick and new worlds start
//! in the morning.

/// Game ticks in one day (15 game minutes at 100 ticks per game second).
pub const DAY_TICKS: u64 = 90_000;
/// Game ticks in one hour of the day.
pub const HOUR_TICKS: u64 = DAY_TICKS / 24;
/// Minutes in a day.
const DAY_MINUTES: u64 = 24 * 60;
/// Where tick 0 falls in the day: 08:00.
const START_OFFSET: u64 = 8 * HOUR_TICKS;

/// The part of the day, by the clock: night 21:00 to 05:00, dawn 05:00 to 07:00, day 07:00
/// to 19:00, dusk 19:00 to 21:00.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DayPhase {
    Night,
    Dawn,
    Day,
    Dusk,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GameTime {
    pub tick: u64,
}

impl GameTime {
    pub fn new() -> Self {
        Self { tick: 0 }
    }

    pub fn advance(&mut self) {
        self.tick += 1;
    }

    /// Ticks since midnight.
    pub fn ticks_into_day(&self) -> u64 {
        (self.tick + START_OFFSET) % DAY_TICKS
    }

    /// The time of day as (hour, minute).
    pub fn clock(&self) -> (u64, u64) {
        let minutes = self.ticks_into_day() * DAY_MINUTES / DAY_TICKS;
        (minutes / 60, minutes % 60)
    }

    pub fn phase(&self) -> DayPhase {
        match self.clock().0 {
            5..=6 => DayPhase::Dawn,
            7..=18 => DayPhase::Day,
            19..=20 => DayPhase::Dusk,
            _ => DayPhase::Night,
        }
    }

    pub fn is_night(&self) -> bool {
        self.phase() == DayPhase::Night
    }

    /// How bright the sky is, out of 1000: full by day, none at night, easing through dawn
    /// and dusk.
    pub fn daylight_permille(&self) -> u64 {
        let into = self.ticks_into_day();
        let ramp = 2 * HOUR_TICKS;
        match self.phase() {
            DayPhase::Day => 1000,
            DayPhase::Night => 0,
            DayPhase::Dawn => (into - 5 * HOUR_TICKS) * 1000 / ramp,
            DayPhase::Dusk => 1000 - (into - 19 * HOUR_TICKS) * 1000 / ramp,
        }
    }

    /// Ticks until the clock next reads `hour:minute` (zero if it reads that now).
    pub fn ticks_until(&self, hour: u64, minute: u64) -> u64 {
        let target = (hour * 60 + minute) * DAY_TICKS / DAY_MINUTES;
        (target + DAY_TICKS - self.ticks_into_day()) % DAY_TICKS
    }
}

impl Default for GameTime {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(tick: u64) -> GameTime {
        GameTime { tick }
    }

    #[test]
    fn a_new_world_starts_at_eight_in_the_morning() {
        assert_eq!(at(0).clock(), (8, 0));
        assert_eq!(at(0).phase(), DayPhase::Day);
        assert_eq!(at(0).daylight_permille(), 1000);
    }

    #[test]
    fn phases_follow_the_clock_and_wrap_each_day() {
        let cases = [
            (4 * HOUR_TICKS, (12, 0), DayPhase::Day),
            (11 * HOUR_TICKS, (19, 0), DayPhase::Dusk),
            (13 * HOUR_TICKS, (21, 0), DayPhase::Night),
            (16 * HOUR_TICKS, (0, 0), DayPhase::Night),
            (21 * HOUR_TICKS, (5, 0), DayPhase::Dawn),
            (23 * HOUR_TICKS, (7, 0), DayPhase::Day),
        ];
        for (tick, clock, phase) in cases {
            for day in [0, 3] {
                let t = at(tick + day * DAY_TICKS);
                assert_eq!((t.clock(), t.phase()), (clock, phase), "tick {tick}");
            }
        }
        // 15 game minutes a day: 90 presses of the 10-second pass-time key
        assert_eq!(DAY_TICKS, 90 * 1000);
    }

    #[test]
    fn dusk_and_dawn_ease_the_light() {
        let dusk = at(12 * HOUR_TICKS); // 20:00
        assert_eq!(dusk.daylight_permille(), 500);
        let dawn = at(21 * HOUR_TICKS + HOUR_TICKS / 2); // 05:30
        assert_eq!(dawn.daylight_permille(), 250);
    }

    #[test]
    fn ticks_until_only_looks_forward() {
        let morning = at(0);
        assert_eq!(morning.ticks_until(8, 0), 0);
        assert_eq!(morning.ticks_until(0, 0), 16 * HOUR_TICKS);
        let later = at(morning.ticks_until(0, 0));
        assert_eq!(later.clock(), (0, 0));
        assert_eq!(later.ticks_until(23, 0), 23 * HOUR_TICKS);
    }
}

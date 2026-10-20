use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// What a fake clock reports as the wall-clock time at start: 2026-01-01 00:00:00 UTC.
pub const FAKE_EPOCH_UNIX: u64 = 1_767_225_600;

/// Source of time for everything in the client that changes behavior or rendering over time.
///
/// Gameplay and UI code must read time through `App::clock` rather than `Instant::now()`,
/// so headless runs can drive the client with fake time and stay deterministic.
#[derive(Debug, Clone)]
pub enum Clock {
    /// Wall-clock time, used by the interactive client.
    Real,
    /// Manually advanced time. `base` is captured once; only differences between
    /// instants are ever used, so output does not depend on when the run started.
    Fake { base: Instant, offset: Duration },
}

impl Clock {
    pub fn fake() -> Self {
        Clock::Fake {
            base: Instant::now(),
            offset: Duration::ZERO,
        }
    }

    pub fn now(&self) -> Instant {
        match self {
            Clock::Real => Instant::now(),
            Clock::Fake { base, offset } => *base + *offset,
        }
    }

    /// Time elapsed since `earlier` according to this clock.
    pub fn since(&self, earlier: Instant) -> Duration {
        self.now().saturating_duration_since(earlier)
    }

    /// Milliseconds for animation phases (text effects, flashing).
    pub fn animation_millis(&self) -> u64 {
        match self {
            Clock::Real => SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            Clock::Fake { offset, .. } => offset.as_millis() as u64,
        }
    }

    /// Seconds since the Unix epoch, for dates and seeds. A fake clock starts at
    /// `FAKE_EPOCH_UNIX` and moves only when advanced.
    pub fn unix_seconds(&self) -> u64 {
        match self {
            Clock::Real => SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            Clock::Fake { offset, .. } => FAKE_EPOCH_UNIX + offset.as_secs(),
        }
    }

    /// Advance a fake clock. Has no effect on a real clock.
    pub fn advance(&mut self, by: Duration) {
        if let Clock::Fake { offset, .. } = self {
            *offset += by;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fake_clock_only_moves_when_advanced() {
        let mut clock = Clock::fake();
        let start = clock.now();
        assert_eq!(clock.since(start), Duration::ZERO);
        clock.advance(Duration::from_millis(1500));
        assert_eq!(clock.since(start), Duration::from_millis(1500));
        assert_eq!(clock.animation_millis(), 1500);
    }

    #[test]
    fn fake_unix_time_only_changes_when_advanced() {
        let mut clock = Clock::fake();
        assert_eq!(clock.unix_seconds(), FAKE_EPOCH_UNIX);
        assert_eq!(clock.unix_seconds(), FAKE_EPOCH_UNIX);
        clock.advance(Duration::from_secs(90));
        assert_eq!(clock.unix_seconds(), FAKE_EPOCH_UNIX + 90);
    }
}

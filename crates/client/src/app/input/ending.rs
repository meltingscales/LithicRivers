//! The closing sequence: when an Aperture opens, a few full-screen pages over its art, then
//! the credits scrolling on the clock. Any key steps to the next page; on the credits it
//! returns to the world, where the Aperture stands open.

use crate::App;
use crossterm::event::KeyCode;

/// The closing pages, before the credits. They show nothing of the far side.
pub const PAGES: [&[&str]; 4] = [
    &[
        "The ring takes the last of what you fed it.",
        "For a moment, nothing. Then the air inside it folds, and keeps folding.",
    ],
    &[
        "Thale saw it open once, for four seconds.",
        "It has been open longer than that now.",
        "It does not turn toward you. Not yet.",
    ],
    &[
        "Under the ash, the frames he built to remember its shape",
        "stop dreaming of it.",
        "You were one of them. You finished what they were made for.",
    ],
    &[
        "The Aperture stands open.",
        "What waits past it, and whether it is ready for you,",
        "is another story.",
    ],
];

/// Milliseconds the credits take to scroll one line.
pub const CREDITS_LINE_MS: u128 = 800;

/// Whether the closing sequence is playing: an Aperture opened and its sequence hasn't
/// finished.
pub fn showing(app: &App) -> bool {
    app.splash.state == crate::SplashState::MainUI
        && lithicrivers_core::aperture::ending_due(&app.core.game.world).is_some()
}

/// Whether the credits page is up.
pub fn on_credits(app: &App) -> bool {
    app.panels.ending_page >= PAGES.len()
}

/// Lines of credits scrolled so far.
pub fn credits_scroll(app: &App) -> u16 {
    app.panels
        .ending_credits_since
        .map_or(0, |since| {
            (app.clock.since(since).as_millis() / CREDITS_LINE_MS) as u16
        })
        .min(app.panels.credits.text.lines().count() as u16)
}

/// Keys while the sequence plays. Returns false when it isn't.
pub fn handle_ending_input(app: &mut App, _key: KeyCode) -> bool {
    if !showing(app) {
        return false;
    }
    if on_credits(app) {
        finish(app);
        return true;
    }
    app.panels.ending_page += 1;
    if on_credits(app) {
        app.panels.ending_credits_since = Some(app.clock.now());
    }
    true
}

/// Back to the world: the opening is seen and stays saved with the Aperture.
fn finish(app: &mut App) {
    let game = &mut app.core.game;
    if let Some(core) = lithicrivers_core::aperture::ending_due(&game.world) {
        if let Ok(mut a) = game
            .world
            .get::<&mut lithicrivers_core::aperture::Aperture>(core)
        {
            a.ending_seen = true;
        }
    }
    game.res.log("The Aperture stands open.");
    app.panels.ending_page = 0;
    app.panels.ending_credits_since = None;
}

use anyhow::Result;
use lithicrivers_core::Game;
use tracing_subscriber::{fmt, EnvFilter};

fn main() -> Result<()> {
    // Logging
    let _ = fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();

    let _game = Game::new(42u64);
    // TUI removed: game loop and rendering must be replaced with alternative logic or removed entirely.
    Ok(())
}

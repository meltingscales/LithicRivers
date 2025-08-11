use anyhow::Result;
use clap::Parser;
use lithicrivers_core::Game;
use lithicrivers_tui::TuiApp;
use tracing_subscriber::{fmt, EnvFilter};

#[derive(Parser, Debug)]
#[command(name = "lithicrivers", version, about = "LithicRivers Rust migration scaffold")] 
struct Args {
    /// World seed
    #[arg(long, default_value_t = 42u64)]
    seed: u64,
}

fn main() -> Result<()> {
    // Logging
    let _ = fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();

    let args = Args::parse();

    let mut game = Game::new(args.seed);
    let mut tui = TuiApp::new()?;

    // Simple fixed-timestep loop (~30 FPS render, 100ms per tick)
    let tick_ms = 100u64;
    let mut last_tick = std::time::Instant::now();

    loop {
        // Tick
        if last_tick.elapsed() >= std::time::Duration::from_millis(tick_ms) {
            game.tick();
            last_tick = std::time::Instant::now();
        }

        // Draw
        tui.draw_once(&game)?;

        // Input: press 'q' to quit
        if TuiApp::poll_quit_event(10)? { break; }
    }

    tui.teardown()?;
    Ok(())
}

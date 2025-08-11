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

    // Simple loop: poll input, apply intent, tick once, render. ~100 FPS cap by sleep.
    loop {
        if tui.handle_input(&mut game, 10)? { break; }
        game.tick();
        tui.draw_once(&mut game)?;
    }

    tui.teardown()?;
    Ok(())
}

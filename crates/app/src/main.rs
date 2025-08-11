use anyhow::Result;
use lithicrivers_core::Game;
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
    // TUI removed: game loop and rendering must be replaced with alternative logic or removed entirely.
    Ok(())
}

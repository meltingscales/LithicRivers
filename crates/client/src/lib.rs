//! The LithicRivers client: the game window, the terminal UI, headless runs and MCP.
//! Desktop builds start it through `run_desktop` (see `main.rs`).

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
mod app;
mod app_state;
mod bench;
mod boot_message;
mod clock;
mod dialogue_presenter;
#[cfg(test)]
mod factory_tests;
#[cfg(test)]
mod farming_tests;
mod headless;
mod mcp;
#[cfg(test)]
mod menu_navigation_tests;
#[cfg(test)]
mod save_reload_tests;
#[cfg(test)]
mod snapshot_tests;
mod tutorialsystem;
mod ui;
mod window;
mod worlds;

use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use rust_embed::RustEmbed;
use std::{
    error::Error,
    io::{self, IsTerminal},
    time::Duration,
};
use tracing_subscriber::EnvFilter;

// Tracing file appender for log file output
use tracing_appender as _tracing_appender_hidden; // avoid "unused extern crate" lint

#[derive(RustEmbed)]
#[folder = "assets/"]
struct EmbeddedAssets;

use crate::app_state::*;
mod audio;
mod rendering_helpers;
mod sprite_constants;
mod sprite_loader;
pub(crate) use crate::app::client_app::{App, MenuTab, SplashState};
pub(crate) use crate::ui::layout::ui;
use crate::{
    app::handle_input,
    sprite_loader::{Scale, SpriteLoader},
};

/// Run the desktop client: parse the command line, then start a headless mode, the game
/// window, or the terminal UI.
pub fn run_desktop() -> Result<(), Box<dyn Error>> {
    #[cfg(windows)]
    attach_parent_console();

    let seed_val = init_logging_and_seed();

    // Headless scripted run: no terminal, no audio, fake clock
    let terminal_mode = match run_headless_from_args(seed_val)? {
        Launch::Exit(code) => std::process::exit(code),
        Launch::Interactive { terminal } => terminal,
    };

    let mut app = App::new_with_seed(seed_val);

    // The game's own window, unless the player asked for the terminal
    if !terminal_mode {
        let event_loop =
            winit::event_loop::EventLoop::new().map_err(|e| format!("cannot open a window: {e}"));
        match event_loop.and_then(|event_loop| window::run(&mut app, event_loop)) {
            Ok(()) => {
                log_shutdown(&mut app);
                return Ok(());
            }
            Err(e) => {
                tracing::error!(target: "game", "game window unavailable: {e}");
                if !io::stdin().is_terminal() {
                    show_error(&format!(
                        "LithicRivers couldn't open its window: {e}\n\nYou can start it from a terminal with --terminal instead."
                    ));
                    std::process::exit(1);
                }
                eprintln!("LithicRivers couldn't open its window ({e}); using this terminal.");
            }
        }
    }

    // Ensure we restore the terminal if a panic occurs
    std::panic::set_hook(Box::new(|info| {
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen, DisableMouseCapture);
        eprintln!("\n\nPanic: {info}");
    }));

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

/// Read the config's logging level and default seed, and start writing logs to
/// `LithicRivers.log` (rotated daily) in the data folder. Returns the default seed.
fn init_logging_and_seed() -> u64 {
    let config = lithicrivers_core::game_config::GameConfig::new();
    let log_level = config
        .get_setting("game", "LOGGINGLEVEL")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("LOGGINGLEVEL must be set in config"));
    let seed_val: u64 = config
        .get_setting("game", "DEFAULT_SEED")
        .and_then(|v| v.as_u64())
        .unwrap_or_else(|| panic!("DEFAULT_SEED must be set in config"));
    let file_appender = _tracing_appender_hidden::rolling::daily(
        lithicrivers_core::config::data_path("."),
        "LithicRivers.log",
    );
    let (non_blocking, guard) = _tracing_appender_hidden::non_blocking(file_appender);
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(log_level.to_lowercase()));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(non_blocking)
        .with_ansi(false)
        .try_init();
    // Keep the guard alive for the program's lifetime so logs flush
    let _ = Box::leak(Box::new(guard));
    seed_val
}

/// Android's entry point: the game in its window, with every file in the app's storage.
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(android_app: winit::platform::android::activity::AndroidApp) {
    use winit::platform::android::EventLoopBuilderExtAndroid;
    if let Some(dir) = android_app.internal_data_path() {
        lithicrivers_core::config::set_base_dir(dir);
    }
    let seed = init_logging_and_seed();
    let mut app = App::new_with_seed(seed);
    app.ui.window.mobile = true;
    let event_loop = winit::event_loop::EventLoop::builder()
        .with_android_app(android_app)
        .build();
    match event_loop {
        Ok(event_loop) => {
            if let Err(e) = window::run(&mut app, event_loop) {
                tracing::error!(target: "game", "game window failed: {e}");
            }
        }
        Err(e) => tracing::error!(target: "game", "cannot start the event loop: {e}"),
    }
    log_shutdown(&mut app);
    // winit allows one event loop per process, so a later launch of the activity in this
    // process could never draw. Ending the process makes the next launch start fresh.
    std::process::exit(0);
}

/// Built as a window app, the Windows client has no console of its own. Terminal and
/// headless runs started from a console use that console instead.
#[cfg(windows)]
fn attach_parent_console() {
    let wants_console = std::env::args()
        .skip(1)
        .any(|a| matches!(a.as_str(), "--terminal" | "--script" | "--bench" | "--mcp"));
    if wants_console {
        // Fails harmlessly when there is no parent console (or output is piped)
        unsafe {
            winapi::um::wincon::AttachConsole(winapi::um::wincon::ATTACH_PARENT_PROCESS);
        }
    }
}

/// Tell the player something went wrong where they will see it: a message box on
/// Windows, where a window app has no console, and standard error elsewhere.
fn show_error(message: &str) {
    tracing::error!(target: "game", "{message}");
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        let wide = |s: &str| {
            std::ffi::OsStr::new(s)
                .encode_wide()
                .chain(std::iter::once(0))
                .collect::<Vec<u16>>()
        };
        let (text, title) = (wide(message), wide("LithicRivers"));
        unsafe {
            winapi::um::winuser::MessageBoxW(
                std::ptr::null_mut(),
                text.as_ptr(),
                title.as_ptr(),
                winapi::um::winuser::MB_OK | winapi::um::winuser::MB_ICONERROR,
            );
        }
    }
    #[cfg(not(windows))]
    eprintln!("{message}");
}

/// How the client was asked to run.
enum Launch {
    /// A headless mode ran; exit with this status
    Exit(i32),
    /// Start the interactive client, in the terminal or in the game window
    Interactive { terminal: bool },
}

/// Handle `--script <path> [--seed <u64>] [--size <cols>x<rows>]`, `--bench <script>...`,
/// `--mcp` and `--terminal`.
fn run_headless_from_args(default_seed: u64) -> Result<Launch, Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let mut script_path = None;
    let mut bench_paths: Vec<String> = Vec::new();
    let mut mcp = false;
    let mut terminal = false;
    let mut overrides = headless::Overrides::default();
    while let Some(arg) = args.next() {
        let mut value = || args.next().ok_or_else(|| format!("{arg} expects a value"));
        match arg.as_str() {
            "--script" => script_path = Some(value()?),
            "--mcp" => mcp = true,
            "--terminal" => terminal = true,
            "--bench" => {
                bench_paths.push(value()?);
                bench_paths.extend(args.by_ref());
            }
            "--seed" => overrides.seed = Some(value()?.parse()?),
            "--size" => {
                let v = value()?;
                overrides.size =
                    Some(headless::parse_size(&v).ok_or_else(|| format!("invalid --size '{v}'"))?);
            }
            other => return Err(format!("unknown argument '{other}'").into()),
        }
    }
    if !bench_paths.is_empty() {
        if mcp || script_path.is_some() || overrides.seed.is_some() || overrides.size.is_some() {
            return Err("--bench takes only scenario script paths".into());
        }
        let code = bench::run(
            &bench_paths,
            &mut io::stdout().lock(),
            &mut io::stderr().lock(),
        );
        return Ok(Launch::Exit(code));
    }
    if mcp {
        if script_path.is_some() || overrides.size.is_some() {
            return Err("--mcp can't be combined with --script or --size".into());
        }
        mcp::serve_stdio(overrides.seed.unwrap_or(default_seed))?;
        return Ok(Launch::Exit(0));
    }
    let Some(path) = script_path else {
        if overrides.seed.is_some() || overrides.size.is_some() {
            return Err("--seed and --size require --script".into());
        }
        return Ok(Launch::Interactive { terminal });
    };
    let text = std::fs::read_to_string(&path).map_err(|e| format!("cannot read {path}: {e}"))?;
    let code = headless::run_script(
        &text,
        default_seed,
        overrides,
        &mut io::stdout().lock(),
        &mut io::stderr().lock(),
    );
    Ok(Launch::Exit(code))
}

/// A crossterm mouse event as the game's `MouseInput` (button releases and drags are
/// ignored).
fn mouse_input(me: &crossterm::event::MouseEvent) -> Option<app::input::mouse::MouseInput> {
    use app::input::mouse::{MouseButton, MouseInput, MouseKind};
    use crossterm::event::{KeyModifiers, MouseButton as Button, MouseEventKind};
    let kind = match me.kind {
        MouseEventKind::Down(Button::Left) => MouseKind::Click(MouseButton::Left),
        MouseEventKind::Down(Button::Right) => MouseKind::Click(MouseButton::Right),
        MouseEventKind::Down(Button::Middle) => MouseKind::Click(MouseButton::Middle),
        MouseEventKind::ScrollUp => MouseKind::ScrollUp,
        MouseEventKind::ScrollDown => MouseKind::ScrollDown,
        MouseEventKind::Moved => MouseKind::Move,
        _ => return None,
    };
    Some(MouseInput {
        kind,
        col: me.column,
        row: me.row,
        shift: me.modifiers.contains(KeyModifiers::SHIFT),
    })
}

/// Advance the client by one frame: combat auto-ticks and splash/UI timers.
/// Shared by the interactive loop and headless runs; reads time only from `app.clock`.
fn step(app: &mut App) {
    app.update_demo();
    // Auto-advance ticks when combat is active (one tick per COMBAT_MS_PER_TICK)
    if app.combat.is_active() {
        let tick_ms = app
            .core
            .game
            .res
            .config
            .get_setting("game", "COMBAT_MS_PER_TICK")
            .and_then(|v| v.as_u64())
            .unwrap_or_else(|| {
                panic!(
                    "COMBAT_MS_PER_TICK must be set in config. raw value: {:?}",
                    app.core
                        .game
                        .res
                        .config
                        .get_setting("game", "COMBAT_MS_PER_TICK")
                )
            });
        let tick_interval = Duration::from_millis(tick_ms);

        let current_time = app.clock.now();
        let should_tick = match app.last_combat_tick {
            None => {
                app.last_combat_tick = Some(current_time);
                false
            }
            Some(last_time) => {
                if current_time.saturating_duration_since(last_time) >= tick_interval {
                    app.last_combat_tick = Some(current_time);
                    true
                } else {
                    false
                }
            }
        };

        if should_tick {
            let tick_result = app.core.game.tick();
            app.apply_tick_result(&tick_result);
        }
    } else {
        // Reset tick timer when not in combat
        app.last_combat_tick = None;
    }

    app.on_tick();
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<(), Box<dyn Error>>
where
    B::Error: 'static,
{
    loop {
        step(app);

        terminal.draw(|f| ui(f, app))?;

        if crossterm::event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Press {
                        handle_input(app, key.code)?;
                    }
                }
                Event::Mouse(me) => {
                    if let Some(input) = mouse_input(&me) {
                        handle_mouse_event(app, input);
                    }
                }
                _ => {}
            }
        }

        if app.core.should_quit {
            log_shutdown(app);
            return Ok(());
        }
    }
}

/// A mouse event from either frontend: on the boot log the wheel scrolls the log,
/// anywhere else the game handles it.
fn handle_mouse_event(app: &mut App, input: app::input::mouse::MouseInput) {
    use app::input::mouse::MouseKind;
    if let SplashState::BootMessage = app.splash.state {
        match input.kind {
            MouseKind::ScrollDown => {
                let total_lines = app.splash.boot_display_text.lines().count() as u16;
                let visible_lines = 20; // Approximate visible lines
                if app.splash.boot_scroll + visible_lines < total_lines {
                    app.splash.boot_scroll = app.splash.boot_scroll.saturating_add(3);
                }
            }
            MouseKind::ScrollUp => {
                app.splash.boot_scroll = app.splash.boot_scroll.saturating_sub(3);
            }
            _ => {}
        }
        return;
    }
    app::input::mouse::handle_mouse(app, input);
}

fn log_shutdown(app: &mut App) {
    app.core.game.res.log("Shutting down...");
    tracing::info!(
        target: "game",
        "shutdown tick={} view_z={} seed={}",
        app.core.game.res.time.tick,
        app.ui.view_z,
        app.core.game.res.world_state.seed
    );
}

// Helper function to get player's inventory as a HashMap

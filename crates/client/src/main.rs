use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
mod app;
mod app_state;
mod boot_message;
mod input;
mod ui;

use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Style},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Tabs, Wrap},
    Frame, Terminal,
};
use rust_embed::RustEmbed;
use std::{error::Error, io, time::Duration, time::Instant};
use tracing_subscriber::EnvFilter;

// Tracing file appender for log file output
use tracing_appender as _tracing_appender_hidden; // avoid "unused extern crate" lint

#[derive(RustEmbed)]
#[folder = "assets/"]
struct EmbeddedAssets;

use crate::app_state::*;
use lithicrivers_core::{
    components::{BattleDelay, Combat, GameEntity, Position},
    config::ConfigManager,
    Entity,
};
mod audio;
mod rendering_helpers;
mod sprite_constants;
mod sprite_loader;
use crate::{
    app::handle_input,
    sprite_loader::{Scale, SpriteLoader},
    ui::{
        centered_rect,
        panels::{
            render_body_panel, render_combat_panel, render_crafting_panel, render_credits_panel,
            render_game_view, render_help_panel, render_inventory_list_only,
            render_inventory_panel, render_look_panel, render_menu_panel, render_quit_panel,
        },
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SplashState {
    Logo,
    GameTitle,
    BootMessage,
    MainUI,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MenuTab {
    World,
    Body,
    Inventory,
    Crafting,
    Menu,
    Help,
    Credits,
    Quit,
}

const BOOT_MESSAGE_TYPEWRITER_MS: u64 = 100;

impl MenuTab {
    #[allow(dead_code)]
    const COUNT: usize = 8;

    fn next(self) -> Self {
        match self {
            MenuTab::World => MenuTab::Body,
            MenuTab::Body => MenuTab::Inventory,
            MenuTab::Inventory => MenuTab::Crafting,
            MenuTab::Crafting => MenuTab::Menu,
            MenuTab::Menu => MenuTab::Help,
            MenuTab::Help => MenuTab::Credits,
            MenuTab::Credits => MenuTab::Quit,
            MenuTab::Quit => MenuTab::World,
        }
    }

    fn prev(self) -> Self {
        match self {
            MenuTab::World => MenuTab::Quit,
            MenuTab::Body => MenuTab::World,
            MenuTab::Inventory => MenuTab::Body,
            MenuTab::Crafting => MenuTab::Inventory,
            MenuTab::Menu => MenuTab::Crafting,
            MenuTab::Help => MenuTab::Menu,
            MenuTab::Credits => MenuTab::Help,
            MenuTab::Quit => MenuTab::Credits,
        }
    }

    fn as_index(&self) -> usize {
        *self as usize
    }
}

/// Main application state - organized into logical subsystems for better maintainability
struct App {
    /// Core game engine systems
    pub core: CoreState,
    /// UI framework state  
    pub ui: UiState,
    /// Audio system
    pub audio: AudioState,
    /// Logging configuration
    pub logging: LoggingState,
    /// Combat system
    pub combat: CombatUiState,
    /// UI panel states
    pub panels: PanelStates,
    /// Splash screen system
    pub splash: SplashScreenState,
    /// Last time we advanced a combat tick (for 10 ticks/second)
    pub last_combat_tick: Option<std::time::Instant>,
}

impl App {
    fn on_tick(&mut self) {
        // Handle splash screen timing
        if let Some(start_time) = self.splash.start_time {
            match self.splash.state {
                SplashState::Logo => {
                    if start_time.elapsed() >= Duration::from_secs(1) {
                        self.splash.state = SplashState::GameTitle;
                        self.splash.start_time = Some(Instant::now());
                    }
                }
                SplashState::GameTitle => {
                    if start_time.elapsed() >= Duration::from_secs(1) {
                        self.splash.state = SplashState::BootMessage;
                        self.splash.start_time = Some(Instant::now());
                        self.splash.boot_display_text.clear();
                        self.splash.boot_line_index = 0;
                        self.splash.last_line_time = Instant::now();
                        self.splash.boot_complete = false;
                    }
                }
                SplashState::BootMessage => {
                    // Handle typewriter effect
                    if !self.splash.boot_complete {
                        let elapsed = self.splash.last_line_time.elapsed();
                        if elapsed >= Duration::from_millis(BOOT_MESSAGE_TYPEWRITER_MS) {
                            // ~(1000/x) characters per second
                            self.splash.last_line_time = Instant::now();

                            // Get all text as a single string with newlines
                            let full_text = self.splash.boot_message_lines.join("\n");

                            if self.splash.boot_line_index < full_text.len() {
                                // Move to next line
                                if let Some(next_newline) =
                                    full_text[self.splash.boot_line_index..].find('\n')
                                {
                                    self.splash.boot_line_index += next_newline + 1;
                                } else {
                                    self.splash.boot_line_index = full_text.len();
                                }
                                self.splash.boot_display_text =
                                    full_text[..self.splash.boot_line_index].to_string();
                            } else if !self.splash.boot_complete {
                                self.splash.boot_complete = true;
                                // Set a minimum display time after completion
                                self.splash.start_time = Some(Instant::now());
                            }
                        }
                    } else if start_time.elapsed() >= Duration::from_secs(2) {
                        // 2 seconds after completion, move to main UI
                        self.splash.state = SplashState::MainUI;
                    }
                }
                _ => {}
            }
        }
    }

    fn snap_view_to_player_z(&mut self) {
        if let Some(e) = self.core.game.get_player_entity() {
            if let Ok(pos) = self
                .core
                .game
                .world
                .get::<&lithicrivers_core::components::Position>(e)
            {
                // Snap entire viewport center and Z slice to player
                self.ui.view_x = pos.x;
                self.ui.view_y = pos.y;
                self.ui.view_z = pos.z;
            }
        }
    }

    fn activate_menu(&mut self) {
        match self.ui.current_tab {
            MenuTab::World => {
                // World (already active view)
                self.core.game.res.log("World map active");
            }
            MenuTab::Body => {
                // Body
                self.core.game.res.log("Body panel active");
            }
            MenuTab::Inventory => {
                // Inventory (placeholder)
                self.core.game.res.log("Inventory panel (WIP)");
            }
            MenuTab::Menu => {
                // Menu
                self.core
                    .game
                    .res
                    .log("Menu panel active (press S to Save, L to Load)");
            }
            MenuTab::Help => {
                // Help
                self.core.game.res.log("Help panel active");
            }
            MenuTab::Credits => {
                // Credits
                self.core
                    .game
                    .res
                    .log("Credits panel active (Up/Down to scroll)");
            }
            MenuTab::Quit => {
                // Quit
                self.core.game.res.log("Quit requested (menu)");
                tracing::info!(target: "game", "quit_requested input=menu tick={}", self.core.game.res.gametick);
                self.core.should_quit = true;
            }
            MenuTab::Crafting => {
                // Crafting
                self.core.game.res.log("Crafting panel active");
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Load configuration to determine logging and seed before starting app
    let cm = ConfigManager::new();
    let log_level = cm
        .get_setting("game", "LOGGINGLEVEL")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("LOGGINGLEVEL must be set in config"));
    let seed_val: u64 = cm
        .get_setting("game", "DEFAULT_SEED")
        .and_then(|v| v.as_u64())
        .unwrap_or_else(|| panic!("DEFAULT_SEED must be set in config"));

    // Initialize tracing to write logs to LithicRivers.log (rotated daily)
    {
        let file_appender = _tracing_appender_hidden::rolling::daily(".", "LithicRivers.log");
        let (non_blocking, _guard) = _tracing_appender_hidden::non_blocking(file_appender);
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(log_level.to_lowercase()));
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_writer(non_blocking)
            .with_ansi(false)
            .init();
        // Keep _guard alive for program lifetime to ensure logs flush properly
        let _ = Box::leak(Box::new(_guard));
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

    // Create app and run it
    let mut app = App::new_with_seed(seed_val);
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

/// Update combat timers and trigger world ticks when moves execute
/// Returns true if combat should exit (due to escape)
fn update_combat_timing(
    _app: &mut App,
    _delta_time: std::time::Duration,
) -> Result<bool, Box<dyn std::error::Error>> {
    // This function is deprecated - combat timing is now handled by ActionQueue system in core
    // Always return false (don't exit combat from here)
    Ok(false)
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<(), Box<dyn Error>> {
    let mut last_update = std::time::Instant::now();

    loop {
        let now = std::time::Instant::now();
        let delta_time = now.duration_since(last_update);
        last_update = now;

        // Update combat timing (but don't tick world yet!)
        let should_exit_combat = update_combat_timing(app, delta_time)?;

        // Exit combat if escape was successful
        if should_exit_combat {
            app.combat = CombatUiState::None;
        }

        // Auto-advance ticks when combat is active (10 ticks per second)
        if app.combat.is_active() {
            const TICK_INTERVAL: std::time::Duration = std::time::Duration::from_millis(100); // 10 ticks per second

            let current_time = std::time::Instant::now();
            let should_tick = match app.last_combat_tick {
                None => {
                    app.last_combat_tick = Some(current_time);
                    false
                }
                Some(last_time) => {
                    if current_time.duration_since(last_time) >= TICK_INTERVAL {
                        app.last_combat_tick = Some(current_time);
                        true
                    } else {
                        false
                    }
                }
            };

            if should_tick {
                app.core.game.tick();
            }
        } else {
            // Reset tick timer when not in combat
            app.last_combat_tick = None;
        }

        terminal.draw(|f| ui(f, app))?;

        if crossterm::event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Press {
                        handle_input(app, key.code)?;
                    }
                }
                Event::Mouse(me) => {
                    if let crossterm::event::MouseEventKind::ScrollDown = me.kind {
                        // Scroll down (move view down, which means increase scroll position)
                        if let SplashState::BootMessage = app.splash.state {
                            let total_lines = app.splash.boot_display_text.lines().count() as u16;
                            let visible_lines = 20; // Approximate visible lines
                            if app.splash.boot_scroll + visible_lines < total_lines {
                                app.splash.boot_scroll = app.splash.boot_scroll.saturating_add(3);
                                // Scroll 3 lines at a time
                            }
                        }
                    } else if let crossterm::event::MouseEventKind::ScrollUp = me.kind {
                        // Scroll up (move view up, which means decrease scroll position)
                        if let SplashState::BootMessage = app.splash.state {
                            app.splash.boot_scroll = app.splash.boot_scroll.saturating_sub(3);
                            // Scroll 3 lines at a time
                        }
                    } else {
                        // Log other mouse events
                        tracing::debug!("UNHANDLED Mouse event: {:?}", me);
                    }
                }
                _ => {}
            }
        }

        app.on_tick();

        if app.core.should_quit {
            app.core.game.res.log("Shutting down...");
            tracing::info!(
                target: "game",
                "shutdown tick={} view_z={} seed={}",
                app.core.game.res.gametick,
                app.ui.view_z,
                app.core.game.res.seed
            );
            return Ok(());
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    // Show splash screens if needed
    match app.splash.state {
        SplashState::Logo => {
            // Show centered logo
            let logo_paragraph = Paragraph::new(app.splash.logo_text.as_str())
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::NONE));

            // Center the logo in the middle of the screen
            let area = centered_rect(50, 50, f.size());
            f.render_widget(logo_paragraph, area);
            return;
        }
        SplashState::GameTitle => {
            // Show centered game title
            let title_paragraph = Paragraph::new(app.splash.game_title_text.as_str())
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::NONE));

            // Center the title in the middle of the screen
            let area = centered_rect(70, 70, f.size());
            f.render_widget(title_paragraph, area);
            return;
        }
        SplashState::BootMessage => {
            // Show boot message with glitch effect
            let block = Block::default()
                .borders(Borders::ALL)
                .border_set(border::THICK)
                .title(" System Boot ")
                .title_alignment(Alignment::Center)
                .border_style(Style::default().fg(Color::LightBlue));

            // Inner and centered content area with padding
            let inner = block.inner(Rect::new(0, 0, f.size().width, f.size().height));
            let area = centered_rect(80, 60, inner);

            // Clear content area before drawing
            f.render_widget(Clear, area);

            // Calculate available height for text (accounting for borders and padding)
            let inner_area = area.inner(&Margin {
                horizontal: 2,
                vertical: 2,
            });
            let text_area = inner_area.inner(&Margin {
                horizontal: 1,
                vertical: 1,
            });

            // Calculate visible lines and update scroll position if needed
            let visible_lines = text_area.height.saturating_sub(2); // Leave room for border
            let total_lines = app.splash.boot_display_text.lines().count() as u16;

            // Auto-scroll if we're at the bottom
            if app.splash.boot_scroll + visible_lines >= total_lines.saturating_sub(1) {
                app.splash.boot_scroll = total_lines.saturating_sub(visible_lines);
            }

            // Create a scrollable paragraph
            let paragraph = Paragraph::new(app.splash.boot_display_text.as_str())
                .block(Block::default().borders(Borders::NONE))
                .wrap(Wrap { trim: false })
                .scroll((app.splash.boot_scroll, 0));

            // Render content with padding
            f.render_widget(paragraph, inner_area);

            // Add scrollbar if needed
            if total_lines > visible_lines {
                // Create a simple scrollbar on the right
                let scrollbar_area = Rect {
                    x: inner_area.right() - 1,
                    y: inner_area.y,
                    width: 1,
                    height: inner_area.height,
                };

                // Calculate scrollbar thumb position and height
                let thumb_height = (visible_lines as f32 / total_lines as f32
                    * visible_lines as f32)
                    .max(1.0) as u16;
                let thumb_position = (app.splash.boot_scroll as f32
                    / (total_lines - visible_lines) as f32
                    * (visible_lines - thumb_height) as f32)
                    .round() as u16;

                // Draw scrollbar track
                let track_span = Span::styled("│", Style::default().fg(Color::DarkGray));
                for y in inner_area.top()..inner_area.bottom() {
                    f.render_widget(
                        Paragraph::new(track_span.clone()),
                        Rect::new(scrollbar_area.x, y, 1, 1),
                    );
                }

                // Draw scrollbar thumb
                let thumb_span = Span::styled("▐", Style::default().fg(Color::LightBlue));
                for y in 0..thumb_height {
                    let y_pos = inner_area.y + thumb_position + y;
                    if y_pos < inner_area.bottom() {
                        f.render_widget(
                            Paragraph::new(thumb_span.clone()),
                            Rect::new(scrollbar_area.x, y_pos, 1, 1),
                        );
                    }
                }
            }

            // Render border last to ensure it's on top
            f.render_widget(block, area);
            return;
        }
        SplashState::MainUI => {
            // Normal UI rendering continues below
        }
    }

    // Only render the main UI if we're past all splash screens
    if app.splash.state != SplashState::MainUI {
        return;
    }

    let root_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Length(1), // Title line
            Constraint::Min(0),    // Main area (map + inventory)
            Constraint::Length(7), // Message log
            Constraint::Length(3), // Bottom menu bar (needs 3 for borders + content)
        ])
        .split(f.size());

    // Title
    let title = Paragraph::new("LithicRivers (Ratatui Client)")
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center);
    f.render_widget(title, root_chunks[0]);

    // Main area depends on selected tab
    match app.ui.current_tab {
        MenuTab::World => {
            if app.combat.is_active() {
                // Combat mode: show combat panel on the left (50%), game view on the right (50%)
                let main_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(root_chunks[1]);
                render_combat_panel(f, app, main_chunks[0]);
                render_game_view(f, app, main_chunks[1]);
            } else if app.panels.look.mode {
                // With Look mode: show look panel on the right
                let main_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Min(20), Constraint::Length(24)])
                    .split(root_chunks[1]);
                render_game_view(f, app, main_chunks[0]);
                render_look_panel(f, app, main_chunks[1]);
            } else {
                // Show inventory list as sidebar, but hide Item detail panel
                let main_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Min(20), Constraint::Length(24)])
                    .split(root_chunks[1]);
                render_game_view(f, app, main_chunks[0]);
                render_inventory_list_only(f, app, main_chunks[1]);
            }
        }
        MenuTab::Crafting => {
            // Crafting panel
            render_crafting_panel(f, app, root_chunks[1]);
        }
        MenuTab::Body => {
            // Body panel
            render_body_panel(f, app, root_chunks[1]);
        }
        MenuTab::Inventory => {
            // Inventory panel
            render_inventory_panel(f, app, root_chunks[1]);
        }
        MenuTab::Menu => {
            // Menu panel
            render_menu_panel(f, app, root_chunks[1]);
        }
        MenuTab::Help => {
            // Help: controls
            render_help_panel(f, app, root_chunks[1]);
        }
        MenuTab::Credits => {
            // Credits: scrollable
            render_credits_panel(f, app, root_chunks[1]);
        }
        MenuTab::Quit => {
            // Quit: show diamond pattern panel
            render_quit_panel(f, app, root_chunks[1]);
        }
    }

    // Message log
    render_message_log(f, app, root_chunks[2]);

    // Bottom menu bar
    render_bottom_menu(f, app, root_chunks[3]);
}

fn render_message_log(f: &mut Frame, app: &mut App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();
    // Account for the border (top+bottom) since Paragraph has a Block
    let visible_rows = area.height.saturating_sub(2) as usize;
    let start = if app.core.game.res.messages.len() > visible_rows {
        app.core.game.res.messages.len() - visible_rows
    } else {
        0
    };
    for msg in app.core.game.res.messages.iter().skip(start) {
        lines.push(Line::from(Span::raw(msg.clone())));
    }
    let paragraph = Paragraph::new(lines)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Messages")
                .style(Style::default().fg(Color::White)),
        );
    f.render_widget(paragraph, area);
}

fn render_bottom_menu(f: &mut Frame, app: &mut App, area: Rect) {
    // Remember for click handling
    app.ui.bottom_menu_rect = Some(area);
    // All tabs white; selected tab green
    let titles = vec![
        Span::raw("World"),
        Span::raw("Body"),
        Span::raw("Inventory"),
        Span::raw("Crafting"),
        Span::raw("Menu"),
        Span::raw("Help"),
        Span::raw("Credits"),
        Span::raw("Quit"),
    ];
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Menu"))
        .select(app.ui.current_tab.as_index())
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Green));
    f.render_widget(tabs, area);
}

// Helper function to get player's inventory as a HashMap

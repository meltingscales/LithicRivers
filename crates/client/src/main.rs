use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
mod app;
mod app_state;
mod boot_message;
mod dialogue_engine;
mod dialogue_presenter;
mod input;
mod ui;

use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
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
use crate::dialogue_presenter::DialoguePresenter;
use lithicrivers_core::{
    components::{BattleDelay, Combat, GameEntity, Position},
    config::ConfigManager,
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
            render_game_view, render_help_panel, render_hotbar_panel, render_inventory_list_only,
            render_inventory_panel, render_look_panel, render_menu_panel, render_modes_panel,
            render_quit_panel,
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
                tracing::info!(target: "game", "quit_requested input=menu tick={}", self.core.game.res.time.tick);
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
    let config = lithicrivers_core::game_config::GameConfig::new();
    let log_level = config
        .get_setting("game", "LOGGINGLEVEL")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("LOGGINGLEVEL must be set in config"));
    let seed_val: u64 = config
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

        // Auto-advance ticks when combat is active (1 tick per second)
        if app.combat.is_active() {
            let TICK_INTERVAL: std::time::Duration = std::time::Duration::from_millis(
                app.core
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
                    }),
            );

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
                app.core.game.res.time.tick,
                app.ui.view_z,
                app.core.game.res.world_state.seed
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

    // Check if we should show hotbar (place mode + world tab)
    let show_hotbar = app.ui.current_tab == MenuTab::World
        && matches!(app.panels.build.mode, crate::app_state::BuildMode::Place);

    let root_chunks = if show_hotbar {
        Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints([
                Constraint::Length(1), // Title line
                Constraint::Min(0),    // Main area (map + inventory)
                Constraint::Length(3), // Hotbar
                Constraint::Length(7), // Message log
                Constraint::Length(3), // Bottom menu bar (needs 3 for borders + content)
            ])
            .split(f.size())
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints([
                Constraint::Length(1), // Title line
                Constraint::Min(0),    // Main area (map + inventory)
                Constraint::Length(7), // Message log
                Constraint::Length(3), // Bottom menu bar (needs 3 for borders + content)
            ])
            .split(f.size())
    };

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
                // With Look mode: show modes panel and look panel on the right
                let main_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Min(20), Constraint::Length(24)])
                    .split(root_chunks[1]);
                render_game_view(f, app, main_chunks[0]);

                // Split right panel vertically: modes (3 lines) + look panel (rest)
                let right_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(3), Constraint::Min(0)])
                    .split(main_chunks[1]);
                render_modes_panel(f, app, right_chunks[0]);
                render_look_panel(f, app, right_chunks[1]);
            } else {
                // Show modes panel and inventory list as sidebar
                let main_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Min(20), Constraint::Length(24)])
                    .split(root_chunks[1]);
                render_game_view(f, app, main_chunks[0]);

                // Split right panel vertically: modes (3 lines) + inventory (rest)
                let right_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(3), Constraint::Min(0)])
                    .split(main_chunks[1]);
                render_modes_panel(f, app, right_chunks[0]);
                render_inventory_list_only(f, app, right_chunks[1]);
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

    // Render corpse looting modals if active
    render_corpse_looting_modals(f, app);

    // Render NPC interaction modals if active
    render_npc_interaction_modals(f, app);

    // Render hotbar assignment modal if active
    render_hotbar_assignment_modal(f, app);

    // Render hotbar if in place mode
    if show_hotbar {
        render_hotbar_panel(f, app, root_chunks[2]);
        // Message log
        render_message_log(f, app, root_chunks[3]);
        // Bottom menu bar
        render_bottom_menu(f, app, root_chunks[4]);
    } else {
        // Message log
        render_message_log(f, app, root_chunks[2]);
        // Bottom menu bar
        render_bottom_menu(f, app, root_chunks[3]);
    }
}

fn render_message_log(f: &mut Frame, app: &mut App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();
    // Account for the border (top+bottom) since Paragraph has a Block
    let visible_rows = area.height.saturating_sub(2) as usize;
    let messages = &app.core.game.res.events.get_message_log().messages;
    let start = if messages.len() > visible_rows {
        messages.len() - visible_rows
    } else {
        0
    };
    for msg in messages.iter().skip(start) {
        let color = match msg.color {
            lithicrivers_core::message_log::MessageColor::Default => Color::White,
            lithicrivers_core::message_log::MessageColor::Red => Color::Red,
            lithicrivers_core::message_log::MessageColor::Yellow => Color::Yellow,
            lithicrivers_core::message_log::MessageColor::Green => Color::Green,
            lithicrivers_core::message_log::MessageColor::Blue => Color::Blue,
            lithicrivers_core::message_log::MessageColor::Cyan => Color::Cyan,
            lithicrivers_core::message_log::MessageColor::Magenta => Color::Magenta,
        };
        lines.push(Line::from(Span::styled(
            &msg.text,
            Style::default().fg(color),
        )));
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

/// Render corpse looting modals when active
fn render_corpse_looting_modals(f: &mut Frame, app: &mut App) {
    let looting_state = app.panels.corpse_looting.clone();
    match looting_state {
        CorpseLootingState::SelectingCorpse {
            adjacent_entities,
            selected_corpse,
        } => {
            render_corpse_selection_modal(f, app, &adjacent_entities, selected_corpse);
        }
        CorpseLootingState::LootingCorpse {
            entity,
            selected_loot_item,
            selected_player_item,
            loot_panel_focus,
        } => {
            render_corpse_loot_modal(
                f,
                app,
                entity,
                selected_loot_item,
                selected_player_item,
                loot_panel_focus,
            );
        }
        CorpseLootingState::None => {
            // No modal to render
        }
    }
}

/// Render the corpse selection modal when multiple corpses are nearby
fn render_corpse_selection_modal(
    f: &mut Frame,
    app: &mut App,
    adjacent_entities: &[hecs::Entity],
    selected_corpse: usize,
) {
    use lithicrivers_core::components::{EntityKind, Inventory as InvComp};
    use ratatui::{
        style::Modifier,
        text::{Line, Span},
        widgets::{Clear, List, ListItem},
    };

    // Create modal area (centered, 50% width, 40% height)
    let area = centered_rect(50, 40, f.size());

    // Clear the background
    f.render_widget(Clear, area);

    // Main modal block
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Choose Corpse to Loot ")
        .title_alignment(Alignment::Center)
        .style(Style::default());

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Create list of corpses
    let corpse_items: Vec<ListItem> = adjacent_entities
        .iter()
        .enumerate()
        .filter_map(|(i, &entity)| {
            // Get corpse info
            if let (Ok(entity_kind), Ok(inv)) = (
                app.core.game.world.get::<&EntityKind>(entity),
                app.core.game.world.get::<&InvComp>(entity),
            ) {
                if *entity_kind == EntityKind::Corpse {
                    let is_selected = selected_corpse == i;
                    let style = if is_selected {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };

                    // Show corpse name and item count
                    let item_count = inv.slots.len();
                    let text = format!("Corpse ({} items)", item_count);

                    return Some(ListItem::new(Line::from(Span::styled(text, style))));
                }
            }
            None
        })
        .collect();

    let list =
        List::new(corpse_items).highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_widget(list, inner);

    // Controls at bottom
    let controls = Paragraph::new("↑↓: Select | Enter: Loot | Esc: Cancel")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));

    if inner.height > 1 {
        let controls_area = Rect {
            x: inner.x,
            y: inner.y + inner.height - 1,
            width: inner.width,
            height: 1,
        };
        f.render_widget(controls, controls_area);
    }
}

/// Render the corpse looting modal with inventory transfer interface
fn render_corpse_loot_modal(
    f: &mut Frame,
    app: &mut App,
    corpse_entity: hecs::Entity,
    selected_loot_item: usize,
    selected_player_item: usize,
    loot_panel_focus: bool,
) {
    use lithicrivers_core::components::{itemkind_name, Inventory as InvComp};
    use ratatui::{
        style::Modifier,
        text::{Line, Span},
        widgets::{Clear, List, ListItem},
    };

    // Get corpse inventory
    let corpse_inv = if let Ok(inv) = app.core.game.world.get::<&InvComp>(corpse_entity) {
        inv.clone()
    } else {
        return; // Corpse no longer exists
    };

    // Get player inventory
    let player_inv = if let Some(player_entity) = app.core.game.get_player_entity() {
        if let Ok(inv) = app.core.game.world.get::<&InvComp>(player_entity) {
            inv.clone()
        } else {
            return; // Player no longer exists
        }
    } else {
        return;
    };

    // Create modal area (centered, 70% width, 60% height)
    let area = centered_rect(70, 60, f.size());

    // Clear the background
    f.render_widget(Clear, area);

    // Main modal block
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Looting Corpse ")
        .title_alignment(Alignment::Center)
        .style(Style::default());

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Split into two columns
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);

    // Left side: Corpse inventory
    let corpse_block = Block::default()
        .borders(Borders::ALL)
        .title(" Corpse Items ")
        .border_style(if loot_panel_focus {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });

    let corpse_items: Vec<ListItem> = corpse_inv
        .slots
        .iter()
        .enumerate()
        .map(|(i, stack)| {
            let is_selected = loot_panel_focus && selected_loot_item == i;
            let style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(Span::styled(
                format!("{:2} x {}", stack.qty, itemkind_name(stack.kind)),
                style,
            )))
        })
        .collect();

    let corpse_list = List::new(corpse_items).block(corpse_block);
    f.render_widget(corpse_list, chunks[0]);

    // Right side: Player inventory
    let player_block = Block::default()
        .borders(Borders::ALL)
        .title(" Your Items ")
        .border_style(if !loot_panel_focus {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });

    let player_items: Vec<ListItem> = player_inv
        .slots
        .iter()
        .enumerate()
        .map(|(i, stack)| {
            let is_selected = !loot_panel_focus && selected_player_item == i;
            let style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(Span::styled(
                format!("{:2} x {}", stack.qty, itemkind_name(stack.kind)),
                style,
            )))
        })
        .collect();

    let player_list = List::new(player_items).block(player_block);
    f.render_widget(player_list, chunks[1]);

    // Controls at bottom
    let controls = Paragraph::new("←→: Switch Panel | ↑↓: Select | Enter: Take Item | Esc: Close")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));

    if inner.height > 1 {
        let controls_area = Rect {
            x: inner.x,
            y: inner.y + inner.height - 1,
            width: inner.width,
            height: 1,
        };
        f.render_widget(controls, controls_area);
    }
}

/// Render NPC interaction modals when active
fn render_npc_interaction_modals(f: &mut Frame, app: &mut App) {
    use crate::app_state::NPCInteractionState;
    let interaction_state = app.panels.npc_interaction.clone();
    match interaction_state {
        NPCInteractionState::SelectingNPC {
            adjacent_npcs,
            selected_npc,
        } => {
            render_npc_selection_modal(f, app, &adjacent_npcs, selected_npc);
        }
        NPCInteractionState::InDialogue {
            npc_entity,
            conversation,
            selected_choice,
        } => {
            render_npc_dialogue_modal(f, app, npc_entity, selected_choice);
        }
        NPCInteractionState::None => {
            // No modal to render
        }
    }
}

/// Render the NPC selection modal when multiple NPCs are nearby
fn render_npc_selection_modal(
    f: &mut Frame,
    app: &mut App,
    adjacent_npcs: &[(hecs::Entity, String)],
    selected_npc: usize,
) {
    // Create modal area (centered, 60% width, 50% height)
    let area = centered_rect(60, 50, f.size());

    // Clear the background
    f.render_widget(Clear, area);

    // Create the modal block
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .title(" Choose NPC to Talk To ")
        .title_style(
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(Color::Black).fg(Color::White));

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Create the list of NPCs
    let mut npc_lines = Vec::new();
    for (i, (npc_entity, npc_name)) in adjacent_npcs.iter().enumerate() {
        let prefix = if i == selected_npc { "→ " } else { "  " };

        // Get NPC position for display
        let pos_info = if let Ok(pos) = app
            .core
            .game
            .world
            .get::<&lithicrivers_core::components::Position>(*npc_entity)
        {
            format!(" at ({}, {})", pos.x, pos.y)
        } else {
            String::new()
        };

        let line = format!("{}{}{}", prefix, npc_name, pos_info);
        npc_lines.push(line);
    }

    // Render the list
    let npc_text = npc_lines.join("\n");
    let paragraph = Paragraph::new(npc_text)
        .alignment(Alignment::Left)
        .style(Style::default().fg(Color::White));

    if inner.height > 2 {
        let text_area = Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width,
            height: inner.height - 2,
        };
        f.render_widget(paragraph, text_area);
    }

    // Controls at bottom
    let controls = Paragraph::new("↑↓: Select | Enter: Talk | Esc: Cancel")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));

    if inner.height > 1 {
        let controls_area = Rect {
            x: inner.x,
            y: inner.y + inner.height - 1,
            width: inner.width,
            height: 1,
        };
        f.render_widget(controls, controls_area);
    }
}

/// Render the NPC dialogue modal
fn render_npc_dialogue_modal(
    f: &mut Frame,
    app: &mut App,
    npc_entity: hecs::Entity,
    selected_choice: usize,
) {
    // Create modal area (most of screen)
    let area = centered_rect(90, 80, f.size());

    // Clear the background
    f.render_widget(Clear, area);

    // Create the modal block
    let npc_name = if let Ok(dialogue) = app
        .core
        .game
        .world
        .get::<&lithicrivers_core::components::Dialogue>(npc_entity)
    {
        dialogue.name.clone()
    } else {
        "Unknown NPC".to_string()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .title(format!(" Talking to {} ", npc_name))
        .title_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(Color::Black).fg(Color::White));

    let inner = block.inner(area);
    f.render_widget(block, inner);

    // Get current dialogue using the new DialoguePresenter
    let dialogue_text =
        if let NPCInteractionState::InDialogue { conversation, .. } = &app.panels.npc_interaction {
            DialoguePresenter::format_dialogue_text(
                &app.panels.dialogue_engine,
                conversation,
                selected_choice,
            )
        } else {
            format!("{}: \"Hello, traveler!\"", npc_name)
        };

    let paragraph = Paragraph::new(dialogue_text)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(Color::White));

    if inner.height > 2 {
        let text_area = Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width,
            height: inner.height - 2,
        };
        f.render_widget(paragraph, text_area);
    }

    // Controls at bottom
    let controls = Paragraph::new("Up/Down: Select Choice | Enter: Choose | Esc: End Conversation")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));

    if inner.height > 1 {
        let controls_area = Rect {
            x: inner.x,
            y: inner.y + inner.height - 1,
            width: inner.width,
            height: 1,
        };
        f.render_widget(controls, controls_area);
    }
}

/// Render hotbar assignment modal when active
fn render_hotbar_assignment_modal(f: &mut Frame, app: &mut App) {
    use crate::app_state::HotbarAssignmentState;

    // Extract the values we need to avoid borrow conflicts
    match &app.panels.hotbar_assignment {
        HotbarAssignmentState::ChoosingBlock {
            hotbar_slot,
            available_blocks,
            selected_block,
        } => {
            let slot = *hotbar_slot;
            let blocks = available_blocks.clone();
            let selected = *selected_block;
            render_block_picker_modal(f, app, slot, &blocks, selected);
        }
        HotbarAssignmentState::None => {
            // No modal to render
        }
    }
}

/// Render the block picker modal for hotbar assignment
fn render_block_picker_modal(
    f: &mut Frame,
    app: &mut App,
    hotbar_slot: usize,
    available_blocks: &[lithicrivers_core::components::ItemKind],
    selected_block: usize,
) {
    use lithicrivers_core::components::itemkind_name;
    use ratatui::{
        style::Modifier,
        text::{Line, Span},
        widgets::{Clear, List, ListItem},
    };

    // Create modal area (centered, 50% width, 40% height)
    let area = centered_rect(50, 40, f.size());

    // Clear the background
    f.render_widget(Clear, area);

    // Main modal block
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Choose Block for F{} ", hotbar_slot + 1))
        .title_alignment(Alignment::Center)
        .style(Style::default());

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Create list of available blocks
    let block_items: Vec<ListItem> = available_blocks
        .iter()
        .enumerate()
        .map(|(i, &block_kind)| {
            let is_selected = selected_block == i;
            let style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            // Show block icon and name using actual sprite
            let sprite_name = lithicrivers_core::components::itemkind_sprite_name(block_kind);
            let icon = if let Some(slash_pos) = sprite_name.find('/') {
                let (category, name) = sprite_name.split_at(slash_pos);
                let name = &name[1..]; // Remove the '/'
                let sprite_data = app.core.sprite_loader.load_sprite(name, category);
                // Get the first character from the 1x1 sprite (sprites[0])
                sprite_data
                    .sprites
                    .get(0)
                    .and_then(|s| s.chars().next())
                    .unwrap_or('?')
            } else {
                '?'
            };
            let text = format!("{} {}", icon, itemkind_name(block_kind));

            ListItem::new(Line::from(Span::styled(text, style)))
        })
        .collect();

    let list =
        List::new(block_items).highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_widget(list, inner);

    // Controls at bottom
    let controls = Paragraph::new("↑↓: Select | Enter: Assign | Esc: Cancel")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));

    if inner.height > 1 {
        let controls_area = Rect {
            x: inner.x,
            y: inner.y + inner.height - 1,
            width: inner.width,
            height: 1,
        };
        f.render_widget(controls, controls_area);
    }
}

// Helper function to get player's inventory as a HashMap

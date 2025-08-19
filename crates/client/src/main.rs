use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseEvent,
        MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs, Wrap},
    Frame, Terminal,
};
use rust_embed::RustEmbed;
use std::{error::Error, io, time::Duration};

#[derive(RustEmbed)]
#[folder = "assets/"]
struct EmbeddedAssets;

use lithicrivers_core::Game;
mod audio;
mod sprite_loader;
use crate::sprite_loader::{
    sprite_block_for_entity, sprite_block_for_fluid, sprite_block_for_tile, Scale, SpriteLoader,
};

use lithicrivers_core::components::{Inventory as InvComp, ItemKind};
use lithicrivers_core::model::body::{Body, BodyPart, BodyPartState, BodyPartType};

struct App {
    game: Game,
    sprite_loader: SpriteLoader,
    should_quit: bool,
    // UI state: remember bottom menu rect for click handling
    bottom_menu_rect: Option<Rect>,
    menu_index: usize,
    scale: Scale,
    audio: audio::AudioManager,
    // Credits panel state
    credits_text: String,
    credits_scroll: u16,
}

fn render_help_panel(f: &mut Frame, _app: &mut App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(Span::styled(
        "Controls",
        Style::default().fg(Color::Cyan),
    )));
    lines.push(Line::from(""));
    // Movement
    lines.push(Line::from(Span::raw("Movement (Numpad):")));
    lines.push(Line::from(Span::raw("  7 8 9  - diagonals/cardinals")));
    lines.push(Line::from(Span::raw("  4 5 6  - 5 to wait")));
    lines.push(Line::from(Span::raw("  1 2 3")));
    lines.push(Line::from(""));
    // Vertical
    lines.push(Line::from(Span::raw("Vertical movement:")));
    lines.push(Line::from(Span::raw("  <  - move up a Z-level")));
    lines.push(Line::from(Span::raw("  >  - move down a Z-level")));
    lines.push(Line::from(Span::raw(
        "  PageUp/PageDown - change viewed Z slice",
    )));
    lines.push(Line::from(""));
    // Actions
    lines.push(Line::from(Span::raw("Actions:")));
    lines.push(Line::from(Span::raw("  m  - mine (plays SFX on success)")));
    lines.push(Line::from(""));
    // Zoom
    lines.push(Line::from(Span::raw("Zoom:")));
    lines.push(Line::from(Span::raw("  =/+ - zoom in")));
    lines.push(Line::from(Span::raw("  -   - zoom out")));
    lines.push(Line::from(Span::raw("  0   - reset zoom")));
    lines.push(Line::from(""));
    // Save/Load
    lines.push(Line::from(Span::raw("Save/Load:")));
    lines.push(Line::from(Span::raw("  S - save to save.json")));
    lines.push(Line::from(Span::raw("  L - load from save.json")));
    lines.push(Line::from(""));
    // Menu
    lines.push(Line::from(Span::raw("Menu navigation:")));
    lines.push(Line::from(Span::raw(
        "  Left/Right - switch tabs (World, Body, Inventory, Menu, Help, Quit)",
    )));
    lines.push(Line::from(Span::raw(
        "  Enter/Space - activate selected tab",
    )));
    lines.push(Line::from(Span::raw("  Mouse - click tab labels")));
    lines.push(Line::from(""));
    // Quit
    lines.push(Line::from(Span::raw("General:")));
    lines.push(Line::from(Span::raw("  q - quit")));

    let block = Block::default().borders(Borders::ALL).title("Help");
    let inner = block.inner(area);
    let p = Paragraph::new(lines).alignment(Alignment::Left);
    f.render_widget(p, inner);
    f.render_widget(block, area);
}

fn render_credits_panel(f: &mut Frame, app: &mut App, area: Rect) {
    // Build a scrollable paragraph from preloaded embedded text
    let block = Block::default().borders(Borders::ALL).title("Credits");
    let inner = block.inner(area);
    let para = Paragraph::new(app.credits_text.clone())
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false })
        .scroll((app.credits_scroll, 0));
    f.render_widget(para, inner);
    f.render_widget(block, area);
}

impl App {
    fn new() -> App {
        // Initialize game and sprite loader
        let game = Game::new(12345);
        let mut sprite_loader = SpriteLoader::new(None);
        // Preload all assets to eliminate runtime I/O during rendering
        sprite_loader.preload_all();

        // Initialize audio manager
        let mut audio = audio::AudioManager::new().unwrap_or_else(|e| {
            panic!("Failed to initialize audio: {}", e);
        });

        // Set volumes
        audio.set_music_volume(0.60);
        audio.set_sfx_volume(0.35);

        // Register and play the opening music
        let music_track = audio::AudioTrack::new("crates/client/assets/sound/music/opening.mp3");
        audio.register_music("opening", music_track);
        if let Err(e) = audio.play_music("opening") {
            panic!("Failed to play music: {}", e);
        }

        let wood_crack =
            audio::AudioTrack::new("crates/client/assets/sound/effects/wood_crack.mp3");
        audio.register_sound_effect("wood_crack", wood_crack);

        // Build credits text from embedded config files
        let version = EmbeddedAssets::get("config/VERSION")
            .map(|d| String::from_utf8_lossy(&d.data).to_string())
            .unwrap_or_else(|| "(missing VERSION)".to_string());
        let app_id = EmbeddedAssets::get("config/STEAM_APP_ID")
            .map(|d| String::from_utf8_lossy(&d.data).to_string())
            .unwrap_or_else(|| "(missing STEAM_APP_ID)".to_string());
        let credits_body = EmbeddedAssets::get("config/credits.txt")
            .map(|d| String::from_utf8_lossy(&d.data).to_string())
            .unwrap_or_else(|| "(missing credits.txt)".to_string());
        let credits_text = format!(
            "Version: {}\nSTEAM_APP_ID: {}\n\n{}",
            version.trim(),
            app_id.trim(),
            credits_body
        );

        App {
            game,
            sprite_loader,
            should_quit: false,
            bottom_menu_rect: None,
            audio,
            menu_index: 0,
            scale: Scale::Small,
            credits_text,
            credits_scroll: 0,
        }
    }

    fn on_tick(&mut self) {
        // Turn-based: do not auto-tick. Ticks only occur on player actions in handle_input().
    }

    fn snap_view_to_player_z(&mut self) {
        if let Some(e) = self.game.res.player_entity {
            if let Ok(pos) = self
                .game
                .world
                .get::<&lithicrivers_core::components::Position>(e)
            {
                self.game.res.view_z = pos.z;
            }
        }
    }

    fn handle_input(&mut self, key: KeyCode) -> Result<(), Box<dyn Error>> {
        match key {
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            // Activate selected menu by Enter/Space
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.activate_menu();
            }
            // Cycle menu with left/right (7 tabs)
            KeyCode::Left => {
                if self.menu_index == 0 {
                    self.menu_index = 6;
                } else {
                    self.menu_index -= 1;
                }
            }
            KeyCode::Right => {
                self.menu_index = (self.menu_index + 1) % 7;
            }
            // Mining
            KeyCode::Char('m') => {
                self.game.queue_mine();
                let mining_success = self.game.tick();
                if mining_success {
                    let _ = self.audio.play_sound_effect("wood_crack");
                }
            }
            // Movement using numpad keys (cardinal + diagonal)
            KeyCode::Char('8') => {
                self.game.queue_player_move(0, -1);
                self.game.tick();
                self.snap_view_to_player_z();
            }
            KeyCode::Char('2') => {
                self.game.queue_player_move(0, 1);
                self.game.tick();
                self.snap_view_to_player_z();
            }
            KeyCode::Char('4') => {
                self.game.queue_player_move(-1, 0);
                self.game.tick();
                self.snap_view_to_player_z();
            }
            KeyCode::Char('6') => {
                self.game.queue_player_move(1, 0);
                self.game.tick();
                self.snap_view_to_player_z();
            }
            KeyCode::Char('7') => {
                self.game.queue_player_move(-1, -1);
                self.game.tick();
                self.snap_view_to_player_z();
            }
            KeyCode::Char('9') => {
                self.game.queue_player_move(1, -1);
                self.game.tick();
                self.snap_view_to_player_z();
            }
            KeyCode::Char('1') => {
                self.game.queue_player_move(-1, 1);
                self.game.tick();
                self.snap_view_to_player_z();
            }
            KeyCode::Char('3') => {
                self.game.queue_player_move(1, 1);
                self.game.tick();
                self.snap_view_to_player_z();
            }
            KeyCode::Char('5') => {
                self.game.queue_player_move(0, 0);
                self.game.tick();
            }
            // Z-level viewing: PageUp/PageDown change viewed slice
            KeyCode::PageUp => {
                self.game.res.view_z = self.game.res.view_z.saturating_add(1);
            }
            KeyCode::PageDown => {
                self.game.res.view_z = self.game.res.view_z.saturating_sub(1);
            }
            // Vertical movement: '<' up, '>' down (like DF variants)
            KeyCode::Char('<') => {
                self.game.queue_player_move_z(1);
                self.game.tick();
                self.snap_view_to_player_z();
            }
            KeyCode::Char('>') => {
                self.game.queue_player_move_z(-1);
                self.game.tick();
                self.snap_view_to_player_z();
            }
            // Zoom controls: '=' zoom in, '-' zoom out, '0' reset
            KeyCode::Char('=') | KeyCode::Char('+') => {
                self.scale = match self.scale {
                    Scale::Small => Scale::Medium,
                    Scale::Medium => Scale::Large,
                    Scale::Large => Scale::Large,
                };
            }
            KeyCode::Char('-') => {
                self.scale = match self.scale {
                    Scale::Large => Scale::Medium,
                    Scale::Medium => Scale::Small,
                    Scale::Small => Scale::Small,
                };
            }
            KeyCode::Char('0') => {
                self.scale = Scale::Small;
            }
            // Credits panel scrolling
            KeyCode::Up => {
                if self.menu_index == 5 {
                    self.credits_scroll = self.credits_scroll.saturating_sub(1);
                }
            }
            KeyCode::Down => {
                if self.menu_index == 5 {
                    self.credits_scroll = self.credits_scroll.saturating_add(1);
                }
            }
            KeyCode::PageUp => {
                if self.menu_index == 5 {
                    self.credits_scroll = self.credits_scroll.saturating_sub(10);
                } else {
                    self.game.res.view_z = self.game.res.view_z.saturating_add(1);
                }
            }
            KeyCode::PageDown => {
                if self.menu_index == 5 {
                    self.credits_scroll = self.credits_scroll.saturating_add(10);
                } else {
                    self.game.res.view_z = self.game.res.view_z.saturating_sub(1);
                }
            }
            // Save/Load (debug): 'S' to save JSON, 'L' to load JSON
            KeyCode::Char('S') => {
                self.game
                    .save_json("save.json")
                    .expect("Save failed: JSON serialization error");
                self.game.res.log("Saved to save.json");
            }
            KeyCode::Char('L') => {
                self.game
                    .load_json("save.json")
                    .expect("Load failed: JSON deserialization error");
                self.game.res.log("Loaded from save.json");
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_mouse(&mut self, me: MouseEvent) -> Result<(), Box<dyn Error>> {
        if let MouseEventKind::Down(_btn) = me.kind {
            if let Some(rect) = self.bottom_menu_rect {
                // Convert to u16 to i32 safely
                let mx = me.column as i32;
                let my = me.row as i32;
                let rx = rect.x as i32;
                let ry = rect.y as i32;
                let rw = rect.width as i32;
                let rh = rect.height as i32;
                if mx >= rx && mx < rx + rw && my >= ry && my < ry + rh {
                    // Map click to tab index (7 tabs)
                    let seg = rw / 7;
                    let relx = mx - rx;
                    self.menu_index = if relx < seg {
                        0
                    } else if relx < seg * 2 {
                        1
                    } else if relx < seg * 3 {
                        2
                    } else if relx < seg * 4 {
                        3
                    } else if relx < seg * 5 {
                        4
                    } else if relx < seg * 6 {
                        5
                    } else {
                        6
                    };
                    self.activate_menu();
                }
            }
        }
        Ok(())
    }

    fn activate_menu(&mut self) {
        match self.menu_index {
            0 => {
                // World (already active view)
                self.game.res.log("World map active");
            }
            1 => {
                // Body
                self.game.res.log("Body panel active");
            }
            2 => {
                // Inventory (placeholder)
                self.game.res.log("Inventory panel (WIP)");
            }
            3 => {
                // Menu
                self.game
                    .res
                    .log("Menu panel active (press S to Save, L to Load)");
            }
            4 => {
                // Help
                self.game.res.log("Help panel active");
            }
            5 => {
                // Credits
                self.game
                    .res
                    .log("Credits panel active (Up/Down to scroll)");
            }
            _ => {
                // Quit
                self.should_quit = true;
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
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
    let mut app = App::new();
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

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<(), Box<dyn Error>> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if crossterm::event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Press {
                        app.handle_input(key.code)?;
                    }
                }
                Event::Mouse(me) => {
                    app.handle_mouse(me)?;
                }
                _ => {}
            }
        }

        app.on_tick();

        if app.should_quit {
            return Ok(());
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let root_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Length(1), // Title line
            Constraint::Min(0),    // Main area (map + inventory)
            Constraint::Length(5), // Message log
            Constraint::Length(3), // Bottom menu bar (needs 3 for borders + content)
        ])
        .split(f.size());

    // Title
    let title = Paragraph::new("LithicRivers (Ratatui Client)")
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center);
    f.render_widget(title, root_chunks[0]);

    // Main area depends on selected tab
    if app.menu_index == 0 {
        // World: map with inventory sidebar
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(20), Constraint::Length(24)])
            .split(root_chunks[1]);
        render_game_view(f, app, main_chunks[0]);
        render_inventory_panel(f, app, main_chunks[1]);
    } else if app.menu_index == 1 {
        // Body: fullscreen body panel
        render_body_panel(f, app, root_chunks[1]);
    } else if app.menu_index == 2 {
        // Inventory: fullscreen inventory panel
        render_inventory_panel(f, app, root_chunks[1]);
    } else if app.menu_index == 3 {
        // Menu: save/load panel
        render_menu_panel(f, app, root_chunks[1]);
    } else if app.menu_index == 4 {
        // Help: controls
        render_help_panel(f, app, root_chunks[1]);
    } else if app.menu_index == 5 {
        // Credits: scrollable
        render_credits_panel(f, app, root_chunks[1]);
    } else {
        // Quit selected: do nothing special here; run loop will exit
    }

    // Message log
    render_message_log(f, app, root_chunks[2]);

    // Bottom menu bar
    render_bottom_menu(f, app, root_chunks[3]);
}

fn render_game_view(f: &mut Frame, app: &mut App, area: Rect) {
    // Get the game view from the core
    let view = app.game.build_view();

    // Create the game display text
    let mut lines = Vec::new();

    // Render the map to exactly the panel's area
    let view_h = view.map_lines.len();
    let view_w = if view_h > 0 {
        view.map_lines[0].len()
    } else {
        0
    };
    let target_cols = area.width as usize;
    let target_rows = area.height as usize;

    // Dimensions of the provided view window
    let view_h = view_h;
    let view_w = view_w;

    // Sync world generation Z with current view slice
    app.game.res.world.set_generation_z(app.game.res.view_z);

    // Compute world-space bounds for current viewport and prefetch chunks
    let left = view.player_pos.x - (target_cols as i32 / 2);
    let top = view.player_pos.y - (target_rows as i32 / 2);
    let right = left + target_cols as i32 - 1;
    let bottom = top + target_rows as i32 - 1;
    app.game.res.world.prefetch_rect(left, top, right, bottom);

    for row in 0..target_rows {
        let mut spans = Vec::with_capacity(target_cols);
        for col in 0..target_cols {
            let world_x = left + col as i32;
            let world_y = top + row as i32;

            // Base tile color/glyph
            let tile_kind = app.game.res.world.get_tile_cached(world_x, world_y);

            // Overlay from fluids/entities when within original view bounds
            let rel_x = (world_x - (view.player_pos.x - (view_w as i32 / 2))) as isize;
            let rel_y = (world_y - (view.player_pos.y - (view_h as i32 / 2))) as isize;

            // Check fluids first
            let fluid_pos = lithicrivers_core::components::Position {
                x: world_x,
                y: world_y,
                z: app.game.res.view_z,
            };
            if let Some(fluid) = app.game.res.fluids.get_fluid(fluid_pos) {
                let (block, color) =
                    sprite_block_for_fluid(&mut app.sprite_loader, fluid.fluid_type, app.scale)
                        .unwrap_or_else(|| {
                            panic!(
                                "Could not find sprite for fluid type: {:?}",
                                fluid.fluid_type
                            )
                        });
                let ch = block.chars().next().unwrap_or(' ');
                spans.push(Span::styled(ch.to_string(), Style::default().fg(color)));
                continue;
            }

            // If within the original view window, use its overlay character for entities
            let mut used_overlay = false;
            if rel_x >= 0 && rel_y >= 0 && (rel_y as usize) < view_h && (rel_x as usize) < view_w {
                let ch = view.map_lines[rel_y as usize]
                    .chars()
                    .nth(rel_x as usize)
                    .unwrap_or(' ');
                if ch != ' ' {
                    let (block, color) =
                        sprite_block_for_entity(&mut app.sprite_loader, ch, app.scale);
                    let ech = block.chars().next().unwrap_or(' ');
                    spans.push(Span::styled(ech.to_string(), Style::default().fg(color)));
                    used_overlay = true;
                }
            }

            if !used_overlay {
                let (block, color) =
                    sprite_block_for_tile(&mut app.sprite_loader, tile_kind, app.scale)
                        .unwrap_or_else(|| {
                            panic!("Could not find sprite for tile kind: {:?}", tile_kind)
                        });
                let ch = block.chars().next().unwrap_or(' ');
                spans.push(Span::styled(ch.to_string(), Style::default().fg(color)));
            }
        }
        lines.push(Line::from(spans));
    }

    let paragraph = Paragraph::new(lines)
        .block(Block::default())
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

fn render_inventory_panel(f: &mut Frame, app: &mut App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();
    if let Some(e) = app.game.res.player_entity {
        if let Ok(inv) = app.game.world.get::<&InvComp>(e) {
            if inv.slots.is_empty() {
                lines.push(Line::from(Span::raw("(Empty)")));
            } else {
                for s in &inv.slots {
                    lines.push(Line::from(Span::raw(format!(
                        "{} x{}",
                        kind_name(s.kind),
                        s.qty
                    ))));
                }
            }
        } else {
            lines.push(Line::from(Span::raw("(No Inventory component)")));
        }
    } else {
        lines.push(Line::from(Span::raw("(No player)")));
    }
    let content = Paragraph::new(lines)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Inventory")
                .style(Style::default().fg(Color::White)),
        );
    f.render_widget(content, area);
}

fn render_message_log(f: &mut Frame, app: &mut App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();
    let start = if app.game.res.messages.len() > area.height as usize {
        app.game.res.messages.len() - area.height as usize
    } else {
        0
    };
    for msg in app.game.res.messages.iter().skip(start) {
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
    app.bottom_menu_rect = Some(area);
    let titles = vec![
        Span::styled(" World ", Style::default().fg(Color::Green)),
        Span::styled(" Body ", Style::default().fg(Color::LightBlue)),
        Span::styled(" Inventory ", Style::default().fg(Color::Yellow)),
        Span::styled(" Menu ", Style::default().fg(Color::Magenta)),
        Span::styled(" Help ", Style::default().fg(Color::White)),
        Span::styled(" Credits ", Style::default().fg(Color::Gray)),
        Span::styled(" Quit ", Style::default().fg(Color::Red)),
    ];
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Menu"))
        .select(app.menu_index)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Cyan));
    f.render_widget(tabs, area);
}

fn render_menu_panel(f: &mut Frame, app: &mut App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(Span::styled(
        "Game Menu",
        Style::default().fg(Color::Cyan),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::raw("S - Save to save.json")));
    lines.push(Line::from(Span::raw("L - Load from save.json")));
    lines.push(Line::from(Span::raw("Q - Quit")));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::raw(format!(
        "Seed: {}",
        app.game.res.seed
    ))));
    lines.push(Line::from(Span::raw(format!(
        "Tick: {}",
        app.game.res.gametick
    ))));
    lines.push(Line::from(Span::raw(format!(
        "View Z: {}",
        app.game.res.view_z
    ))));
    if let Some(e) = app.game.res.player_entity {
        if let Ok(pos) = app
            .game
            .world
            .get::<&lithicrivers_core::components::Position>(e)
        {
            lines.push(Line::from(Span::raw(format!(
                "Player: ({}, {}, {})",
                pos.x, pos.y, pos.z
            ))));
        }
    }
    let block = Block::default().borders(Borders::ALL).title("Menu");
    let inner = block.inner(area);
    let p = Paragraph::new(lines).alignment(Alignment::Left);
    f.render_widget(p, inner);
    f.render_widget(block, area);
}

fn render_body_panel(f: &mut Frame, app: &mut App, area: Rect) {
    // Split area: left ASCII overview, right list
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(28), Constraint::Min(20)])
        .split(area);

    // Build from ECS
    let mut list_lines: Vec<Line> = Vec::new();
    let mut ascii_lines_opt: Option<Vec<Line<'static>>> = None;
    if let Some(e) = app.game.res.player_entity {
        if let Ok(body) = app.game.world.get::<&Body>(e) {
            // Build list
            let mut parts: Vec<&BodyPart> = body.parts.values().collect();
            parts.sort_by_key(|p| p.part_type as i32);
            for p in parts {
                let (label, color) = match p.state {
                    BodyPartState::Missing => ("Missing", Color::DarkGray),
                    BodyPartState::Damaged => ("Damaged", Color::Yellow),
                    BodyPartState::Functional => ("Functional", Color::Green),
                    BodyPartState::Enhanced => ("Enhanced", Color::Cyan),
                };
                list_lines.push(Line::from(Span::styled(
                    format!("{:>9}: {}", p.name, label),
                    Style::default().fg(color),
                )));
            }
            // Build ASCII from borrowed body
            ascii_lines_opt = Some(build_body_ascii(&*body));
        }
    }
    if list_lines.is_empty() {
        list_lines.push(Line::from(Span::raw("(No Body data)")));
    }

    // Left: ASCII overview
    let ascii_block = Block::default().borders(Borders::ALL).title("Body");
    let ascii_inner = ascii_block.inner(chunks[0]);
    let ascii_lines = ascii_lines_opt.unwrap_or_else(|| vec![Line::from(Span::raw("(No Body)"))]);
    let ascii_para = Paragraph::new(ascii_lines).alignment(Alignment::Left);
    f.render_widget(ascii_para, ascii_inner);
    f.render_widget(ascii_block, chunks[0]);

    // Right: textual list
    let list_para = Paragraph::new(list_lines)
        .alignment(Alignment::Left)
        .block(Block::default().borders(Borders::ALL).title("Parts"));
    f.render_widget(list_para, chunks[1]);
}

fn build_body_ascii(body: &Body) -> Vec<Line<'static>> {
    // Simple 13x13 schematic using markers for parts:
    // H head, X torso, A/a arms, L/l legs, space background
    let art = [
        "      HHH     ",
        "     HHHHH    ",
        "      HHH     ",
        "   A  XXX  a  ",
        "  A  XXXXX  a ",
        " A   XXXXX   a",
        "     XXXXX    ",
        "     XXXXX    ",
        "     XX XX    ",
        "     L   l    ",
        "     L   l    ",
        "     L   l    ",
        "    L     l   ",
    ];

    // Helper to get state color by marker
    let color_for = |marker: char| -> Color {
        let (part_type, present) = match marker {
            'H' => (BodyPartType::Head, true),
            'X' => (BodyPartType::Torso, true),
            'A' => (BodyPartType::LeftArm, true),
            'a' => (BodyPartType::RightArm, true),
            'L' => (BodyPartType::LeftLeg, true),
            'l' => (BodyPartType::RightLeg, true),
            _ => (BodyPartType::Head, false),
        };
        if !present {
            return Color::DarkGray;
        }
        let state = body
            .parts
            .get(&part_type)
            .map(|p| p.state)
            .unwrap_or(BodyPartState::Missing);
        match state {
            BodyPartState::Missing => Color::Black,
            BodyPartState::Damaged => Color::Red,
            BodyPartState::Functional => Color::Green,
            BodyPartState::Enhanced => Color::Cyan,
        }
    };

    let mut out: Vec<Line> = Vec::new();
    for row in art {
        let mut spans: Vec<Span> = Vec::new();
        for ch in row.chars() {
            if ch == ' ' {
                spans.push(Span::raw(" "));
            } else {
                let color = color_for(ch);
                spans.push(Span::styled("█", Style::default().fg(color)));
            }
        }
        out.push(Line::from(spans));
    }
    out
}

fn kind_name(kind: ItemKind) -> &'static str {
    match kind {
        ItemKind::Wood => "Wood",
        ItemKind::Acorn => "Acorn",
        ItemKind::Stick => "Stick",
    }
}

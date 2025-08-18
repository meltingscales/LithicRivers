use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseEvent, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap, Tabs, Gauge, Clear},
    Frame, Terminal,
};
use std::{
    error::Error,
    io,
    time::Duration,
};

use lithicrivers_core::Game;
mod sprite_loader;
use crate::sprite_loader::{sprite_for_fluid, sprite_for_tile, SpriteLoader, color_for_entity};

#[derive(Debug, Default, Clone, Copy)]
struct BodyPart<'a> {
    name: &'a str,
    hp: f32, // 0.0 - 1.0
}

struct App {
    game: Game,
    sprite_loader: SpriteLoader,
    should_quit: bool,
    // UI state: remember bottom menu rect for click handling
    bottom_menu_rect: Option<Rect>,
    menu_index: usize,
    body_parts: Vec<BodyPart<'static>>, // temporary demo data
}

impl App {
    fn new() -> App {
        // Initialize game and sprite loader
        let game = Game::new(12345);
        let mut sprite_loader = SpriteLoader::new(None);
        // Preload all assets to eliminate runtime I/O during rendering
        sprite_loader.preload_all();
        App {
            game,
            sprite_loader,
            should_quit: false,
            bottom_menu_rect: None,
            menu_index: 0,
            body_parts: vec![
                BodyPart { name: "Head", hp: 0.7 },
                BodyPart { name: "Torso", hp: 0.9 },
                BodyPart { name: "Left Arm", hp: 0.5 },
                BodyPart { name: "Right Arm", hp: 0.85 },
                BodyPart { name: "Left Leg", hp: 0.6 },
                BodyPart { name: "Right Leg", hp: 0.95 },
            ],
        }
    }

    fn on_tick(&mut self) {
        // Turn-based: do not auto-tick. Ticks only occur on player actions in handle_input().
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
            // Cycle menu with left/right (4 tabs)
            KeyCode::Left => {
                if self.menu_index == 0 { self.menu_index = 3; } else { self.menu_index -= 1; }
            }
            KeyCode::Right => {
                self.menu_index = (self.menu_index + 1) % 4;
            }
            // Mining
            KeyCode::Char('m') => {
                self.game.queue_mine();
                self.game.tick();
            }
            // Movement using numpad keys (cardinal + diagonal)
            KeyCode::Char('8') => {
                self.game.queue_player_move(0, -1);
                self.game.tick();
            }
            KeyCode::Char('2') => {
                self.game.queue_player_move(0, 1);
                self.game.tick();
            }
            KeyCode::Char('4') => {
                self.game.queue_player_move(-1, 0);
                self.game.tick();
            }
            KeyCode::Char('6') => {
                self.game.queue_player_move(1, 0);
                self.game.tick();
            }
            KeyCode::Char('7') => {
                self.game.queue_player_move(-1, -1);
                self.game.tick();
            }
            KeyCode::Char('9') => {
                self.game.queue_player_move(1, -1);
                self.game.tick();
            }
            KeyCode::Char('1') => {
                self.game.queue_player_move(-1, 1);
                self.game.tick();
            }
            KeyCode::Char('3') => {
                self.game.queue_player_move(1, 1);
                self.game.tick();
            }
            KeyCode::Char('5') => {
                self.game.queue_player_move(0, 0);
                self.game.tick();
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
                    // Map click to tab index (4 tabs)
                    let seg = rw / 4;
                    let relx = mx - rx;
                    self.menu_index = if relx < seg { 0 } else if relx < seg * 2 { 1 } else if relx < seg * 3 { 2 } else { 3 };
                    self.activate_menu();
                }
            }
        }
        Ok(())
    }

    fn activate_menu(&mut self) {
        match self.menu_index {
            0 => { // World (already active view)
                self.game.res.log("World map active");
            }
            1 => { // Body (placeholder data)
                self.game.res.log("Body panel active");
            }
            2 => { // Inventory (placeholder)
                self.game.res.log("Inventory panel (WIP)");
            }
            _ => { // Quit
                self.should_quit = true;
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
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
            Constraint::Length(1),        // Title line
            Constraint::Min(0),           // Main area (map + inventory)
            Constraint::Length(5),        // Message log
            Constraint::Length(3),        // Bottom menu bar (needs 3 for borders + content)
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
            .constraints([
                Constraint::Min(20),
                Constraint::Length(24),
            ])
            .split(root_chunks[1]);
        render_game_view(f, app, main_chunks[0]);
        render_inventory_panel(f, app, main_chunks[1]);
    } else if app.menu_index == 1 {
        // Body: fullscreen body panel
        render_body_panel(f, app, root_chunks[1]);
    } else if app.menu_index == 2 {
        // Inventory: fullscreen inventory panel
        render_inventory_panel(f, app, root_chunks[1]);
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
    let view_w = if view_h > 0 { view.map_lines[0].len() } else { 0 };
    let target_cols = area.width as usize;
    let target_rows = area.height as usize;

    // Dimensions of the provided view window
    let view_h = view_h;
    let view_w = view_w;

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
            let fluid_pos = lithicrivers_core::components::Position { x: world_x, y: world_y, z: 0 };
            if let Some(fluid) = app.game.res.fluids.get_fluid(fluid_pos) {
                let (glyph, color) = sprite_for_fluid(&mut app.sprite_loader, fluid.fluid_type)
                    .unwrap_or_else(|| panic!("Could not find sprite for fluid type: {:?}", fluid.fluid_type));
                spans.push(Span::styled(glyph.to_string(), Style::default().fg(color)));
                continue;
            }

            // If within the original view window, use its overlay character for entities
            let mut used_overlay = false;
            if rel_x >= 0 && rel_y >= 0 && (rel_y as usize) < view_h && (rel_x as usize) < view_w {
                let ch = view.map_lines[rel_y as usize].chars().nth(rel_x as usize).unwrap_or(' ');
                if ch != ' ' {
                    let entity_color = color_for_entity(&mut app.sprite_loader, ch);
                    spans.push(Span::styled(ch.to_string(), Style::default().fg(entity_color)));
                    used_overlay = true;
                }
            }

            if !used_overlay {
                let (glyph, color) = sprite_for_tile(&mut app.sprite_loader, tile_kind)
                    .unwrap_or_else(|| panic!("Could not find sprite for tile kind: {:?}", tile_kind));
                spans.push(Span::styled(glyph.to_string(), Style::default().fg(color)));
            }
        }
        lines.push(Line::from(spans));
    }

    let paragraph = Paragraph::new(lines)
        .block(Block::default())
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

fn render_inventory_panel(f: &mut Frame, _app: &mut App, area: Rect) {
    let content = Paragraph::new("(Inventory WIP)")
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
    } else { 0 };
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
        Span::styled(" Quit ", Style::default().fg(Color::Red)),
    ];
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Menu"))
        .select(app.menu_index)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Cyan));
    f.render_widget(tabs, area);
}

fn render_body_panel(f: &mut Frame, app: &mut App, area: Rect) {
    // Outer block
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Body")
        .style(Style::default().fg(Color::White));

    let inner = block.inner(area);

    // Layout: one row per body part, fixed height 2-3 each
    let constraints: Vec<Constraint> = (0..app.body_parts.len())
        .map(|_| Constraint::Length(3))
        .collect();
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    // Clear inner and render gauges
    f.render_widget(Clear, inner);
    for (i, part) in app.body_parts.iter().enumerate() {
        let color = if part.hp > 0.7 { Color::Green } else if part.hp > 0.3 { Color::Yellow } else { Color::Red };
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::NONE))
            .gauge_style(Style::default().fg(color))
            .ratio(part.hp as f64)
            .label(format!("{:>9}: {:.0}%", part.name, part.hp * 100.0));
        if i < rows.len() { f.render_widget(gauge, rows[i]); }
    }

    // Render border last
    f.render_widget(block, area);
}


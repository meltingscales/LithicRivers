use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame, Terminal,
};
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};

use lithicrivers_core::Game;
use lithicrivers_core::tiles::TileKind;
use lithicrivers_core::resources::world::CHUNK_SIZE;
mod sprite_loader;
use crate::sprite_loader::{SpriteLoader, SpriteData};
mod palette;
use crate::palette::{color_for_tile, color_for_fluid};

struct App {
    game: Game,
    sprite_loader: SpriteLoader,
    should_quit: bool,
    tick_rate: Duration,
    last_tick: Instant,
}

impl App {
    fn new() -> App {
        App {
            game: Game::new(12345),
            sprite_loader: SpriteLoader::new(None),
            should_quit: false,
            tick_rate: Duration::from_millis(250),
            last_tick: Instant::now(),
        }
    }

    fn on_tick(&mut self) {
        // Auto-tick the game at regular intervals
        if self.last_tick.elapsed() >= self.tick_rate {
            self.game.tick();
            self.last_tick = Instant::now();
        }
    }

    fn handle_input(&mut self, key: KeyCode) -> Result<(), Box<dyn Error>> {
        match key {
            KeyCode::Char('q') => {
                self.should_quit = true;
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
            // Arrow keys as alternative
            KeyCode::Up => {
                self.game.queue_player_move(0, -1);
                self.game.tick();
            }
            KeyCode::Down => {
                self.game.queue_player_move(0, 1);
                self.game.tick();
            }
            KeyCode::Left => {
                self.game.queue_player_move(-1, 0);
                self.game.tick();
            }
            KeyCode::Right => {
                self.game.queue_player_move(1, 0);
                self.game.tick();
            }
            _ => {}
        }
        Ok(())
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

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<(), Box<dyn Error>> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if crossterm::event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.handle_input(key.code)?;
                }
            }
        }

        app.on_tick();

        if app.should_quit {
            return Ok(());
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.size());

    // Title
    let title = Paragraph::new("LithicRivers (Ratatui Client)")
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White)),
        );
    f.render_widget(title, chunks[0]);

    // Game view
    render_game_view(f, app, chunks[1]);

    // Status/Controls
    let controls = Paragraph::new("Controls: Arrow keys or numpad (1-9) to move, 'q' to quit")
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White)),
        );
    f.render_widget(controls, chunks[2]);
}

fn render_game_view<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    // Get the game view from the core
    let view = app.game.build_view();
    
    // Create the game display text
    let mut lines = Vec::new();
    
    // Add game info
    lines.push(Line::from(vec![
        Span::styled(
            format!("Tick: {} | Player: ({}, {})", 
                view.gametick, 
                view.player_pos.x, 
                view.player_pos.y
            ),
            Style::default().fg(Color::White)
        )
    ]));
    lines.push(Line::from("")); // Empty line
    
    // Render the map
    for (y, line) in view.map_lines.iter().enumerate() {
        let mut spans = Vec::new();
        for (x, ch) in line.chars().enumerate() {
            let world_x = view.player_pos.x - (view.map_lines[0].len() as i32 / 2) + x as i32;
            let world_y = view.player_pos.y - (view.map_lines.len() as i32 / 2) + y as i32;
            
            // Get tile info for coloring
            let tile_kind = app.game.res.world.get_tile(world_x, world_y);
            let ratatui_color = tile_kind_to_ratatui_color(tile_kind);
            
            // Check for fluids
            let fluid_pos = lithicrivers_core::components::Position { x: world_x, y: world_y, z: 0 };
            if let Some(fluid) = app.game.res.fluids.get_fluid(fluid_pos) {
                let fluid_color = fluid_type_to_ratatui_color(fluid.fluid_type);
                spans.push(Span::styled(ch.to_string(), Style::default().fg(fluid_color)));
            } else if ch != ' ' {
                // Entity overlay (player, sheep, etc.)
                let entity_color = if ch == '@' { Color::Magenta } 
                                 else if ch == 's' || ch == 'S' { Color::Yellow }
                                 else { Color::White };
                spans.push(Span::styled(ch.to_string(), Style::default().fg(entity_color)));
            } else {
                // Regular tile
                let display_char = match tile_kind {
                    TileKind::Rock => '#',
                    TileKind::Dirt => '.',
                    TileKind::Grass => ',',
                    TileKind::Tree => 'T',
                    TileKind::Air => ' ',
                    _ => '?',
                };
                spans.push(Span::styled(display_char.to_string(), Style::default().fg(ratatui_color)));
            }
        }
        lines.push(Line::from(spans));
    }
    
    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Game World")
                .style(Style::default().fg(Color::White)),
        )
        .wrap(Wrap { trim: false });
    
    f.render_widget(paragraph, area);
}

fn tile_kind_to_ratatui_color(kind: TileKind) -> Color {
    match kind {
        TileKind::Rock => Color::Gray,
        TileKind::Dirt => Color::Rgb(139, 69, 19), // Brown
        TileKind::Grass => Color::Green,
        TileKind::Tree => Color::Rgb(34, 139, 34), // Forest green
        TileKind::Air => Color::White,
        _ => Color::White,
    }
}

fn fluid_type_to_ratatui_color(fluid_type: lithicrivers_core::resources::fluids::FluidType) -> Color {
    use lithicrivers_core::resources::fluids::FluidType;
    match fluid_type {
        FluidType::Water => Color::Blue,
        FluidType::Oil => Color::Rgb(64, 64, 64), // Dark gray
        FluidType::Blood => Color::Red,
        FluidType::Acid => Color::Rgb(255, 255, 0), // Bright yellow
        FluidType::Lava => Color::Rgb(255, 69, 0), // Red-orange
    }
}
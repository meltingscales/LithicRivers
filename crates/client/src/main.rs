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
    time::Duration,
};

use lithicrivers_core::tiles::TileKind;
use lithicrivers_core::Game;
mod sprite_loader;
use crate::sprite_loader::{sprite_for_fluid, sprite_for_tile, SpriteData, SpriteLoader};
mod palette;
use crate::palette::{color_for_fluid, color_for_tile};

struct App {
    game: Game,
    sprite_loader: SpriteLoader,
    should_quit: bool,
}

fn parse_color_string(s: &str) -> Option<Color> {
    // Support #RRGGBB
    let s = s.trim();
    if let Some(hex) = s.strip_prefix('#') {
        if hex.len() == 6 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&hex[0..2], 16),
                u8::from_str_radix(&hex[2..4], 16),
                u8::from_str_radix(&hex[4..6], 16),
            ) {
                return Some(Color::Rgb(r, g, b));
            }
        }
    }
    None
}

fn first_sprite_char(sd: &SpriteData) -> char {
    if let Some(first) = sd.sprites.first() {
        first.chars().next().unwrap_or(' ')
    } else {
        ' '
    }
}


impl App {
    fn new() -> App {
        App {
            game: Game::new(12345),
            sprite_loader: SpriteLoader::new(None),
            should_quit: false,
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

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<(), Box<dyn Error>> {
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

fn ui(f: &mut Frame, app: &mut App) {
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

fn render_game_view(f: &mut Frame, app: &mut App, area: Rect) {
    // Get the game view from the core
    let view = app.game.build_view();

    // Create the game display text
    let mut lines = Vec::new();

    // Add game info
    lines.push(Line::from(vec![Span::styled(
        format!(
            "Tick: {} | Player: ({}, {})",
            view.gametick, view.player_pos.x, view.player_pos.y
        ),
        Style::default().fg(Color::White),
    )]));
    lines.push(Line::from("")); // Empty line

    // Render the map
    for (y, line) in view.map_lines.iter().enumerate() {
        let mut spans = Vec::new();
        for (x, ch) in line.chars().enumerate() {
            let world_x = view.player_pos.x - (view.map_lines[0].len() as i32 / 2) + x as i32;
            let world_y = view.player_pos.y - (view.map_lines.len() as i32 / 2) + y as i32;

            // Get tile info for coloring/sprites
            let tile_kind = app.game.res.world.get_tile(world_x, world_y);
            let ratatui_color = color_for_tile(tile_kind);

            // Check for fluids
            let fluid_pos = lithicrivers_core::components::Position {
                x: world_x,
                y: world_y,
                z: 0,
            };
            if let Some(fluid) = app.game.res.fluids.get_fluid(fluid_pos) {
                // Try sprite for fluid
                let (glyph, color) = sprite_for_fluid(&mut app.sprite_loader, fluid.fluid_type)
                    .unwrap_or_else(|| ('~', color_for_fluid(fluid.fluid_type)));
                spans.push(Span::styled(glyph.to_string(), Style::default().fg(color)));
            } else if ch != ' ' {
                // Entity overlay (player, sheep, etc.)
                // For now, keep simple fallback colors for entities
                let entity_color = if ch == '@' {
                    Color::Magenta
                } else if ch == 's' || ch == 'S' {
                    Color::Yellow
                } else {
                    Color::White
                };
                spans.push(Span::styled(ch.to_string(), Style::default().fg(entity_color)));
            } else {
                // Regular tile
                let (glyph, color) = sprite_for_tile(&mut app.sprite_loader, tile_kind)
                    .unwrap_or_else(|| panic!("Could not find sprite for tile kind: {:?}", tile_kind));
                spans.push(Span::styled(glyph.to_string(), Style::default().fg(color)));
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


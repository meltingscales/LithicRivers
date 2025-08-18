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

// use lithicrivers_core::tiles::TileKind; // Not needed directly here
use lithicrivers_core::Game;
mod sprite_loader;
use crate::sprite_loader::{sprite_for_fluid, sprite_for_tile, SpriteData, SpriteLoader, color_for_entity};

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
        .margin(0)
        .constraints([
            Constraint::Length(1),        // Title line
            Constraint::Min(0),           // World viewport
            Constraint::Length(3),        // Bottom bar with borders
        ])
        .split(f.size());

    // Title
    let title = Paragraph::new("LithicRivers (Ratatui Client)")
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center);
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
                .title("Status & Controls")
                .style(Style::default().fg(Color::White)),
        );
    f.render_widget(controls, chunks[2]);
}

fn render_game_view(f: &mut Frame, app: &mut App, area: Rect) {
    // Get the game view from the core
    let view = app.game.build_view();

    // Create the game display text
    let mut lines = Vec::new();

    // Render the map; height = full area, width = min(area width, provided view width)
    let view_h = view.map_lines.len();
    let view_w = if view_h > 0 { view.map_lines[0].len() } else { 0 };
    let target_cols = std::cmp::min(area.width as usize, view_w);
    let target_rows = area.height as usize;

    // Dimensions of the provided view window
    let view_h = view_h;
    let view_w = view_w;

    for row in 0..target_rows {
        let mut spans = Vec::with_capacity(target_cols);
        for col in 0..target_cols {
            let world_x = view.player_pos.x - (target_cols as i32 / 2) + col as i32;
            let world_y = view.player_pos.y - (target_rows as i32 / 2) + row as i32;

            // Base tile color/glyph
            let tile_kind = app.game.res.world.get_tile(world_x, world_y);

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

    // Center horizontally if we're narrower than available area
    let render_width = target_cols as u16;
    let render_area = if render_width < area.width {
        Rect {
            x: area.x + (area.width - render_width) / 2,
            y: area.y,
            width: render_width,
            height: area.height,
        }
    } else {
        area
    };

    f.render_widget(paragraph, render_area);
}


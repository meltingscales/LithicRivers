use anyhow::Result;
use crossterm::{cursor, event, execute, terminal};
use crossterm::event::{Event, KeyCode};
use ratatui::{backend::CrosstermBackend, Terminal};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::text::{Line, Span};

use lithicrivers_core::Game;

pub struct TuiApp {
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
}

impl TuiApp {
    pub fn new() -> Result<Self> {
        let mut stdout = std::io::stdout();
        execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;
        terminal::enable_raw_mode()?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }

    pub fn teardown(&mut self) -> Result<()> {
        terminal::disable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(),
            terminal::LeaveAlternateScreen,
            cursor::Show
        )?;
        self.terminal.show_cursor()?;
        Ok(())
    }

    pub fn draw_once(&mut self, game: &Game) -> Result<()> {
        let view = game.build_view();
        self.terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(1),
                ])
                .split(size);

            let header = Paragraph::new(Line::from(vec![
                Span::raw("LithicRivers (Rust migration scaffold) | "),
                Span::raw(format!("tick={} ", view.gametick)),
                Span::raw(format!("player=({}, {}, {})", view.player_pos.x, view.player_pos.y, view.player_pos.z)),
            ]))
            .block(Block::default().borders(Borders::ALL).title("Status"));

            let map_text = view.map_lines.join("\n");
            let body = Paragraph::new(map_text)
                .block(Block::default().borders(Borders::ALL).title("Map (press 'q' to quit)"));

            f.render_widget(header, chunks[0]);
            f.render_widget(body, chunks[1]);
        })?;
        Ok(())
    }

    pub fn handle_input(game: &mut Game, timeout_ms: u64) -> Result<bool> {
        if event::poll(std::time::Duration::from_millis(timeout_ms))? {
            if let Event::Key(k) = event::read()? {
                match k.code {
                    KeyCode::Esc | KeyCode::Char('q') => return Ok(true),
                    // Arrow keys
                    KeyCode::Up => game.queue_player_move(0, -1),
                    KeyCode::Down => game.queue_player_move(0, 1),
                    KeyCode::Left => game.queue_player_move(-1, 0),
                    KeyCode::Right => game.queue_player_move(1, 0),
                    // Numpad (reported as chars typically when numlock is on)
                    KeyCode::Char('7') => game.queue_player_move(-1, -1),
                    KeyCode::Char('8') => game.queue_player_move(0, -1),
                    KeyCode::Char('9') => game.queue_player_move(1, -1),
                    KeyCode::Char('4') => game.queue_player_move(-1, 0),
                    KeyCode::Char('5') => { /* wait / no-op */ }
                    KeyCode::Char('6') => game.queue_player_move(1, 0),
                    KeyCode::Char('1') => game.queue_player_move(-1, 1),
                    KeyCode::Char('2') => game.queue_player_move(0, 1),
                    KeyCode::Char('3') => game.queue_player_move(1, 1),
                    // vi-keys as a bonus
                    KeyCode::Char('h') => game.queue_player_move(-1, 0),
                    KeyCode::Char('j') => game.queue_player_move(0, 1),
                    KeyCode::Char('k') => game.queue_player_move(0, -1),
                    KeyCode::Char('l') => game.queue_player_move(1, 0),
                    KeyCode::Char('y') => game.queue_player_move(-1, -1),
                    KeyCode::Char('u') => game.queue_player_move(1, -1),
                    KeyCode::Char('b') => game.queue_player_move(-1, 1),
                    KeyCode::Char('n') => game.queue_player_move(1, 1),
                    _ => {}
                }
            }
        }
        Ok(false)
    }
}

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

    pub fn poll_quit_event(timeout_ms: u64) -> Result<bool> {
        if event::poll(std::time::Duration::from_millis(timeout_ms))? {
            if let Event::Key(k) = event::read()? {
                if k.code == KeyCode::Char('q') { return Ok(true); }
            }
        }
        Ok(false)
    }
}

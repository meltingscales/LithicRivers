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
    zoom: u16, // 1, 2, or 3
}

impl TuiApp {
    pub fn new() -> Result<Self> {
        let mut stdout = std::io::stdout();
        execute!(
            stdout,
            terminal::EnterAlternateScreen,
            cursor::Hide,
            event::EnableMouseCapture
        )?;
        terminal::enable_raw_mode()?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal, zoom: 1 })
    }

    pub fn teardown(&mut self) -> Result<()> {
        terminal::disable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(),
            terminal::LeaveAlternateScreen,
            cursor::Show,
            event::DisableMouseCapture
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
                Span::raw(format!("player=({}, {}, {}) ", view.player_pos.x, view.player_pos.y, view.player_pos.z)),
                Span::raw(format!("zoom={}x", self.zoom)),
            ]))
            .block(Block::default().borders(Borders::ALL).title("Status"));

            // Compute inner area (exclude borders) for scaling calculations
            let body_area = chunks[1];
            let inner_w = body_area.width.saturating_sub(2); // approx inside the block borders
            let inner_h = body_area.height.saturating_sub(2);
            let map_text = build_scaled_view(&view.map_lines, view.player_pos.x, view.player_pos.y, inner_w, inner_h, self.zoom);
            let body = Paragraph::new(map_text)
                .block(Block::default().borders(Borders::ALL).title("Map (press 'q' to quit)"));

            f.render_widget(header, chunks[0]);
            f.render_widget(body, chunks[1]);
        })?;
        Ok(())
    }

    pub fn handle_input(&mut self, game: &mut Game, timeout_ms: u64) -> Result<bool> {
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
                    // Zoom controls
                    KeyCode::Char('+') | KeyCode::Char('=') => {
                        // zoom in
                        if self.zoom < 3 { self.zoom += 1; }
                    }
                    KeyCode::Char('-') => {
                        // zoom out
                        if self.zoom > 1 { self.zoom -= 1; }
                    }
                    _ => {}
                }
            }
        }
        Ok(false)
    }
}

fn build_scaled_view(map_lines: &Vec<String>, center_x: i32, center_y: i32, area_w: u16, area_h: u16, zoom: u16) -> String {
    let zoom = zoom.max(1).min(3);
    let tiles_w = std::cmp::max(1, (area_w as usize) / (zoom as usize));
    let tiles_h = std::cmp::max(1, (area_h as usize) / (zoom as usize));
    if map_lines.is_empty() { return String::new(); }
    let world_h = map_lines.len() as isize;
    let world_w = map_lines[0].chars().count() as isize;

    let cx = center_x as isize;
    let cy = center_y as isize;
    let half_w = (tiles_w as isize) / 2;
    let half_h = (tiles_h as isize) / 2;
    let left = cx - half_w;
    let top = cy - half_h;

    let mut out_lines: Vec<String> = Vec::new();
    for ty in 0..tiles_h as isize {
        // Build one tile row scaled horizontally
        let mut base_row = String::new();
        let wy = top + ty;
        for tx in 0..tiles_w as isize {
            let wx = left + tx;
            let ch = if wx >= 0 && wy >= 0 && wx < world_w && wy < world_h {
                // Safe to index
                let line = &map_lines[wy as usize];
                line.chars().nth(wx as usize).unwrap_or(' ')
            } else {
                ' '
            };
            for _ in 0..zoom { base_row.push(ch); }
        }
        // Clamp to area width
        if base_row.chars().count() > area_w as usize {
            base_row = base_row.chars().take(area_w as usize).collect();
        }
        // Repeat vertically according to zoom
        for _ in 0..zoom {
            if out_lines.len() >= area_h as usize { break; }
            out_lines.push(base_row.clone());
        }
        if out_lines.len() >= area_h as usize { break; }
    }
    // Pad if needed
    while out_lines.len() < area_h as usize {
        out_lines.push(String::new());
    }
    out_lines.join("\n")
}

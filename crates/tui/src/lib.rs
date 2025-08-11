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

#[cfg(test)]
mod tests {
    use super::build_scaled_view;

    fn sample_window() -> Vec<String> {
        // 9x5 window with player '@' at center (x=4,y=2)
        let rows = vec![
            ".........",
            "..###....",
            "....@....",
            "....#....",
            ".........",
        ];
        rows.into_iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn center_crop_keeps_player_visible_zoom1() {
        let window = sample_window();
        // Area slightly smaller than window so we crop
        let out = build_scaled_view(&window, 8, 5, 1);
        assert!(out.contains('@'), "player '@' not visible in zoom1 cropped output:\n{}", out);
    }

    #[test]
    fn scaling_zoom2_keeps_player_visible() {
        let window = sample_window();
        // At zoom=2, tiles = area/2. Choose area such that tiles are 4x3 (smaller than 9x5), forcing center crop
        let out = build_scaled_view(&window, 8, 6, 2);
        assert!(out.contains('@'), "player '@' not visible in zoom2 output:\n{}", out);
    }
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

    pub fn draw_once(&mut self, game: &mut Game) -> Result<()> {
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
            let map_text = build_scaled_view(&view.map_lines, inner_w, inner_h, self.zoom);
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

fn build_scaled_view(map_lines: &Vec<String>, area_w: u16, area_h: u16, zoom: u16) -> String {
    let zoom = zoom.max(1).min(3);
    let tiles_w = std::cmp::max(1, (area_w as usize) / (zoom as usize));
    let tiles_h = std::cmp::max(1, (area_h as usize) / (zoom as usize));
    if map_lines.is_empty() { return String::new(); }
    let world_h = map_lines.len();
    let world_w = map_lines[0].chars().count();
    // Center-crop within the provided window so the player (at window center) stays visible
    let start_x = if world_w > tiles_w { (world_w - tiles_w) / 2 } else { 0 };
    let start_y = if world_h > tiles_h { (world_h - tiles_h) / 2 } else { 0 };
    // We assume map_lines is already a window centered on the player.
    // So we just scale and clamp to the available area.

    let mut out_lines: Vec<String> = Vec::new();
    for ty in 0..tiles_h as usize {
        // Build one tile row scaled horizontally
        let mut base_row = String::new();
        let wy = start_y + ty;
        for tx in 0..tiles_w as usize {
            let wx = start_x + tx;
            let ch = if wy < world_h && wx < world_w {
                let line = &map_lines[wy];
                line.chars().nth(wx).unwrap_or(' ')
            } else { ' ' };
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

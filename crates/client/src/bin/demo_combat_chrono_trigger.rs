use std::error::Error;
use std::io::{self, Stdout};
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Wrap},
    Frame, Terminal,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug, Clone)]
struct Enemy {
    name: String,
    health: u32,
    max_health: u32,
    portrait_seed: (f64, f64, f64), // (cx, cy, scale) for Mandelbrot
}

impl Enemy {
    fn new(name: &str, max_health: u32, seed: (f64, f64, f64)) -> Self {
        Self {
            name: name.to_string(),
            health: max_health,
            max_health,
            portrait_seed: seed,
        }
    }

    fn render_portrait(&self, width: usize, height: usize) -> String {
        let (cx, cy, scale) = self.portrait_seed;
        render_mandelbrot(width, height, cx, cy, scale)
    }

    fn health_percentage(&self) -> u16 {
        ((self.health as f32 / self.max_health as f32) * 100.0) as u16
    }
}

struct App {
    enemies: Vec<Enemy>,
    current_enemy: usize,
}

impl App {
    fn new() -> Self {

        let mut mandel_coords = vec![];
        for _ in 0..10 {
            mandel_coords.push((rand::random::<f64>() * 2.0 - 1.0, rand::random::<f64>() * 2.0 - 1.0, rand::random::<f64>() * 2.0 + 1.0));
        }
        
        // Create 1-5 random enemies with different seeds for variety
        let mut enemies = vec![
            Enemy::new("Gato", 120, mandel_coords[0]),
            Enemy::new("Nu", 180, mandel_coords[1]),
            Enemy::new("Retinite", 250, mandel_coords[2]),
            Enemy::new("Rend", 120, mandel_coords[3]),
            Enemy::new("Mete", 120, mandel_coords[4]),
        ];
        
        // Randomly select 1-5 enemies
        use rand::seq::SliceRandom;
        use rand::thread_rng;
        let mut rng = thread_rng();
        let count = rand::random::<usize>() % 5 + 1;
        enemies.shuffle(&mut rng);
        enemies.truncate(count);

        Self {
            enemies,
            current_enemy: 0,
        }
    }

    fn next_enemy(&mut self) {
        self.current_enemy = (self.current_enemy + 1) % self.enemies.len().max(1);
    }

    fn prev_enemy(&mut self) {
        if self.current_enemy == 0 {
            self.current_enemy = self.enemies.len().saturating_sub(1);
        } else {
            self.current_enemy -= 1;
        }
    }
}

fn main() -> Result<()> {
    let mut terminal = setup_terminal()?;
    let res = run_app(&mut terminal);
    restore_terminal(terminal)?;
    if let Err(err) = res {
        eprintln!("{err:?}");
    }
    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    let mut app = App::new();

    loop {
        terminal.draw(|f| ui(f, &app))?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Right | KeyCode::Char('d') => app.next_enemy(),
                    KeyCode::Left | KeyCode::Char('a') => app.prev_enemy(),
                    KeyCode::Char('h') => {
                        // Simulate hitting the current enemy
                        if let Some(enemy) = app.enemies.get_mut(app.current_enemy) {
                            let damage = rand::random::<u32>() % 20 + 10;
                            enemy.health = enemy.health.saturating_sub(damage);
                            if enemy.health == 0 {
                                app.enemies.remove(app.current_enemy);
                                if app.current_enemy >= app.enemies.len() && !app.enemies.is_empty() {
                                    app.current_enemy = app.enemies.len() - 1;
                                }
                                if app.enemies.is_empty() {
                                    return Ok(());
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(mut terminal: Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

fn ui(f: &mut Frame, app: &App) {
    let size = f.size();

    // Outer frame
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Chrono Trigger-Style Combat ")
        .title_alignment(Alignment::Center);
    let inner = block.inner(size);
    f.render_widget(block, size);

    if app.enemies.is_empty() {
        let victory = Paragraph::new("Victory!")
            .style(Style::default().fg(Color::Green))
            .alignment(Alignment::Center);
        f.render_widget(victory, inner);
        return;
    }

    // Layout: enemies area + footer
    let chunks = Layout::vertical([Constraint::Min(1), Constraint::Length(3)]).split(inner);

    // Create a row for each enemy
    let enemy_chunks = Layout::horizontal(
        app.enemies
            .iter()
            .map(|_| Constraint::Ratio(1, app.enemies.len() as u32))
            .collect::<Vec<_>>(),
    )
    .split(chunks[0]);

    // Render each enemy
    for (i, (enemy, area)) in app.enemies.iter().zip(enemy_chunks.iter()).enumerate() {
        let is_selected = i == app.current_enemy;
        let border_style = if is_selected {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let enemy_block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title_alignment(Alignment::Center);
        
        let inner_area = enemy_block.inner(*area);
        f.render_widget(enemy_block, *area);

        // Layout for each enemy: portrait on top, name and health below
        let enemy_layout = Layout::vertical([
            Constraint::Length(8),  // Portrait
            Constraint::Length(1),  // Name
            Constraint::Length(1),  // Health bar
            Constraint::Min(1),     // Spacer
        ]).split(inner_area);

        // Render portrait (12x8 as per requirements)
        let portrait = enemy.render_portrait(12, 8);
        let portrait_para = Paragraph::new(portrait)
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .alignment(Alignment::Center);
        f.render_widget(portrait_para, enemy_layout[0]);

        // Render enemy name
        let name_style = if is_selected {
            Style::default().fg(Color::Yellow).bold()
        } else {
            Style::default()
        };
        let name_para = Paragraph::new(Line::from(vec![
            Span::styled(&enemy.name, name_style)
        ])).alignment(Alignment::Center);
        f.render_widget(name_para, enemy_layout[1]);

        // Render health bar
        let health_gauge = Gauge::default()
            .block(Block::default())
            .gauge_style(Style::default()
                .fg(Color::Red)
                .bg(Color::DarkGray)
                .add_modifier(ratatui::style::Modifier::BOLD))
            .ratio(enemy.health_percentage() as f64 / 100.0)
            .label(format!("HP: {}/{} ", enemy.health, enemy.max_health));
        f.render_widget(health_gauge, enemy_layout[2]);
    }

    // Footer with controls
    let controls = Line::from(vec![
        "←/→ or A/D: Select ".into(),
        "H: Hit ".into(),
        "Q: Quit".into(),
    ]);
    let footer = Paragraph::new(controls)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
    f.render_widget(footer, chunks[1]);
}

// Center a rect of exact size (w,h) inside "area".
fn center_rect_exact(w: u16, h: u16, area: Rect) -> Rect {
    let w = w.min(area.width);
    let h = h.min(area.height);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    Rect {
        x,
        y,
        width: w,
        height: h,
    }
}

// Render Mandelbrot set into an ASCII buffer sized (w x h)
fn render_mandelbrot(w: usize, h: usize, cx: f64, cy: f64, scale: f64) -> String {
    let palette: &[char] = &[' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'];
    let max_iter = 60usize;

    // Map pixel to complex plane
    let aspect = w as f64 / h as f64;
    let (span_x, span_y) = if aspect >= 1.0 {
        (scale * aspect, scale)
    } else {
        (scale, scale / aspect)
    };
    let left = cx - span_x / 2.0;
    let top = cy - span_y / 2.0;

    let mut out = String::with_capacity((w + 1) * h);
    for j in 0..h {
        for i in 0..w {
            let x0 = left + (i as f64 / (w - 1).max(1) as f64) * span_x;
            let y0 = top + (j as f64 / (h - 1).max(1) as f64) * span_y;
            let mut x = 0.0;
            let mut y = 0.0;
            let mut it = 0usize;
            while x * x + y * y <= 4.0 && it < max_iter {
                let xt = x * x - y * y + x0;
                y = 2.0 * x * y + y0;
                x = xt;
                it += 1;
            }
            let idx = if it >= max_iter {
                palette.len() - 1
            } else {
                (it * (palette.len() - 1)) / max_iter
            };
            out.push(palette[idx]);
        }
        if j + 1 < h {
            out.push('\n');
        }
    }
    out
}

// Add this to Cargo.toml:
// [dependencies]
// rand = "0.8.5"
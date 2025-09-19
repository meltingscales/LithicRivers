use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, symbols::border, widgets::*};
use std::{
    cmp::max,
    collections::VecDeque,
    fs, io,
    path::Path,
    time::{Duration, Instant},
};

const GLITCH_FRAMES: usize = 10;
const TARGET_FPS: u64 = 30; // ~33ms per frame

struct App {
    images: Vec<String>,
    frame: usize,
    last_tick: Instant,
    should_quit: bool,
    area: Rect,
}

impl App {
    fn new(size: Rect) -> Self {
        let images = build_intro_images(size);
        let area = intro_area(size);
        Self {
            images,
            frame: 0,
            last_tick: Instant::now(),
            should_quit: false,
            area,
        }
    }

    fn on_tick(&mut self) {
        let frame_time = Duration::from_millis(1000 / TARGET_FPS);
        if self.last_tick.elapsed() >= frame_time {
            self.last_tick = Instant::now();
            if self.frame + 1 < self.images.len() {
                self.frame += 1;
            }
        }
    }

    fn on_resize(&mut self, size: Rect) {
        self.images = build_intro_images(size);
        self.frame = self.frame.min(self.images.len().saturating_sub(1));
        self.area = intro_area(size);
    }
}

fn main() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let size = terminal.size()?;
    let mut app = App::new(size);
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        // Input handling and ticking
        if event::poll(Duration::from_millis(5))? {
            match event::read()? {
                Event::Key(KeyEvent { code, kind, .. }) => {
                    if kind == KeyEventKind::Press {
                        match code {
                            KeyCode::Char('q') | KeyCode::Esc => {
                                app.should_quit = true;
                            }
                            _ => {}
                        }
                    }
                }
                Event::Resize(_, _) => {
                    let size = terminal.size()?;
                    app.on_resize(size);
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

fn ui(f: &mut Frame, app: &App) {
    let size = f.size();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::THICK)
        .title(" Intro ")
        .title_alignment(Alignment::Center)
        .border_style(Style::default().fg(Color::LightBlue));

    // Inner and centered content area
    let inner = block.inner(Rect::new(0, 0, size.width, size.height));
    let area = centered_rect(80, 60, inner);

    // Clear content area before drawing
    f.render_widget(Clear, area);

    // Current image frame
    let text = if app.images.is_empty() {
        "".to_string()
    } else {
        app.images[app.frame].clone()
    };

    let paragraph = Paragraph::new(text).block(Block::default().borders(Borders::NONE));
    f.render_widget(paragraph, area);

    // Render border last
    f.render_widget(block, area);
}

fn intro_area(size: Rect) -> Rect {
    let block = Block::default();
    let inner = block.inner(Rect::new(0, 0, size.width, size.height));
    centered_rect(80, 60, inner)
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn build_intro_images(size: Rect) -> Vec<String> {
    let lines = read_boot_lines();

    // Margins akin to Python version
    let top_margin = max(1, size.height as i64 / 6) as u16;
    let left_margin = max(2, size.width as i64 / 12) as u16;
    let viewport_height = size
        .height
        .saturating_sub(top_margin)
        .saturating_sub(2)
        .max(1);
    let available_width = size
        .width
        .saturating_sub(left_margin)
        .saturating_sub(1)
        .max(1);

    // Sliding window buffer of fixed height
    let mut window: VecDeque<String> = VecDeque::with_capacity(viewport_height as usize);
    let mut images: Vec<String> = Vec::new();

    for (line_idx, raw) in lines.iter().enumerate() {
        let mut base_line = raw.clone();
        if base_line.len() as u16 > available_width {
            base_line.truncate(available_width as usize);
        }

        // Build glitch frames for this line
        for i in 0..GLITCH_FRAMES {
            let rate = 0.50 * (1.0 - (i as f32 / GLITCH_FRAMES as f32)) + 0.02; // matches Python falloff

            // Maintain temporary presence of the line during glitching
            if window.len() < window.capacity() {
                window.push_back(base_line.clone());
            }

            // Build buffer with the last line replaced by the corrupted frame
            let mut buffer: Vec<String> = window.iter().cloned().collect();
            if !buffer.is_empty() {
                let seed = (line_idx as u64) << 32 | (i as u64);
                let last = buffer.len() - 1;
                buffer[last] = corrupt_text(&base_line, rate, seed);
            }

            // Left-pad and join
            let padded: Vec<String> = buffer
                .into_iter()
                .map(|mut l| {
                    if l.len() as u16 > available_width {
                        l.truncate(available_width as usize);
                    }
                    while (l.len() as u16) < available_width {
                        l.push(' ');
                    }
                    l
                })
                .collect();
            images.push(padded.join("\n"));

            // Pop to keep the line transient until it settles
            if !window.is_empty() {
                window.pop_back();
            }
        }

        // Finally append the settled clean line and capture a frame
        if window.len() < window.capacity() {
            window.push_back(base_line.clone());
        } else if !window.is_empty() {
            // If at capacity, drop oldest to append newest (keep latest lines visible)
            window.pop_front();
            window.push_back(base_line.clone());
        }

        let padded: Vec<String> = window
            .iter()
            .cloned()
            .map(|mut l| {
                if l.len() as u16 > available_width {
                    l.truncate(available_width as usize);
                }
                while (l.len() as u16) < available_width {
                    l.push(' ');
                }
                l
            })
            .collect();
        images.push(padded.join("\n"));
    }

    // Idle tail ~1.5s on the last image
    if let Some(last) = images.last().cloned() {
        let idle_tail = (TARGET_FPS * 3 / 2) as usize;
        for _ in 0..idle_tail {
            images.push(last.clone());
        }
    }

    images
}

fn read_boot_lines() -> Vec<String> {
    // Resolve to crates/client/assets/config/boot_message.dat regardless of CWD
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("config")
        .join("boot_message.dat");
    match fs::read_to_string(&path) {
        Ok(contents) => contents.lines().map(|s| s.to_string()).collect(),
        Err(_) => panic!("Failed to read boot message from {}", path.display()),
    }
}

// Deterministic text corruption similar to python's corrupt_text(corruption_rate)
fn corrupt_text(line: &str, rate: f32, seed: u64) -> String {
    if rate <= 0.0 {
        return line.to_string();
    }

    let mut out = String::with_capacity(line.len());
    let mut rng = SplitMix64::new(seed ^ 0x9E3779B97F4A7C15);
    for ch in line.chars() {
        // Keep spaces
        if ch == ' ' || ch == '\t' {
            out.push(ch);
            continue;
        }
        let r = rng.next_f32();
        if r < rate {
            // Replace with a visible ASCII glitch char
            let glyphs: &[char] = &[
                '#', '%', '@', '*', '+', '=', 'x', 'X', '?', '!', '/', '\\', '~', '-', '_',
            ];
            let idx = (rng.next_u32() as usize) % glyphs.len();
            out.push(glyphs[idx]);
        } else {
            out.push(ch);
        }
    }
    out
}

// Minimal deterministic RNG (SplitMix64)
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }

    fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }

    fn next_f32(&mut self) -> f32 {
        // [0,1)
        let v = self.next_u32();
        (v as f32) / (u32::MAX as f32 + 1.0)
    }
}

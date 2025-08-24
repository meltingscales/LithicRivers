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
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

struct App {
    idx: usize,
}

impl App {
    fn new() -> Self {
        Self { idx: 0 }
    }

    fn next(&mut self) {
        self.idx = (self.idx + 1) % VIEWPORTS.len();
    }

    fn prev(&mut self) {
        if self.idx == 0 {
            self.idx = VIEWPORTS.len() - 1;
        } else {
            self.idx -= 1;
        }
    }
}

// Hardcoded Mandelbrot viewports (label, width, height, center_re, center_im, scale)
// scale: how many complex units span the shorter dimension; smaller = zoomed in
const VIEWPORTS: &[(&str, usize, usize, f64, f64, f64)] = &[
    ("8x12", 8, 12, -0.5, 0.0, 3.0),
    ("10x10", 10, 10, -0.5, 0.0, 3.0),
    ("12x8", 12, 8, -0.5, 0.0, 3.0),
    ("16x12", 16, 12, -0.75, 0.0, 2.5),
    ("20x14", 20, 14, -0.745, 0.112, 0.04), // a bit zoomed near Seahorse Valley
];

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
                    KeyCode::Right | KeyCode::Char('n') => app.next(),
                    KeyCode::Left | KeyCode::Char('p') => app.prev(),
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
        .title(" Portrait Sprite Size Test ")
        .title_alignment(Alignment::Center);
    let inner = block.inner(size);
    f.render_widget(block, size);

    // Layout: portrait area + footer
    let chunks = Layout::vertical([Constraint::Min(1), Constraint::Length(3)]).split(inner);

    // Render current fractal portrait, centered within chunks[0]
    let (label, pw, ph, cx, cy, scale) = VIEWPORTS[app.idx];

    let portrait_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} | lines:{} cols:{} ", label, ph, pw))
        .title_alignment(Alignment::Center);
    let pb_inner = portrait_block.inner(chunks[0]);
    f.render_widget(portrait_block, chunks[0]);

    let centered = center_rect_exact(pw as u16, ph as u16, pb_inner);

    let buf = render_mandelbrot(pw, ph, cx, cy, scale);

    let portrait_para = Paragraph::new(buf)
        .style(Style::default().fg(Color::White).bg(Color::Black))
        .block(Block::default());

    f.render_widget(portrait_para, centered);

    // Footer with controls
    let footer = Paragraph::new(Line::from(vec!["←/→ or p/n to cycle • q to quit".into()]))
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
    // Maintain aspect ratio: the shorter terminal dimension spans `scale` units
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

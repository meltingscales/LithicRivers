use std::{
    error::Error,
    io::{self, Stdout},
    time::Duration,
};

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Layout},
    style::{Color, Stylize},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

struct App {
    fg_color: Color,
    bg_color: Color,
    show_hex: bool,
}

impl App {
    fn new() -> Self {
        Self {
            fg_color: Color::White,
            bg_color: Color::Black,
            show_hex: false,
        }
    }
    
    fn toggle_hex(&mut self) {
        self.show_hex = !self.show_hex;
    }
    
    fn next_fg_color(&mut self) {
        self.fg_color = match self.fg_color {
            Color::Black => Color::DarkGray,
            Color::DarkGray => Color::Gray,
            Color::Gray => Color::White,
            Color::White => Color::Red,
            Color::Red => Color::Green,
            Color::Green => Color::Yellow,
            Color::Yellow => Color::Blue,
            Color::Blue => Color::Magenta,
            Color::Magenta => Color::Cyan,
            Color::Cyan => Color::LightRed,
            Color::LightRed => Color::LightGreen,
            Color::LightGreen => Color::LightYellow,
            Color::LightYellow => Color::LightBlue,
            Color::LightBlue => Color::LightMagenta,
            Color::LightMagenta => Color::LightCyan,
            _ => Color::Black,
        };
    }
    
    fn next_bg_color(&mut self) {
        self.bg_color = match self.bg_color {
            Color::Black => Color::DarkGray,
            Color::DarkGray => Color::Gray,
            Color::Gray => Color::White,
            Color::White => Color::Red,
            Color::Red => Color::Green,
            Color::Green => Color::Yellow,
            Color::Yellow => Color::Blue,
            Color::Blue => Color::Magenta,
            Color::Magenta => Color::Cyan,
            Color::Cyan => Color::LightRed,
            Color::LightRed => Color::LightGreen,
            Color::LightGreen => Color::LightYellow,
            Color::LightYellow => Color::LightBlue,
            Color::LightBlue => Color::LightMagenta,
            Color::LightMagenta => Color::LightCyan,
            _ => Color::Black,
        };
    }
    
    fn get_color_name(color: Color) -> &'static str {
        match color {
            Color::Black => "Black",
            Color::DarkGray => "DarkGray",
            Color::Gray => "Gray",
            Color::White => "White",
            Color::Red => "Red",
            Color::Green => "Green",
            Color::Yellow => "Yellow",
            Color::Blue => "Blue",
            Color::Magenta => "Magenta",
            Color::Cyan => "Cyan",
            Color::LightRed => "LightRed",
            Color::LightGreen => "LightGreen",
            Color::LightYellow => "LightYellow",
            Color::LightBlue => "LightBlue",
            Color::LightMagenta => "LightMagenta",
            Color::LightCyan => "LightCyan",
            _ => "Custom",
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

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Right => app.next_fg_color(),
                    KeyCode::Left => app.next_bg_color(),
                    KeyCode::Char('h') => app.toggle_hex(),
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
    
    // Main border
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" ASCII Character Set Viewer ")
        .title_alignment(Alignment::Center);
        
    let inner = block.inner(size);
    f.render_widget(block, size);
    
    // Create a grid of characters
    let mut grid = String::new();
    for row in 0..16 {
        for col in 0..16 {
            let code = row * 16 + col;
            if code >= 32 && code <= 126 {  // Printable ASCII range
                if app.show_hex {
                    grid.push_str(&format!("{:02x} ", code));
                } else {
                    if let Some(c) = std::char::from_u32(code as u32) {
                        grid.push(c);
                    } else {
                        grid.push(' ');
                    }
                    grid.push(' ');
                }
            } else {
                grid.push_str("  ");
            }
        }
        grid.push('\n');
    }
    
    // Display the character grid
    let grid_para = Paragraph::new(grid)
        .fg(app.fg_color)
        .bg(app.bg_color)
        .block(Block::default().borders(Borders::NONE));
    
    // Display current colors
    let info = format!(
        "FG: {} | BG: {} | Press 'h' to toggle hex/char view | 'q' to quit",
        App::get_color_name(app.fg_color),
        App::get_color_name(app.bg_color)
    );
    
    let info_para = Paragraph::new(info)
        .block(Block::default().borders(Borders::TOP));
    
    // Layout
    let chunks = Layout::vertical([
        Constraint::Min(16),
        Constraint::Length(3),
    ]).split(inner);
    
    f.render_widget(grid_para, chunks[0]);
    f.render_widget(info_para, chunks[1]);
}

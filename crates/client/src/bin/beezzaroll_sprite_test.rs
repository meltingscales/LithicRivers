use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    symbols::border,
    widgets::*,
};
use std::{io, time::Duration};

#[derive(Debug, Default)]
struct AsciiViewer {
    should_quit: bool,
    fg_color: Color,
    bg_color: Color,
    current_fg: usize,
    current_bg: usize,
    show_hex: bool,
}

impl AsciiViewer {
    fn new() -> Self {
        Self {
            should_quit: false,
            fg_color: Color::White,
            bg_color: Color::Black,
            current_fg: 0,
            current_bg: 0,
            show_hex: false,
        }
    }
    
    fn next_fg_color(&mut self) {
        let colors = Self::get_available_colors();
        self.current_fg = (self.current_fg + 1) % colors.len();
        self.fg_color = colors[self.current_fg];
    }
    
    fn prev_fg_color(&mut self) {
        let colors = Self::get_available_colors();
        self.current_fg = if self.current_fg == 0 {
            colors.len() - 1
        } else {
            self.current_fg - 1
        };
        self.fg_color = colors[self.current_fg];
    }
    
    fn next_bg_color(&mut self) {
        let colors = Self::get_available_colors();
        self.current_bg = (self.current_bg + 1) % colors.len();
        self.bg_color = colors[self.current_bg];
    }
    
    fn prev_bg_color(&mut self) {
        let colors = Self::get_available_colors();
        self.current_bg = if self.current_bg == 0 {
            colors.len() - 1
        } else {
            self.current_bg - 1
        };
        self.bg_color = colors[self.current_bg];
    }
    
    fn get_available_colors() -> Vec<Color> {
        vec![
            Color::Black,
            Color::DarkGray,
            Color::Gray,
            Color::White,
            Color::Red,
            Color::Green,
            Color::Yellow,
            Color::Blue,
            Color::Magenta,
            Color::Cyan,
            Color::LightRed,
            Color::LightGreen,
            Color::LightYellow,
            Color::LightBlue,
            Color::LightMagenta,
            Color::LightCyan,
            Color::DarkRed,
            Color::DarkGreen,
            Color::DarkYellow,
            Color::DarkBlue,
            Color::DarkMagenta,
            Color::DarkCyan,
            Color::Indexed(8),   // Dark gray
            Color::Indexed(15),  // Bright white
            Color::Indexed(16),  // Start of 256-color palette
            Color::Indexed(231), // End of grayscale
            Color::Indexed(196), // Red
            Color::Indexed(46),  // Green
            Color::Indexed(226), // Yellow
            Color::Indexed(21),  // Blue
            Color::Indexed(201), // Magenta
            Color::Indexed(51),  // Cyan
        ]
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
            Color::DarkRed => "DarkRed",
            Color::DarkGreen => "DarkGreen",
            Color::DarkYellow => "DarkYellow",
            Color::DarkBlue => "DarkBlue",
            Color::DarkMagenta => "DarkMagenta",
            Color::DarkCyan => "DarkCyan",
            Color::Indexed(n) => match n {
                8 => "DarkGray (8)",
                15 => "BrightWhite (15)",
                16 => "Color16",
                231 => "White (231)",
                196 => "BrightRed (196)",
                46 => "BrightGreen (46)",
                226 => "BrightYellow (226)",
                21 => "BrightBlue (21)",
                201 => "BrightMagenta (201)",
                51 => "BrightCyan (51)",
                _ => "Custom",
            },
            _ => "Custom",
        }
    }
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = AsciiViewer::new();
    let res = run_app(&mut terminal, app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}")
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: AsciiViewer) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Char('Q') => app.should_quit = true,
                        KeyCode::Right => app.next_fg_color(),
                        KeyCode::Left => app.prev_fg_color(),
                        KeyCode::Up => app.next_bg_color(),
                        KeyCode::Down => app.prev_bg_color(),
                        KeyCode::Char('h') => app.show_hex = !app.show_hex,
                        _ => {}
                    }
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn ui(f: &mut Frame, app: &AsciiViewer) {
    let size = f.size();
    
    // Main border
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::THICK)
        .title(" ASCII Character Set Viewer ")
        .title_alignment(Alignment::Center)
        .border_style(Style::default().fg(Color::LightCyan));
    
    // Layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(10),   // Character display
            Constraint::Length(6), // Controls and info
        ])
        .split(block.inner(size));

    // Title
    let title = Paragraph::new("Beezzaroll's ASCII Character Set")
        .alignment(Alignment::Center)
        .block(Block::default());
    
    // Character grid
    let char_block = Block::default()
        .borders(Borders::ALL)
        .title("Printable ASCII Characters (32-126)")
        .border_style(Style::default().fg(Color::Yellow));
    
    // Create a grid of ASCII characters (32-126)
    let rows = (32..127).collect::<Vec<_>>()
        .chunks(16)
        .map(|chunk| {
            let cells = chunk.iter().map(|&c| {
                let ch = c as char;
                let display = if app.show_hex {
                    format!("{:02X}", c)
                } else {
                    format!(" {}", ch)
                };
                
                // Create a cell with the character and current colors
                let cell_style = Style::default()
                    .fg(app.fg_color)
                    .bg(app.bg_color);
                
                Cell::from(display).style(cell_style)
            });
            
            Row::new(cells).height(1)
        });
    
    let table = Table::new(rows)
        .column_spacing(1)
        .widths(&[Constraint::Length(if app.show_hex { 2 } else { 2 }); 16])
        .style(Style::default())
        .column_spacing(1);
    
    // Info text
    let fg_name = AsciiViewer::get_color_name(app.fg_color);
    let bg_name = AsciiViewer::get_color_name(app.bg_color);
    
    let info_text = format!(
        "Foreground: {} (←/→ to change)\n\
Background: {} (↑/↓ to change)\n\
Press 'H' to toggle hex/char view\n\
Press 'Q' to quit",
        fg_name, bg_name
    );
    
    let info = Paragraph::new(info_text)
        .alignment(Alignment::Left)
        .block(Block::default().borders(Borders::NONE));
    
    // Render everything
    f.render_widget(block, size);
    f.render_widget(title, chunks[0]);
    f.render_widget(char_block, chunks[1]);
    f.render_widget(table, chunks[1]);
    f.render_widget(info, chunks[2]);
}

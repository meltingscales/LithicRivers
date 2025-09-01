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
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

struct App {
    fg_color: Color,
    bg_color: Color,
    show_hex: bool,
    char_set: usize,
}

impl App {
    fn new() -> Self {
        Self {
            fg_color: Color::White,
            bg_color: Color::Black,
            show_hex: false,
            char_set: 0,
        }
    }

    fn toggle_hex(&mut self) {
        self.show_hex = !self.show_hex;
    }

    fn next_char_set(&mut self) {
        self.char_set = (self.char_set + 1) % 4;
    }

    fn get_character_set(&self) -> &'static [char] {
        match self.char_set {
            0 => &[
                '∙', '·', '√', 'ⁿ', '²', '■', ' ', '♠', '♣', '♥', '♦', '♫', '☼', '►', '◄', '↕',
                '‼', '¶', '§', '▬', '↨', '↑', '↓', '→', '←', '∟', '↔', '▲', '▼', '⌂', '⌐', '¬',
                '↓', '→', '←', '∟', '↔', '▲', '▼', ' ', '!', '"', '#', '$', '%', '&', '\'', '(',
                ')', '*', '+', ',', '-', '.', '/', '0', '1', '2', '3', '4', '5', '6', '7', '8',
                '9', ':', ';', '<', '=', '>', '?', '@', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H',
                'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X',
                'Y', 'Z', '[', '\\', ']', '^', '_', '`', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h',
                'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x',
                'y', 'z', '{', '|', '}', '~', '⌂', 'Ç', 'ü', 'é', 'â', 'ä', 'à', 'å', 'ç', 'ê',
                'ë', 'è', 'ï', 'î', 'ì', 'Ä', 'Å', 'É', 'æ', 'Æ', 'ô', 'ö', 'ò', 'û', 'ù', 'ÿ',
                'Ö', 'Ü', '¢', '£', '¥', '₧', 'ƒ', 'á', 'í', 'ó', 'ú', 'ñ', 'Ñ', 'ª', 'º', '¿',
                '!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/', '0',
                '1', '2', '3', '4', '5', '6', '7', '8', '9', ':', ';', '<', '=', '>', '?', '@',
                'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P',
                'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '[', '\\', ']', '^', '_',
            ],
            1 => &[
                '╭', '─', '╮', '│', '╯', '─', '╰', '│', '┏', '━', '┓', '┃', '┛', '━', '┗', '┃',
                '┌', '┬', '┐', '├', '┼', '┤', '└', '┴', '┘', '╱', '╲', '╳', '┼', '─', '│', '┌',
                '┐', '└', '┘', '├', '┤', '┬', '┴', '╔', '╗', '╚', '╝', '╠', '╣', '╦', '╩', '╬',
                '═', '║', '╒', '╤', '╕', '╞', '╪', '╡', '╘', '╧', '╛', '╞', '╪', '╡', '╘', '╧',
                '╛', '╓', '╥', '╖', '╟', '╫', '╢', '╙', '╨', '╜', '╓', '╥', '╖', '╟', '╫', '╢',
                '╙', '╨', '╜', '╒', '╤', '╕', '╞', '╪', '╡', '╘', '╧', '╛', '╞', '╪', '╡', '╘',
                '╧', '╛', '▀', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█', '▉', '▊', '▋', '▌', '▍',
                '▎', '▏', '▏', '▎', '▍', '▌', '▋', '▊', '▉', '█', '▇', '▆', '▅', '▄', '▃', '▂',
                '▁', '▔', '▕', '▔', '▔', '▔', '▔', '▔', '▔', '▔', '▔', '▔', '▔', '▔', '▔', '▔',
                '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█', '▉', '▊', '▋', '▌', '▍', '▎', '▏', '▏',
                '▎', '▍', '▌', '▋', '▊', '▉', '█', '▇', '▆', '▅', '▄', '▃', '▂', '▁', '▔', '▔',
                '▔', '▔', '▔', '▔', '▔', '▔', '▔', '▔', '▔', '▔', '▔', '▔', '▔', '▔', '▔', '▔',
                ' ', '!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/',
                '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', ':', ';', '<', '=', '>', '?',
                '@', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O',
                'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '[', '\\', ']', '^', '_',
            ],
            2 => &[
                ' ', '☺', '☻', '♥', '♦', '♣', '♠', '•', '◘', '○', '◙', '♂', '♀', '♪', '♫', '☼',
                '►', '◄', '↕', '‼', '¶', '§', '▬', '↨', '↑', '↓', '→', '←', '∟', '↔', '▲', '▼',
                ' ', '!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/',
                '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', ':', ';', '<', '=', '>', '?',
                '@', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O',
                'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '[', '\\', ']', '^', '_',
                '`', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o',
                'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '{', '|', '}', '~', '⌂',
                'Ç', 'ü', 'é', 'â', 'ä', 'à', 'å', 'ç', 'ê', 'ë', 'è', 'ï', 'î', 'ì', 'Ä', 'Å',
                'É', 'æ', 'Æ', 'ô', 'ö', 'ò', 'û', 'ù', 'ÿ', 'Ö', 'Ü', '¢', '£', '¥', '₧', 'ƒ',
                'á', 'í', 'ó', 'ú', 'ñ', 'Ñ', 'ª', 'º', '¿', '⌐', '¬', '½', '¼', '¡', '«', '»',
                '░', '▒', '▓', '│', '┤', '╡', '╢', '╖', '╕', '╣', '║', '╗', '╝', '╜', '╛', '┐',
                '└', '┴', '┬', '├', '─', '┼', '╞', '╟', '╚', '╔', '╩', '╦', '╠', '═', '╬', '╧',
                '╨', '╤', '╥', '╙', '╘', '╒', '╓', '╫', '╪', '┘', '┌', '█', '▄', '▌', '▐', '▀',
                'α', 'ß', 'Γ', 'π', 'Σ', 'σ', 'µ', 'τ', 'Φ', 'Θ', 'Ω', 'δ', '∞', 'φ', 'ε', '∩',
                '≡', '±', '≥', '≤', '⌠', '⌡', '÷', '≈', '°', '∙', '·', '√', 'ⁿ', '²', '■', ' ',
            ],
            _ => &[
                '█', '▓', '▒', '░', '■', '□', '▪', '▫', '▬', '▲', '▼', '◄', '►', '●', '○', '•',
            ],
        }
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

    fn prev_fg_color(&mut self) {
        self.fg_color = match self.fg_color {
            Color::DarkGray => Color::Black,
            Color::Gray => Color::DarkGray,
            Color::White => Color::Gray,
            Color::Red => Color::White,
            Color::Green => Color::Red,
            Color::Yellow => Color::Green,
            Color::Blue => Color::Yellow,
            Color::Magenta => Color::Blue,
            Color::Cyan => Color::Magenta,
            Color::LightRed => Color::Cyan,
            Color::LightGreen => Color::LightRed,
            Color::LightYellow => Color::LightGreen,
            Color::LightBlue => Color::LightYellow,
            Color::LightMagenta => Color::LightBlue,
            Color::LightCyan => Color::LightMagenta,
            _ => Color::White,
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

    fn prev_bg_color(&mut self) {
        self.bg_color = match self.bg_color {
            Color::DarkGray => Color::Black,
            Color::Gray => Color::DarkGray,
            Color::White => Color::Gray,
            Color::Red => Color::White,
            Color::Green => Color::Red,
            Color::Yellow => Color::Green,
            Color::Blue => Color::Yellow,
            Color::Magenta => Color::Blue,
            Color::Cyan => Color::Magenta,
            Color::LightRed => Color::Cyan,
            Color::LightGreen => Color::LightRed,
            Color::LightYellow => Color::LightGreen,
            Color::LightBlue => Color::LightYellow,
            Color::LightMagenta => Color::LightBlue,
            Color::LightCyan => Color::LightMagenta,
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

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Right => app.next_fg_color(),
                    KeyCode::Left => app.prev_fg_color(),
                    KeyCode::Up => app.next_bg_color(),
                    KeyCode::Down => app.prev_bg_color(),
                    KeyCode::Char('h') => app.toggle_hex(),
                    KeyCode::Char('c') => app.next_char_set(),
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

    // Create a grid of characters from the current character set
    let mut grid = String::new();
    let chars = app.get_character_set();
    let chars_len = chars.len();

    if chars_len == 0 {
        return;
    }

    // Calculate rows and columns to fit the terminal
    let cols = (inner.width as usize).min(32); // Max 32 columns
    let rows = (inner.height as usize).saturating_sub(4).max(1); // Leave space for controls

    for row in 0..rows {
        for col in 0..cols {
            let idx = (row * cols + col) % chars_len;
            if app.show_hex && idx < 256 {
                grid.push_str(&format!("{:02x} ", idx));
            } else {
                grid.push(chars[idx]);
                grid.push(' ');
            }
        }
        grid.push('\n');
    }

    // Display the character grid
    let grid_para = Paragraph::new(grid)
        .fg(app.fg_color)
        .bg(app.bg_color)
        .block(Block::default().borders(Borders::NONE));

    // Display current colors and controls
    let info = format!(
        "FG: {} (←/→) | BG: {} (↑/↓) | Chars: {} | Mode: {} | 'h': Toggle Hex | 'c': Cycle Chars | 'q': Quit",
        App::get_color_name(app.fg_color),
        App::get_color_name(app.bg_color),
        ["Blocks", "Boxes", "CP437", "Emoji"][app.char_set],
        if app.show_hex { "Hex" } else { "Char" }
    );

    let info_para = Paragraph::new(info).block(
        Block::default()
            .borders(Borders::TOP)
            .style(Style::default().fg(Color::Gray)),
    );

    // Layout
    let chunks = Layout::vertical([Constraint::Min(1), Constraint::Length(3)]).split(inner);

    f.render_widget(grid_para, chunks[0]);
    f.render_widget(info_para, chunks[1]);
}

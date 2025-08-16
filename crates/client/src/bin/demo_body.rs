use std::{io, time::Duration};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    symbols::border,
    widgets::*,
};

#[derive(Debug, Default)]
struct BodyPart {
    name: &'static str,
    hp: f32, // 0.0 - 1.0
}

struct App {
    body_parts: Vec<BodyPart>,
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        Self {
            body_parts: vec![
                BodyPart { name: "Head", hp: 0.7 },
                BodyPart { name: "Torso", hp: 0.9 },
                BodyPart { name: "Left Arm", hp: 0.5 },
                BodyPart { name: "Right Arm", hp: 0.85 },
                BodyPart { name: "Left Leg", hp: 0.6 },
                BodyPart { name: "Right Leg", hp: 0.95 },
            ],
            should_quit: false,
        }
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
    let app = App::new();
    let res = run_app(&mut terminal, app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}")
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    app.should_quit = true;
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let size = f.size();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::THICK)
        .title(" Body Status ")
        .title_alignment(Alignment::Center)
        .border_style(Style::default().fg(Color::LightBlue));

    let inner = block.inner(Rect::new(0, 0, size.width, size.height));
    let area = centered_rect(80, 60, inner);

    let body_parts = app.body_parts.iter().map(|part| {
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::NONE))
            .gauge_style(Style::default().fg(match part.hp {
                x if x > 0.7 => Color::Green,
                x if x > 0.3 => Color::Yellow,
                _ => Color::Red,
            }))
            .ratio(part.hp as f64)
            .label(format!("{:>9}: {:.0}%", part.name, part.hp * 100.0));

        ListItem::new("").child(Box::new(gauge))
    }).collect::<Vec<_>>();

    let list = List::new(body_parts)
        .block(block)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol(">");

    f.render_widget(Clear, area);
    f.render_widget(list, area);
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

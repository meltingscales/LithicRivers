use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, symbols::border, widgets::*};
use std::{io, time::Duration};

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
                BodyPart {
                    name: "Head",
                    hp: 0.7,
                },
                BodyPart {
                    name: "Torso",
                    hp: 0.9,
                },
                BodyPart {
                    name: "Left Arm",
                    hp: 0.5,
                },
                BodyPart {
                    name: "Right Arm",
                    hp: 0.85,
                },
                BodyPart {
                    name: "Left Leg",
                    hp: 0.6,
                },
                BodyPart {
                    name: "Right Leg",
                    hp: 0.95,
                },
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

fn ui(f: &mut Frame, app: &App) {
    let size = f.size();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::THICK)
        .title(" Body Status ")
        .title_alignment(Alignment::Center)
        .border_style(Style::default().fg(Color::LightBlue));

    let inner = block.inner(Rect::new(0, 0, size.width, size.height));
    let area = centered_rect(80, 60, inner);

    // Split area into vertical rows for each body part
    let constraints: Vec<Constraint> = (0..app.body_parts.len())
        .map(|_| Constraint::Length(3))
        .collect();
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    // Clear content area
    f.render_widget(Clear, area);

    // Render each gauge in its own row
    for (i, part) in app.body_parts.iter().enumerate() {
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::NONE))
            .gauge_style(Style::default().fg(match part.hp {
                x if x > 0.7 => Color::Green,
                x if x > 0.3 => Color::Yellow,
                _ => Color::Red,
            }))
            .ratio(part.hp as f64)
            .label(format!("{:>9}: {:.0}%", part.name, part.hp * 100.0));

        f.render_widget(gauge, rows[i]);
    }

    // Render the border last so it appears on top
    f.render_widget(block, area);
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

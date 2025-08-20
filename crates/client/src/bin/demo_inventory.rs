use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, symbols::border, widgets::*};
use std::{io, time::Duration};

struct App {
    inventory: Vec<Option<&'static str>>,
    selected: Option<usize>,
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        Self {
            inventory: vec![
                Some("Rusty Knife"),
                Some("Bandage"),
                None,
                None,
                Some("Scrap"),
                None,
                Some("Seed"),
                None,
                None,
                None,
                None,
                Some("Battery"),
                None,
                Some("Water"),
                None,
                None,
            ],
            selected: None,
            should_quit: false,
        }
    }

    fn next(&mut self) {
        let i = match self.selected {
            Some(i) => (i + 1) % self.inventory.len(),
            None => 0,
        };
        self.selected = Some(i);
    }

    fn previous(&mut self) {
        let i = match self.selected {
            Some(i) => {
                if i == 0 {
                    self.inventory.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.selected = Some(i);
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
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => app.should_quit = true,
                        KeyCode::Right | KeyCode::Char('l') => app.next(),
                        KeyCode::Left | KeyCode::Char('h') => app.previous(),
                        KeyCode::Down | KeyCode::Char('j') => {
                            if let Some(selected) = app.selected {
                                let new_selected = (selected + 4).min(app.inventory.len() - 1);
                                app.selected = Some(new_selected);
                            } else {
                                app.selected = Some(0);
                            }
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            if let Some(selected) = app.selected {
                                let new_selected = selected.saturating_sub(4);
                                app.selected = Some(new_selected);
                            } else {
                                app.selected = Some(0);
                            }
                        }
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

fn ui(f: &mut Frame, app: &App) {
    let size = f.size();

    // Create a block for the inventory
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::THICK)
        .title(" Inventory ")
        .title_alignment(Alignment::Center)
        .border_style(Style::default().fg(Color::LightBlue));

    // Create a centered area for the inventory
    let inner = block.inner(Rect::new(0, 0, size.width, size.height));
    let area = centered_rect(70, 70, inner);

    // Create a grid layout for the inventory
    let grid_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(1),    // Grid
            Constraint::Length(3), // Controls
        ])
        .split(area);

    // Render the title
    let title = Paragraph::new("Inventory (demo)")
        .style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    f.render_widget(title, grid_layout[0]);

    // Create a 4x4 grid layout for the inventory items using nested Layouts
    let row_constraints = [
        Constraint::Percentage(25),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
    ];
    let col_constraints = [
        Constraint::Percentage(25),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
    ];

    let row_areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints(row_constraints)
        .split(grid_layout[1]);

    for row in 0..4 {
        let col_areas = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(col_constraints)
            .split(row_areas[row]);

        for col in 0..4 {
            let idx = row * 4 + col;
            let item = &app.inventory[idx];
            let is_selected = app.selected == Some(idx);

            let (text, style) = match item {
                Some(item) => (
                    format!("\n {}", item),
                    Style::default().fg(Color::LightGreen),
                ),
                None => (
                    "\n (empty)".to_string(),
                    Style::default().fg(Color::DarkGray),
                ),
            };

            let mut block = Block::default().borders(Borders::ALL);
            if is_selected {
                block = block
                    .border_style(Style::default().fg(Color::Yellow))
                    .title_style(Style::default().fg(Color::Yellow));
            }

            let cell_block = block
                .title(format!(" {} ", idx + 1))
                .title_alignment(Alignment::Right)
                .padding(Padding::new(1, 1, 1, 1));

            let paragraph = Paragraph::new(text)
                .block(cell_block)
                .style(style)
                .alignment(Alignment::Center);

            f.render_widget(Paragraph::new(""), col_areas[col]); // clear cell
            f.render_widget(paragraph, col_areas[col]);
        }
    }

    // Render controls
    let controls = Paragraph::new("←→↑↓/hjkl: Navigate | q: Quit")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    f.render_widget(controls, grid_layout[2]);

    // Render the border last to be on top
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

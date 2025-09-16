use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};
use std::{collections::HashMap, io, time::Duration};

// Define our item types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Item {
    Wood,
    Stone,
    String,
    Stick,
    Torch,
}

impl Item {
    fn name(&self) -> &'static str {
        match self {
            Item::Wood => "Wood",
            Item::Stone => "Stone",
            Item::String => "String",
            Item::Stick => "Stick",
            Item::Torch => "Torch",
        }
    }

    fn default_quantity() -> u32 {
        12
    }
}

// Define our recipe type
type Recipe = (Vec<(Item, u32)>, Item, u32); // (ingredients, result, quantity)

struct App {
    inventory: HashMap<Item, u32>,
    recipes: Vec<Recipe>,
    selected_recipe: Option<usize>,
    selected_item: Option<usize>,
    should_quit: bool,
    message: Option<String>,
    message_timer: u8,
}

impl App {
    fn new() -> Self {
        // Initialize inventory with default quantities
        let mut inventory = HashMap::new();
        let items = [
            Item::Wood,
            Item::Stone,
            Item::String,
            Item::Stick,
            Item::Torch,
        ];

        for &item in &items {
            inventory.insert(item, Item::default_quantity());
        }

        // Define crafting recipes
        let recipes = vec![
            // Torch: 1x Stick + 1x String
            (vec![(Item::Stick, 1), (Item::String, 1)], Item::Torch, 4),
        ];

        Self {
            inventory,
            recipes,
            selected_recipe: None,
            selected_item: None,
            should_quit: false,
            message: None,
            message_timer: 0,
        }
    }

    fn craft(&mut self, recipe_index: usize) {
        if let Some(recipe) = self.recipes.get(recipe_index) {
            let (ingredients, result, quantity) = recipe;

            // Check if we have all ingredients
            for &(item, required) in ingredients {
                if self.inventory.get(&item).copied().unwrap_or(0) < required {
                    self.set_message(format!("Not enough {}!", item.name()));
                    return;
                }
            }

            // Consume ingredients
            for &(item, required) in ingredients {
                *self.inventory.entry(item).or_default() -= required;
            }

            // Add result
            *self.inventory.entry(*result).or_default() += quantity;

            self.set_message(format!("Crafted {}x {}!", quantity, result.name()));
        }
    }

    fn set_message(&mut self, msg: String) {
        self.message = Some(msg);
        self.message_timer = 30; // Show message for 30 frames
    }

    fn update_message_timer(&mut self) {
        if self.message_timer > 0 {
            self.message_timer -= 1;
            if self.message_timer == 0 {
                self.message = None;
            }
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
        println!("{:?}", err)
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
                        KeyCode::Char('q') | KeyCode::Esc => {
                            app.should_quit = true;
                        }
                        KeyCode::Up => {
                            if let Some(selected) = app.selected_recipe {
                                app.selected_recipe = Some(selected.saturating_sub(1).max(0));
                            } else if let Some(selected) = app.selected_item {
                                app.selected_item = Some(selected.saturating_sub(1).max(0));
                            }
                        }
                        KeyCode::Down => {
                            if let Some(selected) = app.selected_recipe {
                                app.selected_recipe =
                                    Some((selected + 1).min(app.recipes.len().saturating_sub(1)));
                            } else if let Some(selected) = app.selected_item {
                                app.selected_item =
                                    Some((selected + 1).min(app.inventory.len().saturating_sub(1)));
                            } else {
                                app.selected_item = Some(0);
                            }
                        }
                        KeyCode::Left | KeyCode::Right => {
                            // Toggle between inventory and recipes
                            app.selected_recipe = if app.selected_recipe.is_some() {
                                None
                            } else {
                                app.selected_item = None;
                                Some(0)
                            };
                        }
                        KeyCode::Enter => {
                            if let Some(recipe_idx) = app.selected_recipe {
                                app.craft(recipe_idx);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        app.update_message_timer();

        if app.should_quit {
            return Ok(());
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let size = f.size();

    // Create a block for the entire app
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Crafting Demo ")
        .title_alignment(Alignment::Center);

    // Create two columns
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(block.inner(size));

    // Render inventory
    render_inventory(f, app, chunks[0]);

    // Render crafting recipes
    render_recipes(f, app, chunks[1]);

    // Render message if any
    if let Some(msg) = &app.message {
        let block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::DarkGray));

        let paragraph = Paragraph::new(msg.clone())
            .block(block)
            .alignment(Alignment::Center);

        let area = centered_rect(60, 20, size);
        f.render_widget(Clear, area);
        f.render_widget(paragraph, area);
    }

    f.render_widget(block, size);
}

fn render_inventory(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<(&Item, &u32)> = app.inventory.iter().collect();

    // Get the selected recipe's ingredients and result if any
    let (recipe_ingredients, recipe_result) = app.selected_recipe.map_or_else(
        || (Vec::new(), None),
        |idx| {
            if let Some((ingredients, result, _)) = app.recipes.get(idx) {
                let ingredients: Vec<&Item> = ingredients.iter().map(|(item, _)| item).collect();
                (ingredients, Some(result))
            } else {
                (Vec::new(), None)
            }
        },
    );

    let block = Block::default().borders(Borders::ALL).title(" Inventory ");

    let items_list: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(i, (item, &count))| {
            let is_selected = app.selected_item == Some(i) && app.selected_recipe.is_none();
            let is_ingredient = recipe_ingredients.contains(&&item);
            let is_result = recipe_result.map_or(false, |result_item| *result_item == **item);

            let style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else if is_result && app.selected_recipe.is_some() {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else if is_ingredient && app.selected_recipe.is_some() {
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(vec![Span::styled(
                format!("{:2} x {:<15}", count, item.name()),
                style,
            )]))
        })
        .collect();

    let list = List::new(items_list)
        .block(block)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_stateful_widget(
        list,
        area,
        &mut ListState::default().with_selected(app.selected_item),
    );
}

fn render_recipes(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Crafting Recipes ");

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let recipe_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)].as_ref())
        .split(inner_area);

    // Instructions
    let instructions = Paragraph::new("↑/↓: Select  Enter: Craft  ←→: Switch Panel  Q: Quit")
        .alignment(Alignment::Center);
    f.render_widget(instructions, recipe_chunks[0]);

    // Recipe list
    let recipe_list: Vec<ListItem> = app
        .recipes
        .iter()
        .enumerate()
        .map(|(i, (ingredients, result, quantity))| {
            let is_selected = app.selected_recipe == Some(i);
            let style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            // Format ingredients
            let ingredients_text = ingredients
                .iter()
                .map(|(item, count)| format!("{}x {}", count, item.name()))
                .collect::<Vec<_>>()
                .join(", ");

            ListItem::new(Line::from(vec![
                Span::styled(format!("{}x {}", quantity, result.name()), style),
                Span::raw(" ").style(Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("({})", ingredients_text),
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
        })
        .collect();

    let list =
        List::new(recipe_list).highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_stateful_widget(
        list,
        recipe_chunks[1],
        &mut ListState::default().with_selected(app.selected_recipe),
    );
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

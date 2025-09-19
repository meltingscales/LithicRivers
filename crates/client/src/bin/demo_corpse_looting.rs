use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};
use std::{io, time::Duration};

// Define our item types (matching the real game)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Item {
    Leather,
    Meat,
    Wood,
    Stone,
    String,
    Torch,
}

impl Item {
    fn name(&self) -> &'static str {
        match self {
            Item::Leather => "Leather",
            Item::Meat => "Meat",
            Item::Wood => "Wood",
            Item::Stone => "Stone",
            Item::String => "String",
            Item::Torch => "Torch",
        }
    }
}

#[derive(Debug, Clone)]
struct ItemStack {
    item: Item,
    quantity: u32,
}

#[derive(Debug, Clone)]
struct Corpse {
    name: String,
    x: i64,
    y: i64,
    inventory: Vec<ItemStack>,
}

impl Corpse {
    fn new(name: &str, x: i64, y: i64, inventory: Vec<ItemStack>) -> Self {
        Self {
            name: name.to_string(),
            x,
            y,
            inventory,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum UiMode {
    WorldMap,
    SelectingCorpse(Vec<usize>), // List of adjacent corpse indices to choose from
    LootingCorpse(usize),        // Index of corpse being looted
}

struct App {
    player_x: i64,
    player_y: i64,
    player_inventory: Vec<ItemStack>,
    corpses: Vec<Corpse>,
    ui_mode: UiMode,
    selected_loot_item: usize,
    selected_player_item: usize,
    selected_corpse: usize, // For corpse selection mode
    loot_panel_focus: bool, // true = corpse inventory, false = player inventory
    should_quit: bool,
    message: Option<String>,
    message_timer: u8,
}

impl App {
    fn new() -> Self {
        // Create demo corpses showing multi-corpse selection scenarios
        let corpses = vec![
            // Scenario 1: 2 corpses next to each other at (3,2) and (4,2)
            Corpse::new(
                "Feral Dog",
                3,
                2,
                vec![
                    ItemStack {
                        item: Item::Leather,
                        quantity: 2,
                    },
                    ItemStack {
                        item: Item::Meat,
                        quantity: 3,
                    },
                ],
            ),
            Corpse::new(
                "Wolf",
                4,
                2,
                vec![
                    ItemStack {
                        item: Item::Meat,
                        quantity: 1,
                    },
                    ItemStack {
                        item: Item::Leather,
                        quantity: 1,
                    },
                ],
            ),
            // Scenario 2: 5 corpses clustered around (7,8)
            Corpse::new(
                "Bandit Leader",
                7,
                8,
                vec![
                    ItemStack {
                        item: Item::Torch,
                        quantity: 2,
                    },
                    ItemStack {
                        item: Item::String,
                        quantity: 4,
                    },
                ],
            ),
            Corpse::new(
                "Bandit Scout",
                6,
                8,
                vec![
                    ItemStack {
                        item: Item::Wood,
                        quantity: 3,
                    },
                    ItemStack {
                        item: Item::String,
                        quantity: 1,
                    },
                ],
            ),
            Corpse::new(
                "Bandit Archer",
                8,
                8,
                vec![
                    ItemStack {
                        item: Item::Wood,
                        quantity: 2,
                    },
                    ItemStack {
                        item: Item::String,
                        quantity: 2,
                    },
                ],
            ),
            Corpse::new(
                "Bandit Warrior",
                7,
                7,
                vec![
                    ItemStack {
                        item: Item::Leather,
                        quantity: 1,
                    },
                    ItemStack {
                        item: Item::Meat,
                        quantity: 2,
                    },
                ],
            ),
            Corpse::new(
                "Bandit Mage",
                7,
                9,
                vec![
                    ItemStack {
                        item: Item::Torch,
                        quantity: 1,
                    },
                    ItemStack {
                        item: Item::Stone,
                        quantity: 3,
                    },
                ],
            ),
            // Single corpse for contrast
            Corpse::new(
                "Lone Traveler",
                10,
                4,
                vec![
                    ItemStack {
                        item: Item::Wood,
                        quantity: 1,
                    },
                    ItemStack {
                        item: Item::Torch,
                        quantity: 1,
                    },
                ],
            ),
            // Corpse directly under player starting position to test standing on corpse
            Corpse::new(
                "Fallen Hero",
                5,
                5,
                vec![
                    ItemStack {
                        item: Item::Torch,
                        quantity: 3,
                    },
                    ItemStack {
                        item: Item::Stone,
                        quantity: 2,
                    },
                ],
            ),
        ];

        // Player starts with some basic items
        let player_inventory = vec![
            ItemStack {
                item: Item::Wood,
                quantity: 5,
            },
            ItemStack {
                item: Item::Stone,
                quantity: 3,
            },
        ];

        Self {
            player_x: 5,
            player_y: 5,
            player_inventory,
            corpses,
            ui_mode: UiMode::WorldMap,
            selected_loot_item: 0,
            selected_player_item: 0,
            selected_corpse: 0,
            loot_panel_focus: true,
            should_quit: false,
            message: None,
            message_timer: 0,
        }
    }

    fn move_player(&mut self, dx: i64, dy: i64) {
        if matches!(self.ui_mode, UiMode::WorldMap) {
            self.player_x += dx;
            self.player_y += dy;

            // Keep player in bounds (simple demo bounds)
            self.player_x = self.player_x.clamp(0, 12);
            self.player_y = self.player_y.clamp(0, 12);
        }
    }

    fn get_adjacent_corpses(&self) -> Vec<usize> {
        let mut adjacent = Vec::new();
        for (i, corpse) in self.corpses.iter().enumerate() {
            let dx = (corpse.x - self.player_x).abs();
            let dy = (corpse.y - self.player_y).abs();
            // Include all 9 tiles: 3x3 grid centered on player (including player's tile)
            if dx <= 1 && dy <= 1 {
                adjacent.push(i);
            }
        }
        adjacent
    }

    fn try_interact(&mut self) {
        if matches!(self.ui_mode, UiMode::WorldMap) {
            let adjacent = self.get_adjacent_corpses();
            if adjacent.is_empty() {
                self.set_message("No corpses nearby to loot".to_string());
            } else if adjacent.len() == 1 {
                // Single corpse - go directly to looting
                let corpse_idx = adjacent[0];
                self.ui_mode = UiMode::LootingCorpse(corpse_idx);
                self.selected_loot_item = 0;
                self.selected_player_item = 0;
                self.loot_panel_focus = true;
                self.set_message(format!("Looting {}", self.corpses[corpse_idx].name));
            } else {
                // Multiple corpses - show selection screen
                self.ui_mode = UiMode::SelectingCorpse(adjacent);
                self.selected_corpse = 0;
                self.set_message("Choose which corpse to loot".to_string());
            }
        }
    }

    fn select_corpse(&mut self, corpse_idx: usize) {
        self.ui_mode = UiMode::LootingCorpse(corpse_idx);
        self.selected_loot_item = 0;
        self.selected_player_item = 0;
        self.loot_panel_focus = true;
        self.set_message(format!("Looting {}", self.corpses[corpse_idx].name));
    }

    fn cancel_corpse_selection(&mut self) {
        self.ui_mode = UiMode::WorldMap;
        self.set_message("Cancelled interaction".to_string());
    }

    fn close_loot_modal(&mut self) {
        self.ui_mode = UiMode::WorldMap;
        self.set_message("Finished looting".to_string());
    }

    fn take_item_from_corpse(&mut self, corpse_idx: usize, item_idx: usize) {
        let item_name = if let Some(corpse) = self.corpses.get(corpse_idx) {
            if let Some(item_stack) = corpse.inventory.get(item_idx) {
                if item_stack.quantity > 0 {
                    Some(item_stack.item.name().to_string())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        if let Some(name) = item_name {
            if let Some(corpse) = self.corpses.get_mut(corpse_idx) {
                if let Some(item_stack) = corpse.inventory.get_mut(item_idx) {
                    if item_stack.quantity > 0 {
                        // Take one item
                        item_stack.quantity -= 1;

                        // Add to player inventory
                        if let Some(player_stack) = self
                            .player_inventory
                            .iter_mut()
                            .find(|stack| stack.item == item_stack.item)
                        {
                            player_stack.quantity += 1;
                        } else {
                            self.player_inventory.push(ItemStack {
                                item: item_stack.item,
                                quantity: 1,
                            });
                        }

                        // Remove empty stacks
                        corpse.inventory.retain(|stack| stack.quantity > 0);

                        // Adjust selection if needed
                        if self.selected_loot_item >= corpse.inventory.len()
                            && corpse.inventory.len() > 0
                        {
                            self.selected_loot_item = corpse.inventory.len() - 1;
                        }
                    }
                }
            }

            self.set_message(format!("Took {}", name));
        }
    }

    fn set_message(&mut self, msg: String) {
        self.message = Some(msg);
        self.message_timer = 60; // Show message for 60 frames
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
                    match app.ui_mode {
                        UiMode::WorldMap => match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => {
                                app.should_quit = true;
                            }
                            KeyCode::Up => app.move_player(0, -1),
                            KeyCode::Down => app.move_player(0, 1),
                            KeyCode::Left => app.move_player(-1, 0),
                            KeyCode::Right => app.move_player(1, 0),
                            KeyCode::Char('i') | KeyCode::Enter => app.try_interact(),
                            _ => {}
                        },
                        UiMode::SelectingCorpse(ref adjacent_corpses) => {
                            let corpses_clone = adjacent_corpses.clone();
                            match key.code {
                                KeyCode::Esc => app.cancel_corpse_selection(),
                                KeyCode::Up => {
                                    app.selected_corpse = app.selected_corpse.saturating_sub(1);
                                }
                                KeyCode::Down => {
                                    app.selected_corpse = (app.selected_corpse + 1)
                                        .min(corpses_clone.len().saturating_sub(1));
                                }
                                KeyCode::Enter => {
                                    if app.selected_corpse < corpses_clone.len() {
                                        let corpse_idx = corpses_clone[app.selected_corpse];
                                        app.select_corpse(corpse_idx);
                                    }
                                }
                                _ => {}
                            }
                        }
                        UiMode::LootingCorpse(corpse_idx) => match key.code {
                            KeyCode::Esc => app.close_loot_modal(),
                            KeyCode::Left | KeyCode::Right => {
                                app.loot_panel_focus = !app.loot_panel_focus;
                            }
                            KeyCode::Up => {
                                if app.loot_panel_focus {
                                    if let Some(corpse) = app.corpses.get(corpse_idx) {
                                        if !corpse.inventory.is_empty() {
                                            app.selected_loot_item =
                                                app.selected_loot_item.saturating_sub(1);
                                        }
                                    }
                                } else {
                                    if !app.player_inventory.is_empty() {
                                        app.selected_player_item =
                                            app.selected_player_item.saturating_sub(1);
                                    }
                                }
                            }
                            KeyCode::Down => {
                                if app.loot_panel_focus {
                                    if let Some(corpse) = app.corpses.get(corpse_idx) {
                                        if !corpse.inventory.is_empty() {
                                            app.selected_loot_item = (app.selected_loot_item + 1)
                                                .min(corpse.inventory.len() - 1);
                                        }
                                    }
                                } else {
                                    if !app.player_inventory.is_empty() {
                                        app.selected_player_item = (app.selected_player_item + 1)
                                            .min(app.player_inventory.len() - 1);
                                    }
                                }
                            }
                            KeyCode::Enter => {
                                if app.loot_panel_focus {
                                    app.take_item_from_corpse(corpse_idx, app.selected_loot_item);
                                }
                            }
                            _ => {}
                        },
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

    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(size);

    // Split right side for inventory and instructions
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(chunks[1]);

    // Render world map
    render_world_map(f, app, chunks[0]);

    // Render player inventory
    render_player_inventory(f, app, right_chunks[0]);

    // Render instructions
    render_instructions(f, app, right_chunks[1]);

    // Render corpse selection modal if selecting
    if let UiMode::SelectingCorpse(ref adjacent_corpses) = app.ui_mode {
        render_corpse_selection_modal(f, app, adjacent_corpses);
    }

    // Render loot modal if in looting mode
    if let UiMode::LootingCorpse(corpse_idx) = app.ui_mode {
        render_loot_modal(f, app, corpse_idx);
    }

    // Render message if any
    if let Some(msg) = &app.message {
        render_message(f, msg, size);
    }
}

fn render_world_map(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title(" World Map ");

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Create a mini world map
    let map_width = inner.width as usize;
    let map_height = inner.height as usize;

    let mut map_content = String::new();

    for y in 0..map_height {
        for x in 0..map_width {
            let world_x = x as i64;
            let world_y = y as i64;

            // Check what's at this position
            if world_x == app.player_x && world_y == app.player_y {
                map_content.push('@'); // Player
            } else if let Some(_) = app
                .corpses
                .iter()
                .find(|c| c.x == world_x && c.y == world_y)
            {
                map_content.push('%'); // Corpse
            } else {
                map_content.push('.'); // Empty ground
            }
        }
        if y < map_height - 1 {
            map_content.push('\n');
        }
    }

    let map_widget = Paragraph::new(map_content).style(Style::default().fg(Color::Green));

    f.render_widget(map_widget, inner);

    // Show interaction hint
    let adjacent_corpses = app.get_adjacent_corpses();
    let hint = if adjacent_corpses.is_empty() {
        "Move with arrow keys. Find corpses (%) to loot!"
    } else {
        "Press 'i' to interact with nearby corpse"
    };

    let hint_area = Rect {
        x: area.x,
        y: area.y + area.height,
        width: area.width,
        height: 1,
    };

    if hint_area.y < f.size().height {
        let hint_widget = Paragraph::new(hint).style(Style::default().fg(Color::Yellow));
        f.render_widget(hint_widget, hint_area);
    }
}

fn render_player_inventory(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Player Inventory ");

    let items_list: Vec<ListItem> = app
        .player_inventory
        .iter()
        .enumerate()
        .map(|(i, stack)| {
            let is_selected = !app.loot_panel_focus
                && matches!(app.ui_mode, UiMode::LootingCorpse(_))
                && app.selected_player_item == i;

            let style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(Span::styled(
                format!("{:2} x {}", stack.quantity, stack.item.name()),
                style,
            )))
        })
        .collect();

    let list = List::new(items_list).block(block);

    f.render_widget(list, area);
}

fn render_instructions(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title(" Controls ");

    let instructions = match app.ui_mode {
        UiMode::WorldMap => vec![
            "Movement:",
            "  ↑ ↓ ← →  Move player",
            "",
            "Interaction:",
            "  i / Enter  Interact with corpse",
            "",
            "General:",
            "  q / Esc    Quit demo",
            "",
            "Try (3,2) for 2 corpses,",
            "(7,8) for 5, or press 'i'",
            "to loot corpse under you!",
        ],
        UiMode::SelectingCorpse(_) => vec![
            "Multiple Corpses Found:",
            "  ↑ ↓        Select corpse",
            "  Enter      Loot selected",
            "  Esc        Cancel",
            "",
            "Choose which corpse to",
            "interact with from the",
            "selection modal.",
        ],
        UiMode::LootingCorpse(_) => vec![
            "Looting Mode:",
            "  ← →        Switch panel",
            "  ↑ ↓        Select item",
            "  Enter      Take item",
            "  Esc        Close looting",
            "",
            "Navigate between corpse",
            "items and your inventory",
            "to transfer loot.",
        ],
    };

    let content = instructions.join("\n");
    let paragraph = Paragraph::new(content)
        .block(block)
        .style(Style::default().fg(Color::Cyan));

    f.render_widget(paragraph, area);
}

fn render_corpse_selection_modal(f: &mut Frame, app: &App, adjacent_corpses: &[usize]) {
    // Create modal area (centered, 50% width, 40% height)
    let area = centered_rect(50, 40, f.size());

    // Clear the background
    f.render_widget(Clear, area);

    // Main modal block
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Choose Corpse to Loot ")
        .title_alignment(Alignment::Center)
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Create list of corpses
    let corpse_items: Vec<ListItem> = adjacent_corpses
        .iter()
        .enumerate()
        .map(|(i, &corpse_idx)| {
            let corpse = &app.corpses[corpse_idx];
            let is_selected = app.selected_corpse == i;
            let style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            // Show corpse name and item count
            let item_count = corpse.inventory.len();
            let text = format!("{} ({} items)", corpse.name, item_count);

            ListItem::new(Line::from(Span::styled(text, style)))
        })
        .collect();

    let list =
        List::new(corpse_items).highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_widget(list, inner);

    // Controls at bottom
    let controls = Paragraph::new("↑↓: Select | Enter: Loot | Esc: Cancel")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));

    let controls_area = Rect {
        x: inner.x,
        y: inner.y + inner.height - 1,
        width: inner.width,
        height: 1,
    };

    f.render_widget(controls, controls_area);
}

fn render_loot_modal(f: &mut Frame, app: &App, corpse_idx: usize) {
    let corpse = &app.corpses[corpse_idx];

    // Create modal area (centered, 70% width, 60% height)
    let area = centered_rect(70, 60, f.size());

    // Clear the background
    f.render_widget(Clear, area);

    // Main modal block
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Looting {} ", corpse.name))
        .title_alignment(Alignment::Center)
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Split into two columns
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);

    // Left side: Corpse inventory
    let corpse_block = Block::default()
        .borders(Borders::ALL)
        .title(" Corpse Items ")
        .border_style(if app.loot_panel_focus {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });

    let corpse_items: Vec<ListItem> = corpse
        .inventory
        .iter()
        .enumerate()
        .map(|(i, stack)| {
            let is_selected = app.loot_panel_focus && app.selected_loot_item == i;
            let style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(Span::styled(
                format!("{:2} x {}", stack.quantity, stack.item.name()),
                style,
            )))
        })
        .collect();

    let corpse_list = List::new(corpse_items).block(corpse_block);
    f.render_widget(corpse_list, chunks[0]);

    // Right side: Player inventory
    let player_block = Block::default()
        .borders(Borders::ALL)
        .title(" Your Items ")
        .border_style(if !app.loot_panel_focus {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });

    let player_items: Vec<ListItem> = app
        .player_inventory
        .iter()
        .enumerate()
        .map(|(i, stack)| {
            let is_selected = !app.loot_panel_focus && app.selected_player_item == i;
            let style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(Span::styled(
                format!("{:2} x {}", stack.quantity, stack.item.name()),
                style,
            )))
        })
        .collect();

    let player_list = List::new(player_items).block(player_block);
    f.render_widget(player_list, chunks[1]);

    // Controls at bottom
    let controls = Paragraph::new("←→: Switch Panel | ↑↓: Select | Enter: Take Item | Esc: Close")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));

    let controls_area = Rect {
        x: inner.x,
        y: inner.y + inner.height - 1,
        width: inner.width,
        height: 1,
    };

    f.render_widget(controls, controls_area);
}

fn render_message(f: &mut Frame, message: &str, area: Rect) {
    let message_area = Rect {
        x: area.x,
        y: area.y + area.height - 1,
        width: area.width,
        height: 1,
    };

    let message_widget = Paragraph::new(message)
        .style(Style::default().fg(Color::Yellow).bg(Color::DarkGray))
        .alignment(Alignment::Center);

    f.render_widget(message_widget, message_area);
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

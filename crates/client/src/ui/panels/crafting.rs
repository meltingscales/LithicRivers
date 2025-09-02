use crate::ui::panels::get_player_inventory;
use lithicrivers_core::components::{itemkind_name, ItemKind};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

pub fn render_crafting_panel(f: &mut Frame, app: &mut crate::App, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title("Crafting");
    let inner = block.inner(area);

    // Get player's inventory
    let inventory = get_player_inventory(app);

    // Update craft message timer
    if let Some((_, ref mut timer)) = &mut app.craft_message {
        *timer = timer.saturating_sub(1);
        if *timer == 0 {
            app.craft_message = None;
        }
    }

    // Split the area into three parts: recipes list, details, and inventory
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),    // Main content
            Constraint::Length(3), // For the message area
        ])
        .split(inner);

    let main_area = chunks[0];
    let message_area = chunks[1];

    // Split main area into three columns
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33), // Recipe list
            Constraint::Percentage(34), // Recipe details
            Constraint::Percentage(33), // Inventory
        ])
        .split(main_area);

    let list_area = chunks[0];
    let details_area = chunks[1];
    let inventory_area = chunks[2];

    // Render recipes list
    let recipes: Vec<ListItem> = app
        .recipe_handler
        .get_recipes()
        .iter()
        .enumerate()
        .map(|(i, recipe)| {
            let can_craft = app.recipe_handler.can_craft(i, &inventory);
            let style = if i == app.craft_selected {
                if can_craft {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Yellow)
                }
            } else if can_craft {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::DIM)
            };

            ListItem::new(Span::styled(
                format!("• {}", itemkind_name(recipe.result)),
                style,
            ))
        })
        .collect();

    let list = ratatui::widgets::List::new(recipes)
        .highlight_style(Style::default().add_modifier(ratatui::style::Modifier::BOLD))
        .highlight_symbol("> ")
        .block(Block::default().borders(Borders::ALL).title("Recipes"));

    f.render_stateful_widget(
        list,
        list_area,
        &mut ListState::default().with_selected(Some(app.craft_selected)),
    );

    // Render recipe details
    if let Some(recipe) = app.recipe_handler.get_recipes().get(app.craft_selected) {
        let can_craft = app.recipe_handler.can_craft(app.craft_selected, &inventory);
        let mut details = vec![
            Line::from(vec![
                Span::styled("Recipe Name: ", Style::default().fg(Color::Yellow)),
                Span::styled(recipe.name, Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::styled("Result: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    format!("{}x {}", recipe.quantity, itemkind_name(recipe.result)),
                    Style::default().fg(if can_craft { Color::Green } else { Color::Red }),
                ),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Ingredients:",
                Style::default().fg(Color::Yellow),
            )),
        ];

        for &(item, qty) in recipe.ingredients {
            let available = inventory.get(&item).copied().unwrap_or(0);
            let has_enough = available >= qty;
            let style = if has_enough {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            };

            details.push(Line::from(vec![
                Span::styled("  - ", style),
                Span::styled(format!("{}x {}", qty, itemkind_name(item)), style),
                Span::styled(
                    format!(" (have {})", available),
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::DIM),
                ),
            ]));
        }

        let details_block = Block::default()
            .borders(Borders::ALL)
            .title("Recipe Details");

        let details_paragraph = Paragraph::new(details)
            .block(details_block)
            .wrap(Wrap { trim: true });

        f.render_widget(details_paragraph, details_area);
    }

    // Render inventory
    let inventory_block = Block::default().borders(Borders::ALL).title("Inventory");

    let mut inventory_items: Vec<(ItemKind, u32)> = inventory.into_iter().collect();
    inventory_items.sort_by_key(|&(kind, _)| itemkind_name(kind).to_string());

    let inventory_list: Vec<ListItem> = inventory_items
        .into_iter()
        .map(|(kind, qty)| {
            ListItem::new(Line::from(vec![
                Span::raw("• "),
                Span::styled(
                    format!("{}: ", itemkind_name(kind)),
                    Style::default().fg(Color::Cyan),
                ),
                Span::raw(format!("x{}", qty)),
            ]))
        })
        .collect();

    let inventory_widget = List::new(inventory_list).block(inventory_block);

    f.render_widget(inventory_widget, inventory_area);

    // Render message if any
    if let Some((message, _)) = &app.craft_message {
        let message_block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Yellow));

        let message_paragraph = Paragraph::new(message.as_str())
            .block(message_block)
            .alignment(Alignment::Center);

        f.render_widget(message_paragraph, message_area);
    }

    f.render_widget(block, area);
}

use crate::ui::panels::get_player_inventory;
use lithicrivers_core::components::{itemkind_name, ItemKind};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

/// The recipes the tab lists right now, as indices into the recipe list: those whose name,
/// result (name or description) or an ingredient matches the search.
/// How long the "Crafted ..." or "Not enough resources" message stays up.
pub const CRAFT_MESSAGE_DURATION: std::time::Duration = std::time::Duration::from_millis(2500);

pub fn filtered_recipes(app: &mut crate::App) -> Vec<usize> {
    let inventory = get_player_inventory(app);
    let crafting = &mut app.panels.crafting;
    let loader = &mut app.core.sprite_loader;
    let shown: Vec<usize> = crafting
        .recipe_handler
        .get_recipes()
        .iter()
        .enumerate()
        .filter(|&(i, recipe)| {
            if crafting.craftable_only && !crafting.recipe_handler.can_craft(i, &inventory) {
                return false;
            }
            let description = super::item_dictionary::description(loader, recipe.result);
            let mut haystacks = vec![recipe.name, itemkind_name(recipe.result), &description];
            haystacks.extend(recipe.ingredients.iter().map(|&(i, _)| itemkind_name(i)));
            crafting.search.matches(haystacks)
        })
        .map(|(i, _)| i)
        .collect();
    keep_selection(crafting, &shown);
    shown
}

/// Keep the selection on the same recipe when the list changes (the search, the
/// craftable-only filter or the inventory), or on the nearest one still shown.
fn keep_selection(crafting: &mut crate::app_state::CraftingPanelState, shown: &[usize]) {
    if crafting.last_shown != shown {
        if let Some(&recipe) = crafting.last_shown.get(crafting.selected) {
            crafting.selected = shown
                .iter()
                .position(|&i| i >= recipe)
                .unwrap_or(shown.len().saturating_sub(1));
        }
        crafting.last_shown = shown.to_vec();
    }
    crafting.selected = crafting.selected.min(shown.len().saturating_sub(1));
}

pub fn render_crafting_panel(f: &mut Frame, app: &mut crate::App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Line::from("Crafting"));
    let inner = block.inner(area);

    // Get player's inventory
    let inventory = get_player_inventory(app);

    // The craft message goes away after a while, however fast frames are drawn
    if let Some((_, shown_at)) = app.panels.crafting.message {
        if app.clock.since(shown_at) >= CRAFT_MESSAGE_DURATION {
            app.panels.crafting.message = None;
        }
    }

    // The shown recipes, keeping the selection on one of them
    let shown = filtered_recipes(app);
    let total = app.panels.crafting.recipe_handler.get_recipes().len();
    let state = &mut app.panels.crafting;
    state.selected = state.selected.min(shown.len().saturating_sub(1));
    let selected_recipe = shown.get(state.selected).copied();

    // Split the area: the three columns, the search line, and the message area
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),    // Main content
            Constraint::Length(1), // Search line
            Constraint::Length(3), // For the message area
        ])
        .split(inner);

    let main_area = chunks[0];
    let search_area = chunks[1];
    let message_area = chunks[2];
    let search_line = app
        .panels
        .crafting
        .search
        .hint(|c, a| app.core.config_manager.get_printable_key_for_keybind(c, a));
    // The craftable-only toggle's hint follows the search hint, and is clickable
    let toggle_key = app
        .core
        .config_manager
        .get_printable_key_for_keybind("crafting", "TOGGLE_CRAFTABLE_ONLY");
    let craftable_only = app.panels.crafting.craftable_only;
    let toggle_hint = format!(
        "{toggle_key}: craftable only ({})",
        if craftable_only { "on" } else { "off" }
    );
    let gap = "   ";
    let toggle_x = search_area.x + (search_line.chars().count() + gap.len()) as u16;
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(search_line, Style::default().fg(Color::Gray)),
            Span::raw(gap),
            Span::styled(
                toggle_hint.clone(),
                Style::default().fg(if craftable_only {
                    Color::Yellow
                } else {
                    Color::Gray
                }),
            ),
        ])),
        search_area,
    );
    let toggle_width = (toggle_hint.chars().count() as u16)
        .min((search_area.x + search_area.width).saturating_sub(toggle_x));
    if toggle_width > 0 {
        app.ui.click_targets.push(
            Rect::new(toggle_x, search_area.y, toggle_width, 1),
            crate::ui::click_targets::ClickTarget::CraftableToggle,
        );
    }

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
    let all_recipes = app.panels.crafting.recipe_handler.get_recipes();
    let mut recipes: Vec<ListItem> = shown
        .iter()
        .map(|&i| {
            let recipe = &all_recipes[i];
            let can_craft = app.panels.crafting.recipe_handler.can_craft(i, &inventory);
            let style = if Some(i) == selected_recipe {
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
                format!("* {}", itemkind_name(recipe.result)),
                style,
            ))
        })
        .collect();
    if shown.is_empty() {
        let empty = if craftable_only && app.panels.crafting.search.text.is_empty() {
            vec![
                "Nothing craftable yet".to_string(),
                format!("{toggle_key}: show all recipes"),
            ]
        } else {
            vec!["No recipes match".to_string()]
        };
        let gray = Style::default().fg(Color::DarkGray);
        recipes.push(ListItem::new(
            empty
                .into_iter()
                .map(|l| Line::from(Span::styled(l, gray)))
                .collect::<Vec<_>>(),
        ));
    }
    let title = if craftable_only {
        "Recipes (craftable only)".to_string()
    } else if shown.len() == total {
        "Recipes".to_string()
    } else {
        format!("Recipes - showing {} of {total}", shown.len())
    };

    let list = ratatui::widgets::List::new(recipes)
        .highlight_style(Style::default().add_modifier(ratatui::style::Modifier::BOLD))
        .highlight_symbol("> ")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Line::from(title)),
        );

    let recipe_count = shown.len();
    let mut recipe_state =
        ListState::default().with_selected(selected_recipe.map(|_| app.panels.crafting.selected));
    f.render_stateful_widget(list, list_area, &mut recipe_state);
    let first = recipe_state.offset();
    app.ui.click_targets.rows(
        crate::ui::click_targets::inset(list_area),
        first,
        recipe_count.saturating_sub(first),
        crate::ui::click_targets::ClickTarget::RecipeRow,
    );

    // Render recipe details
    if let Some((index, recipe)) =
        selected_recipe.map(|i| (i, &app.panels.crafting.recipe_handler.get_recipes()[i]))
    {
        let can_craft = app
            .panels
            .crafting
            .recipe_handler
            .can_craft(index, &inventory);
        let description = crate::ui::panels::item_dictionary::description(
            &mut app.core.sprite_loader,
            recipe.result,
        );
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
            Line::from(Span::styled(
                description,
                Style::default()
                    .fg(Color::Gray)
                    .add_modifier(Modifier::ITALIC),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Ingredients:",
                Style::default().fg(Color::Yellow),
            )),
        ];

        let raw = lithicrivers_core::crafting::raw_sources();
        let recipes = app.panels.crafting.recipe_handler.get_recipes();
        for line in lithicrivers_core::crafting::ingredient_tree(recipes, recipe, 1, &raw) {
            let available = inventory.get(&line.item).copied().unwrap_or(0);
            let style = if available >= line.needed {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            };
            let marker = if line.raw { "* " } else { "- " };
            details.push(Line::from(vec![
                Span::styled(format!("{}{marker}", "  ".repeat(line.depth + 1)), style),
                Span::styled(
                    format!("{}x {}", line.needed, itemkind_name(line.item)),
                    style,
                ),
                Span::styled(
                    format!(" (have {})", available),
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::DIM),
                ),
            ]));
        }
        details.push(Line::from(Span::styled(
            "* raw material",
            Style::default().fg(Color::DarkGray),
        )));

        let details_block = Block::default()
            .borders(Borders::ALL)
            .title(Line::from("Recipe Details"));

        // Keep leading spaces: they indent the ingredient tree
        let details_paragraph = Paragraph::new(details)
            .block(details_block)
            .wrap(Wrap { trim: false });

        f.render_widget(details_paragraph, details_area);
    }

    // Render inventory
    let inventory_block = Block::default()
        .borders(Borders::ALL)
        .title(Line::from("Inventory"));

    let mut inventory_items: Vec<(ItemKind, u32)> = inventory.into_iter().collect();
    inventory_items.sort_by_key(|&(kind, _)| itemkind_name(kind).to_string());

    let inventory_list: Vec<ListItem> = inventory_items
        .into_iter()
        .map(|(kind, qty)| {
            ListItem::new(Line::from(vec![
                Span::raw("* "),
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
    if let Some((message, _)) = &app.panels.crafting.message {
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

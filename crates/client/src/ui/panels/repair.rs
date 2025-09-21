use crate::ui::panels::get_player_inventory;
use lithicrivers_core::components::{itemkind_name, ItemKind};
use lithicrivers_core::model::body::{Body, BodyPartState, BodyPartType};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

/// Render the repair modal as an overlay
pub fn render_repair_modal(f: &mut Frame, app: &mut crate::App, area: Rect) {
    // Clear the background
    f.render_widget(Clear, area);

    // Create modal background
    let modal_block = Block::default()
        .borders(Borders::ALL)
        .title(" Body Repair ")
        .title_alignment(Alignment::Center)
        .style(Style::default().fg(Color::Yellow));

    let inner = modal_block.inner(area);
    f.render_widget(modal_block, area);

    // Get player's inventory and body
    let inventory = get_player_inventory(app);

    let player_body: Option<Body> = if let Some(e) = app.core.game.get_player_entity() {
        if let Ok(body_ref) = app.core.game.world.get::<&Body>(e) {
            Some((*body_ref).clone())
        } else {
            None
        }
    } else {
        None
    };

    // Split the area into three parts: repair recipes list, details, and body parts/inventory
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),    // Main content
            Constraint::Length(3), // Controls/message area
        ])
        .split(inner);

    let main_area = chunks[0];
    let controls_area = chunks[1];

    // Split main area into three columns
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33), // Repair recipe list
            Constraint::Percentage(34), // Recipe details
            Constraint::Percentage(33), // Body parts/inventory
        ])
        .split(main_area);

    let list_area = chunks[0];
    let details_area = chunks[1];
    let body_parts_area = chunks[2];

    // Get current repair state
    let (selected_repair, selected_body_part) = match &app.panels.body_repair {
        crate::app_state::BodyRepairState::SelectingRepairAndPart {
            selected_repair,
            selected_body_part,
            ..
        } => (*selected_repair, *selected_body_part),
        _ => (0, None),
    };

    // Render repair recipes list
    render_repair_recipes_list(f, list_area, app, selected_repair);

    // Render recipe details
    render_repair_recipe_details(f, details_area, app, selected_repair, &inventory);

    // Render body parts list
    render_body_parts_list(f, body_parts_area, &player_body, selected_body_part);

    // Render controls
    let controls_text =
        "[↑↓] Navigate Recipes | [←→] Cycle Body Parts | [Enter] Repair Selected Part | [Esc] Cancel";
    let controls_para = Paragraph::new(controls_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Controls"));
    f.render_widget(controls_para, controls_area);
}

fn render_repair_recipes_list(f: &mut Frame, area: Rect, app: &crate::App, selected: usize) {
    let recipes = app.core.repair_handler.get_recipes();
    let inventory = get_player_inventory(app);

    let items: Vec<ListItem> = recipes
        .iter()
        .enumerate()
        .map(|(i, recipe)| {
            let is_selected = i == selected;
            let can_craft = app.core.repair_handler.can_repair(i, &inventory);

            let style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else if can_craft {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            };

            let text = format!("{}", recipe.name);
            ListItem::new(text).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Repair Recipes"),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_widget(list, area);
}

fn render_repair_recipe_details(
    f: &mut Frame,
    area: Rect,
    app: &crate::App,
    selected: usize,
    inventory: &std::collections::HashMap<ItemKind, u32>,
) {
    let recipes = app.core.repair_handler.get_recipes();

    let mut lines = vec![];

    if let Some(recipe) = recipes.get(selected) {
        lines.push(Line::from(Span::styled(
            recipe.name,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        lines.push(Line::from("Restores:"));
        lines.push(Line::from(format!(
            "  +{} Durability",
            recipe.durability_restored
        )));
        lines.push(Line::from(""));

        lines.push(Line::from("Compatible with:"));
        if recipe.compatible_parts.is_empty() {
            lines.push(Line::from("  All body parts"));
        } else {
            for part_type in recipe.compatible_parts {
                let part_name = match part_type {
                    BodyPartType::Head => "Head",
                    BodyPartType::Torso => "Torso",
                    BodyPartType::PowerSource => "Power Source",
                    BodyPartType::LeftArm => "Left Arm",
                    BodyPartType::RightArm => "Right Arm",
                    BodyPartType::LeftLeg => "Left Leg",
                    BodyPartType::RightLeg => "Right Leg",
                };
                lines.push(Line::from(format!("  {}", part_name)));
            }
        }
        lines.push(Line::from(""));

        lines.push(Line::from("Requires:"));
        for &(item, required) in recipe.ingredients {
            let available = inventory.get(&item).copied().unwrap_or(0);
            let item_name = itemkind_name(item);

            let color = if available >= required {
                Color::Green
            } else {
                Color::Red
            };

            lines.push(Line::from(Span::styled(
                format!("  {} x{} (have: {})", item_name, required, available),
                Style::default().fg(color),
            )));
        }
    } else {
        lines.push(Line::from("No recipe selected"));
    }

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Recipe Details"),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

fn render_body_parts_list(
    f: &mut Frame,
    area: Rect,
    body: &Option<Body>,
    selected_part: Option<BodyPartType>,
) {
    let mut lines = vec![];

    if let Some(body) = body {
        // Get parts in a consistent order
        let part_order = [
            BodyPartType::Head,
            BodyPartType::Torso,
            BodyPartType::PowerSource,
            BodyPartType::LeftArm,
            BodyPartType::RightArm,
            BodyPartType::LeftLeg,
            BodyPartType::RightLeg,
        ];

        for part_type in part_order {
            if let Some(part) = body.parts.get(&part_type) {
                let is_selected = selected_part == Some(part_type);
                let can_repair = part.state != BodyPartState::Missing;

                let (label, color) = match part.state {
                    BodyPartState::Missing => ("Missing", Color::DarkGray),
                    BodyPartState::Damaged => ("Damaged", Color::Yellow),
                    BodyPartState::Functional => ("Functional", Color::Green),
                    BodyPartState::Enhanced => ("Enhanced", Color::Cyan),
                };

                let mut style = Style::default().fg(color);
                if is_selected && can_repair {
                    style = style.add_modifier(Modifier::BOLD).fg(Color::Yellow);
                } else if is_selected {
                    style = style.add_modifier(Modifier::BOLD);
                } else if !can_repair {
                    style = style.fg(Color::DarkGray);
                }

                let prefix = if is_selected { ">" } else { " " };
                lines.push(Line::from(Span::styled(
                    format!("{} {}: {} ({})", prefix, part.name, label, part.integrity),
                    style,
                )));
            }
        }
    } else {
        lines.push(Line::from("No body data"));
    }

    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("Body Parts"))
        .alignment(Alignment::Left);

    f.render_widget(paragraph, area);
}

use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn render_tutorial_panel(f: &mut Frame, app: &mut crate::App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();

    // no header needed as our panel has a title

    // Check if there's an active tutorial
    if let Some(current_step) = app.ui.tutorial_system.current_step() {
        // Active tutorial - show current step
        let current_title = format!("Current Lesson: [{}]", current_step.title);
        lines.push(Line::from(Span::styled(
            current_title,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        // Show instruction
        lines.push(Line::from(Span::styled(
            "Instructions:",
            Style::default().fg(Color::Green),
        )));
        lines.push(Line::from(current_step.instruction.clone()));
        lines.push(Line::from(""));

        // Show what key to press
        match &current_step.action {
            crate::tutorialsystem::TutorialAction::KeyPress(key) => {
                let key_hint = match key {
                    crossterm::event::KeyCode::Char(c) => {
                        format!("Press [{}] to continue", c.to_uppercase())
                    }
                    _ => format!("Press [{:?}] to continue", key),
                };
                lines.push(Line::from(Span::styled(
                    key_hint,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )));
            }
            crate::tutorialsystem::TutorialAction::Keybind { category, action } => {
                // Get the configured key for this keybind
                let key_name = app
                    .core
                    .config_manager
                    .get_printable_key_for_keybind(category, action);
                let key_hint = format!("Press [{}] to continue", key_name);
                lines.push(Line::from(Span::styled(
                    key_hint,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )));
            }
            crate::tutorialsystem::TutorialAction::KeybindAdjacentToTile {
                category,
                action,
                tile,
            } => {
                // Get the configured key for this keybind
                let key_name = app
                    .core
                    .config_manager
                    .get_printable_key_for_keybind(category, action);

                // Get tile display name from tile registry
                let tile_name = tile.display_name();

                let key_hint = format!("Stand next to a {} and press [{}]", tile_name, key_name);
                lines.push(Line::from(Span::styled(
                    key_hint,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )));
            }
            crate::tutorialsystem::TutorialAction::AnyKey => {
                lines.push(Line::from(Span::styled(
                    "Press any key to continue",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )));
            }
            crate::tutorialsystem::TutorialAction::PickupItem { item, quantity } => {
                let required_qty = quantity.unwrap_or(1);
                let item_name = lithicrivers_core::components::itemkind_name(*item);

                let pickup_hint = if required_qty == 1 {
                    format!("Pick up a {}", item_name.to_lowercase())
                } else {
                    format!("Pick up {} {}", required_qty, item_name.to_lowercase())
                };

                lines.push(Line::from(Span::styled(
                    pickup_hint,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )));

                // Show current progress if player has some of the item
                if let Some((player_entity, _)) = app
                    .core
                    .game
                    .world
                    .query::<(
                        &lithicrivers_core::components::Player,
                        &lithicrivers_core::components::Inventory,
                    )>()
                    .iter()
                    .next()
                {
                    if let Ok(inventory) =
                        app.core
                            .game
                            .world
                            .get::<&lithicrivers_core::components::Inventory>(player_entity)
                    {
                        let current_qty: u32 = inventory
                            .slots
                            .iter()
                            .filter(|stack| stack.kind == *item)
                            .map(|stack| stack.qty)
                            .sum();

                        if current_qty > 0 {
                            let progress_text = format!(
                                "Progress: {} / {} {}",
                                current_qty,
                                required_qty,
                                item_name.to_lowercase()
                            );
                            lines.push(Line::from(Span::styled(
                                progress_text,
                                Style::default().fg(Color::Cyan),
                            )));
                        }
                    }
                }
            }
            crate::tutorialsystem::TutorialAction::MovementKeys {
                required_keys,
                pressed_keys,
            } => {
                let progress_text = format!(
                    "Movement Progress: {} / {} keys pressed",
                    pressed_keys.len(),
                    required_keys.len()
                );
                lines.push(Line::from(Span::styled(
                    progress_text,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )));

                // Show which keys are still needed
                let remaining: Vec<_> = required_keys
                    .iter()
                    .filter(|key| !pressed_keys.contains(key))
                    .collect();

                if !remaining.is_empty() {
                    let remaining_text = format!(
                        "Please press: {}",
                        remaining
                            .iter()
                            .map(|k| match k {
                                crossterm::event::KeyCode::Char(c) => c.to_string(),
                                crossterm::event::KeyCode::Left => "LEFT ARROW".to_string(),
                                crossterm::event::KeyCode::Right => "RIGHT ARROW".to_string(),
                                crossterm::event::KeyCode::Up => "UP ARROW".to_string(),
                                crossterm::event::KeyCode::Down => "DOWN ARROW".to_string(),
                                _ => format!("{:?}", k),
                            })
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                    lines.push(Line::from(Span::styled(
                        remaining_text,
                        Style::default().fg(Color::Yellow),
                    )));
                }
            }
            crate::tutorialsystem::TutorialAction::MultipleKeybinds {
                required_keybinds,
                pressed_keybinds,
            } => {
                let progress_text = format!(
                    "Lesson Progress: {} / {} actions completed",
                    pressed_keybinds.len(),
                    required_keybinds.len()
                );
                lines.push(Line::from(Span::styled(
                    progress_text,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )));

                // Show which keybinds are still needed
                let remaining: Vec<_> = required_keybinds
                    .iter()
                    .filter(|keybind| !pressed_keybinds.contains(keybind))
                    .collect();

                if !remaining.is_empty() {
                    let remaining_text = format!(
                        "Please press: {}",
                        remaining
                            .iter()
                            .map(|(category, action)| {
                                // Get the actual configured key for this keybind
                                let key_name = app
                                    .core
                                    .config_manager
                                    .get_printable_key_for_keybind(category, action);
                                format!("[{}]", key_name)
                            })
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                    lines.push(Line::from(Span::styled(
                        remaining_text,
                        Style::default().fg(Color::Yellow),
                    )));
                }
            }
            _ => {
                lines.push(Line::from(Span::styled(
                    "Follow the instructions above",
                    Style::default().fg(Color::Yellow),
                )));
            }
        }
        lines.push(Line::from(""));

        // Show category progress
        let category_progress = format!(
            "Category Progress: {} / {}",
            app.ui.tutorial_system.current_step_index + 1,
            app.ui.tutorial_system.steps.len()
        );
        lines.push(Line::from(Span::styled(
            category_progress,
            Style::default().fg(Color::Blue),
        )));
        lines.push(Line::from(""));

        // Show completed tutorials
        if !app.ui.tutorial_system.completed_tutorials.is_empty() {
            lines.push(Line::from(Span::styled(
                "Completed:",
                Style::default().fg(Color::Green),
            )));
            for tutorial in &app.ui.tutorial_system.completed_tutorials {
                lines.push(Line::from(format!("  ✓ {}", tutorial)));
            }
        }
    } else {
        // No active tutorial
        lines.push(Line::from(Span::styled(
            "No active tutorial",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(""));

        lines.push(Line::from("Available tutorials:"));

        // Get tutorials grouped by category from the database
        let categories = crate::tutorialsystem::TutorialDatabase::get_tutorials_by_category();

        // Display tutorials grouped by category
        for (category, tutorials) in &categories {
            lines.push(Line::from(Span::styled(
                format!("  {}:", category),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )));
            for tutorial in tutorials {
                // Check if this tutorial is completed
                let is_completed = app
                    .ui
                    .tutorial_system
                    .completed_tutorials
                    .contains(&tutorial.id);
                let status_icon = if is_completed { "✓" } else { "•" };
                let style = if is_completed {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::White)
                };

                lines.push(Line::from(Span::styled(
                    format!("    {} {}", status_icon, tutorial.title),
                    style,
                )));

                // Show description for non-completed tutorials
                if !is_completed {
                    lines.push(Line::from(Span::styled(
                        format!("      {}", tutorial.description),
                        Style::default().fg(Color::DarkGray),
                    )));
                }
            }
            lines.push(Line::from(""));
        }

        // Remove the extra empty line if we added any categories
        if !categories.is_empty() {
            lines.pop(); // Remove last empty line
        }
        lines.push(Line::from(""));

        // Show completed tutorials if any
        if !app.ui.tutorial_system.completed_tutorials.is_empty() {
            lines.push(Line::from(Span::styled(
                "Completed:",
                Style::default().fg(Color::Green),
            )));
            for tutorial in &app.ui.tutorial_system.completed_tutorials {
                lines.push(Line::from(format!("  ✓ {}", tutorial)));
            }
            lines.push(Line::from(""));
        }

        lines.push(Line::from(Span::styled(
            "Press SHIFT+T to select and start any tutorial",
            Style::default().fg(Color::Yellow),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Controls:",
        Style::default().fg(Color::Cyan),
    )));
    lines.push(Line::from(
        "SHIFT-T - Open tutorial modal: pick a new or old tutorial to run through.",
    ));
    lines.push(Line::from("SHIFT-Y - Skip current tutorial step."));
    lines.push(Line::from(
        "/toggle_tutorial - Disable entire tutorial system.",
    ));

    // Create the paragraph widget
    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .title(Line::from("Tutorial").alignment(Alignment::Center))
                .title_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Left);

    f.render_widget(paragraph, area);
}

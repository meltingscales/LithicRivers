use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn render_tutorial_panel(f: &mut Frame, app: &mut crate::App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();

    // Header
    lines.push(Line::from(Span::styled(
        "📚 Tutorial System",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    // Check if there's an active tutorial
    if let Some(current_step) = app.ui.tutorial_system.current_step() {
        // Active tutorial - show current step
        let current_title = format!("Current: {}", current_step.title);
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
                        format!("Press '{}' to continue", c.to_uppercase())
                    }
                    _ => format!("Press {:?} to continue", key),
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
                let key_hint = format!("Press {} to continue", key_name);
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
                        "Still need: {}",
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
            _ => {
                lines.push(Line::from(Span::styled(
                    "Follow the instructions above",
                    Style::default().fg(Color::Yellow),
                )));
            }
        }
        lines.push(Line::from(""));

        // Show progress
        let progress = format!(
            "Progress: {} / {}",
            app.ui.tutorial_system.current_step_index + 1,
            app.ui.tutorial_system.steps.len()
        );
        lines.push(Line::from(Span::styled(
            progress,
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
        lines.push(Line::from("• Movement and Navigation"));
        lines.push(Line::from("• Inventory Management"));
        lines.push(Line::from("• Crafting System"));
        lines.push(Line::from("• Combat Basics"));
        lines.push(Line::from("• Building and Mining"));
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
            "Press F1 to start a tutorial",
            Style::default().fg(Color::Yellow),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Controls:",
        Style::default().fg(Color::Cyan),
    )));
    lines.push(Line::from("ESC - Skip current tutorial"));
    lines.push(Line::from("F1 - Toggle tutorial panel"));

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

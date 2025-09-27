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

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

/// Render the tutorial selection modal
pub fn render_tutorial_selection_modal(f: &mut Frame, app: &mut crate::App) {
    if let crate::app_state::TutorialSelectionState::SelectingTutorial { selected_tutorial } =
        &app.panels.tutorial_selection
    {
        let available_tutorials =
            crate::tutorialsystem::TutorialDatabase::get_available_tutorials();

        if available_tutorials.is_empty() {
            return;
        }

        // Create modal area (centered, 70% width, 60% height)
        let popup_area = centered_rect(70, 60, f.size());

        // Clear the area first
        f.render_widget(Clear, popup_area);

        // Main modal block
        let modal_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title(Line::from("Select Tutorial").alignment(Alignment::Center))
            .title_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );

        let inner = modal_block.inner(popup_area);
        f.render_widget(modal_block, popup_area);

        // Split the inner area into sections
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Instructions
                Constraint::Min(0),    // Tutorial list
                Constraint::Length(2), // Controls
            ])
            .split(inner);

        // Instructions
        let instructions = vec![
            Line::from("Choose a tutorial to start or restart:"),
            Line::from(""),
        ];
        let instructions_paragraph = Paragraph::new(instructions)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Left);
        f.render_widget(instructions_paragraph, chunks[0]);

        // Tutorial list
        let mut list_items = Vec::new();

        // Get tutorials grouped by category (but we need to track indices for selection)
        let mut tutorial_index = 0;
        let categories = crate::tutorialsystem::TutorialDatabase::get_tutorials_by_category();

        for (category, tutorials) in &categories {
            // Category header
            list_items.push(ListItem::new(Line::from(Span::styled(
                format!("{}:", category),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ))));

            for tutorial in tutorials {
                let is_selected = tutorial_index == *selected_tutorial;
                let is_completed = app
                    .ui
                    .tutorial_system
                    .completed_tutorials
                    .contains(&tutorial.id);

                let status_icon = if is_completed { "✓" } else { "•" };
                let style = if is_selected {
                    Style::default().fg(Color::Black).bg(Color::Yellow)
                } else if is_completed {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::White)
                };

                let mut spans = vec![
                    Span::styled("  ", style),
                    Span::styled(format!("{} ", status_icon), style),
                    Span::styled(&tutorial.title, style),
                ];

                if is_completed {
                    spans.push(Span::styled(
                        " (Completed)",
                        Style::default().fg(Color::Green),
                    ));
                }

                list_items.push(ListItem::new(Line::from(spans)));

                // Show description if selected
                if is_selected {
                    list_items.push(ListItem::new(Line::from(Span::styled(
                        format!("    {}", tutorial.description),
                        Style::default().fg(Color::DarkGray),
                    ))));
                }

                tutorial_index += 1;
            }

            // Add spacing between categories
            list_items.push(ListItem::new(Line::from("")));
        }

        let list = List::new(list_items).style(Style::default().fg(Color::White));
        f.render_widget(list, chunks[1]);

        // Controls
        let controls = vec![Line::from(vec![
            Span::styled(
                "↑↓",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Navigate  "),
            Span::styled(
                "Enter",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Select  "),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Cancel"),
        ])];
        let controls_paragraph = Paragraph::new(controls)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center);
        f.render_widget(controls_paragraph, chunks[2]);
    }
}

/// Helper function to create a centered rect
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

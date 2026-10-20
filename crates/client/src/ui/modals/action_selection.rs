//! Multi-action and action selection modals.

use crate::ui::centered_rect;
use crate::App;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Render the multi-action selection modal
pub(crate) fn render_multi_action_selection_modal(f: &mut Frame, app: &mut App) {
    use crate::app_state::MultiActionSelectState;

    match &app.panels.multi_action_select {
        MultiActionSelectState::SelectingAction {
            available_actions,
            selected_action,
        } => {
            let actions = available_actions.clone();
            let selected = *selected_action;
            render_action_selection_modal(f, app, &actions, selected);
        }
        MultiActionSelectState::None => {
            // No modal to render
        }
    }
}

/// Render the action selection modal
fn render_action_selection_modal(
    f: &mut Frame,
    app: &mut App,
    available_actions: &[crate::app_state::InteractionType],
    selected_action: usize,
) {
    use ratatui::{
        style::Modifier,
        widgets::{Clear, List, ListItem},
    };

    // Create modal area (centered, 60% width, 50% height)
    let area = centered_rect(60, 50, f.size());

    // Clear the background
    f.render_widget(Clear, area);

    // Main modal block
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Line::from(" Choose Interaction "))
        .title_alignment(Alignment::Center)
        .style(Style::default());

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Get player position for directional information
    let player_pos = app
        .core
        .game
        .get_player_entity()
        .and_then(|e| {
            app.core
                .game
                .world
                .get::<&lithicrivers_core::components::Position>(e)
                .ok()
        })
        .map(|pos| *pos);

    // Create list of available actions
    let action_items: Vec<ListItem> = available_actions
        .iter()
        .enumerate()
        .map(|(i, action)| {
            let is_selected = selected_action == i;
            let style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let display_text = format!(
                "{} {}",
                action.icon(),
                action.display_name_with_context(&app.core.game.world, player_pos)
            );
            ListItem::new(display_text).style(style)
        })
        .collect();

    let list = List::new(action_items)
        .block(Block::default())
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    // Render the list with selection, leaving space for controls
    let list_height = inner.height.saturating_sub(2);
    let list_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: list_height,
    };

    f.render_widget(list, list_area);

    // Controls at bottom
    let controls = Paragraph::new("↑↓: Select | Enter: Choose | Esc: Cancel")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));

    if inner.height > 1 {
        let controls_area = Rect {
            x: inner.x,
            y: inner.y + inner.height - 1,
            width: inner.width,
            height: 1,
        };
        f.render_widget(controls, controls_area);
    }
}

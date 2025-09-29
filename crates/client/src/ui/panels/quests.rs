use crate::app_state::QuestSection;
use crate::App;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render_quests_panel(f: &mut Frame, app: &App, area: Rect) {
    // Clear the area first
    f.render_widget(Clear, area);

    // Split the area into two panels: Active (left) and Completed (right)
    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Render Active Quests Panel
    render_active_quests_panel(f, app, chunks[0]);

    // Render Completed Quests Panel
    render_completed_quests_panel(f, app, chunks[1]);
}

fn render_active_quests_panel(f: &mut Frame, app: &App, area: Rect) {
    let active_quests: Vec<_> = app
        .core
        .game
        .res
        .active_quests
        .iter()
        .filter(|q| q.state == lithicrivers_core::dialogue::QuestState::Active)
        .collect();

    let mut quest_text = String::new();

    if active_quests.is_empty() {
        quest_text.push_str("(No active quests)\n");
    } else {
        // Show quests with selection highlighting (only for active section)
        for (i, quest) in active_quests.iter().enumerate() {
            let is_selected = app.panels.quests.selected_section == QuestSection::Active
                && i == app.panels.quests.selected_quest_index;
            let selection_marker = if is_selected { "►" } else { " " };

            quest_text.push_str(&format!("{} [ ] {}\n", selection_marker, quest.name));
            if is_selected {
                quest_text.push_str(&format!("    {}\n", quest.description));
                // Show objectives for selected quest
                for objective in quest.objectives.iter() {
                    let status = if objective.completed { "✓" } else { " " };
                    quest_text.push_str(&format!("    [{}] {}\n", status, objective.description));
                }
            }
            quest_text.push('\n');
        }
    }

    // Add navigation instructions for active panel
    if app.panels.quests.selected_section == QuestSection::Active {
        quest_text.push_str("─────────────────────────\n");
        quest_text.push_str("↑/↓ Select   Tab: Switch");
    }

    // Highlight the active panel border
    let border_style = if app.panels.quests.selected_section == QuestSection::Active {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let active_panel = Paragraph::new(quest_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Active Quests")
                .border_style(border_style),
        )
        .wrap(ratatui::widgets::Wrap { trim: true });

    f.render_widget(active_panel, area);
}

fn render_completed_quests_panel(f: &mut Frame, app: &App, area: Rect) {
    let completed_quests: Vec<_> = app
        .core
        .game
        .res
        .active_quests
        .iter()
        .filter(|q| q.state == lithicrivers_core::dialogue::QuestState::Completed)
        .collect();

    let mut quest_text = String::new();

    if completed_quests.is_empty() {
        quest_text.push_str("(No completed quests)\n");
    } else {
        // Show quests with selection highlighting (only for completed section)
        for (i, quest) in completed_quests.iter().enumerate() {
            let is_selected = app.panels.quests.selected_section == QuestSection::Completed
                && i == app.panels.quests.selected_quest_index;
            let selection_marker = if is_selected { "►" } else { " " };

            quest_text.push_str(&format!("{} [✓] {}\n", selection_marker, quest.name));
            if is_selected {
                quest_text.push_str(&format!("    {}\n", quest.description));
            }
            quest_text.push('\n');
        }
    }

    // Add navigation instructions for completed panel
    if app.panels.quests.selected_section == QuestSection::Completed {
        quest_text.push_str("─────────────────────────\n");
        quest_text.push_str("↑/↓ Select   Tab: Switch");
    }

    // Highlight the active panel border
    let border_style = if app.panels.quests.selected_section == QuestSection::Completed {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let completed_panel = Paragraph::new(quest_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Completed Quests")
                .border_style(border_style),
        )
        .wrap(ratatui::widgets::Wrap { trim: true });

    f.render_widget(completed_panel, area);
}

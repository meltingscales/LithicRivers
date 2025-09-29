use crate::App;
use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render_quests_panel(f: &mut Frame, app: &App, area: Rect) {
    // Clear the area first
    f.render_widget(Clear, area);

    let mut quest_text = String::new();

    // Active Quests Section
    quest_text.push_str("Active Quests\n\n");

    let active_quests: Vec<_> = app
        .core
        .game
        .res
        .active_quests
        .iter()
        .filter(|q| q.state == lithicrivers_core::dialogue::QuestState::Active)
        .collect();

    if active_quests.is_empty() {
        quest_text.push_str("(No active quests)\n");
    } else {
        for quest in active_quests {
            quest_text.push_str(&format!("[ ] {}\n", quest.name));
            quest_text.push_str(&format!("    {}\n", quest.description));

            // Show objectives
            for (_i, objective) in quest.objectives.iter().enumerate() {
                let status = if objective.completed { "✓" } else { " " };
                quest_text.push_str(&format!("    [{}] {}\n", status, objective.description));
            }
            quest_text.push('\n');
        }
    }

    quest_text.push_str("\nCompleted Quests\n\n");

    // Completed Quests Section
    let completed_quests: Vec<_> = app
        .core
        .game
        .res
        .active_quests
        .iter()
        .filter(|q| q.state == lithicrivers_core::dialogue::QuestState::Completed)
        .collect();

    if completed_quests.is_empty() {
        quest_text.push_str("(None yet)");
    } else {
        for quest in completed_quests {
            quest_text.push_str(&format!("[✓] {}\n", quest.name));
            quest_text.push_str(&format!("    {}\n\n", quest.description));
        }
    }

    let quest_panel = Paragraph::new(quest_text)
        .block(Block::default().borders(Borders::ALL).title("Quests"))
        .wrap(ratatui::widgets::Wrap { trim: true });

    f.render_widget(quest_panel, area);
}

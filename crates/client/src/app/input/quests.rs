use crate::app_state::QuestSection;
use crate::{App, MenuTab};
use crossterm::event::KeyCode;
use std::error::Error;

pub fn handle_input(app: &mut App, key: KeyCode) -> Result<bool, Box<dyn Error>> {
    // Only handle input when Quests tab is active
    if app.ui.current_tab != MenuTab::Quests {
        return Ok(false);
    }

    match key {
        // Up arrow - previous quest
        KeyCode::Up => {
            let quest_count = match app.panels.quests.selected_section {
                QuestSection::Active => app
                    .core
                    .game
                    .res
                    .active_quests
                    .iter()
                    .filter(|q| q.state == lithicrivers_core::dialogue::QuestState::Active)
                    .count(),
                QuestSection::Completed => app
                    .core
                    .game
                    .res
                    .active_quests
                    .iter()
                    .filter(|q| q.state == lithicrivers_core::dialogue::QuestState::Completed)
                    .count(),
            };

            if quest_count > 0 {
                if app.panels.quests.selected_quest_index > 0 {
                    app.panels.quests.selected_quest_index -= 1;
                } else {
                    app.panels.quests.selected_quest_index = quest_count - 1;
                }
            }
            return Ok(true);
        }
        // Down arrow - next quest
        KeyCode::Down => {
            let quest_count = match app.panels.quests.selected_section {
                QuestSection::Active => app
                    .core
                    .game
                    .res
                    .active_quests
                    .iter()
                    .filter(|q| q.state == lithicrivers_core::dialogue::QuestState::Active)
                    .count(),
                QuestSection::Completed => app
                    .core
                    .game
                    .res
                    .active_quests
                    .iter()
                    .filter(|q| q.state == lithicrivers_core::dialogue::QuestState::Completed)
                    .count(),
            };

            if quest_count > 0 {
                app.panels.quests.selected_quest_index =
                    (app.panels.quests.selected_quest_index + 1) % quest_count;
            }
            return Ok(true);
        }
        // Tab - switch between Active and Completed sections
        KeyCode::Tab => {
            app.panels.quests.selected_section = match app.panels.quests.selected_section {
                QuestSection::Active => QuestSection::Completed,
                QuestSection::Completed => QuestSection::Active,
            };
            // Reset selection when switching sections
            app.panels.quests.selected_quest_index = 0;
            return Ok(true);
        }
        _ => return Ok(false),
    }
}

use crate::App;
use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render_quests_panel(f: &mut Frame, _app: &App, area: Rect) {
    // Clear the area first
    f.render_widget(Clear, area);

    // For now, render a placeholder quest panel
    let quest_text = "Active Quests\n\n[ ] Repair the Broken Android\n    Find a lab-grown diamond and scrap electronics\n    Location: SapienCorp Factory\n\n\nCompleted Quests\n\n(None yet)";

    let quest_panel = Paragraph::new(quest_text)
        .block(Block::default().borders(Borders::ALL).title("Quests"))
        .wrap(ratatui::widgets::Wrap { trim: true });

    f.render_widget(quest_panel, area);
}

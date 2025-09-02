use ratatui::{
    layout::{Alignment, Rect},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn render_credits_panel(f: &mut Frame, app: &mut crate::App, area: Rect) {
    // Build a scrollable paragraph from preloaded embedded text
    let block = Block::default().borders(Borders::ALL).title("Credits");
    let inner = block.inner(area);
    let para = Paragraph::new(app.panels.credits.text.clone())
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false })
        .scroll((app.panels.credits.scroll, 0));
    f.render_widget(para, inner);
    f.render_widget(block, area);
}

use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_quit_panel(f: &mut Frame, _app: &mut crate::App, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title("Quit");
    let inner = block.inner(area);
    let mut lines: Vec<Line<'static>> = Vec::new();
    let width = inner.width as usize;
    let height = inner.height as usize;
    // Build a staggered (diamond-like) pattern:
    // rows alternate between starting with 0 and an offset, then repeating "QUIT" with wide spacing
    let word = "QUIT";
    let sep = "        "; // 8 spaces between words
    let offset = "      "; // 6 spaces offset on alternating rows
    for row in 0..height {
        let mut s = String::new();
        if row % 2 == 1 {
            s.push_str(offset);
        }
        // fill line with repeating pattern
        while s.len() < width + word.len() + sep.len() {
            s.push_str(word);
            s.push_str(sep);
        }
        // Trim to visible width
        s.truncate(width);
        lines.push(Line::from(Span::raw(s)));
    }
    let p = Paragraph::new(lines)
        .alignment(Alignment::Left)
        .style(Style::default().fg(Color::Red));
    f.render_widget(p, inner);
    f.render_widget(block, area);
}

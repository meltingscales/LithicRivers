use super::tutorial_step::{HighlightArea, TutorialStep};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

/// UI component for rendering tutorial overlays and highlights
#[allow(dead_code)]
pub struct TutorialOverlay;

#[allow(dead_code)]
impl TutorialOverlay {
    /// Render the tutorial panel at the bottom of the screen
    pub fn render_tutorial_panel(frame: &mut Frame, area: Rect, step: &TutorialStep) {
        // Tutorial panel at bottom of screen
        let panel_height = 4;
        let panel_area = Rect {
            x: area.x,
            y: area.height.saturating_sub(panel_height),
            width: area.width,
            height: panel_height,
        };

        // Clear the area first
        frame.render_widget(Clear, panel_area);

        // Create tutorial content
        let title_text = format!("📚 Tutorial: {}", step.title);
        let instruction_text = step.instruction.clone();

        let content = format!("{}\n{}", title_text, instruction_text);

        let paragraph = Paragraph::new(content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow))
                    .title("Tutorial")
                    .title_style(
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
            )
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, panel_area);
    }

    /// Render highlight overlay for specific areas
    pub fn render_highlight(frame: &mut Frame, area: Rect, highlight: &HighlightArea) {
        let highlight_area = Rect {
            x: highlight.x.min(area.width.saturating_sub(1)),
            y: highlight.y.min(area.height.saturating_sub(1)),
            width: highlight.width.min(area.width.saturating_sub(highlight.x)),
            height: highlight
                .height
                .min(area.height.saturating_sub(highlight.y)),
        };

        // Create a highlighting border
        let highlight_block = Block::default()
            .borders(Borders::ALL)
            .border_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .title(highlight.description.as_str())
            .title_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_widget(highlight_block, highlight_area);
    }

    /// Render tutorial controls help text
    pub fn render_tutorial_controls(frame: &mut Frame, area: Rect) {
        let help_text = "Press ESC to skip tutorial | F1 to toggle tutorial panel";

        let controls_area = Rect {
            x: area.x,
            y: 0,
            width: area.width,
            height: 1,
        };

        let paragraph = Paragraph::new(help_text)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, controls_area);
    }

    /// Calculate safe area that doesn't overlap with tutorial panel
    pub fn get_main_content_area(total_area: Rect, tutorial_active: bool) -> Rect {
        if tutorial_active {
            // Reserve space at bottom for tutorial panel
            Rect {
                x: total_area.x,
                y: total_area.y + 1, // Reserve top line for controls
                width: total_area.width,
                height: total_area.height.saturating_sub(5), // Reserve bottom for panel
            }
        } else {
            total_area
        }
    }
}

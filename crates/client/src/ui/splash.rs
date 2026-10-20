//! Splash screens shown before the main game UI (logo, game title, boot message).

use crate::ui::centered_rect;
use crate::{App, SplashState};
use ratatui::{
    layout::{Alignment, Margin, Rect},
    style::{Color, Style},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

/// Draw the current splash screen, if one is showing. Returns true when it drew one.
pub(crate) fn render_splash(f: &mut Frame, app: &mut App) -> bool {
    match app.splash.state {
        SplashState::Logo => {
            // Show centered logo
            let logo_paragraph = Paragraph::new(app.splash.logo_text.as_str())
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::NONE));

            // Center the logo in the middle of the screen
            let area = centered_rect(50, 50, f.size());
            f.render_widget(logo_paragraph, area);
            return true;
        }
        SplashState::GameTitle => {
            // Show centered game title
            let title_paragraph = Paragraph::new(app.splash.game_title_text.as_str())
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::NONE));

            // Center the title in the middle of the screen
            let area = centered_rect(70, 70, f.size());
            f.render_widget(title_paragraph, area);
            return true;
        }
        SplashState::BootMessage => {
            // Show boot message with glitch effect
            let block = Block::default()
                .borders(Borders::ALL)
                .border_set(border::THICK)
                .title(Line::from(" System Boot "))
                .title_alignment(Alignment::Center)
                .border_style(Style::default().fg(Color::LightBlue));

            // Inner and centered content area with padding
            let inner = block.inner(Rect::new(0, 0, f.size().width, f.size().height));
            let area = centered_rect(80, 60, inner);

            // Clear content area before drawing
            f.render_widget(Clear, area);

            // Calculate available height for text (accounting for borders and padding)
            let inner_area = area.inner(&Margin {
                horizontal: 2,
                vertical: 2,
            });
            let text_area = inner_area.inner(&Margin {
                horizontal: 1,
                vertical: 1,
            });

            // Calculate visible lines and update scroll position if needed
            let visible_lines = text_area.height.saturating_sub(2); // Leave room for border
            let total_lines = app.splash.boot_display_text.lines().count() as u16;

            // Auto-scroll if we're at the bottom
            if app.splash.boot_scroll + visible_lines >= total_lines.saturating_sub(1) {
                app.splash.boot_scroll = total_lines.saturating_sub(visible_lines);
            }

            // Create a scrollable paragraph
            let paragraph = Paragraph::new(app.splash.boot_display_text.as_str())
                .block(Block::default().borders(Borders::NONE))
                .wrap(Wrap { trim: false })
                .scroll((app.splash.boot_scroll, 0));

            // Render content with padding
            f.render_widget(paragraph, inner_area);

            // Add scrollbar if needed
            if total_lines > visible_lines {
                // Create a simple scrollbar on the right
                let scrollbar_area = Rect {
                    x: inner_area.right() - 1,
                    y: inner_area.y,
                    width: 1,
                    height: inner_area.height,
                };

                // Calculate scrollbar thumb position and height
                let thumb_height = (visible_lines as f32 / total_lines as f32
                    * visible_lines as f32)
                    .max(1.0) as u16;
                let thumb_position = (app.splash.boot_scroll as f32
                    / (total_lines - visible_lines) as f32
                    * (visible_lines - thumb_height) as f32)
                    .round() as u16;

                // Draw scrollbar track
                let track_span = Span::styled("│", Style::default().fg(Color::DarkGray));
                for y in inner_area.top()..inner_area.bottom() {
                    f.render_widget(
                        Paragraph::new(track_span.clone()),
                        Rect::new(scrollbar_area.x, y, 1, 1),
                    );
                }

                // Draw scrollbar thumb
                let thumb_span = Span::styled("▐", Style::default().fg(Color::LightBlue));
                for y in 0..thumb_height {
                    let y_pos = inner_area.y + thumb_position + y;
                    if y_pos < inner_area.bottom() {
                        f.render_widget(
                            Paragraph::new(thumb_span.clone()),
                            Rect::new(scrollbar_area.x, y_pos, 1, 1),
                        );
                    }
                }
            }

            // Render border last to ensure it's on top
            f.render_widget(block, area);
            return true;
        }
        SplashState::MainUI => {
            // Normal UI rendering continues below
        }
    }
    false
}

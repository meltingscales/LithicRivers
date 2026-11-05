//! Splash screens shown before the main game UI (logo, game title, boot message).

use crate::ui::centered_rect;
use crate::{App, SplashState};
use ratatui::{
    layout::{Alignment, Constraint, Margin, Rect},
    style::{Color, Style},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

/// A box exactly as big as some text, centered in an area (and no bigger than the area),
/// so splash art is never cut off while the screen is large enough to hold it.
fn text_rect(text: &str, area: Rect) -> Rect {
    let width = text
        .lines()
        .map(|l| Line::from(l).width())
        .max()
        .unwrap_or(0) as u16;
    let height = text.lines().count() as u16;
    area.centered(
        Constraint::Length(width.min(area.width)),
        Constraint::Length(height.min(area.height)),
    )
}

/// Draw the current splash screen, if one is showing. Returns true when it drew one.
pub(crate) fn render_splash(f: &mut Frame, app: &mut App) -> bool {
    if crate::ui::title::render_title_menu(f, app) {
        return true;
    }
    match app.splash.state {
        SplashState::Logo => {
            // Show centered logo
            let logo_paragraph = Paragraph::new(app.splash.logo_text.as_str())
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::NONE));

            let area = text_rect(&app.splash.logo_text, f.area());
            f.render_widget(logo_paragraph, area);
            return true;
        }
        SplashState::GameTitle => {
            // Show centered game title
            let title_paragraph = Paragraph::new(app.splash.game_title_text.as_str())
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::NONE));

            let area = text_rect(&app.splash.game_title_text, f.area());
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

            f.render_widget(
                crate::ui::boot_backdrop::BootBackdrop {
                    millis: app.clock.animation_millis(),
                },
                f.area(),
            );

            // Inner and centered content area with padding
            let inner = block.inner(Rect::new(0, 0, f.area().width, f.area().height));
            let area = centered_rect(80, 60, inner);

            // Clear content area before drawing
            f.render_widget(Clear, area);

            // The text fills the padded area, minus the scrollbar's column on the right
            let inner_area = area.inner(Margin {
                horizontal: 2,
                vertical: 2,
            });
            let text_area = Rect {
                width: inner_area.width.saturating_sub(2),
                ..inner_area
            };

            let paragraph =
                Paragraph::new(app.splash.boot_display_text.as_str()).wrap(Wrap { trim: false });
            let visible_lines = text_area.height;
            let total_lines = paragraph.line_count(text_area.width) as u16;
            let max_scroll = total_lines.saturating_sub(visible_lines);

            // Follow new lines while the view is at (or a line short of) the bottom
            if app.splash.boot_scroll + 1 >= max_scroll {
                app.splash.boot_scroll = max_scroll;
            }
            app.splash.boot_scroll = app.splash.boot_scroll.min(max_scroll);

            f.render_widget(paragraph.scroll((app.splash.boot_scroll, 0)), text_area);

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
                let thumb_position = (app.splash.boot_scroll as f32 / max_scroll as f32
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
        _ => {
            // Main UI (title-menu screens were drawn above)
        }
    }
    false
}

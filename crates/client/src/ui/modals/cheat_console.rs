//! Cheat console modal.

use crate::app;
use crate::app_state::*;
use crate::ui::centered_rect;
use crate::App;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Render cheat console modal when active
pub(crate) fn render_cheat_console_modal(f: &mut Frame, app: &mut App) {
    if let CheatConsoleState::Open {
        input,
        cursor_position,
        autocomplete_suggestions,
        autocomplete_index: _,
        scroll_offset,
    } = &app.panels.cheat_console
    {
        // Create modal area (centered, 60% width, taller height for multiline help)
        let area = centered_rect(60, 25, f.size());

        // Clear the background
        f.render_widget(Clear, area);

        // Main modal block
        let block = Block::default()
            .borders(Borders::ALL)
            .title(Line::from(" Cheat Console "))
            .title_alignment(Alignment::Center)
            .style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        f.render_widget(block, area);

        // Create input text with cursor
        let mut display_text = input.clone();
        if *cursor_position <= input.len() {
            display_text.insert(*cursor_position, '|');
        }

        // Input field
        let input_paragraph = Paragraph::new(format!("> {}", display_text))
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Left);

        let input_area = Rect {
            x: inner.x,
            y: inner.y + 1,
            width: inner.width,
            height: 1,
        };
        f.render_widget(input_paragraph, input_area);

        // Help text - show autocomplete suggestions if available, otherwise show all commands
        let help_lines = if !autocomplete_suggestions.is_empty() {
            vec![Line::from(format!(
                "Suggestions: {}",
                autocomplete_suggestions.join(", ")
            ))]
        } else {
            let registry = app::input::cheat_console::get_command_registry();
            let commands = registry.get_command_descriptions();
            let mut lines = vec![Line::from("Available commands:")];

            // Show 5 commands at a time (expanded visual size)
            let max_visible_commands = 5;
            let total_commands = commands.len();
            let start_index = *scroll_offset;
            let end_index = (start_index + max_visible_commands).min(total_commands);

            // Show scroll up indicator if we're not at the top
            if start_index > 0 {
                lines.push(Line::from("  ▲ (more commands above)"));
            }

            // Show visible commands
            for command in &commands[start_index..end_index] {
                lines.push(Line::from(format!("  {}", command)));
            }

            // Show scroll down indicator if there are more commands below
            if end_index < total_commands {
                lines.push(Line::from("  ▼ (more commands below)"));
            }

            // Add scroll instructions if there are more commands
            if total_commands > max_visible_commands {
                lines.push(Line::from(""));
                let up_key = app
                    .core
                    .config_manager
                    .get_printable_key_for_keybind("ui", "CHEAT_CONSOLE_SCROLL_UP");
                let down_key = app
                    .core
                    .config_manager
                    .get_printable_key_for_keybind("ui", "CHEAT_CONSOLE_SCROLL_DOWN");
                lines.push(Line::from(format!(
                    "Use {}/{} to scroll through commands",
                    up_key, down_key
                )));
            }

            lines
        };
        let help_paragraph = Paragraph::new(help_lines)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);

        let help_height = if !autocomplete_suggestions.is_empty() {
            1
        } else {
            6
        }; // 1 line for title + 5 commands
        let help_area = Rect {
            x: inner.x,
            y: inner.y + 3,
            width: inner.width,
            height: help_height,
        };
        f.render_widget(help_paragraph, help_area);

        // Controls
        let controls = Paragraph::new("Enter: Execute | Tab: Autocomplete | Esc: Cancel")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));

        if inner.height > 7 {
            // Need more height for multiline help + controls
            let controls_area = Rect {
                x: inner.x,
                y: inner.y + inner.height - 1,
                width: inner.width,
                height: 1,
            };
            f.render_widget(controls, controls_area);
        }
    }
}

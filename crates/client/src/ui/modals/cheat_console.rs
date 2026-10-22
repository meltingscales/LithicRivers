//! Cheat console modal.

use crate::app;
use crate::app_state::*;
use crate::ui::centered_rect;
use crate::App;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

/// Suggestion rows shown at once; the list scrolls to keep the picked one visible.
const MAX_SUGGESTION_ROWS: usize = 8;

/// Render cheat console modal when active
pub(crate) fn render_cheat_console_modal(f: &mut Frame, app: &mut App) {
    if let CheatConsoleState::Open {
        input,
        cursor_position,
        autocomplete_suggestions,
        autocomplete_index,
        scroll_offset,
    } = &app.panels.cheat_console
    {
        // Centered, 60% wide; taller while it lists suggestions
        let height = if autocomplete_suggestions.is_empty() {
            25
        } else {
            60
        };
        let area = centered_rect(60, height, f.size());

        // Clear the background
        f.render_widget(Clear, area);
        app.ui
            .click_targets
            .push(area, crate::ui::click_targets::ClickTarget::ModalArea);

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

        // Suggestions: a scrolling list under the input, one per line
        if !autocomplete_suggestions.is_empty() {
            let rows = (inner.height.saturating_sub(5) as usize)
                .min(MAX_SUGGESTION_ROWS)
                .min(autocomplete_suggestions.len())
                .max(1);
            let selected = autocomplete_index.unwrap_or(0);
            let first = selected.saturating_sub(rows - 1);
            let items: Vec<ListItem> = autocomplete_suggestions
                .iter()
                .enumerate()
                .skip(first)
                .take(rows)
                .map(|(i, s)| {
                    let style = if Some(i) == *autocomplete_index {
                        Style::default().fg(Color::Black).bg(Color::Cyan)
                    } else {
                        Style::default().fg(Color::Gray)
                    };
                    let marker = if Some(i) == *autocomplete_index {
                        "> "
                    } else {
                        "  "
                    };
                    ListItem::new(Line::from(Span::styled(format!("{marker}{s}"), style)))
                })
                .collect();
            let list = List::new(items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(
                        " Suggestions ({}) ",
                        autocomplete_suggestions.len()
                    ))
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
            let list_area = Rect {
                x: inner.x,
                y: inner.y + 2,
                width: inner.width,
                height: rows as u16 + 2,
            };
            f.render_widget(list, list_area);
            app.ui.click_targets.rows(
                crate::ui::click_targets::inset(list_area),
                first,
                rows,
                crate::ui::click_targets::ClickTarget::Suggestion,
            );
        }

        // Help text: every command, when there are no suggestions
        let help_lines = if !autocomplete_suggestions.is_empty() {
            Vec::new()
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
            0
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
        let up = app
            .core
            .config_manager
            .get_printable_key_for_keybind("ui", "CHEAT_CONSOLE_SCROLL_UP");
        let down = app
            .core
            .config_manager
            .get_printable_key_for_keybind("ui", "CHEAT_CONSOLE_SCROLL_DOWN");
        let controls = Paragraph::new(if autocomplete_suggestions.is_empty() {
            "Enter: Execute | Tab: Autocomplete | Esc: Cancel".to_string()
        } else {
            format!("{up}/{down}: pick | Enter: use pick, then run | Tab: complete | Esc: Cancel")
        })
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

//! NPC selection and dialogue modals.

use crate::app_state::*;
use crate::dialogue_presenter::DialoguePresenter;
use crate::ui::centered_rect;
use crate::App;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    symbols::border,
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

/// Render NPC interaction modals when active
pub(crate) fn render_npc_interaction_modals(f: &mut Frame, app: &mut App, viewport_area: Rect) {
    use crate::app_state::NPCInteractionState;
    let interaction_state = app.panels.npc_interaction.clone();
    match interaction_state {
        NPCInteractionState::SelectingNPC {
            adjacent_npcs,
            selected_npc,
        } => {
            render_npc_selection_modal(f, app, &adjacent_npcs, selected_npc, viewport_area);
        }
        NPCInteractionState::InDialogue {
            npc_entity,
            conversation: _,
            selected_choice,
        } => {
            render_npc_dialogue_modal(f, app, npc_entity, selected_choice, viewport_area);
        }
        NPCInteractionState::None => {
            // No modal to render
        }
    }
}

/// Render the NPC selection modal when multiple NPCs are nearby
fn render_npc_selection_modal(
    f: &mut Frame,
    app: &mut App,
    adjacent_npcs: &[(hecs::Entity, String)],
    selected_npc: usize,
    viewport_area: Rect,
) {
    // Create modal area constrained to viewport (centered, 60% width, 50% height)
    let area = centered_rect(60, 50, viewport_area);

    // Clear the background
    f.render_widget(Clear, area);

    // Create the modal block
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .title(Line::from(" Choose NPC to Talk To "))
        .title_style(
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(Color::Black).fg(Color::White));

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Create the list of NPCs
    let mut npc_lines = Vec::new();
    for (i, (npc_entity, npc_name)) in adjacent_npcs.iter().enumerate() {
        let prefix = if i == selected_npc { "→ " } else { "  " };

        // Get NPC position for display
        let pos_info = if let Ok(pos) = app
            .core
            .game
            .world
            .get::<&lithicrivers_core::components::Position>(*npc_entity)
        {
            format!(" at ({}, {})", pos.x, pos.y)
        } else {
            String::new()
        };

        let line = format!("{}{}{}", prefix, npc_name, pos_info);
        npc_lines.push(line);
    }

    // Render the list
    let npc_text = npc_lines.join("\n");
    let paragraph = Paragraph::new(npc_text)
        .alignment(Alignment::Left)
        .style(Style::default().fg(Color::White));

    if inner.height > 2 {
        let text_area = Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width,
            height: inner.height - 2,
        };
        f.render_widget(paragraph, text_area);
    }

    // Controls at bottom
    let controls = Paragraph::new("↑↓: Select | Enter: Talk | Esc: Cancel")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));

    if inner.height > 1 {
        let controls_area = Rect {
            x: inner.x,
            y: inner.y + inner.height - 1,
            width: inner.width,
            height: 1,
        };
        f.render_widget(controls, controls_area);
    }
}

/// Render the NPC dialogue modal
fn render_npc_dialogue_modal(
    f: &mut Frame,
    app: &mut App,
    npc_entity: hecs::Entity,
    selected_choice: usize,
    viewport_area: Rect,
) {
    // Create modal area constrained to viewport (not full screen)
    let area = centered_rect(90, 80, viewport_area);

    // Clear the background
    f.render_widget(Clear, area);

    // Create the modal block
    let npc_name = if let Ok(dialogue) = app
        .core
        .game
        .world
        .get::<&lithicrivers_core::components::Dialogue>(npc_entity)
    {
        dialogue.name.clone()
    } else {
        "Unknown NPC".to_string()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .title(Line::from(format!(" Talking to {} ", npc_name)))
        .title_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(Color::Black).fg(Color::White));

    let inner = block.inner(area);
    f.render_widget(block, inner);
    app.ui
        .click_targets
        .push(area, crate::ui::click_targets::ClickTarget::ModalArea);

    // Get current dialogue and both portraits using the new DialoguePresenter with Summon Night style
    let (dialogue_lines, npc_portrait, player_portrait) = if let NPCInteractionState::InDialogue {
        conversation,
        npc_entity,
        ..
    } = &app.panels.npc_interaction
    {
        DialoguePresenter::format_dialogue_with_portraits(
            &mut app.core.sprite_loader,
            &app.core.game.world,
            *npc_entity,
            &app.panels.dialogue_tree,
            conversation,
            selected_choice,
            &app.core.game.res,
            app.clock.animation_millis(),
        )
    } else {
        use ratatui::text::{Line, Span};
        (
            vec![Line::from(Span::styled(
                format!("{}: \"Hello, traveler!\"", npc_name),
                Style::default().fg(Color::White),
            ))],
            None,
            None,
        )
    };

    // Create Summon Night-style layout: portrait on left, dialogue on right
    if inner.height > 2 {
        let content_area = Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width,
            height: inner.height - 2,
        };

        // Clear the content area to prevent rendering artifacts on resize
        f.render_widget(Clear, content_area);

        // Split the content area horizontally for portrait and text
        let portrait_width = 16; // Width for 12x8 portrait + padding

        // Create Summon Night-style three-panel layout: NPC portrait | dialogue text | player portrait
        let has_npc = npc_portrait.is_some();
        let has_player = player_portrait.is_some();

        let left_width = if has_npc {
            portrait_width.min(content_area.width / 4)
        } else {
            0
        };
        let right_width = if has_player {
            portrait_width.min(content_area.width / 4)
        } else {
            0
        };
        let text_width = content_area.width - left_width - right_width;

        // NPC portrait area (left side)
        if let Some(npc_port) = npc_portrait {
            let npc_area = Rect {
                x: content_area.x,
                y: content_area.y,
                width: left_width,
                height: content_area.height,
            };

            // Create a properly sized bordered area for the 12x8 portrait
            // Add 2 to width and height for borders
            let portrait_bordered_width = 14; // 12 + 2 for borders
            let portrait_bordered_height = 10; // 8 + 2 for borders

            let portrait_area = Rect {
                x: npc_area.x + (npc_area.width.saturating_sub(portrait_bordered_width)) / 2,
                y: npc_area.y + (npc_area.height.saturating_sub(portrait_bordered_height)) / 2,
                width: portrait_bordered_width.min(npc_area.width),
                height: portrait_bordered_height.min(npc_area.height),
            };

            let npc_paragraph = Paragraph::new(npc_port)
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Cyan))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Gray)),
                );
            f.render_widget(npc_paragraph, portrait_area);
        }

        // Player portrait area (right side)
        if let Some(player_port) = player_portrait {
            let player_area = Rect {
                x: content_area.x + left_width + text_width,
                y: content_area.y,
                width: right_width,
                height: content_area.height,
            };

            // Create a properly sized bordered area for the 12x8 portrait
            // Add 2 to width and height for borders
            let portrait_bordered_width = 14; // 12 + 2 for borders
            let portrait_bordered_height = 10; // 8 + 2 for borders

            let portrait_area = Rect {
                x: player_area.x + (player_area.width.saturating_sub(portrait_bordered_width)) / 2,
                y: player_area.y
                    + (player_area.height.saturating_sub(portrait_bordered_height)) / 2,
                width: portrait_bordered_width.min(player_area.width),
                height: portrait_bordered_height.min(player_area.height),
            };

            let player_paragraph = Paragraph::new(player_port)
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Yellow))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Gray)),
                );
            f.render_widget(player_paragraph, portrait_area);
        }

        // Dialogue text area (center)
        let text_area = Rect {
            x: content_area.x + left_width,
            y: content_area.y,
            width: text_width,
            height: content_area.height,
        };

        record_choices(app, &dialogue_lines, text_area);
        let dialogue_paragraph = Paragraph::new(dialogue_lines)
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });
        f.render_widget(dialogue_paragraph, text_area);
    }

    // Controls at bottom
    let controls = Paragraph::new("Up/Down: Select Choice | Enter: Choose | Esc: End Conversation")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));

    if inner.height > 1 {
        let controls_area = Rect {
            x: inner.x,
            y: inner.y + inner.height - 1,
            width: inner.width,
            height: 1,
        };
        f.render_widget(controls, controls_area);
    }
}

/// Record each numbered choice line ("1.  ..." or "> 2.  ...") of the wrapped dialogue text
/// as a click target, as tall as it wraps to.
fn record_choices(app: &App, lines: &[ratatui::text::Line], area: Rect) {
    let mut y = area.y;
    for line in lines {
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        // Lines wrap one at a time, so each line's height is its own wrapped height
        let height = Paragraph::new(line.clone())
            .wrap(Wrap { trim: true })
            .line_count(area.width) as u16;
        let numbered = text.trim_start().trim_start_matches("> ");
        let number = numbered
            .split_once('.')
            .and_then(|(n, _)| n.trim().parse::<usize>().ok());
        if let Some(n) = number.filter(|&n| n > 0) {
            let rect = Rect::new(area.x, y, area.width, height).intersection(area);
            if rect.height > 0 {
                app.ui.click_targets.push(
                    rect,
                    crate::ui::click_targets::ClickTarget::DialogueChoice(n - 1),
                );
            }
        }
        y = y.saturating_add(height);
    }
}

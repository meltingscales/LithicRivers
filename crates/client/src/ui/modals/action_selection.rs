//! Multi-action and action selection modals.

use crate::app_state::InteractionType;
use crate::App;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Render the multi-action selection modal
pub(crate) fn render_multi_action_selection_modal(f: &mut Frame, app: &mut App) {
    use crate::app_state::MultiActionSelectState;

    match &app.panels.multi_action_select {
        MultiActionSelectState::SelectingAction {
            available_actions,
            selected_action,
            ..
        } => {
            let actions = available_actions.clone();
            let selected = *selected_action;
            render_action_selection_modal(f, app, &actions, selected);
        }
        MultiActionSelectState::None => {
            // No modal to render
        }
    }
}

/// Width of the 3x3 grid: three 5-column cells and four borders.
const GRID_WIDTH: u16 = 19;
/// Height of the grid: three 2-row cells and four borders.
const GRID_HEIGHT: u16 = 10;

/// The glyph and color the grid shows for an interaction's thing: its entity's 1x sprite,
/// or the tile's for doors.
fn glyph(app: &mut App, action: &InteractionType) -> (String, Color) {
    use crate::sprite_loader::{sprite_block_for_spriteref, sprite_block_for_tile, Scale};
    let sprite = action.entity().and_then(|e| {
        app.core
            .game
            .world
            .get::<&lithicrivers_core::components::SpriteRef>(e)
            .ok()
            .map(|s| (*s).clone())
    });
    let block = match sprite {
        Some(sprite) => Some(sprite_block_for_spriteref(
            &mut app.core.sprite_loader,
            &sprite,
            Scale::Small,
        )),
        None => {
            let p = action.position();
            let tile = app.core.game.res.world_state.world.get_tile(p.x, p.y, p.z);
            sprite_block_for_tile(&mut app.core.sprite_loader, tile, Scale::Small)
        }
    };
    block
        .and_then(|(text, color)| text.chars().next().map(|c| (c.to_string(), color)))
        .unwrap_or_else(|| (action.icon().to_string(), Color::White))
}

/// Render the action selection modal: the 3x3 grid of the tiles around the player, laid out
/// like the movement keys, and the list of choices beside it, each with its key.
fn render_action_selection_modal(
    f: &mut Frame,
    app: &mut App,
    available_actions: &[InteractionType],
    selected_action: usize,
) {
    use crate::app::input::multi_action::{keybind_for, player_position, GRID};
    use crate::ui::click_targets::ClickTarget;
    use ratatui::{
        style::Modifier,
        text::Span,
        widgets::{Clear, List, ListItem},
    };

    let screen = f.area();
    let width = (screen.width * 6 / 10).max(50).min(screen.width);
    let height = (GRID_HEIGHT + 4).min(screen.height);
    let area = Rect {
        x: screen.x + (screen.width - width) / 2,
        y: screen.y + (screen.height - height) / 2,
        width,
        height,
    };
    f.render_widget(Clear, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Line::from(" Choose Interaction "))
        .title_alignment(Alignment::Center)
        .style(Style::default());
    let inner = block.inner(area);
    f.render_widget(block, area);
    app.ui.click_targets.push(area, ClickTarget::ModalArea);

    let player = player_position(app);
    let key = |app: &App, (category, action): (&str, &str)| {
        app.core
            .config_manager
            .get_printable_key_for_keybind(category, action)
    };
    let offsets: Vec<(i64, i64)> = available_actions
        .iter()
        .map(|a| player.map_or((0, 0), |p| a.offset_from(p)))
        .collect();
    let selected_offset = offsets.get(selected_action).copied();

    // The grid, unless the window is too small for it (then just the list)
    let show_grid = inner.width >= GRID_WIDTH + 20 && inner.height >= GRID_HEIGHT + 2;
    let list_x = if show_grid {
        inner.x + GRID_WIDTH + 3
    } else {
        inner.x
    };
    if show_grid {
        let player_glyph = app.core.game.get_player_entity().and_then(|e| {
            app.core
                .game
                .world
                .get::<&lithicrivers_core::components::SpriteRef>(e)
                .ok()
                .map(|s| (*s).clone())
        });
        let player_glyph = player_glyph
            .map(|s| {
                crate::sprite_loader::sprite_block_for_spriteref(
                    &mut app.core.sprite_loader,
                    &s,
                    crate::sprite_loader::Scale::Small,
                )
            })
            .unwrap_or_else(|| ("@".to_string(), Color::White));
        let mut cells = Vec::new();
        for &(offset, category, action) in &GRID {
            let first = offsets.iter().position(|&o| o == offset);
            let (glyph, color) = match first {
                _ if offset == (0, 0) => player_glyph.clone(),
                Some(i) => glyph(app, &available_actions[i]),
                None => (" ".to_string(), Color::DarkGray),
            };
            let style = if selected_offset == Some(offset) {
                Style::default().fg(Color::Black).bg(Color::Yellow)
            } else if first.is_some() {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let glyph_style = if selected_offset == Some(offset) {
                style
            } else {
                Style::default().fg(color)
            };
            let label: String = key(app, (category, action)).chars().take(5).collect();
            cells.push((
                Span::styled(format!("{label:^5}"), style),
                Span::styled(format!("{glyph:^5}"), glyph_style),
            ));
        }
        let border = Style::default().fg(Color::Gray);
        let mut lines = Vec::new();
        for row in 0..3 {
            let edge = if row == 0 {
                "┌─────┬─────┬─────┐"
            } else {
                "├─────┼─────┼─────┤"
            };
            lines.push(Line::from(Span::styled(edge, border)));
            for part in 0..2 {
                let mut spans = vec![Span::styled("│", border)];
                for cell in &cells[row * 3..row * 3 + 3] {
                    spans.push(if part == 0 {
                        cell.0.clone()
                    } else {
                        cell.1.clone()
                    });
                    spans.push(Span::styled("│", border));
                }
                lines.push(Line::from(spans));
            }
        }
        lines.push(Line::from(Span::styled("└─────┴─────┴─────┘", border)));
        let grid_area = Rect {
            x: inner.x + 1,
            y: inner.y,
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
        };
        f.render_widget(Paragraph::new(lines), grid_area);
        for (i, &(offset, ..)) in GRID.iter().enumerate() {
            let (col, row) = (i as u16 % 3, i as u16 / 3);
            app.ui.click_targets.push(
                Rect::new(grid_area.x + 1 + col * 6, grid_area.y + 1 + row * 3, 5, 2),
                ClickTarget::InteractionCell(offset),
            );
        }
    }

    // The list: every choice with its key; the selected choice's tile in yellow
    let items: Vec<ListItem> = available_actions
        .iter()
        .zip(&offsets)
        .enumerate()
        .map(|(i, (action, &offset))| {
            let style = if i == selected_action {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else if Some(offset) == selected_offset {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };
            let text = format!(
                "[{}] {}",
                key(app, keybind_for(offset)),
                action.label(&app.core.game.world)
            );
            ListItem::new(text).style(style)
        })
        .collect();
    let list_area = Rect {
        x: list_x,
        y: inner.y + 1,
        width: inner.right().saturating_sub(list_x),
        height: inner.height.saturating_sub(4),
    };
    f.render_widget(List::new(items), list_area);
    app.ui.click_targets.rows(
        list_area,
        0,
        available_actions.len(),
        ClickTarget::InteractionChoice,
    );

    // Controls at the bottom
    let hints = [
        "Direction keys: pick a tile".to_string(),
        format!(
            "{}: choose | {}/{}: list | {}: cancel",
            key(app, ("ui", "MENU_ACTIVATE")),
            key(app, ("ui", "MENU_UP")),
            key(app, ("ui", "MENU_DOWN")),
            key(app, ("ui", "CLOSE_HELP_MENU")),
        ),
    ];
    for (i, hint) in hints.into_iter().enumerate() {
        let from_bottom = 2 - i as u16;
        if inner.height > from_bottom {
            f.render_widget(
                Paragraph::new(hint)
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(Color::Gray)),
                Rect::new(inner.x, inner.bottom() - from_bottom, inner.width, 1),
            );
        }
    }
}

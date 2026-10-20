//! Hotbar assignment and block picker modals.

use crate::ui::centered_rect;
use crate::App;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Render hotbar assignment modal when active
pub(crate) fn render_hotbar_assignment_modal(f: &mut Frame, app: &mut App) {
    use crate::app_state::HotbarAssignmentState;

    // Extract the values we need to avoid borrow conflicts
    match &app.panels.hotbar_assignment {
        HotbarAssignmentState::ChoosingBlock {
            hotbar_slot,
            available_blocks,
            selected_block,
        } => {
            let slot = *hotbar_slot;
            let blocks = available_blocks.clone();
            let selected = *selected_block;
            render_block_picker_modal(f, app, slot, &blocks, selected);
        }
        HotbarAssignmentState::None => {
            // No modal to render
        }
    }
}

/// Render the block picker modal for hotbar assignment
fn render_block_picker_modal(
    f: &mut Frame,
    app: &mut App,
    hotbar_slot: usize,
    available_blocks: &[lithicrivers_core::components::ItemKind],
    selected_block: usize,
) {
    use lithicrivers_core::components::itemkind_name;
    use ratatui::{
        style::Modifier,
        text::{Line, Span},
        widgets::{Clear, List, ListItem},
    };

    // Create modal area (centered, 50% width, 40% height)
    let area = centered_rect(50, 40, f.size());

    // Clear the background
    f.render_widget(Clear, area);

    // Main modal block
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Line::from(format!(
            " Choose Block for F{} ",
            hotbar_slot + 1
        )))
        .title_alignment(Alignment::Center)
        .style(Style::default());

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Create list of available blocks with "Clear slot" as first option
    let mut block_items: Vec<ListItem> = Vec::new();

    // Add "Clear slot" as the first option (index 0)
    let clear_slot_selected = selected_block == 0;
    let clear_slot_style = if clear_slot_selected {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let clear_slot_text = "✗ Clear slot";
    block_items.push(ListItem::new(Line::from(Span::styled(
        clear_slot_text,
        clear_slot_style,
    ))));

    // Add actual blocks (indices 1+)
    for (i, &block_kind) in available_blocks.iter().enumerate() {
        let list_index = i + 1; // Offset by 1 because index 0 is "Clear slot"
        let is_selected = selected_block == list_index;
        let style = if is_selected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        // Show block icon and name using actual sprite
        let sprite_name = lithicrivers_core::components::itemkind_sprite_name(block_kind);
        let icon = if let Some(slash_pos) = sprite_name.find('/') {
            let (category, name) = sprite_name.split_at(slash_pos);
            let name = &name[1..]; // Remove the '/'
            let sprite_data = app.core.sprite_loader.load_sprite(name, category);
            // Get the first character from the 1x1 sprite (sprites[0])
            sprite_data
                .sprites
                .first()
                .and_then(|s| s.chars().next())
                .unwrap_or('?')
        } else {
            '?'
        };
        let text = format!("{} {}", icon, itemkind_name(block_kind));

        block_items.push(ListItem::new(Line::from(Span::styled(text, style))));
    }

    let list =
        List::new(block_items).highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_widget(list, inner);

    // Controls at bottom
    let controls = Paragraph::new("↑↓: Select | Enter: Assign | Esc: Cancel")
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

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
    let area = centered_rect(50, 40, f.area());

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
    app.ui
        .click_targets
        .push(area, crate::ui::click_targets::ClickTarget::ModalArea);

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

        let icon = crate::ui::panels::hotbar::item_glyph(app, block_kind);
        let text = format!("{} {}", icon, itemkind_name(block_kind));

        block_items.push(ListItem::new(Line::from(Span::styled(text, style))));
    }

    let list =
        List::new(block_items).highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_widget(list, inner);
    app.ui.click_targets.rows(
        inner,
        0,
        available_blocks.len() + 1,
        crate::ui::click_targets::ClickTarget::BlockPickerRow,
    );

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

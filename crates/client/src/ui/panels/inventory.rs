use crate::rendering_helpers::parse_hex_color;
use lithicrivers_core::components::{
    itemkind_name, itemkind_sprite_name, Inventory as InvComp, ItemKind,
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::collections::HashMap;

pub fn render_inventory_list_only(f: &mut Frame, app: &mut crate::App, area: Rect) {
    // Build list with selection highlight (same as right side of render_inventory_panel)
    let mut list_lines: Vec<Line<'static>> = Vec::new();
    if let Some(e) = app.core.game.res.player_entity {
        if let Ok(inv) = app.core.game.world.get::<&InvComp>(e) {
            if inv.slots.is_empty() {
                list_lines.push(Line::from(Span::raw("(Empty)")));
                app.panels.inventory.selected = 0;
            } else {
                if app.panels.inventory.selected >= inv.slots.len() {
                    app.panels.inventory.selected = inv.slots.len() - 1;
                }
                for (i, s) in inv.slots.iter().enumerate() {
                    let label = format!("{} x{}", itemkind_name(s.kind), s.qty);
                    if i == app.panels.inventory.selected {
                        list_lines.push(Line::from(Span::styled(
                            label,
                            Style::default().fg(Color::Yellow),
                        )));
                    } else {
                        list_lines.push(Line::from(Span::raw(label)));
                    }
                }
            }
        } else {
            list_lines.push(Line::from(Span::raw("(No Inventory component)")));
        }
    } else {
        list_lines.push(Line::from(Span::raw("(No player)")));
    }

    let list_para = Paragraph::new(list_lines)
        .alignment(Alignment::Left)
        .block(Block::default().borders(Borders::ALL).title("Inventory"));
    f.render_widget(list_para, area);
}

pub fn render_inventory_panel(f: &mut Frame, app: &mut crate::App, area: Rect) {
    // Split: left art/details, right list
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(28), Constraint::Min(20)])
        .split(area);

    // Build list with selection highlight
    let mut list_lines: Vec<Line<'static>> = Vec::new();
    let mut selected_kind: Option<ItemKind> = None;
    let mut selected_qty: u32 = 0;
    if let Some(e) = app.core.game.res.player_entity {
        if let Ok(inv) = app.core.game.world.get::<&InvComp>(e) {
            if inv.slots.is_empty() {
                list_lines.push(Line::from(Span::raw("(Empty)")));
                app.panels.inventory.selected = 0;
            } else {
                if app.panels.inventory.selected >= inv.slots.len() {
                    app.panels.inventory.selected = inv.slots.len() - 1;
                }
                for (i, s) in inv.slots.iter().enumerate() {
                    let label = format!("{} x{}", itemkind_name(s.kind), s.qty);
                    if i == app.panels.inventory.selected {
                        selected_kind = Some(s.kind);
                        selected_qty = s.qty;
                        list_lines.push(Line::from(Span::styled(
                            label,
                            Style::default().fg(Color::Yellow),
                        )));
                    } else {
                        list_lines.push(Line::from(Span::raw(label)));
                    }
                }
            }
        } else {
            list_lines.push(Line::from(Span::raw("(No Inventory component)")));
        }
    } else {
        list_lines.push(Line::from(Span::raw("(No player)")));
    }

    // Left: art + description if any selection
    let left_block = Block::default().borders(Borders::ALL).title("Item");
    let left_inner = left_block.inner(chunks[0]);
    let mut left_lines: Vec<Line<'static>> = Vec::new();
    if let Some(kind) = selected_kind {
        let (category, sprite_name) = ("items", itemkind_sprite_name(kind));
        let sd = app.core.sprite_loader.load_sprite(sprite_name, category);
        // Art
        if let Some(block) = sd.art12x8_sprites.first() {
            let color = parse_hex_color(&sd.color);
            for row in block.split('\n') {
                let sp = match color {
                    Some(c) => Span::styled(row.to_string(), Style::default().fg(c)),
                    None => Span::raw(row.to_string()),
                };
                left_lines.push(Line::from(sp));
            }
        }
        // Spacer and description
        left_lines.push(Line::from(""));
        left_lines.push(Line::from(Span::styled(
            format!("{} (x{})", itemkind_name(kind), selected_qty),
            Style::default().fg(Color::Cyan),
        )));
        left_lines.push(Line::from(""));
        left_lines.push(Line::from(Span::raw(sd.description.clone())));
        left_lines.push(Line::from(""));
        left_lines.push(Line::from(Span::raw("Keys:")));
        left_lines.push(Line::from(Span::raw("  d=drop")));
        left_lines.push(Line::from(Span::raw("  .=duplicate")));
        left_lines.push(Line::from(Span::raw("  x=destroy")));
    } else {
        left_lines.push(Line::from(Span::raw("Select an item")));
    }
    let left_para = Paragraph::new(left_lines).alignment(Alignment::Left);
    f.render_widget(left_para, left_inner);
    f.render_widget(left_block, chunks[0]);

    // Right: list panel
    let list_para = Paragraph::new(list_lines)
        .alignment(Alignment::Left)
        .block(Block::default().borders(Borders::ALL).title("Inventory"));
    f.render_widget(list_para, chunks[1]);
}

pub fn get_player_inventory(app: &crate::App) -> HashMap<ItemKind, u32> {
    let mut inventory = HashMap::new();

    if let Some(player_e) = app.core.game.res.player_entity {
        if let Ok(inv) = app
            .core
            .game
            .world
            .get::<&lithicrivers_core::components::Inventory>(player_e)
        {
            for item in &inv.slots {
                *inventory.entry(item.kind).or_insert(0) += item.qty;
            }
        }
    }

    inventory
}

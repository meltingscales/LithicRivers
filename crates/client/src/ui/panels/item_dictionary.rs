//! The Item Dictionary tab: every item with its sprite, name and description, a search over
//! names and descriptions, and the selected item's art, recipes, uses and raw sources.

use crate::rendering_helpers::parse_hex_color;
use crate::sprite_loader::SpriteLoader;
use lithicrivers_core::components::{itemkind_name, itemkind_sprite_name, ItemKind};
use lithicrivers_core::crafting;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};
use strum::IntoEnumIterator;

/// An item's description, from its sprite.
pub fn description(loader: &mut SpriteLoader, item: ItemKind) -> String {
    loader
        .load_sprite(itemkind_sprite_name(item), "items")
        .description
        .clone()
}

/// Items whose name or description contains `search` (ignoring case), in item order.
pub fn matching_items(loader: &mut SpriteLoader, search: &str) -> Vec<ItemKind> {
    let search = search.to_lowercase();
    ItemKind::iter()
        .filter(|&item| {
            search.is_empty()
                || itemkind_name(item).to_lowercase().contains(&search)
                || description(loader, item).to_lowercase().contains(&search)
        })
        .collect()
}

/// The items the tab lists right now.
pub fn filtered_items(app: &mut crate::App) -> Vec<ItemKind> {
    let search = app.panels.item_dictionary.search.clone();
    matching_items(&mut app.core.sprite_loader, &search)
}

fn colored(loader: &mut SpriteLoader, item: ItemKind, text: String) -> Span<'static> {
    let color = parse_hex_color(
        &loader
            .load_sprite(itemkind_sprite_name(item), "items")
            .color,
    );
    Span::styled(text, Style::default().fg(color.unwrap_or(Color::White)))
}

fn recipe_line(recipe: &lithicrivers_core::recipe_handler::Recipe) -> String {
    let ingredients: Vec<String> = recipe
        .ingredients
        .iter()
        .map(|&(i, n)| format!("{n} {}", itemkind_name(i)))
        .collect();
    format!(
        "  {} x{} <- {}",
        recipe.name,
        recipe.quantity,
        ingredients.join(", ")
    )
}

pub fn render_item_dictionary_panel(f: &mut Frame, app: &mut crate::App, area: Rect) {
    let items = filtered_items(app);
    let state = &mut app.panels.item_dictionary;
    state.selected = state.selected.min(items.len().saturating_sub(1));
    let (selected, searching, search) = (state.selected, state.searching, state.search.clone());

    let block = Block::default()
        .borders(Borders::ALL)
        .title(Line::from(" Item Dictionary "));
    let inner = block.inner(area);
    f.render_widget(block, area);
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(inner);
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(rows[0]);

    // The list: 1x sprite and name
    let loader = &mut app.core.sprite_loader;
    let list_items: Vec<ListItem> = items
        .iter()
        .map(|&item| {
            let glyph = loader
                .load_sprite(itemkind_sprite_name(item), "items")
                .sprites
                .first()
                .cloned()
                .unwrap_or_default();
            ListItem::new(Line::from(vec![
                colored(loader, item, format!("{glyph} ")),
                Span::raw(itemkind_name(item)),
            ]))
        })
        .collect();
    let mut list_state = ListState::default();
    list_state.select((!items.is_empty()).then_some(selected));
    let list = List::new(list_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Items ({}) ", items.len())),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    f.render_stateful_widget(list, columns[0], &mut list_state);

    // Details of the selected item: text on the left, art on the right
    let mut lines: Vec<Line> = Vec::new();
    let mut art_lines: Vec<String> = Vec::new();
    let mut art_color = Color::White;
    if let Some(&item) = items.get(selected) {
        let sd = loader
            .load_sprite(itemkind_sprite_name(item), "items")
            .clone();
        let color = parse_hex_color(&sd.color).unwrap_or(Color::White);
        lines.push(Line::from(Span::styled(
            itemkind_name(item),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(sd.description.clone()));
        lines.push(Line::from(""));
        // 12x8 art above the 3x and 2x sprites, in its own column
        art_lines.extend(
            sd.art12x8_sprites
                .first()
                .map_or(vec![], |a| a.lines().map(String::from).collect()),
        );
        for sprite in sd.sprites.iter().skip(1).rev() {
            art_lines.push(String::new());
            art_lines.extend(sprite.lines().map(|l| format!("  {l}")));
        }
        art_color = color;
        let recipes = app.panels.crafting.recipe_handler.get_recipes();
        let made = crafting::made_by(recipes, item);
        let used = crafting::used_in(recipes, item);
        let sources = crafting::raw_sources_of(item);
        if !sources.is_empty() {
            lines.push(Line::from("Found:"));
            lines.extend(
                sources
                    .iter()
                    .map(|s| Line::from(format!("  {}", s.describe()))),
            );
        }
        if !made.is_empty() {
            lines.push(Line::from("Made by:"));
            lines.extend(made.iter().map(|r| Line::from(recipe_line(r))));
        }
        if !used.is_empty() {
            lines.push(Line::from("Used in:"));
            lines.extend(used.iter().map(|r| Line::from(recipe_line(r))));
        }
        if sources.is_empty() && made.is_empty() && used.is_empty() {
            lines.push(Line::from(Span::styled(
                "No known source or use yet.",
                Style::default().fg(Color::DarkGray),
            )));
        }
    } else {
        lines.push(Line::from("No items match the search."));
    }
    let details = Block::default().borders(Borders::ALL);
    let details_inner = details.inner(columns[1]);
    f.render_widget(details, columns[1]);
    let parts = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(10), Constraint::Length(14)])
        .split(details_inner);
    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), parts[0]);
    let art: Vec<Line> = art_lines
        .into_iter()
        .map(|l| Line::from(Span::styled(l, Style::default().fg(art_color))))
        .collect();
    f.render_widget(Paragraph::new(art), parts[1]);

    // The search line
    let key = |c, a| app.core.config_manager.get_printable_key_for_keybind(c, a);
    let search_line = if searching {
        format!(
            "Search: {search}_   ({} done, {} clear)",
            key("dictionary", "END_SEARCH"),
            key("ui", "CLOSE_HELP_MENU")
        )
    } else if search.is_empty() {
        format!("{}: search", key("dictionary", "SEARCH"))
    } else {
        format!(
            "Search: {search}   ({} edit, {} clear)",
            key("dictionary", "SEARCH"),
            key("ui", "CLOSE_HELP_MENU")
        )
    };
    f.render_widget(
        Paragraph::new(Span::styled(search_line, Style::default().fg(Color::Gray))),
        rows[1],
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_item_has_a_sprite_and_description() {
        let mut loader = SpriteLoader::new(None);
        let missing: Vec<&str> = ItemKind::iter()
            .filter(|&item| description(&mut loader, item).trim().is_empty())
            .map(itemkind_name)
            .collect();
        assert!(
            missing.is_empty(),
            "items without a description: {missing:?}"
        );
    }

    #[test]
    fn search_matches_names_and_descriptions() {
        let mut loader = SpriteLoader::new(None);
        let burn = matching_items(&mut loader, "BURN");
        assert!(burn.contains(&ItemKind::BurnerGenerator), "by name");
        for item in &burn {
            let text = format!(
                "{} {}",
                itemkind_name(*item),
                description(&mut loader, *item)
            );
            assert!(text.to_lowercase().contains("burn"), "{text}");
        }
        assert_eq!(
            matching_items(&mut loader, "").len(),
            ItemKind::iter().count()
        );
    }
}

#[cfg(test)]
mod headless_tests {
    use crate::headless::chain_commands;

    #[test]
    fn searching_burn_filters_the_list() {
        let mut runner = chain_commands(&[
            "skip-splash",
            "key ui.MENU_NEXT 6",
            "key dictionary.SEARCH",
            "type \"burn\"",
        ]);
        let screen = runner.screen_text();
        assert!(screen.contains("Burner Generator"), "{screen}");
        assert!(screen.contains("Search: burn_"));
        assert!(runner.app.panels.item_dictionary.searching);
        let listed = super::filtered_items(&mut runner.app);
        assert!(listed.contains(&lithicrivers_core::components::ItemKind::BurnerGenerator));
        assert!(
            !listed.contains(&lithicrivers_core::components::ItemKind::Nail),
            "Nail doesn't mention burning"
        );
    }
}

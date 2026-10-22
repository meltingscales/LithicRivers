//! The Blueprints window: every saved blueprint with its size and cost, the selected one
//! highlighted. Renaming shows the name being typed in place.

use crate::ui::centered_rect;
use crate::App;
use lithicrivers_core::components::itemkind_name;
use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
    Frame,
};

pub(crate) fn render_blueprint_window(f: &mut Frame, app: &App) {
    let Some(window) = &app.panels.blueprint_window else {
        return;
    };
    let library = &app.core.game.res.blueprints;
    let items: Vec<ListItem> = if library.blueprints.is_empty() {
        vec![ListItem::new("  (no blueprints yet: copy an area first)")]
    } else {
        library
            .blueprints
            .iter()
            .enumerate()
            .map(|(i, b)| {
                let name = match &window.renaming {
                    Some(typed) if i == library.selected => format!("{typed}_"),
                    _ => b.name.clone(),
                };
                let cost: Vec<String> = b
                    .cost()
                    .into_iter()
                    .map(|(item, n)| format!("{n} {}", itemkind_name(item)))
                    .collect();
                ListItem::new(format!(
                    "{name}  {}x{}: {}",
                    b.width,
                    b.height,
                    cost.join(", ")
                ))
            })
            .collect()
    };
    let key = |c, a| app.core.config_manager.get_printable_key_for_keybind(c, a);
    let hints = if window.renaming.is_some() {
        format!(
            " {}: keep name  {}: cancel ",
            key("blueprints", "END_RENAME"),
            key("ui", "CLOSE_HELP_MENU")
        )
    } else {
        format!(
            " {}: use  {}: rename  {}: delete  {}: close ",
            key("ui", "MENU_ACTIVATE"),
            key("blueprints", "RENAME"),
            key("blueprints", "DELETE"),
            key("ui", "CLOSE_HELP_MENU")
        )
    };
    let area = centered_rect(70, 60, f.size());
    f.render_widget(Clear, area);
    app.ui
        .click_targets
        .push(area, crate::ui::click_targets::ClickTarget::ModalArea);
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Blueprints ")
                .title_alignment(Alignment::Center)
                .title_bottom(hints),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    let mut state = ListState::default();
    if !library.blueprints.is_empty() {
        state.select(Some(library.selected));
    }
    f.render_stateful_widget(list, area, &mut state);
    let first = state.offset();
    app.ui.click_targets.rows(
        crate::ui::click_targets::inset(area),
        first,
        library.blueprints.len().saturating_sub(first),
        crate::ui::click_targets::ClickTarget::BlueprintRow,
    );
}

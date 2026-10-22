//! The filter picker: a list of "no filter" and every item, with the current filter marked.

use crate::ui::centered_rect;
use crate::App;
use lithicrivers_core::components::itemkind_name;
use lithicrivers_core::factory::FactoryBlock;
use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
    Frame,
};

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    chars
        .next()
        .map_or(String::new(), |c| c.to_uppercase().chain(chars).collect())
}

pub(crate) fn render_filter_picker(f: &mut Frame, app: &App) {
    let Some(picker) = app.panels.filter_picker else {
        return;
    };
    let world = &app.core.game.world;
    let current = lithicrivers_core::logistics::filter(world, picker.entity);
    let name = world
        .get::<&FactoryBlock>(picker.entity)
        .map_or("block", |b| b.kind.name());
    let items: Vec<ListItem> = crate::app::input::filter_picker::rows()
        .into_iter()
        .map(|row| {
            let label = row.map_or("(no filter)", itemkind_name);
            let mark = if row == current { "* " } else { "  " };
            ListItem::new(format!("{mark}{label}"))
        })
        .collect();
    let key = |c, a| app.core.config_manager.get_printable_key_for_keybind(c, a);
    let area = centered_rect(40, 70, f.size());
    f.render_widget(Clear, area);
    app.ui
        .click_targets
        .push(area, crate::ui::click_targets::ClickTarget::ModalArea);
    let row_count = crate::app::input::filter_picker::rows().len();
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {}: pick a filter ", capitalize(name)))
                .title_alignment(Alignment::Center)
                .title_bottom(format!(
                    " {}: set  {}: close ",
                    key("ui", "MENU_ACTIVATE"),
                    key("ui", "CLOSE_HELP_MENU")
                )),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    let mut state = ListState::default();
    state.select(Some(picker.selected));
    f.render_stateful_widget(list, area, &mut state);
    let first = state.offset();
    app.ui.click_targets.rows(
        crate::ui::click_targets::inset(area),
        first,
        row_count.saturating_sub(first),
        crate::ui::click_targets::ClickTarget::FilterRow,
    );
}

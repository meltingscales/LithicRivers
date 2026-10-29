//! Title menu screens: title, world list, world actions, delete confirmation, create world,
//! options.

use crate::app::title_menu::{
    CreateField, DELETE_CHOICES, OPTIONS_ROWS, TITLE_CHOICES, WORLD_ACTIONS,
};
use crate::ui::centered_rect;
use crate::ui::click_targets::ClickTarget;
use crate::{App, SplashState};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

/// Menu rows with the selected one marked and highlighted.
fn menu_lines(items: &[String], selected: usize) -> Vec<Line<'static>> {
    items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            if i == selected {
                Line::from(Span::styled(
                    format!("> {item} <"),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ))
            } else {
                Line::from(format!("  {item}  "))
            }
        })
        .collect()
}

/// Record `count` menu rows drawn inside the bordered `rect`, starting `first_line` lines
/// below the top of its inside.
fn record_rows(
    app: &App,
    rect: Rect,
    first_line: u16,
    count: usize,
    target: impl Fn(usize) -> ClickTarget,
) {
    let inner = Rect::new(
        rect.x + 1,
        rect.y + 1 + first_line,
        rect.width.saturating_sub(2),
        rect.height.saturating_sub(2 + first_line),
    );
    app.ui.click_targets.rows(inner, 0, count, target);
}

fn key(app: &App, action: &str) -> &'static str {
    app.core
        .config_manager
        .get_printable_key_for_keybind("title", action)
}

fn hint(app: &App, extra: &str) -> Line<'static> {
    Line::from(Span::styled(
        format!(
            "{}/{} move   {} select   {} back{extra}",
            key(app, "UP"),
            key(app, "DOWN"),
            key(app, "SELECT"),
            key(app, "BACK")
        ),
        Style::default().fg(Color::DarkGray),
    ))
}

fn date(unix: u64) -> String {
    chrono::DateTime::from_timestamp(unix as i64, 0)
        .map(|d| d.format("%Y-%m-%d %H:%M UTC").to_string())
        .unwrap_or_else(|| "?".to_string())
}

fn on_off(on: bool) -> &'static str {
    if on {
        "ON"
    } else {
        "OFF"
    }
}

/// A bordered box centered on the screen with the given lines. Returns where it was drawn.
fn boxed(f: &mut Frame, area: Rect, title: &str, lines: Vec<Line<'static>>) -> Rect {
    let height = (lines.len() as u16 + 4).min(area.height);
    let width = lines
        .iter()
        .map(|l| l.width() as u16 + 6)
        .max()
        .unwrap_or(20)
        .max(title.len() as u16 + 6)
        .min(area.width);
    let rect = Rect {
        x: area.x + (area.width - width) / 2,
        y: area.y + (area.height - height) / 2,
        width,
        height,
    };
    f.render_widget(Clear, rect);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {title} "))
        .title_alignment(Alignment::Center);
    let paragraph = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false })
        .block(block);
    f.render_widget(paragraph, rect);
    rect
}

fn world_rows(app: &App) -> Vec<String> {
    let mut rows: Vec<String> = app
        .title
        .worlds
        .iter()
        .map(|w| match &w.meta {
            Some(meta) => format!(
                "{}{}  (seed {}, last played {})",
                meta.name,
                world_type_label(meta.settings.world_type),
                meta.seed,
                date(meta.last_played_unix)
            ),
            None => format!(
                "{}  (unreadable)",
                w.folder.file_name().unwrap_or_default().to_string_lossy()
            ),
        })
        .collect();
    rows.push("Create New World".to_string());
    rows.push("Back".to_string());
    rows
}

/// " (Void)" after a debug world's name; nothing for Normal worlds.
fn world_type_label(world_type: lithicrivers_core::resources::WorldType) -> String {
    match world_type {
        lithicrivers_core::resources::WorldType::Normal => String::new(),
        t => format!(" ({})", t.name()),
    }
}

fn render_world_list(f: &mut Frame, app: &App, area: Rect) {
    let mut lines = vec![Line::from("")];
    lines.extend(menu_lines(&world_rows(app), app.title.world_selected));
    lines.push(Line::from(""));
    if let Some(message) = &app.title.message {
        lines.push(Line::from(Span::styled(
            message.clone(),
            Style::default().fg(Color::Red),
        )));
    }
    lines.push(hint(app, ""));
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Select World ")
        .title_alignment(Alignment::Center);
    let paragraph = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false })
        .block(block);
    let rect = centered_rect(80, 80, area);
    f.render_widget(paragraph, rect);
    record_rows(
        app,
        rect,
        1,
        app.title.worlds.len() + 2,
        ClickTarget::WorldRow,
    );
}

fn selected_world_name(app: &App) -> String {
    app.title
        .selected_world()
        .map(|w| match &w.meta {
            Some(meta) => meta.name.clone(),
            None => w
                .folder
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
        })
        .unwrap_or_default()
}

fn render_create(f: &mut Frame, app: &App, area: Rect) {
    let form = &app.title.create;
    let cursor = |i: usize| if form.field == i { "_" } else { "" };
    let rows: Vec<String> = form
        .fields()
        .into_iter()
        .enumerate()
        .map(|(i, field)| {
            let label = field.label();
            match field {
                CreateField::Name => format!("{label}: [{}{}]", form.name, cursor(i)),
                CreateField::Seed => format!("{label}: [{}{}]", form.seed, cursor(i)),
                CreateField::Cheats => format!("{label}: {}", on_off(form.settings.cheats_allowed)),
                CreateField::DeveloperMode => {
                    format!("{label}: {}", on_off(form.settings.developer_mode))
                }
                CreateField::WorldType => {
                    format!("{label}: < {} >", form.settings.world_type.name())
                }
                CreateField::Tutorial => {
                    format!("{label}: {}", on_off(form.settings.show_tutorial))
                }
                CreateField::Create | CreateField::Cancel => label.to_string(),
            }
        })
        .collect();
    let mut lines = vec![Line::from("")];
    lines.extend(menu_lines(&rows, form.field));
    lines.push(Line::from(""));
    // A taken name is allowed, but the new world gets a number so the list tells them apart
    let typed = match form.name.trim() {
        "" => "New World",
        name => name,
    };
    let name = form.world_name(&app.title.worlds);
    if name != typed {
        let shown = typed;
        lines.push(Line::from(Span::styled(
            format!("A world named \"{shown}\" already exists: this one will be \"{name}\"."),
            Style::default().fg(Color::Yellow),
        )));
    }
    lines.push(Line::from(Span::styled(
        "Leave the seed blank for a random world; any text works as a seed.",
        Style::default().fg(Color::DarkGray),
    )));
    lines.push(hint(
        app,
        &format!(
            "   {}/{} toggle   {} erase",
            key(app, "LEFT"),
            key(app, "RIGHT"),
            key(app, "DELETE_CHAR")
        ),
    ));
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Create New World ")
        .title_alignment(Alignment::Center);
    let paragraph = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false })
        .block(block);
    let rect = centered_rect(80, 80, area);
    f.render_widget(paragraph, rect);
    record_rows(app, rect, 1, form.fields().len(), ClickTarget::CreateField);
}

fn render_options(f: &mut Frame, app: &App, area: Rect) {
    let percent = |key: &str| (app.volume_setting(key) * 100.0).round() as u32;
    let window = app.ui.window;
    let on_off = |on: bool, yes: &str, no: &str| if on { yes } else { no }.to_string();
    let window_value = |value: String| {
        if window.active {
            value
        } else {
            "only in the game window".to_string()
        }
    };
    let rows = vec![
        format!("{}: {}%", OPTIONS_ROWS[0], percent("MUSIC_VOLUME")),
        format!("{}: {}%", OPTIONS_ROWS[1], percent("SFX_VOLUME")),
        format!(
            "{}: {}",
            OPTIONS_ROWS[2],
            window_value(format!("{}px", window.font_px))
        ),
        format!(
            "{}: {}",
            OPTIONS_ROWS[3],
            window_value(on_off(window.fullscreen, "on", "off"))
        ),
        format!(
            "{}: {}",
            OPTIONS_ROWS[4],
            window_value(on_off(window.show_display, "shown", "hidden"))
        ),
        OPTIONS_ROWS[5].to_string(),
    ];
    let mut lines = vec![Line::from("")];
    lines.extend(menu_lines(&rows, app.title.options_selected));
    lines.push(Line::from(""));
    lines.push(hint(
        app,
        &format!("   {}/{} change", key(app, "LEFT"), key(app, "RIGHT")),
    ));
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Options ")
        .title_alignment(Alignment::Center);
    let paragraph = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(block);
    let rect = centered_rect(70, 60, area);
    f.render_widget(paragraph, rect);
    record_rows(app, rect, 1, OPTIONS_ROWS.len(), ClickTarget::OptionsRow);
}

fn render_title(f: &mut Frame, app: &App, area: Rect) {
    // Pad the art's lines to one width so centering keeps them aligned
    let art: Vec<&str> = app.splash.game_title_text.lines().collect();
    let width = art.iter().map(|l| l.chars().count()).max().unwrap_or(0);
    let mut lines: Vec<Line> = art
        .iter()
        .map(|l| Line::from(format!("{l:<width$}")))
        .collect();
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        app.title.splash_text().to_string(),
        Style::default().fg(Color::Yellow),
    )));
    lines.push(Line::from(""));
    let choices: Vec<String> = TITLE_CHOICES.iter().map(|c| c.to_string()).collect();
    lines.extend(menu_lines(&choices, app.title.selected));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!(
            "{}/{} move   {} select",
            key(app, "UP"),
            key(app, "DOWN"),
            key(app, "SELECT")
        ),
        Style::default().fg(Color::DarkGray),
    )));
    let height = (lines.len() as u16).min(area.height);
    let rect = Rect {
        y: area.y + (area.height - height) / 2,
        height,
        ..area
    };
    f.render_widget(Paragraph::new(lines).alignment(Alignment::Center), rect);
    // The choices follow the art, a blank line, the splash text and another blank line
    let first = art.len() as u16 + 3;
    let choices_area = Rect {
        y: rect.y + first,
        height: rect.height.saturating_sub(first),
        ..rect
    };
    app.ui.click_targets.rows(
        choices_area,
        0,
        TITLE_CHOICES.len(),
        ClickTarget::TitleChoice,
    );
}

/// Draw the current title-menu screen. Returns false if the state isn't a title screen.
pub(crate) fn render_title_menu(f: &mut Frame, app: &App) -> bool {
    let area = f.area();
    match app.splash.state {
        SplashState::Title => render_title(f, app, area),
        SplashState::WorldList => render_world_list(f, app, area),
        SplashState::WorldActions => {
            render_world_list(f, app, area);
            let actions: Vec<String> = WORLD_ACTIONS.iter().map(|a| a.to_string()).collect();
            let lines = menu_lines(&actions, app.title.world_action);
            let rect = boxed(f, area, &selected_world_name(app), lines);
            app.ui.click_targets.push(rect, ClickTarget::ModalArea);
            record_rows(app, rect, 0, WORLD_ACTIONS.len(), ClickTarget::WorldAction);
        }
        SplashState::RenameWorld => {
            render_world_list(f, app, area);
            let lines = vec![
                Line::from(format!("New name: [{}_]", app.title.rename)),
                Line::from(""),
                Line::from(Span::styled(
                    format!(
                        "{} rename   {} erase   {} cancel",
                        key(app, "SELECT"),
                        key(app, "DELETE_CHAR"),
                        key(app, "BACK")
                    ),
                    Style::default().fg(Color::DarkGray),
                )),
            ];
            let rect = boxed(f, area, "Rename World", lines);
            app.ui.click_targets.push(rect, ClickTarget::ModalArea);
        }
        SplashState::ConfirmDelete => {
            render_world_list(f, app, area);
            let mut lines = vec![
                Line::from(format!("Delete \"{}\"?", selected_world_name(app))),
                Line::from("This can't be undone."),
                Line::from(""),
            ];
            let choices: Vec<String> = DELETE_CHOICES.iter().map(|c| c.to_string()).collect();
            lines.extend(menu_lines(&choices, app.title.delete_choice));
            let rect = boxed(f, area, "Delete World", lines);
            app.ui.click_targets.push(rect, ClickTarget::ModalArea);
            record_rows(
                app,
                rect,
                3,
                DELETE_CHOICES.len(),
                ClickTarget::DeleteChoice,
            );
        }
        SplashState::CreateWorld => render_create(f, app, area),
        SplashState::Options => render_options(f, app, area),
        _ => return false,
    }
    true
}

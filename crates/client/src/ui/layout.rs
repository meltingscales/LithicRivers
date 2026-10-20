//! Frame layout: title, tab content, message log, bottom menu, and modal overlays in draw order.

use crate::ui::centered_rect;
use crate::ui::modals::{
    render_cheat_console_modal, render_corpse_looting_modals, render_hotbar_assignment_modal,
    render_multi_action_selection_modal, render_npc_interaction_modals,
};
use crate::ui::panels::{
    render_body_panel, render_combat_panel, render_crafting_panel, render_credits_panel,
    render_game_view, render_global_map_panel, render_help_panel, render_hotbar_panel,
    render_inventory_list_only, render_inventory_panel, render_look_panel, render_menu_panel,
    render_modes_panel, render_quests_panel, render_quit_panel, render_repair_modal,
    render_tutorial_panel, render_tutorial_selection_modal,
};
use crate::{App, MenuTab, SplashState};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

pub(crate) fn ui(f: &mut Frame, app: &mut App) {
    // Update tutorial highlights (remove expired ones)
    let now = app.clock.now();
    app.ui.tutorial_system.update_highlights(now);

    // Show splash screens if needed
    if crate::ui::splash::render_splash(f, app) {
        return;
    }

    // Only render the main UI if we're past all splash screens
    if app.splash.state != SplashState::MainUI {
        return;
    }

    // Check if we should show hotbar (place mode + world tab)
    let show_hotbar = app.ui.current_tab == MenuTab::World
        && matches!(app.panels.build.mode, crate::app_state::BuildMode::Place);

    let root_chunks = if show_hotbar {
        Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints([
                Constraint::Length(1), // Title line
                Constraint::Min(0),    // Main area (map + inventory)
                Constraint::Length(3), // Hotbar
                Constraint::Length(7), // Message log
                Constraint::Length(3), // Bottom menu bar (needs 3 for borders + content)
            ])
            .split(f.size())
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints([
                Constraint::Length(1), // Title line
                Constraint::Min(0),    // Main area (map + inventory)
                Constraint::Length(7), // Message log
                Constraint::Length(3), // Bottom menu bar (needs 3 for borders + content)
            ])
            .split(f.size())
    };

    // Title
    let title = Paragraph::new("LithicRivers")
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center);
    f.render_widget(title, root_chunks[0]);

    // Main area depends on selected tab
    match app.ui.current_tab {
        MenuTab::World => {
            if app.combat.is_active() {
                // Combat mode: show combat panel on the left (50%), game view on the right (50%)
                let main_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(root_chunks[1]);
                render_combat_panel(f, app, main_chunks[0]);
                render_game_view(f, app, main_chunks[1]);
            } else if app.panels.look.mode {
                // With Look mode: show modes panel and look panel on the right
                let main_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Min(20), Constraint::Length(24)])
                    .split(root_chunks[1]);
                render_game_view(f, app, main_chunks[0]);

                // Split right panel vertically: modes (3 lines) + look panel (rest)
                let right_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(3), Constraint::Min(0)])
                    .split(main_chunks[1]);
                render_modes_panel(f, app, right_chunks[0]);
                render_look_panel(f, app, right_chunks[1]);
            } else {
                // Show modes panel and inventory list as sidebar
                let main_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Min(20), Constraint::Length(24)])
                    .split(root_chunks[1]);
                render_game_view(f, app, main_chunks[0]);

                // Split right panel vertically: modes (3 lines) + inventory (rest)
                let right_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(3), Constraint::Min(0)])
                    .split(main_chunks[1]);
                render_modes_panel(f, app, right_chunks[0]);
                render_inventory_list_only(f, app, right_chunks[1]);
            }
        }
        MenuTab::Tutorial => {
            // Tutorial panel on left side (30%), game view on right (70%)
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
                .split(root_chunks[1]);
            render_tutorial_panel(f, app, main_chunks[0]);
            render_game_view(f, app, main_chunks[1]);
        }
        MenuTab::GlobalMap => {
            // Global Map panel
            render_global_map_panel(f, app, root_chunks[1]);
        }
        MenuTab::Quests => {
            // Quests panel
            render_quests_panel(f, app, root_chunks[1]);
        }
        MenuTab::Crafting => {
            // Crafting panel
            render_crafting_panel(f, app, root_chunks[1]);
        }
        MenuTab::Body => {
            // Body panel
            render_body_panel(f, app, root_chunks[1]);
        }
        MenuTab::Inventory => {
            // Inventory panel
            render_inventory_panel(f, app, root_chunks[1]);
        }
        MenuTab::Menu => {
            // Menu panel
            render_menu_panel(f, app, root_chunks[1]);
        }
        MenuTab::Help => {
            // Help: controls
            render_help_panel(f, app, root_chunks[1]);
        }
        MenuTab::Credits => {
            // Credits: scrollable
            render_credits_panel(f, app, root_chunks[1]);
        }
        MenuTab::Quit => {
            // Quit: show diamond pattern panel
            render_quit_panel(f, app, root_chunks[1]);
        }
    }

    // Render corpse looting modals if active
    render_corpse_looting_modals(f, app);

    // Calculate game viewport area for modal positioning
    let game_viewport = calculate_game_viewport_area(f, app);

    // Render NPC interaction modals if active (constrained to game viewport)
    render_npc_interaction_modals(f, app, game_viewport);

    // Render hotbar assignment modal if active
    render_hotbar_assignment_modal(f, app);

    // Render multi-action selection modal if active
    render_multi_action_selection_modal(f, app);

    // Render repair modal if active
    if matches!(
        app.panels.body_repair,
        crate::app_state::BodyRepairState::SelectingRepairAndPart { .. }
    ) {
        let modal_area = centered_rect(75, 65, f.size());
        render_repair_modal(f, app, modal_area);
    }

    // Render hotbar if in place mode
    if show_hotbar {
        render_hotbar_panel(f, app, root_chunks[2]);
        // Message log
        render_message_log(f, app, root_chunks[3]);
        // Bottom menu bar
        render_bottom_menu(f, app, root_chunks[4]);
    } else {
        // Message log
        render_message_log(f, app, root_chunks[2]);
        // Bottom menu bar
        render_bottom_menu(f, app, root_chunks[3]);
    }

    // Render tutorial selection modal if active
    render_tutorial_selection_modal(f, app);

    // Render cheat console if active (on top of everything else)
    render_cheat_console_modal(f, app);
}

fn render_message_log(f: &mut Frame, app: &mut App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();
    // Account for the border (top+bottom) since Paragraph has a Block
    let visible_rows = area.height.saturating_sub(2) as usize;
    let messages = &app.core.game.res.events.get_message_log().messages;
    let start = if messages.len() > visible_rows {
        messages.len() - visible_rows
    } else {
        0
    };
    for msg in messages.iter().skip(start) {
        let color = match msg.color {
            lithicrivers_core::message_log::MessageColor::Default => Color::White,
            lithicrivers_core::message_log::MessageColor::Red => Color::Red,
            lithicrivers_core::message_log::MessageColor::Yellow => Color::Yellow,
            lithicrivers_core::message_log::MessageColor::Green => Color::Green,
            lithicrivers_core::message_log::MessageColor::Blue => Color::Blue,
            lithicrivers_core::message_log::MessageColor::Cyan => Color::Cyan,
            lithicrivers_core::message_log::MessageColor::Magenta => Color::Magenta,
        };
        lines.push(Line::from(Span::styled(
            &msg.text,
            Style::default().fg(color),
        )));
    }
    let paragraph = Paragraph::new(lines)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Line::from("Messages"))
                .style(Style::default().fg(Color::White)),
        );
    f.render_widget(paragraph, area);
}

fn render_bottom_menu(f: &mut Frame, app: &mut App, area: Rect) {
    // Remember for click handling
    app.ui.bottom_menu_rect = Some(area);
    // All tabs white; selected tab green
    let mut titles = vec![Span::raw("World")];

    // Conditionally add Tutorial tab if visible
    if app.ui.tutorial_visible {
        titles.push(Span::raw("Tutorial"));
    }

    titles.extend(vec![
        Span::raw("Global Map"),
        Span::raw("Quests"),
        Span::raw("Body"),
        Span::raw("Inventory"),
        Span::raw("Crafting"),
        Span::raw("Menu"),
        Span::raw("Help"),
        Span::raw("Credits"),
        Span::raw("Quit"),
    ]);

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Line::from("Menu")),
        )
        .select(app.ui.current_tab.as_index(app.ui.tutorial_visible))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Green));
    f.render_widget(tabs, area);
}

/// Calculate the game viewport area based on current UI layout
fn calculate_game_viewport_area(f: &Frame, app: &App) -> Rect {
    use crate::app_state::BuildMode;

    // Replicate the main UI layout logic to find game viewport bounds
    let show_hotbar =
        app.ui.current_tab == MenuTab::World && matches!(app.panels.build.mode, BuildMode::Place);

    let root_chunks = if show_hotbar {
        Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints([
                Constraint::Length(1), // Title line
                Constraint::Min(0),    // Main area (map + inventory)
                Constraint::Length(3), // Hotbar
                Constraint::Length(7), // Message log
                Constraint::Length(3), // Bottom menu bar
            ])
            .split(f.size())
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints([
                Constraint::Length(1), // Title line
                Constraint::Min(0),    // Main area (map + inventory)
                Constraint::Length(7), // Message log
                Constraint::Length(3), // Bottom menu bar
            ])
            .split(f.size())
    };

    // Calculate game view area based on current tab and combat state
    match app.ui.current_tab {
        MenuTab::World => {
            if app.combat.is_active() {
                // Combat mode: game view is right 50%
                let main_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(root_chunks[1]);
                main_chunks[1]
            } else if app.panels.look.mode {
                // With Look mode: game view is left side
                let main_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Min(20), Constraint::Length(24)])
                    .split(root_chunks[1]);
                main_chunks[0]
            } else {
                // Normal mode: game view is left side
                let main_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Min(20), Constraint::Length(24)])
                    .split(root_chunks[1]);
                main_chunks[0]
            }
        }
        MenuTab::Tutorial => {
            // Tutorial mode: game view is right 70%
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
                .split(root_chunks[1]);
            main_chunks[1]
        }
        MenuTab::GlobalMap
        | MenuTab::Quests
        | MenuTab::Crafting
        | MenuTab::Body
        | MenuTab::Help
        | MenuTab::Inventory
        | MenuTab::Menu
        | MenuTab::Credits
        | MenuTab::Quit => {
            // For non-world tabs, use the full main area
            root_chunks[1]
        }
    }
}

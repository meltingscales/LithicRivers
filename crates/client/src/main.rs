use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
mod app;
mod app_state;
mod boot_message;
mod clock;
mod dialogue_presenter;
mod headless;
mod mcp;
#[cfg(test)]
mod snapshot_tests;
mod tutorialsystem;
mod ui;

use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::border,
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame, Terminal,
};
use rust_embed::RustEmbed;
use std::{error::Error, io, time::Duration};
use tracing_subscriber::EnvFilter;

// Tracing file appender for log file output
use tracing_appender as _tracing_appender_hidden; // avoid "unused extern crate" lint

#[derive(RustEmbed)]
#[folder = "assets/"]
struct EmbeddedAssets;

use crate::app_state::*;
use crate::dialogue_presenter::DialoguePresenter;
mod audio;
mod rendering_helpers;
mod sprite_constants;
mod sprite_loader;
pub(crate) use crate::app::client_app::{App, MenuTab, SplashState};
pub(crate) use crate::ui::layout::ui;
use crate::{
    app::handle_input,
    sprite_loader::{Scale, SpriteLoader},
    ui::centered_rect,
};

fn main() -> Result<(), Box<dyn Error>> {
    // Load configuration to determine logging and seed before starting app
    let config = lithicrivers_core::game_config::GameConfig::new();
    let log_level = config
        .get_setting("game", "LOGGINGLEVEL")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("LOGGINGLEVEL must be set in config"));
    let seed_val: u64 = config
        .get_setting("game", "DEFAULT_SEED")
        .and_then(|v| v.as_u64())
        .unwrap_or_else(|| panic!("DEFAULT_SEED must be set in config"));

    // Initialize tracing to write logs to LithicRivers.log (rotated daily)
    {
        let file_appender = _tracing_appender_hidden::rolling::daily(".", "LithicRivers.log");
        let (non_blocking, _guard) = _tracing_appender_hidden::non_blocking(file_appender);
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(log_level.to_lowercase()));
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_writer(non_blocking)
            .with_ansi(false)
            .init();
        // Keep _guard alive for program lifetime to ensure logs flush properly
        let _ = Box::leak(Box::new(_guard));
    }

    // Headless scripted run: no terminal, no audio, fake clock
    if let Some(code) = run_headless_from_args(seed_val)? {
        std::process::exit(code);
    }

    // Ensure we restore the terminal if a panic occurs
    std::panic::set_hook(Box::new(|info| {
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen, DisableMouseCapture);
        eprintln!("\n\nPanic: {info}");
    }));

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let mut app = App::new_with_seed(seed_val);
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

/// Handle `--script <path> [--seed <u64>] [--size <cols>x<rows>]` and `--mcp`.
/// Returns the exit status if a headless mode ran, or `None` to start the interactive client.
fn run_headless_from_args(default_seed: u64) -> Result<Option<i32>, Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let mut script_path = None;
    let mut mcp = false;
    let mut overrides = headless::Overrides::default();
    while let Some(arg) = args.next() {
        let mut value = || args.next().ok_or_else(|| format!("{arg} expects a value"));
        match arg.as_str() {
            "--script" => script_path = Some(value()?),
            "--mcp" => mcp = true,
            "--seed" => overrides.seed = Some(value()?.parse()?),
            "--size" => {
                let v = value()?;
                overrides.size =
                    Some(headless::parse_size(&v).ok_or_else(|| format!("invalid --size '{v}'"))?);
            }
            other => return Err(format!("unknown argument '{other}'").into()),
        }
    }
    if mcp {
        if script_path.is_some() || overrides.size.is_some() {
            return Err("--mcp can't be combined with --script or --size".into());
        }
        mcp::serve_stdio(overrides.seed.unwrap_or(default_seed))?;
        return Ok(Some(0));
    }
    let Some(path) = script_path else {
        if overrides.seed.is_some() || overrides.size.is_some() {
            return Err("--seed and --size require --script".into());
        }
        return Ok(None);
    };
    let text = std::fs::read_to_string(&path).map_err(|e| format!("cannot read {path}: {e}"))?;
    let code = headless::run_script(
        &text,
        default_seed,
        overrides,
        &mut io::stdout().lock(),
        &mut io::stderr().lock(),
    );
    Ok(Some(code))
}

/// Advance the client by one frame: combat auto-ticks and splash/UI timers.
/// Shared by the interactive loop and headless runs; reads time only from `app.clock`.
fn step(app: &mut App) {
    // Auto-advance ticks when combat is active (one tick per COMBAT_MS_PER_TICK)
    if app.combat.is_active() {
        let tick_ms = app
            .core
            .game
            .res
            .config
            .get_setting("game", "COMBAT_MS_PER_TICK")
            .and_then(|v| v.as_u64())
            .unwrap_or_else(|| {
                panic!(
                    "COMBAT_MS_PER_TICK must be set in config. raw value: {:?}",
                    app.core
                        .game
                        .res
                        .config
                        .get_setting("game", "COMBAT_MS_PER_TICK")
                )
            });
        let tick_interval = Duration::from_millis(tick_ms);

        let current_time = app.clock.now();
        let should_tick = match app.last_combat_tick {
            None => {
                app.last_combat_tick = Some(current_time);
                false
            }
            Some(last_time) => {
                if current_time.saturating_duration_since(last_time) >= tick_interval {
                    app.last_combat_tick = Some(current_time);
                    true
                } else {
                    false
                }
            }
        };

        if should_tick {
            let tick_result = app.core.game.tick();
            app.apply_tick_result(&tick_result);
        }
    } else {
        // Reset tick timer when not in combat
        app.last_combat_tick = None;
    }

    app.on_tick();
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<(), Box<dyn Error>> {
    loop {
        step(app);

        terminal.draw(|f| ui(f, app))?;

        if crossterm::event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Press {
                        handle_input(app, key.code)?;
                    }
                }
                Event::Mouse(me) => {
                    if let crossterm::event::MouseEventKind::ScrollDown = me.kind {
                        // Scroll down (move view down, which means increase scroll position)
                        if let SplashState::BootMessage = app.splash.state {
                            let total_lines = app.splash.boot_display_text.lines().count() as u16;
                            let visible_lines = 20; // Approximate visible lines
                            if app.splash.boot_scroll + visible_lines < total_lines {
                                app.splash.boot_scroll = app.splash.boot_scroll.saturating_add(3);
                                // Scroll 3 lines at a time
                            }
                        }
                    } else if let crossterm::event::MouseEventKind::ScrollUp = me.kind {
                        // Scroll up (move view up, which means decrease scroll position)
                        if let SplashState::BootMessage = app.splash.state {
                            app.splash.boot_scroll = app.splash.boot_scroll.saturating_sub(3);
                            // Scroll 3 lines at a time
                        }
                    } else {
                        // Log other mouse events
                        tracing::debug!("UNHANDLED Mouse event: {:?}", me);
                    }
                }
                _ => {}
            }
        }

        if app.core.should_quit {
            app.core.game.res.log("Shutting down...");
            tracing::info!(
                target: "game",
                "shutdown tick={} view_z={} seed={}",
                app.core.game.res.time.tick,
                app.ui.view_z,
                app.core.game.res.world_state.seed
            );
            return Ok(());
        }
    }
}

/// Render corpse looting modals when active
fn render_corpse_looting_modals(f: &mut Frame, app: &mut App) {
    let looting_state = app.panels.corpse_looting.clone();
    match looting_state {
        CorpseLootingState::SelectingCorpse {
            adjacent_entities,
            selected_corpse,
        } => {
            render_corpse_selection_modal(f, app, &adjacent_entities, selected_corpse);
        }
        CorpseLootingState::LootingCorpse {
            entity,
            selected_loot_item,
            selected_player_item,
            loot_panel_focus,
        } => {
            render_corpse_loot_modal(
                f,
                app,
                entity,
                selected_loot_item,
                selected_player_item,
                loot_panel_focus,
            );
        }
        CorpseLootingState::None => {
            // No modal to render
        }
    }
}

/// Render the corpse selection modal when multiple corpses are nearby
fn render_corpse_selection_modal(
    f: &mut Frame,
    app: &mut App,
    adjacent_entities: &[hecs::Entity],
    selected_corpse: usize,
) {
    use lithicrivers_core::components::{EntityKind, Inventory as InvComp};
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
        .title(Line::from(" Choose Corpse to Loot "))
        .title_alignment(Alignment::Center)
        .style(Style::default());

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Create list of corpses
    let corpse_items: Vec<ListItem> = adjacent_entities
        .iter()
        .enumerate()
        .filter_map(|(i, &entity)| {
            // Get corpse info
            if let (Ok(entity_kind), Ok(inv)) = (
                app.core.game.world.get::<&EntityKind>(entity),
                app.core.game.world.get::<&InvComp>(entity),
            ) {
                if *entity_kind == EntityKind::Corpse {
                    let is_selected = selected_corpse == i;
                    let style = if is_selected {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };

                    // Show corpse name and item count
                    let item_count = inv.slots.len();
                    let text = format!("Corpse ({} items)", item_count);

                    return Some(ListItem::new(Line::from(Span::styled(text, style))));
                }
            }
            None
        })
        .collect();

    let list =
        List::new(corpse_items).highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_widget(list, inner);

    // Controls at bottom
    let controls = Paragraph::new("↑↓: Select | Enter: Loot | Esc: Cancel")
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

/// Render the corpse looting modal with inventory transfer interface
fn render_corpse_loot_modal(
    f: &mut Frame,
    app: &mut App,
    corpse_entity: hecs::Entity,
    selected_loot_item: usize,
    selected_player_item: usize,
    loot_panel_focus: bool,
) {
    use lithicrivers_core::components::{itemkind_name, Inventory as InvComp};
    use ratatui::{
        style::Modifier,
        text::{Line, Span},
        widgets::{Clear, List, ListItem},
    };

    // Get corpse inventory
    let corpse_inv = if let Ok(inv) = app.core.game.world.get::<&InvComp>(corpse_entity) {
        inv.clone()
    } else {
        return; // Corpse no longer exists
    };

    // Get player inventory
    let player_inv = if let Some(player_entity) = app.core.game.get_player_entity() {
        if let Ok(inv) = app.core.game.world.get::<&InvComp>(player_entity) {
            inv.clone()
        } else {
            return; // Player no longer exists
        }
    } else {
        return;
    };

    // Create modal area (centered, 70% width, 60% height)
    let area = centered_rect(70, 60, f.size());

    // Clear the background
    f.render_widget(Clear, area);

    // Main modal block
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Line::from(" Looting Corpse "))
        .title_alignment(Alignment::Center)
        .style(Style::default());

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Split into two columns
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);

    // Left side: Corpse inventory
    let corpse_block = Block::default()
        .borders(Borders::ALL)
        .title(Line::from(" Corpse Items "))
        .border_style(if loot_panel_focus {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });

    let corpse_items: Vec<ListItem> = corpse_inv
        .slots
        .iter()
        .enumerate()
        .map(|(i, stack)| {
            let is_selected = loot_panel_focus && selected_loot_item == i;
            let style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(Span::styled(
                format!("{:2} x {}", stack.qty, itemkind_name(stack.kind)),
                style,
            )))
        })
        .collect();

    let corpse_list = List::new(corpse_items).block(corpse_block);
    f.render_widget(corpse_list, chunks[0]);

    // Right side: Player inventory
    let player_block = Block::default()
        .borders(Borders::ALL)
        .title(Line::from(" Your Items "))
        .border_style(if !loot_panel_focus {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });

    let player_items: Vec<ListItem> = player_inv
        .slots
        .iter()
        .enumerate()
        .map(|(i, stack)| {
            let is_selected = !loot_panel_focus && selected_player_item == i;
            let style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(Span::styled(
                format!("{:2} x {}", stack.qty, itemkind_name(stack.kind)),
                style,
            )))
        })
        .collect();

    let player_list = List::new(player_items).block(player_block);
    f.render_widget(player_list, chunks[1]);

    // Controls at bottom
    let controls = Paragraph::new("←→: Switch Panel | ↑↓: Select | Enter: Take Item | Esc: Close")
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

/// Render NPC interaction modals when active
fn render_npc_interaction_modals(f: &mut Frame, app: &mut App, viewport_area: Rect) {
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

/// Render hotbar assignment modal when active
fn render_hotbar_assignment_modal(f: &mut Frame, app: &mut App) {
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

/// Render the multi-action selection modal
fn render_multi_action_selection_modal(f: &mut Frame, app: &mut App) {
    use crate::app_state::MultiActionSelectState;

    match &app.panels.multi_action_select {
        MultiActionSelectState::SelectingAction {
            available_actions,
            selected_action,
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

/// Render the action selection modal
fn render_action_selection_modal(
    f: &mut Frame,
    app: &mut App,
    available_actions: &[crate::app_state::InteractionType],
    selected_action: usize,
) {
    use ratatui::{
        style::Modifier,
        widgets::{Clear, List, ListItem},
    };

    // Create modal area (centered, 60% width, 50% height)
    let area = centered_rect(60, 50, f.size());

    // Clear the background
    f.render_widget(Clear, area);

    // Main modal block
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Line::from(" Choose Interaction "))
        .title_alignment(Alignment::Center)
        .style(Style::default());

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Get player position for directional information
    let player_pos = app
        .core
        .game
        .get_player_entity()
        .and_then(|e| {
            app.core
                .game
                .world
                .get::<&lithicrivers_core::components::Position>(e)
                .ok()
        })
        .map(|pos| *pos);

    // Create list of available actions
    let action_items: Vec<ListItem> = available_actions
        .iter()
        .enumerate()
        .map(|(i, action)| {
            let is_selected = selected_action == i;
            let style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let display_text = format!(
                "{} {}",
                action.icon(),
                action.display_name_with_context(&app.core.game.world, player_pos)
            );
            ListItem::new(display_text).style(style)
        })
        .collect();

    let list = List::new(action_items)
        .block(Block::default())
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    // Render the list with selection, leaving space for controls
    let list_height = inner.height.saturating_sub(2);
    let list_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: list_height,
    };

    f.render_widget(list, list_area);

    // Controls at bottom
    let controls = Paragraph::new("↑↓: Select | Enter: Choose | Esc: Cancel")
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

/// Render cheat console modal when active
fn render_cheat_console_modal(f: &mut Frame, app: &mut App) {
    if let CheatConsoleState::Open {
        input,
        cursor_position,
        autocomplete_suggestions,
        autocomplete_index: _,
        scroll_offset,
    } = &app.panels.cheat_console
    {
        // Create modal area (centered, 60% width, taller height for multiline help)
        let area = centered_rect(60, 25, f.size());

        // Clear the background
        f.render_widget(Clear, area);

        // Main modal block
        let block = Block::default()
            .borders(Borders::ALL)
            .title(Line::from(" Cheat Console "))
            .title_alignment(Alignment::Center)
            .style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        f.render_widget(block, area);

        // Create input text with cursor
        let mut display_text = input.clone();
        if *cursor_position <= input.len() {
            display_text.insert(*cursor_position, '|');
        }

        // Input field
        let input_paragraph = Paragraph::new(format!("> {}", display_text))
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Left);

        let input_area = Rect {
            x: inner.x,
            y: inner.y + 1,
            width: inner.width,
            height: 1,
        };
        f.render_widget(input_paragraph, input_area);

        // Help text - show autocomplete suggestions if available, otherwise show all commands
        let help_lines = if !autocomplete_suggestions.is_empty() {
            vec![Line::from(format!(
                "Suggestions: {}",
                autocomplete_suggestions.join(", ")
            ))]
        } else {
            let registry = app::input::cheat_console::get_command_registry();
            let commands = registry.get_command_descriptions();
            let mut lines = vec![Line::from("Available commands:")];

            // Show 5 commands at a time (expanded visual size)
            let max_visible_commands = 5;
            let total_commands = commands.len();
            let start_index = *scroll_offset;
            let end_index = (start_index + max_visible_commands).min(total_commands);

            // Show scroll up indicator if we're not at the top
            if start_index > 0 {
                lines.push(Line::from("  ▲ (more commands above)"));
            }

            // Show visible commands
            for command in &commands[start_index..end_index] {
                lines.push(Line::from(format!("  {}", command)));
            }

            // Show scroll down indicator if there are more commands below
            if end_index < total_commands {
                lines.push(Line::from("  ▼ (more commands below)"));
            }

            // Add scroll instructions if there are more commands
            if total_commands > max_visible_commands {
                lines.push(Line::from(""));
                let up_key = app
                    .core
                    .config_manager
                    .get_printable_key_for_keybind("ui", "CHEAT_CONSOLE_SCROLL_UP");
                let down_key = app
                    .core
                    .config_manager
                    .get_printable_key_for_keybind("ui", "CHEAT_CONSOLE_SCROLL_DOWN");
                lines.push(Line::from(format!(
                    "Use {}/{} to scroll through commands",
                    up_key, down_key
                )));
            }

            lines
        };
        let help_paragraph = Paragraph::new(help_lines)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);

        let help_height = if !autocomplete_suggestions.is_empty() {
            1
        } else {
            6
        }; // 1 line for title + 5 commands
        let help_area = Rect {
            x: inner.x,
            y: inner.y + 3,
            width: inner.width,
            height: help_height,
        };
        f.render_widget(help_paragraph, help_area);

        // Controls
        let controls = Paragraph::new("Enter: Execute | Tab: Autocomplete | Esc: Cancel")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));

        if inner.height > 7 {
            // Need more height for multiline help + controls
            let controls_area = Rect {
                x: inner.x,
                y: inner.y + inner.height - 1,
                width: inner.width,
                height: 1,
            };
            f.render_widget(controls, controls_area);
        }
    }
}

// Helper function to get player's inventory as a HashMap

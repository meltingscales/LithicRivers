use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

pub fn render_combat_panel(f: &mut Frame, app: &mut crate::App, area: Rect) {
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .title(" Combat ")
        .title_alignment(Alignment::Center)
        .style(Style::default().fg(Color::Red));

    let inner = outer_block.inner(area);
    f.render_widget(outer_block, area);

    // Get current selection and timers from combat state
    let (current_move, current_enemy, enemy_timers, player_action_timer) = match &app.combat {
        crate::CombatUiState::Active {
            current_move,
            current_enemy,
            enemy_timers,
            player_action_timer,
        } => (
            *current_move,
            *current_enemy,
            enemy_timers.clone(),
            *player_action_timer,
        ),
        _ => (0, 0, vec![], None), // Fallback, shouldn't happen when this function is called
    };

    // Get real player and enemy data from the game world
    let (player_health, player_energy, available_moves, cooldowns) = get_player_combat_data(app);
    let combat_enemies = get_enemy_combat_data(app);

    // Ensure selections are within bounds
    let current_move = current_move.min(available_moves.len().saturating_sub(1));
    let current_enemy = current_enemy.min(combat_enemies.len().saturating_sub(1));

    // Victory check
    if combat_enemies.is_empty() || combat_enemies.iter().all(|e| e.health <= 0) {
        let victory = Paragraph::new("Victory!")
            .style(Style::default().fg(Color::Green))
            .alignment(Alignment::Center);
        f.render_widget(victory, inner);
        return;
    }

    // Main layout: top player info, middle enemies, bottom moves/controls
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Player health/mana bars
            Constraint::Min(8),     // Enemies area
            Constraint::Length(15), // Moves area (increased to fit player timer + 4 moves)
            Constraint::Length(3),  // Message/status area
            Constraint::Length(1),  // Controls
        ])
        .split(inner);

    // Render player info
    render_player_info(f, chunks[0], player_health, player_energy);

    // Render enemies with real timers
    render_enemies(f, chunks[1], &combat_enemies, current_enemy, &enemy_timers);

    // Debug moves area
    app.core.game.res.log(format!(
        "Moves area: {}x{}",
        chunks[2].width, chunks[2].height
    ));

    // Render moves with player action timer
    render_moves(
        f,
        chunks[2],
        &available_moves,
        &cooldowns,
        player_energy,
        current_move,
        player_action_timer,
        app,
    );

    // Message area (placeholder for now)
    let message_para = Paragraph::new("Combat active - Select your move!")
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(message_para, chunks[3]);

    // Controls
    let controls = Paragraph::new(
        "[←→] Select Target | [↑↓] Select Move | [1-4] Quick Select | [SPACE] Use Move",
    )
    .style(Style::default().fg(Color::Gray))
    .alignment(Alignment::Center);
    f.render_widget(controls, chunks[4]);
}

// Old render_player_info function removed - using new one with Body and Energy

fn render_enemies(
    f: &mut Frame,
    area: Rect,
    enemies: &[CombatEnemy],
    current_enemy: usize,
    enemy_timers: &[u32],
) {
    if enemies.is_empty() {
        return;
    }

    // Create horizontal layout for multiple enemies
    let constraints: Vec<Constraint> = (0..enemies.len())
        .map(|_| Constraint::Ratio(1, enemies.len() as u32))
        .collect();
    let enemy_chunks = Layout::horizontal(constraints).split(area);

    for (i, (enemy, chunk)) in enemies.iter().zip(enemy_chunks.iter()).enumerate() {
        let is_selected = i == current_enemy;
        let timer = enemy_timers.get(i).copied().unwrap_or(0);
        render_single_enemy(f, *chunk, enemy, is_selected, timer);
    }
}

fn render_single_enemy(
    f: &mut Frame,
    area: Rect,
    enemy: &CombatEnemy,
    is_selected: bool,
    timer_ms: u32,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(if is_selected {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Simple ASCII art placeholder (later this will use sprite system)
    let portrait_lines = vec!["  /\\_/\\  ", " ( o.o ) ", "  > ^ <  "];

    let portrait = Paragraph::new(portrait_lines.join("\n"))
        .style(Style::default().fg(Color::Green))
        .alignment(Alignment::Center);

    // Layout: portrait at top, health bar at bottom
    let enemy_layout = Layout::vertical([
        Constraint::Min(3),    // Portrait area
        Constraint::Length(3), // Health bar
        Constraint::Length(1), // Attack timer
    ])
    .split(inner);

    f.render_widget(portrait, enemy_layout[0]);

    // Health bar
    let health_ratio = enemy.health as f64 / enemy.max_health as f64;
    let health_bar = Gauge::default()
        .block(
            Block::default()
                .title(enemy.name.as_str())
                .borders(Borders::ALL),
        )
        .gauge_style(Style::default().fg(Color::Red).bg(Color::DarkGray))
        .ratio(health_ratio)
        .label(format!(" {}/{} ", enemy.health, enemy.max_health));

    f.render_widget(health_bar, enemy_layout[1]);

    // Attack timer using real combat timing - only show whole seconds to prevent constant re-renders
    let attack_timer = if timer_ms > 0 {
        let seconds = timer_ms / 1000;
        format!("⏳ {}s", seconds)
    } else {
        "⚡ ATTACKING!".to_string()
    };

    let timer = Paragraph::new(attack_timer)
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center);

    f.render_widget(timer, enemy_layout[2]);
}

fn render_action_queue(f: &mut Frame, area: Rect, app: &mut crate::App) {
    use ratatui::text::Line;
    use ratatui::widgets::Wrap;

    // Mock queue data for now - later this will come from the actual combat system
    let mut lines = vec![Line::from("Current:".bold())];

    // Show player action timer if active
    if let crate::CombatUiState::Active {
        player_action_timer,
        current_move,
        ..
    } = &app.combat
    {
        if let Some(timer_ms) = player_action_timer {
            let seconds = timer_ms / 1000;
            lines.push(Line::from(format!("▶ Executing... {}s", seconds)));
        } else {
            lines.push(Line::from("▶ Waiting..."));
        }
    } else {
        lines.push(Line::from("▶ Waiting..."));
    }

    // Mock queue items - in the real system this would show actual queued actions
    lines.push(Line::from(""));
    lines.push(Line::from("Queue:".bold()));
    lines.push(Line::from("• Enemy 1 attacks"));
    lines.push(Line::from("• Enemy 2 attacks"));

    let block = Block::default().borders(Borders::ALL).title("Action Queue");
    let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

fn render_moves(
    f: &mut Frame,
    area: Rect,
    moves: &[lithicrivers_core::moves::Move],
    cooldowns: &lithicrivers_core::moves::MoveCooldowns,
    energy: lithicrivers_core::components::Energy,
    current_move: usize,
    player_action_timer: Option<u32>,
    app: &mut crate::App,
) {
    // Split the moves area horizontally: moves on left, queue on right
    let horizontal_chunks = Layout::horizontal([
        Constraint::Percentage(70), // Moves area
        Constraint::Percentage(30), // Queue area
    ])
    .split(area);

    let moves_area = horizontal_chunks[0];
    let queue_area = horizontal_chunks[1];
    // Debug: log the area we're working with
    app.core.game.res.log(format!(
        "render_moves area: {}x{} at ({},{})",
        area.width, area.height, area.x, area.y
    ));
    // Always split moves area to show player action timer at the top (always visible)
    let layout = Layout::vertical([Constraint::Length(3), Constraint::Min(12)]).split(moves_area);
    let moves_only_area = layout[1];

    // Always render player action timer - show timer or "Ready" state
    let timer_text = if let Some(timer_ms) = player_action_timer {
        let seconds = timer_ms / 1000;
        format!("⚡ Executing move... {}s", seconds)
    } else {
        "⚡ Ready".to_string()
    };

    let timer_para = Paragraph::new(timer_text)
        .style(if player_action_timer.is_some() {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Yellow)
        })
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Player Action"),
        );
    f.render_widget(timer_para, layout[0]);

    app.core.game.res.log(format!(
        "Player action timer area: {}x{}, Moves area: {}x{}",
        layout[0].width, layout[0].height, layout[1].width, layout[1].height
    ));

    let move_blocks = moves
        .iter()
        .enumerate()
        .map(|(i, mv)| {
            let is_selected = i == current_move;
            let can_use_energy = energy.current >= mv.energy_cost;
            let can_use_cooldown = cooldowns.can_use(mv.move_type);
            let can_use = can_use_energy && can_use_cooldown;

            let cooldown = if cooldowns.get_cooldown(mv.move_type) > 0 {
                format!(" ({})", cooldowns.get_cooldown(mv.move_type))
            } else {
                String::new()
            };

            // Always use the same basic format to ensure consistent rendering
            let prefix = if !can_use { "[X] " } else { "" };
            let move_name = if mv.name.is_empty() {
                "Unknown"
            } else {
                &mv.name
            };
            let content = format!(
                "{}{}: {}{} - {}E",
                prefix,
                i + 1,
                move_name,
                cooldown,
                mv.energy_cost
            );

            // Ensure content is never empty
            let final_content = if content.trim().is_empty() {
                format!("{}: Move - 0E", i + 1)
            } else {
                content
            };

            // Debug: log the move content
            app.core.game.res.log(format!(
                "Move {}: '{}' (can_use: {}, selected: {})",
                i, final_content, can_use, is_selected
            ));

            let style = if !can_use {
                Style::default().fg(Color::DarkGray)
            } else if is_selected {
                Style::default().fg(Color::Yellow).bold()
            } else {
                Style::default().fg(Color::White)
            };

            let border_style = if !can_use {
                Style::default().fg(Color::DarkGray)
            } else if is_selected {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };

            Paragraph::new(final_content).style(style).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(border_style),
            )
        })
        .collect::<Vec<_>>();

    // Debug: log move information
    app.core
        .game
        .res
        .log(format!("Number of moves: {}", move_blocks.len()));
    app.core.game.res.log(format!(
        "Moves area for blocks: {}x{} at ({},{})",
        moves_only_area.width, moves_only_area.height, moves_only_area.x, moves_only_area.y
    ));

    // Ensure we have constraints for each move
    let num_moves = move_blocks.len();
    let constraints: Vec<Constraint> = (0..num_moves).map(|_| Constraint::Length(3)).collect();

    let move_chunks = Layout::vertical(constraints).split(moves_only_area);

    app.core.game.res.log(format!(
        "Generated {} chunks for {} moves",
        move_chunks.len(),
        num_moves
    ));

    // Render each move block, but ensure we don't go out of bounds
    for (i, block) in move_blocks.into_iter().enumerate() {
        if i < move_chunks.len() {
            let chunk = move_chunks[i];
            app.core.game.res.log(format!(
                "Move {}: chunk {}x{} at ({},{})",
                i, chunk.width, chunk.height, chunk.x, chunk.y
            ));
            if chunk.height > 0 && chunk.width > 0 {
                f.render_widget(block, chunk);
            } else {
                app.core
                    .game
                    .res
                    .log(format!("Skipping move {} - zero size chunk", i));
            }
        } else {
            app.core
                .game
                .res
                .log(format!("Move {} out of bounds - no chunk available", i));
        }
    }

    // Render the action queue on the right side
    render_action_queue(f, queue_area, app);
}

// Helper functions to get real game data
fn get_player_combat_data(
    app: &mut crate::App,
) -> (
    Option<lithicrivers_core::model::body::Body>,
    lithicrivers_core::components::Energy,
    Vec<lithicrivers_core::moves::Move>,
    lithicrivers_core::moves::MoveCooldowns,
) {
    use lithicrivers_core::components::Energy;
    use lithicrivers_core::model::body::Body;
    use lithicrivers_core::moves::{get_available_moves, MoveCooldowns};

    // Find the player entity and get their body and energy
    let mut player_body = None;
    let mut player_energy = Energy::new(100); // Default energy
    let mut cooldowns = MoveCooldowns::new();

    if let Some(player_entity) = app.core.game.res.player_entity {
        // Try to get player body
        if let Ok(body) = app.core.game.world.get::<&Body>(player_entity) {
            player_body = Some((*body).clone());
        }

        // Get player energy - should always exist
        player_energy = *app
            .core
            .game
            .world
            .get::<&Energy>(player_entity)
            .expect("Player entity must have Energy component");

        // Get cooldowns - should always exist
        cooldowns = (*app
            .core
            .game
            .world
            .get::<&MoveCooldowns>(player_entity)
            .expect("Player entity must have MoveCooldowns component"))
        .clone();
    }

    (player_body, player_energy, get_available_moves(), cooldowns)
}

#[derive(Debug, Clone)]
struct CombatEnemy {
    name: String,
    health: u32,
    max_health: u32,
}

fn get_enemy_combat_data(app: &mut crate::App) -> Vec<CombatEnemy> {
    use lithicrivers_core::components::{Combat, GameEntity, Health, Position};

    let mut enemies = Vec::new();

    // Find all combat entities near the player
    if let Some(player_entity) = app.core.game.res.player_entity {
        if let Ok(player_pos) = app.core.game.world.get::<&Position>(player_entity) {
            let player_pos = *player_pos;

            // Look for nearby combat entities
            for (entity, (pos, _, _)) in app
                .core
                .game
                .world
                .query::<(&Position, &Combat, &GameEntity)>()
                .iter()
            {
                if entity == player_entity {
                    continue;
                } // Skip player

                let dx = player_pos.x - pos.x;
                let dy = player_pos.y - pos.y;
                let distance_sq = dx * dx + dy * dy;

                if distance_sq <= 1 {
                    // Adjacent enemies - health should always exist
                    let health = *app
                        .core
                        .game
                        .world
                        .get::<&Health>(entity)
                        .expect("Combat entities must have Health component");

                    enemies.push(CombatEnemy {
                        name: format!("Enemy {}", entity.id()), // Use entity ID for now
                        health: health.current,
                        max_health: health.max,
                    });
                }
            }
        }
    }

    enemies
}

fn render_player_info(
    f: &mut Frame,
    area: Rect,
    body: Option<lithicrivers_core::model::body::Body>,
    energy: lithicrivers_core::components::Energy,
) {
    use lithicrivers_core::moves::{calculate_body_integrity, get_body_status_description};

    // Calculate body integrity (replaces health for robots)
    let (integrity_ratio, integrity_label) = if let Some(ref body) = body {
        let integrity = calculate_body_integrity(body);
        let status = get_body_status_description(body);
        (integrity as f64, status)
    } else {
        (1.0, "No Body Data".to_string())
    };

    let energy_ratio = energy.percentage() as f64;

    let integrity_bar = Gauge::default()
        .block(
            Block::default()
                .title("Body Integrity")
                .borders(Borders::ALL),
        )
        .gauge_style(Style::default().fg(Color::Cyan).bg(Color::DarkGray))
        .ratio(integrity_ratio)
        .label(format!(" {} ", integrity_label));

    let energy_bar = Gauge::default()
        .block(Block::default().title("Energy").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Yellow).bg(Color::DarkGray))
        .ratio(energy_ratio)
        .label(format!(" {}/{} ", energy.current, energy.max));

    let bars = Layout::horizontal([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)]).split(area);

    f.render_widget(integrity_bar, bars[0]);
    f.render_widget(energy_bar, bars[1]);
}

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
    let (current_move, current_enemy, enemy_timers, scroll_offset) = match &app.combat {
        crate::CombatUiState::Active {
            current_move,
            current_enemy,
            enemy_timers,
            move_scroll_offset,
        } => (
            *current_move,
            *current_enemy,
            enemy_timers.clone(),
            *move_scroll_offset,
        ),
        _ => (0, 0, vec![], 0), // Fallback, shouldn't happen when this function is called
    };

    // Get real player and enemy data from the game world
    let (player_health, player_energy, available_moves) = get_player_combat_data(app);
    let combat_enemies = get_enemy_combat_data(app);

    // Ensure selections are within bounds
    let current_move = current_move.min(available_moves.len().saturating_sub(1));
    let current_enemy = current_enemy.min(combat_enemies.len().saturating_sub(1));

    // Update the combat state with clamped values to prevent desync
    if let crate::CombatUiState::Active {
        current_enemy: ref mut state_enemy,
        current_move: ref mut state_move,
        ..
    } = app.combat
    {
        *state_enemy = current_enemy;
        *state_move = current_move;
    }

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
    render_enemies(
        f,
        chunks[1],
        &combat_enemies,
        current_enemy,
        &enemy_timers,
        app,
    );

    // Render moves
    render_moves(
        f,
        chunks[2],
        &available_moves,
        player_energy,
        current_move,
        scroll_offset,
        app,
    );

    // Message area (placeholder for now)
    let message_para = Paragraph::new("Combat active - Select your move!")
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(message_para, chunks[3]);

    // Controls
    let controls = Paragraph::new("[←→] Target | [↑↓] Move | [1-4] Quick | [SPACE] Use")
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
    app: &mut crate::App,
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
        render_single_enemy(f, *chunk, enemy, is_selected, timer, app);
    }
}

fn render_single_enemy(
    f: &mut Frame,
    area: Rect,
    enemy: &CombatEnemy,
    is_selected: bool,
    timer_ms: u32,
    app: &mut crate::App,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(if is_selected {
            Style::default().fg(Color::Green)
        } else {
            Style::default()
        });

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Use sprite system for enemy portraits - get the actual 12x8 art
    let portrait = if let Some(sprite_ref) = &enemy.sprite_ref {
        let sprite_data = app.core.sprite_loader.load_by_spriteref(sprite_ref);

        // Get the first 12x8 art sprite (main portrait)
        let sprite_block = sprite_data
            .art12x8_sprites
            .get(0)
            .cloned()
            .unwrap_or_else(|| {
                panic!(
                    "Missing 12x8 art for sprite '{}::{}'",
                    sprite_ref.category, sprite_ref.name
                )
            });

        let sprite_color = crate::sprite_loader::parse_color_string(&sprite_data.color)
            .unwrap_or_else(|| {
                panic!(
                    "Invalid color for sprite '{}::{}'",
                    sprite_ref.category, sprite_ref.name
                )
            });

        Paragraph::new(sprite_block)
            .style(Style::default().fg(sprite_color))
            .alignment(Alignment::Center)
    } else {
        // No fallback - entities without sprites show empty space
        Paragraph::new("")
            .style(Style::default())
            .alignment(Alignment::Center)
    };

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
        format!("{}s", seconds)
    } else {
        "".to_string()
    };

    let timer = Paragraph::new(attack_timer)
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center);

    f.render_widget(timer, enemy_layout[2]);
}

fn render_action_queue(f: &mut Frame, area: Rect, app: &mut crate::App) {
    use ratatui::text::Line;
    use ratatui::widgets::Wrap;

    let mut lines = vec![Line::from("Current:".bold())];

    // Get real action queue data from the player entity
    if let Some(player_entity) = app.core.game.get_player_entity() {
        if let Ok(queue) = app
            .core
            .game
            .world
            .get::<&lithicrivers_core::moves::ActionQueue>(player_entity)
        {
            // Show current action
            if let Some(current_action) = &queue.current_action {
                let ticks = current_action.remaining_time_ticks;
                let action_text = match &current_action.action {
                    lithicrivers_core::moves::CombatAction::PlayerMove {
                        move_data,
                        target_entity,
                        ..
                    } => {
                        if let Some(target) = target_entity {
                            // Try to get target name from the world
                            let target_name =
                                if let Some(player_entity) = app.core.game.get_player_entity() {
                                    if *target == player_entity {
                                        "Player".to_string()
                                    } else {
                                        format!("Enemy {}", target.id())
                                    }
                                } else {
                                    format!("Entity {}", target.id())
                                };
                            format!("{} → {}", move_data.move_type.human_name(), target_name)
                        } else {
                            format!("{} (no target)", move_data.move_type.human_name())
                        }
                    }
                    lithicrivers_core::moves::CombatAction::EnemyAttack {
                        target_entity, ..
                    } => {
                        let target_name =
                            if let Some(player_entity) = app.core.game.get_player_entity() {
                                if *target_entity == player_entity {
                                    "Player".to_string()
                                } else {
                                    format!("Entity {}", target_entity.id())
                                }
                            } else {
                                format!("Entity {}", target_entity.id())
                            };
                        format!("Enemy Attack → {}", target_name)
                    }
                };
                lines.push(Line::from(format!("> {} ({}t)", action_text, ticks)));
            } else {
                lines.push(Line::from("> Ready"));
            }

            // Show queued actions
            lines.push(Line::from(""));
            lines.push(Line::from("Queue:".bold()));

            if queue.actions.is_empty() {
                lines.push(Line::from("* No actions queued"));
            } else {
                for (i, action) in queue.actions.iter().enumerate() {
                    if i >= 3 {
                        // Limit display to first 3 queued actions
                        lines.push(Line::from(format!(
                            "* ... and {} more",
                            queue.actions.len() - 3
                        )));
                        break;
                    }

                    let action_text = match &action.action {
                        lithicrivers_core::moves::CombatAction::PlayerMove {
                            move_data,
                            target_entity,
                            ..
                        } => {
                            if let Some(target) = target_entity {
                                // Try to get target name from the world
                                let target_name = if let Some(player_entity) =
                                    app.core.game.get_player_entity()
                                {
                                    if *target == player_entity {
                                        "Player".to_string()
                                    } else {
                                        format!("Enemy {}", target.id())
                                    }
                                } else {
                                    format!("Entity {}", target.id())
                                };
                                format!("{} → {}", move_data.move_type.human_name(), target_name)
                            } else {
                                format!("{} (no target)", move_data.move_type.human_name())
                            }
                        }
                        lithicrivers_core::moves::CombatAction::EnemyAttack {
                            target_entity,
                            ..
                        } => {
                            let target_name =
                                if let Some(player_entity) = app.core.game.get_player_entity() {
                                    if *target_entity == player_entity {
                                        "Player".to_string()
                                    } else {
                                        format!("Entity {}", target_entity.id())
                                    }
                                } else {
                                    format!("Entity {}", target_entity.id())
                                };
                            format!("Enemy Attack → {}", target_name)
                        }
                    };
                    lines.push(Line::from(format!("* {}", action_text)));
                }
            }
        } else {
            lines.push(Line::from("> No queue"));
        }
    } else {
        lines.push(Line::from("> No player"));
    }

    // Show enemy actions too
    lines.push(Line::from(""));
    lines.push(Line::from("Enemies:".bold()));

    let mut enemy_count = 0;
    for (entity, (_, combat, _)) in app
        .core
        .game
        .world
        .query::<(
            &lithicrivers_core::components::Position,
            &lithicrivers_core::components::Combat,
            &lithicrivers_core::components::GameEntity,
        )>()
        .iter()
    {
        if Some(entity) == app.core.game.get_player_entity() || !combat.triggered {
            continue;
        }

        enemy_count += 1;
        if enemy_count > 3 {
            lines.push(Line::from("* ..."));
            break;
        }

        if let Ok(queue) = app
            .core
            .game
            .world
            .get::<&lithicrivers_core::moves::ActionQueue>(entity)
        {
            if let Some(current) = &queue.current_action {
                let ticks = current.remaining_time_ticks;
                lines.push(Line::from(format!("* Enemy {} ({}t)", enemy_count, ticks)));
            } else {
                lines.push(Line::from(format!("* Enemy {} ready", enemy_count)));
            }
        } else {
            lines.push(Line::from(format!("* Enemy {} preparing", enemy_count)));
        }
    }

    let block = Block::default().borders(Borders::ALL).title("Action Queue");
    let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

fn render_moves(
    f: &mut Frame,
    area: Rect,
    moves: &[lithicrivers_core::moves::Move],
    energy: lithicrivers_core::components::Energy,
    current_move: usize,
    scroll_offset: usize,
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

    // Always split moves area to show player action timer at the top (always visible)
    let layout = Layout::vertical([Constraint::Length(3), Constraint::Min(12)]).split(moves_area);
    let moves_only_area = layout[1];

    // Render player action timer based on ActionQueue system
    let timer_text = if let Some(player_entity) = app.core.game.get_player_entity() {
        if let Ok(queue) = app
            .core
            .game
            .world
            .get::<&lithicrivers_core::moves::ActionQueue>(player_entity)
        {
            if let Some(current_action) = &queue.current_action {
                let ticks = current_action.remaining_time_ticks;
                let action_text = match &current_action.action {
                    lithicrivers_core::moves::CombatAction::PlayerMove {
                        move_data,
                        target_entity,
                        ..
                    } => {
                        if let Some(target) = target_entity {
                            let target_name =
                                if let Some(player_entity) = app.core.game.get_player_entity() {
                                    if *target == player_entity {
                                        "Player".to_string()
                                    } else {
                                        format!("Enemy {}", target.id())
                                    }
                                } else {
                                    format!("Entity {}", target.id())
                                };
                            format!("{} → {}", move_data.move_type.human_name(), target_name)
                        } else {
                            format!("{} (no target)", move_data.move_type.human_name())
                        }
                    }
                    _ => "Action".to_string(),
                };
                format!("{} ({}t)", action_text, ticks)
            } else {
                "Ready".to_string()
            }
        } else {
            "Ready".to_string()
        }
    } else {
        "No Player".to_string()
    };

    let has_action = if let Some(player_entity) = app.core.game.get_player_entity() {
        app.core
            .game
            .world
            .get::<&lithicrivers_core::moves::ActionQueue>(player_entity)
            .map(|queue| queue.current_action.is_some())
            .unwrap_or(false)
    } else {
        false
    };

    let timer_para = Paragraph::new(timer_text)
        .style(if has_action {
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

    // Calculate how many moves we can fit in the available space
    let mut available_height = moves_only_area.height as usize;

    // Reserve space for scroll indicators if needed
    if scroll_offset > 0 {
        available_height = available_height.saturating_sub(1);
    }
    if scroll_offset + available_height < moves.len() {
        available_height = available_height.saturating_sub(1);
    }

    let visible_moves_count = available_height.min(moves.len() - scroll_offset);
    let end_offset = (scroll_offset + visible_moves_count).min(moves.len());
    let visible_moves = &moves[scroll_offset..end_offset];

    let move_blocks = visible_moves
        .iter()
        .enumerate()
        .map(|(visible_i, mv)| {
            let actual_i = scroll_offset + visible_i;
            let is_selected = actual_i == current_move;
            let can_use_energy = energy.current >= mv.energy_cost;
            let can_use = can_use_energy; // Only check energy, action queue handles timing

            // Always use the same basic format to ensure consistent rendering
            let prefix = if !can_use { "[X] " } else { "" };
            let move_name = if mv.name.is_empty() {
                "Unknown"
            } else {
                &mv.name
            };
            let content = format!(
                "{}{}: {} - {}E - {}t",
                prefix,
                actual_i + 1,
                move_name,
                mv.energy_cost,
                mv.execution_time_ticks,
            );

            // // Debug: log the move content
            // app.core.game.res.log(format!(
            //     "Move {}: '{}' (can_use: {}, selected: {})",
            //     actual_i, content, can_use, is_selected
            // ));

            let style = if !can_use {
                Style::default().fg(Color::DarkGray)
            } else if is_selected {
                Style::default().fg(Color::Green).bold()
            } else {
                Style::default().fg(Color::White)
            };

            let _border_style = if !can_use {
                Style::default().fg(Color::DarkGray)
            } else if is_selected {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            };

            Paragraph::new(content).style(style)
        })
        .collect::<Vec<_>>();

    // Create constraints for visible moves, plus scroll indicators
    let num_visible = move_blocks.len();
    let mut constraints: Vec<Constraint> = vec![];

    // Add scroll indicator at top if needed
    if scroll_offset > 0 {
        constraints.push(Constraint::Length(1));
    }

    // Add constraints for each visible move
    for _ in 0..num_visible {
        constraints.push(Constraint::Length(1));
    }

    // Add scroll indicator at bottom if needed
    if scroll_offset + visible_moves_count < moves.len() {
        constraints.push(Constraint::Length(1));
    }

    let move_chunks = Layout::vertical(constraints).split(moves_only_area);
    let mut chunk_index = 0;

    // Render top scroll indicator
    if scroll_offset > 0 {
        let scroll_up = Paragraph::new("▲ More above")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::NONE));
        f.render_widget(scroll_up, move_chunks[chunk_index]);
        chunk_index += 1;
    }

    // Render each move block
    for (i, block) in move_blocks.into_iter().enumerate() {
        if chunk_index + i < move_chunks.len() {
            let chunk = move_chunks[chunk_index + i];
            if chunk.height > 0 && chunk.width > 0 {
                f.render_widget(block, chunk);
            }
        }
    }
    chunk_index += num_visible;

    // Render bottom scroll indicator
    if scroll_offset + visible_moves_count < moves.len() {
        if chunk_index < move_chunks.len() {
            let scroll_down = Paragraph::new("▼ More below")
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::NONE));
            f.render_widget(scroll_down, move_chunks[chunk_index]);
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
) {
    use lithicrivers_core::components::Energy;
    use lithicrivers_core::model::body::Body;
    use lithicrivers_core::moves::get_available_moves;

    // Find the player entity and get their body and energy
    let mut player_body = None;
    let mut player_energy = Energy::new(100); // Default energy

    if let Some(player_entity) = app.core.game.get_player_entity() {
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
    }

    (player_body, player_energy, get_available_moves())
}

#[derive(Debug, Clone)]
struct CombatEnemy {
    name: String,
    health: u32,
    max_health: u32,
    sprite_ref: Option<lithicrivers_core::components::SpriteRef>,
}

fn get_enemy_combat_data(app: &mut crate::App) -> Vec<CombatEnemy> {
    use lithicrivers_core::components::{Combat, GameEntity, Health, Position, SpriteRef};

    let mut enemies = Vec::new();

    // Find all combat entities near the player
    if let Some(player_entity) = app.core.game.get_player_entity() {
        if let Ok(player_pos) = app.core.game.world.get::<&Position>(player_entity) {
            let player_pos = *player_pos;

            // Look for nearby combat entities that are actually in combat
            for (entity, (pos, combat, _, sprite_ref)) in app
                .core
                .game
                .world
                .query::<(&Position, &Combat, &GameEntity, Option<&SpriteRef>)>()
                .iter()
            {
                if entity == player_entity {
                    continue;
                } // Skip player

                // Define the 8 adjacent positions around the player (same as combat_trigger_system)
                let adjacent_positions = [
                    (player_pos.x - 1, player_pos.y - 1), // NW
                    (player_pos.x, player_pos.y - 1),     // N
                    (player_pos.x + 1, player_pos.y - 1), // NE
                    (player_pos.x - 1, player_pos.y),     // W
                    (player_pos.x + 1, player_pos.y),     // E
                    (player_pos.x - 1, player_pos.y + 1), // SW
                    (player_pos.x, player_pos.y + 1),     // S
                    (player_pos.x + 1, player_pos.y + 1), // SE
                ];

                // Check if enemy is in any of the 8 adjacent positions
                let is_adjacent = adjacent_positions.iter().any(|&(adj_x, adj_y)| {
                    pos.x == adj_x && pos.y == adj_y && pos.z == player_pos.z
                });

                if is_adjacent && combat.triggered {
                    // Skip dead enemies
                    if app
                        .core
                        .game
                        .world
                        .get::<&lithicrivers_core::components::Dead>(entity)
                        .is_ok()
                    {
                        continue;
                    }

                    // Only include enemies that are actually in combat
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
                        sprite_ref: sprite_ref.cloned(),
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

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

// Mock combat data structures - these will be replaced with actual game data
#[derive(Debug, Clone)]
struct CombatMove {
    name: String,
    mana_cost: u32,
    cooldown: u32,
    current_cooldown: u32,
}

#[derive(Debug, Clone)]
struct CombatPlayer {
    health: u32,
    max_health: u32,
    mana: u32,
    max_mana: u32,
    moves: Vec<CombatMove>,
}

#[derive(Debug, Clone)]
struct CombatEnemy {
    name: String,
    health: u32,
    max_health: u32,
    attack_timer: u32,
    attack_speed: u32,
}

// For now, create mock data
impl CombatPlayer {
    fn mock() -> Self {
        Self {
            health: 180,
            max_health: 200,
            mana: 60,
            max_mana: 100,
            moves: vec![
                CombatMove {
                    name: "Melee".to_string(),
                    mana_cost: 0,
                    cooldown: 2,
                    current_cooldown: 0,
                },
                CombatMove {
                    name: "Fireball (AoE)".to_string(),
                    mana_cost: 40,
                    cooldown: 3,
                    current_cooldown: 2,
                },
                CombatMove {
                    name: "Tackle".to_string(),
                    mana_cost: 20,
                    cooldown: 5,
                    current_cooldown: 0,
                },
                CombatMove {
                    name: "Escape".to_string(),
                    mana_cost: 20,
                    cooldown: 0,
                    current_cooldown: 0,
                },
            ],
        }
    }

    fn can_use_move(&self, move_index: usize) -> bool {
        if let Some(mv) = self.moves.get(move_index) {
            mv.current_cooldown == 0 && self.mana >= mv.mana_cost
        } else {
            false
        }
    }
}

impl CombatEnemy {
    fn mock() -> Vec<Self> {
        vec![
            Self {
                name: "Gato".to_string(),
                health: 85,
                max_health: 120,
                attack_timer: 3,
                attack_speed: 8,
            },
            Self {
                name: "Nu".to_string(),
                health: 160,
                max_health: 180,
                attack_timer: 0,
                attack_speed: 12,
            },
        ]
    }
}

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

    // For now, use mock data but with real selection state
    let player = CombatPlayer::mock();
    let enemies = CombatEnemy::mock();

    // Ensure selections are within bounds
    let current_move = current_move.min(player.moves.len().saturating_sub(1));
    let current_enemy = current_enemy.min(enemies.len().saturating_sub(1));

    // Victory check
    if enemies.is_empty() || enemies.iter().all(|e| e.health == 0) {
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
    render_player_info(f, chunks[0], &player);

    // Render enemies with real timers
    render_enemies(f, chunks[1], &enemies, current_enemy, &enemy_timers);

    // Debug moves area
    app.core.game.res.log(format!(
        "Moves area: {}x{}",
        chunks[2].width, chunks[2].height
    ));

    // Render moves with player action timer
    render_moves(
        f,
        chunks[2],
        &player,
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

fn render_player_info(f: &mut Frame, area: Rect, player: &CombatPlayer) {
    let health_ratio = player.health as f64 / player.max_health as f64;
    let mana_ratio = player.mana as f64 / player.max_mana as f64;

    let health_bar = Gauge::default()
        .block(Block::default().title("HP").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Red).bg(Color::DarkGray))
        .ratio(health_ratio)
        .label(format!(" {}/{} ", player.health, player.max_health));

    let mana_bar = Gauge::default()
        .block(Block::default().title("MP").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Blue).bg(Color::DarkGray))
        .ratio(mana_ratio)
        .label(format!(" {}/{} ", player.mana, player.max_mana));

    let bars = Layout::horizontal([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)]).split(area);

    f.render_widget(health_bar, bars[0]);
    f.render_widget(mana_bar, bars[1]);
}

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
    player: &CombatPlayer,
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

    let move_blocks = player
        .moves
        .iter()
        .enumerate()
        .map(|(i, mv)| {
            let is_selected = i == current_move;
            let can_use = player.can_use_move(i);
            let cooldown = if mv.current_cooldown > 0 {
                format!(" ({})", mv.current_cooldown)
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
                "{}{}: {}{} - {} MP",
                prefix,
                i + 1,
                move_name,
                cooldown,
                mv.mana_cost
            );

            // Ensure content is never empty
            let final_content = if content.trim().is_empty() {
                format!("{}: Move - 0 MP", i + 1)
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

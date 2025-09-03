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

    // For now, just render mock data - later this will use actual game state
    let player = CombatPlayer::mock();
    let enemies = CombatEnemy::mock();
    let current_move = 0;
    let current_enemy = 0;

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
            Constraint::Length(12), // Moves area
            Constraint::Length(3),  // Message/status area
            Constraint::Length(1),  // Controls
        ])
        .split(inner);

    // Render player info
    render_player_info(f, chunks[0], &player);

    // Render enemies
    render_enemies(f, chunks[1], &enemies, current_enemy);

    // Render moves
    render_moves(f, chunks[2], &player, current_move);

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

fn render_enemies(f: &mut Frame, area: Rect, enemies: &[CombatEnemy], current_enemy: usize) {
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
        render_single_enemy(f, *chunk, enemy, is_selected);
    }
}

fn render_single_enemy(f: &mut Frame, area: Rect, enemy: &CombatEnemy, is_selected: bool) {
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

    // Attack timer
    let attack_timer = if enemy.attack_timer > 0 {
        format!("⏳ {}/{}", enemy.attack_timer, enemy.attack_speed)
    } else {
        "⚡ Ready!".to_string()
    };

    let timer = Paragraph::new(attack_timer)
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center);

    f.render_widget(timer, enemy_layout[2]);
}

fn render_moves(f: &mut Frame, area: Rect, player: &CombatPlayer, current_move: usize) {
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

            let style = if !can_use {
                Style::default().fg(Color::DarkGray)
            } else if is_selected {
                Style::default().fg(Color::Yellow).bold()
            } else {
                Style::default()
            };

            let content = format!("{}: {}{} - {} MP", i + 1, mv.name, cooldown, mv.mana_cost);

            Paragraph::new(content)
                .style(style)
                .block(Block::default().borders(Borders::ALL))
        })
        .collect::<Vec<_>>();

    let move_chunks = Layout::vertical(
        move_blocks
            .iter()
            .map(|_| Constraint::Length(3))
            .collect::<Vec<_>>(),
    )
    .split(area);

    for (i, block) in move_blocks.into_iter().enumerate() {
        if i < move_chunks.len() {
            f.render_widget(block, move_chunks[i]);
        }
    }
}

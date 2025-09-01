use rand::seq::SliceRandom;
use std::collections::VecDeque;
use std::error::Error;
use std::io::{self, Stdout};
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, Borders, Gauge, Paragraph, Wrap},
    Frame, Terminal,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

const PLAYER_MAX_HEALTH: u32 = 200;
const PLAYER_MAX_MANA: u32 = 100;
const MANA_REGEN: u32 = 5;
const TICK_RATE: u64 = 250; // ms

#[derive(Debug, Clone, Copy, PartialEq)]
enum MoveType {
    Escape,
    Fireball,
    Melee,
    Tackle,
}

#[derive(Debug, Clone)]
struct Move {
    name: String,
    move_type: MoveType,
    damage: u32,
    mana_cost: u32,
    cooldown: u32, // in ticks
    current_cooldown: u32,
    effect: Option<Effect>,
    time_cost: u32, // How many ticks this move takes to execute
}

#[derive(Debug, Clone)]
struct Effect {
    duration: u32, // in ticks
    effect_type: EffectType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum EffectType {
    Stun,
}

#[derive(Debug, Clone)]
struct ActiveEffect {
    effect: Effect,
    remaining_ticks: u32,
    target_index: usize,
}

#[derive(Debug, Clone)]
struct Player {
    health: u32,
    max_health: u32,
    mana: u32,
    max_mana: u32,
    moves: Vec<Move>,
}

impl Player {
    fn new() -> Self {
        Self {
            health: PLAYER_MAX_HEALTH,
            max_health: PLAYER_MAX_HEALTH,
            mana: PLAYER_MAX_MANA,
            max_mana: PLAYER_MAX_MANA,
            moves: vec![
                Move {
                    name: "Melee".to_string(),
                    move_type: MoveType::Melee,
                    damage: 10,
                    mana_cost: 0,
                    cooldown: 2, // 2-tick cooldown to prevent spamming
                    current_cooldown: 0,
                    effect: None,
                    time_cost: 3, // Fastest move
                },
                Move {
                    name: "Escape".to_string(),
                    move_type: MoveType::Escape,
                    damage: 0,
                    mana_cost: 20,
                    cooldown: 0,
                    current_cooldown: 0,
                    effect: None,
                    time_cost: 5,
                },
                Move {
                    name: "Fireball (AoE)".to_string(),
                    move_type: MoveType::Fireball,
                    damage: 30,
                    mana_cost: 40,
                    cooldown: 3,
                    current_cooldown: 0,
                    effect: None,
                    time_cost: 8,
                },
                Move {
                    name: "Tackle".to_string(),
                    move_type: MoveType::Tackle,
                    damage: 15,
                    mana_cost: 20,
                    cooldown: 5,
                    current_cooldown: 0,
                    effect: Some(Effect {
                        duration: 3,
                        effect_type: EffectType::Stun,
                    }),
                    time_cost: 6,
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

    fn use_move(&mut self, move_index: usize) -> Option<Move> {
        // Check if we can use the move first
        if !self.can_use_move(move_index) {
            return None;
        }

        // Now we can safely get mutable access since we've done all immutable checks
        if let Some(mv) = self.moves.get_mut(move_index) {
            self.mana = self.mana.saturating_sub(mv.mana_cost);
            mv.current_cooldown = mv.cooldown;
            return Some(mv.clone());
        }
        None
    }

    fn regen(&mut self) {
        self.mana = (self.mana + MANA_REGEN).min(self.max_mana);
        self.health = self.health.min(self.max_health);
    }

    fn update_cooldowns(&mut self) {
        for mv in &mut self.moves {
            if mv.current_cooldown > 0 {
                mv.current_cooldown -= 1;
            }
        }
    }
}

#[derive(Debug, Clone)]
struct Enemy {
    name: String,
    health: u32,
    max_health: u32,
    seed: (f64, f64, f64), // (cx, cy, scale) for Mandelbrot
    effects: Vec<ActiveEffect>,
    is_stunned: bool,
    attack_timer: u32,
    attack_speed: u32, // Lower is faster
    attack_damage: u32,
}

impl Enemy {
    fn is_stunned(&self) -> bool {
        self.is_stunned
    }

    fn add_effect(&mut self, effect: Effect, target_index: usize) {
        let is_stun = effect.effect_type == EffectType::Stun;
        self.effects.push(ActiveEffect {
            effect,
            remaining_ticks: self.effects.last().map_or(0, |e| e.effect.duration),
            target_index,
        });

        if is_stun {
            self.is_stunned = true;
        }
    }

    fn update_effects(&mut self) {
        self.is_stunned = false;
        self.effects.retain_mut(|effect| {
            effect.remaining_ticks = effect.remaining_ticks.saturating_sub(1);
            if effect.remaining_ticks == 0 {
                false
            } else {
                if effect.effect.effect_type == EffectType::Stun {
                    self.is_stunned = true;
                }
                true
            }
        });
    }
}

impl Enemy {
    fn new(
        name: &str,
        max_health: u32,
        attack_speed: u32,
        attack_damage: u32,
        seed: (f64, f64, f64),
    ) -> Self {
        Self {
            name: name.to_string(),
            health: max_health,
            max_health,
            seed,
            effects: Vec::new(),
            is_stunned: false,
            attack_timer: 0,
            attack_speed,
            attack_damage,
        }
    }

    fn render_portrait(&self, width: usize, height: usize) -> String {
        let (cx, cy, scale) = self.seed;
        render_mandelbrot(width, height, cx, cy, scale)
    }

    fn health_percentage(&self) -> u16 {
        ((self.health as f32 / self.max_health as f32) * 100.0) as u16
    }
}

#[derive(Debug)]
enum Action {
    PlayerMove {
        move_index: usize,
        target_index: Option<usize>,
        time_remaining: u32,
    },
    EnemyAttack {
        enemy_index: usize,
        damage: u32,
        time_remaining: u32,
    },
}

struct App {
    player: Player,
    enemies: Vec<Enemy>,
    current_enemy: usize,
    current_move: usize,
    message: Option<(String, Instant)>,
    last_tick: Instant,
    tick_count: u64,
    action_queue: VecDeque<Action>,
    current_action: Option<Action>,
}

impl App {
    fn new() -> Self {
        let now = Instant::now();
        let mut mandel_coords = vec![];
        for _ in 0..10 {
            mandel_coords.push((
                rand::random::<f64>() * 2.0 - 1.0,
                rand::random::<f64>() * 2.0 - 1.0,
                rand::random::<f64>() * 0.5 + 0.5,
            ));
        }

        // Create 1-5 random enemies with different seeds for variety
        let mut enemies = vec![
            Enemy::new("Gato", 120, 8, 5, mandel_coords[0]),
            Enemy::new("Nu", 180, 12, 3, mandel_coords[1]),
            Enemy::new("Retinite", 250, 6, 4, mandel_coords[2]),
            Enemy::new("Rend", 120, 10, 6, mandel_coords[3]),
            Enemy::new("Mete", 120, 8, 5, mandel_coords[4]),
        ];

        // Randomly select 1-5 enemies
        let num_enemies = 1 + rand::random::<usize>() % 5;
        enemies.shuffle(&mut rand::thread_rng());
        enemies.truncate(num_enemies);

        Self {
            player: Player::new(),
            enemies,
            current_enemy: 0,
            current_move: 0,
            message: None,
            last_tick: now,
            tick_count: 0,
            action_queue: VecDeque::new(),
            current_action: None,
        }
    }

    fn next_enemy(&mut self) {
        if self.enemies.len() <= 1 {
            return;
        }
        self.current_enemy = (self.current_enemy + 1) % self.enemies.len();
    }

    fn prev_enemy(&mut self) {
        if self.enemies.len() <= 1 {
            return;
        }
        self.current_enemy = if self.current_enemy == 0 {
            self.enemies.len() - 1
        } else {
            self.current_enemy - 1
        };
    }

    fn next_move(&mut self) {
        if !self.player.moves.is_empty() {
            self.current_move = (self.current_move + 1) % self.player.moves.len();
        }
    }

    fn prev_move(&mut self) {
        if !self.player.moves.is_empty() {
            self.current_move = if self.current_move == 0 {
                self.player.moves.len() - 1
            } else {
                self.current_move - 1
            };
        }
    }

    fn use_current_move(&mut self) -> bool {
        self.queue_player_move(self.current_move)
    }

    fn queue_player_move(&mut self, move_index: usize) -> bool {
        if self.enemies.is_empty() || !self.player.can_use_move(move_index) {
            return false;
        }

        // Get a mutable reference to the move to update cooldown
        if let Some(mv) = self.player.moves.get_mut(move_index) {
            // Deduct mana and set cooldown
            self.player.mana = self.player.mana.saturating_sub(mv.mana_cost);
            mv.current_cooldown = mv.cooldown;

            // Queue the action with appropriate time cost
            let action = match mv.move_type {
                MoveType::Melee => Action::PlayerMove {
                    move_index,
                    target_index: Some(self.current_enemy),
                    time_remaining: mv.time_cost,
                },
                MoveType::Escape => {
                    Action::PlayerMove {
                        move_index,
                        target_index: None, // No target for escape
                        time_remaining: mv.time_cost,
                    }
                }
                MoveType::Fireball | MoveType::Tackle => Action::PlayerMove {
                    move_index,
                    target_index: Some(self.current_enemy),
                    time_remaining: mv.time_cost,
                },
            };

            self.action_queue.push_back(action);
            true
        } else {
            false
        }
    }

    fn queue_enemy_attacks(&mut self) {
        for (i, enemy) in self.enemies.iter_mut().enumerate() {
            if !enemy.is_stunned && enemy.attack_timer == 0 {
                // Queue enemy attack
                self.action_queue.push_back(Action::EnemyAttack {
                    enemy_index: i,
                    damage: enemy.attack_damage,
                    time_remaining: 5, // Base time cost for enemy attacks
                });

                // Reset attack timer with some randomness
                enemy.attack_timer = enemy.attack_speed + (rand::random::<u32>() % 5);
            }
        }
    }

    fn process_action(&mut self) {
        if let Some(action) = self.current_action.take() {
            match action {
                Action::PlayerMove {
                    move_index,
                    target_index,
                    ..
                } => {
                    if let Some(mv) = self.player.moves.get(move_index) {
                        let message = match mv.move_type {
                            MoveType::Melee => {
                                if let Some(enemy_idx) = target_index {
                                    if let Some(enemy) = self.enemies.get_mut(enemy_idx) {
                                        enemy.health = enemy.health.saturating_sub(mv.damage);
                                        Some(format!(
                                            "Melee hits {} for {} damage!",
                                            enemy.name, mv.damage
                                        ))
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            }
                            MoveType::Escape => {
                                self.enemies.clear();
                                Some("You escaped from battle!".to_string())
                            }
                            MoveType::Fireball => {
                                for enemy in &mut self.enemies {
                                    enemy.health = enemy.health.saturating_sub(mv.damage);
                                }
                                Some(format!(
                                    "Fireball hits all enemies for {} damage!",
                                    mv.damage
                                ))
                            }
                            MoveType::Tackle => {
                                if let Some(enemy_idx) = target_index {
                                    if let Some(enemy) = self.enemies.get_mut(enemy_idx) {
                                        enemy.health = enemy.health.saturating_sub(mv.damage);
                                        if let Some(effect) = &mv.effect {
                                            enemy.add_effect(effect.clone(), enemy_idx);
                                        }
                                        Some(format!(
                                            "Tackle hits {} for {} damage and stuns!",
                                            enemy.name, mv.damage
                                        ))
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            }
                        };

                        if let Some(msg) = message {
                            self.message = Some((msg, Instant::now()));
                        }
                    }
                }
                Action::EnemyAttack {
                    enemy_index,
                    damage,
                    ..
                } => {
                    self.player.health = self.player.health.saturating_sub(damage);
                    if let Some(enemy) = self.enemies.get(enemy_index) {
                        self.message = Some((
                            format!("{} attacks for {} damage!", enemy.name, damage),
                            Instant::now(),
                        ));
                    }
                }
            }
        }
    }

    fn update(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_tick).as_millis() >= TICK_RATE as u128 {
            self.tick_count += 1;
            self.last_tick = now;

            // Process current action if any
            if let Some(action) = &mut self.current_action {
                match action {
                    Action::PlayerMove { time_remaining, .. }
                    | Action::EnemyAttack { time_remaining, .. } => {
                        *time_remaining = time_remaining.saturating_sub(1);
                        if *time_remaining == 0 {
                            self.process_action();
                            // After processing an action, check for enemy attacks
                            self.queue_enemy_attacks();
                        }
                    }
                }
            } else if let Some(next_action) = self.action_queue.pop_front() {
                self.current_action = Some(next_action);
            } else {
                // No current action and nothing in queue, handle normal updates
                self.player.update_cooldowns();
                self.player.regen();

                // Update enemy effects
                for enemy in &mut self.enemies {
                    enemy.update_effects();
                }

                // If we get here, both action queue and current action are empty
                // Queue any pending enemy attacks
                self.queue_enemy_attacks();

                // If we still have no actions, reset enemy attack timers to prevent stalling
                if self.action_queue.is_empty() && self.current_action.is_none() {
                    for enemy in &mut self.enemies {
                        if enemy.attack_timer > 0 {
                            enemy.attack_timer -= 1;
                        }
                    }
                }
            }

            // Remove defeated enemies
            self.enemies.retain(|e| e.health > 0);

            // Reset current enemy if needed
            if !self.enemies.is_empty() && self.current_enemy >= self.enemies.len() {
                self.current_enemy = self.enemies.len() - 1;
            }

            // Clear old messages after 2 seconds
            if let Some((_, time)) = &self.message {
                if now.duration_since(*time).as_secs() >= 2 {
                    self.message = None;
                }
            }
        }
    }
}

fn main() -> Result<()> {
    let mut terminal = setup_terminal()?;
    let res = run_app(&mut terminal);
    restore_terminal(terminal)?;
    if let Err(err) = res {
        eprintln!("{err:?}");
    }
    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    let mut app = App::new();

    loop {
        app.update();
        terminal.draw(|f| ui(f, &mut app))?;

        if event::poll(Duration::from_millis(16))? {
            // ~60fps
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    // Allow enemy selection at any time
                    KeyCode::Char('h') | KeyCode::Left | KeyCode::Char('a') => {
                        app.prev_enemy();
                    }
                    KeyCode::Char('l') | KeyCode::Right | KeyCode::Char('d') => {
                        app.next_enemy();
                    }
                    // Allow move selection at any time
                    KeyCode::Char('k') | KeyCode::Up | KeyCode::Char('w') => {
                        app.prev_move();
                    }
                    KeyCode::Char('j') | KeyCode::Down | KeyCode::Char('s') => {
                        app.next_move();
                    }
                    // Queue moves at any time
                    KeyCode::Char(' ') | KeyCode::Enter => {
                        app.use_current_move();
                    }
                    // Allow direct move selection with number keys at any time
                    KeyCode::Char(c @ '1'..='4') => {
                        let move_idx = (c as u8 - b'1') as usize;
                        if move_idx < app.player.moves.len() {
                            app.current_move = move_idx;
                            app.use_current_move();
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(mut terminal: Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

fn render_player_info(f: &mut Frame, area: Rect, app: &App) {
    let player = &app.player;
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

fn render_moves(f: &mut Frame, area: Rect, app: &App) {
    let move_blocks = app
        .player
        .moves
        .iter()
        .enumerate()
        .map(|(i, mv)| {
            let is_selected = i == app.current_move;
            let can_use = app.player.can_use_move(i);
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
        f.render_widget(block, move_chunks[i]);
    }
}

fn render_message(f: &mut Frame, area: Rect, message: &str) {
    let message_para = Paragraph::new(message)
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(message_para, area);
}

fn render_move_queue(f: &mut Frame, area: Rect, app: &App) {
    let current_action = match &app.current_action {
        Some(Action::PlayerMove { move_index, .. }) => {
            if let Some(mv) = app.player.moves.get(*move_index) {
                format!("▶ {}", mv.name)
            } else {
                String::new()
            }
        }
        Some(Action::EnemyAttack { enemy_index, .. }) => {
            if let Some(enemy) = app.enemies.get(*enemy_index) {
                format!("▶ {} attacks!", enemy.name)
            } else {
                String::new()
            }
        }
        None => "Waiting...".to_string(),
    };

    let queued_actions: Vec<String> = app
        .action_queue
        .iter()
        .map(|action| match action {
            Action::PlayerMove { move_index, .. } => {
                if let Some(mv) = app.player.moves.get(*move_index) {
                    format!("• {}", mv.name)
                } else {
                    String::from("• ???")
                }
            }
            Action::EnemyAttack { enemy_index, .. } => {
                if let Some(enemy) = app.enemies.get(*enemy_index) {
                    format!("• {} attacks!", enemy.name)
                } else {
                    String::from("• ???")
                }
            }
        })
        .collect();

    let mut lines = vec![Line::from("Current:".to_string().bold())];
    lines.push(Line::from(current_action));

    if !queued_actions.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from("Queue:".to_string().bold()));
        for action in queued_actions {
            lines.push(Line::from(action));
        }
    }

    let block = Block::default().borders(Borders::ALL).title("Action Queue");

    let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

fn render_enemy_info(f: &mut Frame, enemy: &Enemy, area: Rect, is_selected: bool) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(if is_selected {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Render enemy portrait
    let portrait =
        enemy.render_portrait(inner.width as usize, (inner.height * 2 / 3).min(8) as usize);

    let portrait_block = Block::default()
        .borders(Borders::NONE)
        .style(Style::default().bg(Color::Black));
    let portrait_area = center_rect_exact(
        inner.width.min(20),
        (inner.height * 2 / 3).min(8) + 2,
        inner,
    );

    f.render_widget(portrait_block, portrait_area);
    f.render_widget(
        Paragraph::new(portrait)
            .style(Style::default().fg(Color::Green))
            .alignment(Alignment::Center),
        portrait_area,
    );

    // Render health bar below portrait
    let health_ratio = enemy.health as f64 / enemy.max_health as f64;
    let health_bar = Gauge::default()
        .block(
            Block::default()
                .title(enemy.name.clone())
                .borders(Borders::ALL),
        )
        .gauge_style(Style::default().fg(Color::Red).bg(Color::DarkGray))
        .ratio(health_ratio)
        .label(format!(" {}/{} ", enemy.health, enemy.max_health));

    // Render attack timer
    let attack_timer = if enemy.attack_timer > 0 {
        format!("⏳ {}/{}", enemy.attack_timer, enemy.attack_speed)
    } else {
        "⚡ Ready!".to_string()
    };

    let layout = Layout::vertical([
        Constraint::Length(portrait_area.height + 2), // Portrait with padding
        Constraint::Length(3),                        // Health bar
        Constraint::Length(1),                        // Attack timer
    ])
    .split(inner);

    let timer = Paragraph::new(attack_timer)
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center);

    f.render_widget(health_bar, layout[1]);
    f.render_widget(timer, layout[2]);
}

fn ui(f: &mut Frame, app: &mut App) {
    let size = f.size();

    // Outer frame
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Chrono Trigger-Style Combat ")
        .title_alignment(Alignment::Center);
    let _inner = block.inner(size);
    f.render_widget(block, size);

    // Main layout
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
        .split(f.size());

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Player info
            Constraint::Min(12),    // Enemies
            Constraint::Length(15), // Moves
            Constraint::Length(3),  // Message
            Constraint::Length(1),  // Controls
        ])
        .split(chunks[0]);

    // Victory screen
    if app.enemies.is_empty() {
        let victory = Paragraph::new("Victory!")
            .style(Style::default().fg(Color::Green))
            .alignment(Alignment::Center);
        f.render_widget(victory, chunks[0]);
        return;
    }

    // Render player info and enemies
    render_player_info(f, left_chunks[0], app);

    // Create a row for each enemy
    let enemy_chunks = if !app.enemies.is_empty() {
        let constraints: Vec<Constraint> = (0..app.enemies.len())
            .map(|_| Constraint::Ratio(1, app.enemies.len() as u32))
            .collect();
        Layout::horizontal(constraints)
            .split(left_chunks[1])
            .to_vec()
    } else {
        vec![left_chunks[1]]
    };

    // Render enemies
    for (i, (enemy, area)) in app.enemies.iter().zip(enemy_chunks.iter()).enumerate() {
        let is_selected = i == app.current_enemy;
        render_enemy_info(f, enemy, *area, is_selected);
    }

    // Render moves and message
    render_moves(f, left_chunks[2], app);

    if let Some((msg, _)) = &app.message {
        render_message(f, left_chunks[3], msg);
    }

    // Render the move queue on the right side
    render_move_queue(f, chunks[1], app);

    // Controls help
    let controls = Paragraph::new(
        "[←→] Select Target | [↑↓] Select Move | [1-4] Quick Select | [SPACE] Use Move | [q] Quit",
    )
    .style(Style::default().fg(Color::Gray))
    .alignment(Alignment::Center);
    f.render_widget(controls, left_chunks[4]);
}

// Center a rect of exact size (w,h) inside "area".
fn center_rect_exact(w: u16, h: u16, area: Rect) -> Rect {
    let w = w.min(area.width);
    let h = h.min(area.height);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    Rect {
        x,
        y,
        width: w,
        height: h,
    }
}

// Render Mandelbrot set into an ASCII buffer sized (w x h)
fn render_mandelbrot(w: usize, h: usize, cx: f64, cy: f64, scale: f64) -> String {
    let palette: &[char] = &[' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'];
    let max_iter = 60usize;

    // Map pixel to complex plane
    let aspect = w as f64 / h as f64;
    let (span_x, span_y) = if aspect >= 1.0 {
        (scale * aspect, scale)
    } else {
        (scale, scale / aspect)
    };
    let left = cx - span_x / 2.0;
    let top = cy - span_y / 2.0;

    let mut out = String::with_capacity((w + 1) * h);
    for j in 0..h {
        for i in 0..w {
            let x0 = left + (i as f64 / (w - 1).max(1) as f64) * span_x;
            let y0 = top + (j as f64 / (h - 1).max(1) as f64) * span_y;
            let mut x = 0.0;
            let mut y = 0.0;
            let mut it = 0usize;
            while x * x + y * y <= 4.0 && it < max_iter {
                let xt = x * x - y * y + x0;
                y = 2.0 * x * y + y0;
                x = xt;
                it += 1;
            }
            let idx = if it >= max_iter {
                palette.len() - 1
            } else {
                (it * (palette.len() - 1)) / max_iter
            };
            out.push(palette[idx]);
        }
        if j + 1 < h {
            out.push('\n');
        }
    }
    out
}

// Add this to Cargo.toml:
// [dependencies]
// rand = "0.8.5"

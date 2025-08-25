use std::error::Error;
use std::io::{self, Stdout};
use std::time::{Duration, Instant};
use rand::Rng;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
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
    cooldown: u32,      // in ticks
    current_cooldown: u32,
    effect: Option<Effect>,
}

#[derive(Debug, Clone)]
struct Effect {
    duration: u32,      // in ticks
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
                    cooldown: 0,
                    current_cooldown: 0,
                    effect: None,
                },
                Move {
                    name: "Escape".to_string(),
                    move_type: MoveType::Escape,
                    damage: 0,
                    mana_cost: 20,
                    cooldown: 0,
                    current_cooldown: 0,
                    effect: None,
                },
                Move {
                    name: "Fireball (AoE)".to_string(),
                    move_type: MoveType::Fireball,
                    damage: 30,
                    mana_cost: 40,
                    cooldown: 3,
                    current_cooldown: 0,
                    effect: None,
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
    fn new(name: &str, max_health: u32, attack_speed: u32, attack_damage: u32, seed: (f64, f64, f64)) -> Self {
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

struct App {
    player: Player,
    enemies: Vec<Enemy>,
    current_enemy: usize,
    current_move: usize,
    message: Option<(String, Instant)>,
    last_tick: Instant,
    tick_count: u64,
}

impl App {
    fn new() -> Self {
        let now = Instant::now();
        let mut mandel_coords = vec![];
        for _ in 0..10 {
            mandel_coords.push((
                rand::random::<f64>() * 2.0 - 1.0,
                rand::random::<f64>() * 2.0 - 1.0,
                rand::random::<f64>() * 2.0 + 1.0,
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
        use rand::seq::SliceRandom;
        use rand::thread_rng;
        let mut rng = thread_rng();
        let count = rand::random::<usize>() % 5 + 1;
        enemies.shuffle(&mut rng);
        enemies.truncate(count);

        Self {
            player: Player::new(),
            enemies,
            current_enemy: 0,
            current_move: 0,
            message: None,
            last_tick: now,
            tick_count: 0,
        }
    }

    fn next_enemy(&mut self) {
        if !self.enemies.is_empty() {
            self.current_enemy = (self.current_enemy + 1) % self.enemies.len();
        }
    }

    fn prev_enemy(&mut self) {
        if !self.enemies.is_empty() {
            if self.current_enemy == 0 {
                self.current_enemy = self.enemies.len().saturating_sub(1);
            } else {
                self.current_enemy -= 1;
            }
        }
    }

    fn next_move(&mut self) {
        self.current_move = (self.current_move + 1) % self.player.moves.len();
    }

    fn prev_move(&mut self) {
        if self.current_move == 0 {
            self.current_move = self.player.moves.len().saturating_sub(1);
        } else {
            self.current_move -= 1;
        }
    }

    fn use_current_move(&mut self) {
        if self.enemies.is_empty() {
            return;
        }

        if let Some(mv) = self.player.use_move(self.current_move) {
            let message = match mv.move_type {
                MoveType::Melee => {
                    if let Some(enemy) = self.enemies.get_mut(self.current_enemy) {
                        enemy.health = enemy.health.saturating_sub(mv.damage);
                        Some(format!("Melee hits {} for {} damage!", enemy.name, mv.damage))
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
                    Some(format!("Fireball hits all enemies for {} damage!", mv.damage))
                }
                MoveType::Tackle => {
                    if let Some(enemy) = self.enemies.get_mut(self.current_enemy) {
                        enemy.health = enemy.health.saturating_sub(mv.damage);
                        if let Some(effect) = &mv.effect {
                            enemy.add_effect(effect.clone(), self.current_enemy);
                        }
                        Some(format!("Tackle hits {} for {} damage!", enemy.name, mv.damage))
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

    fn update(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_tick).as_millis() >= TICK_RATE as u128 {
            self.tick_count += 1;
            self.last_tick = now;
            
            // Update cooldowns and regen
            self.player.update_cooldowns();
            self.player.regen();
            
            // Update enemy effects
            for enemy in &mut self.enemies {
                enemy.update_effects();
            }
            
            // Update enemy attack timers
            for enemy in &mut self.enemies {
                if !enemy.is_stunned {
                    if enemy.attack_timer > 0 {
                        enemy.attack_timer -= 1;
                    } else {
                        // Enemy attacks!
                        let damage = enemy.attack_damage;
                        self.player.health = self.player.health.saturating_sub(damage);
                        self.message = Some((
                            format!("{} attacks for {} damage!", enemy.name, damage),
                            Instant::now()
                        ));
                        
                        // Reset attack timer with some randomness
                        enemy.attack_timer = enemy.attack_speed + (rand::random::<u32>() % 5);
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

        if event::poll(Duration::from_millis(16))? { // ~60fps
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('1') => app.current_move = 0,
                    KeyCode::Char('2') => app.current_move = 1,
                    KeyCode::Char('3') => app.current_move = 2,
                    KeyCode::Right | KeyCode::Char('d') => app.next_enemy(),
                    KeyCode::Left | KeyCode::Char('a') => app.prev_enemy(),
                    KeyCode::Down | KeyCode::Char('s') => app.next_move(),
                    KeyCode::Up | KeyCode::Char('w') => app.prev_move(),
                    KeyCode::Char(' ') | KeyCode::Enter => app.use_current_move(),
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

    let bars = Layout::horizontal([
        Constraint::Ratio(1, 2),
        Constraint::Ratio(1, 2),
    ]).split(area);

    f.render_widget(health_bar, bars[0]);
    f.render_widget(mana_bar, bars[1]);
}

fn render_moves(f: &mut Frame, area: Rect, app: &App) {
    let move_blocks = app.player.moves.iter().enumerate().map(|(i, mv)| {
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
        
        let content = format!("{}: {}{} - {} MP", 
            i + 1, 
            mv.name, 
            cooldown,
            mv.mana_cost
        );
        
        Paragraph::new(content)
            .style(style)
            .block(Block::default().borders(Borders::ALL))
    }).collect::<Vec<_>>();
    
    let move_chunks = Layout::vertical(
        move_blocks.iter().map(|_| Constraint::Length(3)).collect::<Vec<_>>()
    ).split(area);
    
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

fn render_enemy_info(f: &mut Frame, enemy: &Enemy, area: Rect, is_selected: bool) {
    let border_style = if is_selected {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style);
    
    let inner_area = block.inner(area);
    f.render_widget(block, area);
    
    // Add attack timer indicator
    let timer_bar = if enemy.attack_timer > 0 {
        format!("Attack in: {}/{}", enemy.attack_timer, enemy.attack_speed)
    } else {
        "Attacking!".to_string()
    };

    let health_label = format!("{} {}/{}", enemy.name, enemy.health, enemy.max_health);
    
    let health_bar = Gauge::default()
        .block(Block::default().title(health_label).borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Red).bg(Color::DarkGray))
        .ratio(enemy.health_percentage() as f64 / 100.0);
        
    let timer_gauge = Gauge::default()
        .block(Block::default().title(timer_bar).borders(Borders::NONE))
        .gauge_style(Style::default().fg(Color::Cyan).bg(Color::DarkGray))
        .ratio(1.0 - (enemy.attack_timer as f64 / enemy.attack_speed as f64).max(0.0).min(1.0));

    let chunks = Layout::vertical([
        Constraint::Length(8), // Portrait
        Constraint::Length(3), // Health bar
        Constraint::Length(2), // Attack timer
    ]).spacing(1)
      .margin(1)
      .split(inner_area);

    let portrait = enemy.render_portrait(12, 8);
    let portrait_para = Paragraph::new(portrait)
        .style(Style::default().fg(Color::White).bg(Color::Black))
        .alignment(Alignment::Center);
    f.render_widget(portrait_para, chunks[0]);

    f.render_widget(health_bar, chunks[1]);
    f.render_widget(timer_gauge, chunks[2]);
}

fn ui(f: &mut Frame, app: &mut App) {
    let size = f.size();

    // Outer frame
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Chrono Trigger-Style Combat ")
        .title_alignment(Alignment::Center);
    let inner = block.inner(size);
    f.render_widget(block, size);

    if app.enemies.is_empty() {
        let victory = Paragraph::new("Victory!")
            .style(Style::default().fg(Color::Green))
            .alignment(Alignment::Center);
        f.render_widget(victory, inner);
        return;
    }

    // Main layout: player info, enemies, moves, message
    let main_chunks = Layout::vertical([
        Constraint::Length(3),  // Player info
        Constraint::Min(12),    // Enemies
        Constraint::Length(12), // Moves
        Constraint::Length(3),  // Message
        Constraint::Length(3),  // Controls
    ]).split(inner);

    // Render player info
    render_player_info(f, main_chunks[0], app);

    // Layout for enemies area
    let enemy_area = main_chunks[1];
    let message_area = main_chunks[3];
    
    // Show message if any
    if let Some((msg, _)) = &app.message {
        render_message(f, message_area, msg);
    }

    // Create a row for each enemy
    let enemy_chunks: Vec<Rect> = if !app.enemies.is_empty() {
        let constraints: Vec<_> = (0..app.enemies.len())
            .map(|_| Constraint::Ratio(1, app.enemies.len() as u32))
            .collect();
        Layout::horizontal(constraints).split(enemy_area).to_vec()
    } else {
        vec![enemy_area]
    };

    // Render each enemy
    for (i, (enemy, area)) in app.enemies.iter().zip(enemy_chunks.iter()).enumerate() {
        let is_selected = i == app.current_enemy;
        render_enemy_info(f, enemy, *area, is_selected);
    }

    // Render moves
    render_moves(f, main_chunks[2], app);

    // Controls help
    let controls = Line::from(vec![
        "W/↑, S/↓: Select Move ".into(),
        "A/←, D/→: Select Enemy ".into(),
        "1,2,3: Quick Select ".into(),
        "Space/Enter: Use Move ".into(),
        "Q: Quit".into(),
    ]);
    let footer = Paragraph::new(controls)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(footer, main_chunks[4]);
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

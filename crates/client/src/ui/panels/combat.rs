//! The combat panel: the robot's parts and energy, a card for each enemy in the fight
//! (portrait, health, distance, ammo and what it is doing), the move list and the action
//! queue.

use lithicrivers_core::combat_state_manager::CombatStateManager;
use lithicrivers_core::components::{Energy, EntityKind, Health, Position, SpriteRef};
use lithicrivers_core::model::body::{Body, BodyPartState, BodyPartType};
use lithicrivers_core::moves::{ActionQueue, CombatAction, Move, QueuedAction};
use lithicrivers_core::systems::ranged::{ammo_left, hit_chance, tile_distance};
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

const BORDER: Color = Color::Rgb(150, 60, 50);
const DIM: Color = Color::DarkGray;

pub fn render_combat_panel(f: &mut Frame, app: &mut crate::App, area: Rect) {
    let outer = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER))
        .title(Line::from(" COMBAT ".bold().fg(Color::Red)))
        .title_alignment(Alignment::Center);
    let inner = outer.inner(area);
    f.render_widget(outer, area);

    let (current_move, current_enemy, scroll_offset) = match &app.combat {
        crate::CombatUiState::Active {
            current_move,
            current_enemy,
            move_scroll_offset,
            ..
        } => (*current_move, *current_enemy, *move_scroll_offset),
        _ => (0, 0, 0),
    };

    let enemies = enemy_cards(app);
    let moves = crate::app::input::combat::current_moves(app);
    let current_move = current_move.min(moves.len().saturating_sub(1));
    let current_enemy = current_enemy.min(enemies.len().saturating_sub(1));
    if let crate::CombatUiState::Active {
        current_enemy: ref mut state_enemy,
        current_move: ref mut state_move,
        ..
    } = app.combat
    {
        *state_enemy = current_enemy;
        *state_move = current_move;
    }

    if enemies.is_empty() {
        let victory = Paragraph::new("Victory!".bold())
            .style(Style::default().fg(Color::Green))
            .alignment(Alignment::Center);
        f.render_widget(victory, inner);
        return;
    }

    let bottom = (moves.len() as u16 + 1).clamp(5, 8);
    let [status, energy, enemies_area, bottom_area, controls] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Min(6),
        Constraint::Length(bottom),
        Constraint::Length(1),
    ])
    .areas(inner);

    render_body(f, status, app);
    render_energy(f, energy, app);
    render_enemies(f, enemies_area, &enemies, current_enemy, app);
    let [moves_area, queue_area] =
        Layout::horizontal([Constraint::Percentage(58), Constraint::Percentage(42)])
            .areas(bottom_area);
    render_moves(f, moves_area, &moves, current_move, scroll_offset, app);
    render_action_queue(f, queue_area, app);
    render_controls(f, controls, enemies.len() > 1, app);
}

/// The robot's parts as a row of small bars: name above, integrity inside.
fn render_body(f: &mut Frame, area: Rect, app: &crate::App) {
    let Some(player) = app.core.game.get_player_entity() else {
        return;
    };
    let Ok(body) = app.core.game.world.get::<&Body>(player) else {
        return;
    };
    let parts = [
        (BodyPartType::Head, "HEAD"),
        (BodyPartType::Torso, "TORSO"),
        (BodyPartType::PowerSource, "CORE"),
        (BodyPartType::LeftArm, "L.ARM"),
        (BodyPartType::RightArm, "R.ARM"),
        (BodyPartType::LeftLeg, "L.LEG"),
        (BodyPartType::RightLeg, "R.LEG"),
    ];
    let columns = Layout::horizontal(vec![Constraint::Ratio(1, parts.len() as u32); parts.len()])
        .spacing(1)
        .split(area);
    for ((part_type, name), column) in parts.iter().zip(columns.iter()) {
        let Some(part) = body.parts.get(part_type) else {
            continue;
        };
        let (ratio, color, label) = match part.state {
            BodyPartState::Missing => (0.0, Color::Gray, "--".to_string()),
            BodyPartState::Damaged => (
                part.integrity as f64 / 100.0,
                if part.integrity == 0 {
                    Color::Red
                } else {
                    Color::Yellow
                },
                part.integrity.to_string(),
            ),
            BodyPartState::Functional => (
                part.integrity as f64 / 100.0,
                Color::Green,
                part.integrity.to_string(),
            ),
            BodyPartState::Enhanced => (
                part.integrity as f64 / 100.0,
                Color::Cyan,
                part.integrity.to_string(),
            ),
        };
        let [name_row, bar_row] =
            Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas(*column);
        f.render_widget(
            Paragraph::new(*name)
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center),
            name_row,
        );
        f.render_widget(
            Gauge::default()
                .gauge_style(Style::default().fg(color).bg(Color::Rgb(40, 40, 40)))
                .ratio(ratio.min(1.0))
                .label(Span::styled(
                    label,
                    Style::default().fg(Color::White).bold(),
                )),
            bar_row,
        );
    }
}

/// Energy, with what queued moves will spend taken off the bar.
fn render_energy(f: &mut Frame, area: Rect, app: &crate::App) {
    let Some(player) = app.core.game.get_player_entity() else {
        return;
    };
    let Ok(energy) = app.core.game.world.get::<&Energy>(player).map(|e| *e) else {
        return;
    };
    let pending = calculate_pending_energy_consumption(app);
    let left = energy.current.saturating_sub(pending);
    let label = if pending > 0 {
        format!(
            "ENERGY {}/{}  (-{} queued)",
            energy.current, energy.max, pending
        )
    } else {
        format!("ENERGY {}/{}", energy.current, energy.max)
    };
    let color = if left == 0 && pending > 0 {
        Color::Red
    } else {
        Color::Yellow
    };
    f.render_widget(
        Gauge::default()
            .gauge_style(Style::default().fg(color).bg(Color::Rgb(40, 40, 40)))
            .ratio((left as f64 / energy.max.max(1) as f64).min(1.0))
            .label(Span::styled(
                label,
                Style::default().fg(Color::White).bold(),
            )),
        area,
    );
}

/// What the combat panel shows of one enemy.
struct EnemyCard {
    entity: hecs::Entity,
    name: String,
    health: Health,
    distance: i64,
    /// Shots left and the chance to hit from here, for enemies that shoot
    gun: Option<(u32, u32)>,
    sprite_ref: Option<SpriteRef>,
}

fn enemy_cards(app: &crate::App) -> Vec<EnemyCard> {
    let world = &app.core.game.world;
    let Some(player_pos) = app
        .core
        .game
        .get_player_entity()
        .and_then(|p| world.get::<&Position>(p).ok().map(|p| *p))
    else {
        return vec![];
    };
    CombatStateManager::enemies(world)
        .into_iter()
        .filter_map(|entity| {
            let pos = *world.get::<&Position>(entity).ok()?;
            let health = *world.get::<&Health>(entity).ok()?;
            let distance = tile_distance(pos, player_pos);
            let gun = world
                .get::<&lithicrivers_core::components::RangedAttack>(entity)
                .ok()
                .map(|g| {
                    (
                        ammo_left(world, entity),
                        hit_chance(g.accuracy_at_1, g.falloff, distance),
                    )
                });
            Some(EnemyCard {
                entity,
                name: enemy_name(app, entity),
                health,
                distance,
                gun,
                sprite_ref: world.get::<&SpriteRef>(entity).ok().map(|s| (*s).clone()),
            })
        })
        .collect()
}

fn render_enemies(
    f: &mut Frame,
    area: Rect,
    enemies: &[EnemyCard],
    current_enemy: usize,
    app: &mut crate::App,
) {
    let shown = enemies.len().min(3);
    let columns = Layout::horizontal(vec![Constraint::Ratio(1, shown as u32); shown]).split(area);
    for (i, (enemy, column)) in enemies.iter().zip(columns.iter()).enumerate() {
        render_enemy(f, *column, enemy, i == current_enemy, app);
    }
}

fn render_enemy(
    f: &mut Frame,
    area: Rect,
    enemy: &EnemyCard,
    selected: bool,
    app: &mut crate::App,
) {
    let (border, title) = if selected {
        (
            Style::default().fg(Color::Green),
            Line::from(vec![
                Span::styled("▶ ", Style::default().fg(Color::Green)),
                Span::styled(enemy.name.clone(), Style::default().fg(Color::Green).bold()),
            ]),
        )
    } else {
        (
            Style::default().fg(DIM),
            Line::from(Span::styled(
                enemy.name.clone(),
                Style::default().fg(Color::Gray),
            )),
        )
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border)
        .title(title);
    let inner = block.inner(area);
    f.render_widget(block, area);

    // Your guns' odds against the selected enemy, under its card
    let aim = if selected { your_aim(app, enemy) } else { None };
    let [portrait_area, health_area, info_area, aim_area] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(aim.is_some() as u16),
    ])
    .areas(inner);
    if let Some(aim) = aim {
        f.render_widget(Paragraph::new(aim).alignment(Alignment::Center), aim_area);
    }

    if let Some(sprite_ref) = &enemy.sprite_ref {
        let sprite = app.core.sprite_loader.load_by_spriteref(sprite_ref);
        let color = crate::sprite_loader::parse_color_string(&sprite.color).unwrap_or(Color::White);
        if let Some(art) = sprite.art12x8_sprites.first() {
            // Drop the art's empty rows, then center what is left in the space there is
            let rows: Vec<&str> = art.lines().collect();
            let first = rows.iter().position(|r| !r.trim().is_empty()).unwrap_or(0);
            let last = rows.iter().rposition(|r| !r.trim().is_empty()).unwrap_or(0);
            let rows = &rows[first..=last.max(first)];
            let fit = rows.len().min(portrait_area.height as usize);
            // Short on room: keep the top, where the head and eyes are
            let lines: Vec<Line> = rows[..fit]
                .iter()
                .map(|r| Line::from(r.to_string()))
                .collect();
            let pad = (portrait_area.height as usize - fit) / 2;
            let portrait = Rect {
                y: portrait_area.y + pad as u16,
                height: fit as u16,
                ..portrait_area
            };
            f.render_widget(
                Paragraph::new(lines)
                    .style(Style::default().fg(color))
                    .alignment(Alignment::Center),
                portrait,
            );
        }
    }

    let ratio = enemy.health.current as f64 / enemy.health.max.max(1) as f64;
    f.render_widget(
        Gauge::default()
            .gauge_style(Style::default().fg(Color::Red).bg(Color::Rgb(40, 40, 40)))
            .ratio(ratio.min(1.0))
            .label(Span::styled(
                format!("{}/{}", enemy.health.current, enemy.health.max),
                Style::default().fg(Color::White).bold(),
            )),
        health_area,
    );

    let tiles = if enemy.distance == 1 {
        "1 tile".to_string()
    } else {
        format!("{} tiles", enemy.distance)
    };
    let mut info = vec![Span::styled(tiles, Style::default().fg(Color::Gray))];
    if let Some((ammo, chance)) = enemy.gun {
        let ammo_style = if ammo == 0 {
            Style::default().fg(DIM)
        } else {
            Style::default().fg(Color::Yellow)
        };
        info.push(Span::styled(" · ", Style::default().fg(DIM)));
        info.push(Span::styled(format!("{ammo} rds"), ammo_style));
        if ammo > 0 {
            info.push(Span::styled(
                format!(" {chance}%"),
                Style::default().fg(Color::Red),
            ));
        }
    }
    if let Some(doing) = enemy_activity(app, enemy.entity) {
        info.push(Span::styled(" · ", Style::default().fg(DIM)));
        info.push(Span::styled(doing, Style::default().fg(Color::Cyan)));
    }
    f.render_widget(
        Paragraph::new(Line::from(info)).alignment(Alignment::Center),
        info_area,
    );
}

/// Each equipped gun's chance to hit `enemy` from here and the rounds carried for it:
/// "Pipe Gun 54% (3)". `None` without an equipped gun.
fn your_aim(app: &crate::App, enemy: &EnemyCard) -> Option<Line<'static>> {
    use lithicrivers_core::firearms;
    let world = &app.core.game.world;
    let player = app.core.game.get_player_entity()?;
    let body = world.get::<&Body>(player).ok()?;
    let inv = world
        .get::<&lithicrivers_core::components::Inventory>(player)
        .ok()?;
    let mut spans = vec![Span::styled("You: ", Style::default().fg(Color::Gray))];
    for arm in firearms::ARMS {
        let Some(gun) = body.weapons.get(&arm).and_then(|g| firearms::gun(*g)) else {
            continue;
        };
        let state = body
            .parts
            .get(&arm)
            .map(|p| p.state)
            .unwrap_or(BodyPartState::Missing);
        if state == BodyPartState::Missing {
            continue;
        }
        if spans.len() > 1 {
            spans.push(Span::styled(" | ", Style::default().fg(DIM)));
        }
        let rounds = inv.count(gun.ammo);
        let name = lithicrivers_core::components::itemkind_name(gun.item);
        let style = if rounds == 0 || enemy.distance > gun.range {
            Style::default().fg(DIM)
        } else {
            Style::default().fg(Color::Green)
        };
        let chance = if enemy.distance > gun.range {
            "out of range".to_string()
        } else {
            format!("{}%", firearms::gun_hit_chance(gun, enemy.distance, state))
        };
        spans.push(Span::styled(format!("{name} {chance} ({rounds})"), style));
    }
    (spans.len() > 1).then(|| Line::from(spans))
}

/// What an enemy is about to do, from its current action: "aims 3t".
fn enemy_activity(app: &crate::App, entity: hecs::Entity) -> Option<String> {
    let queue = app.core.game.world.get::<&ActionQueue>(entity).ok()?;
    let current = queue.current_action.as_ref()?;
    let verb = match current.action {
        CombatAction::EnemyAttack { .. } => "strikes",
        CombatAction::EnemyShoot { .. } => "aims",
        CombatAction::EnemyStep { away: true } => "backs off",
        CombatAction::EnemyStep { away: false } => "advances",
        CombatAction::PlayerMove { .. } => return None,
    };
    Some(format!("{verb} {}t", current.remaining_time_ticks))
}

fn render_moves(
    f: &mut Frame,
    area: Rect,
    moves: &[Move],
    current_move: usize,
    scroll_offset: usize,
    app: &mut crate::App,
) {
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(DIM))
        .title(Line::from("Moves".fg(Color::Gray)));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let energy = app
        .core
        .game
        .get_player_entity()
        .and_then(|p| {
            app.core
                .game
                .world
                .get::<&Energy>(p)
                .ok()
                .map(|e| e.current)
        })
        .unwrap_or(0);
    let available = energy.saturating_sub(calculate_pending_energy_consumption(app));
    let rows = inner.height as usize;
    let scroll_offset = scroll_offset.min(moves.len().saturating_sub(1));
    for (row, (i, mv)) in moves
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(rows)
        .enumerate()
    {
        let usable = available >= mv.energy_cost;
        let selected = i == current_move;
        let marker = if selected { "▶ " } else { "  " };
        let cost = format!("{:>3}E {:>2}t", mv.energy_cost, mv.execution_time_ticks);
        let name_width =
            (inner.width as usize).saturating_sub(marker.chars().count() + cost.len() + 1);
        let mut name = mv.name.clone();
        name.truncate(name_width);
        let style = if !usable {
            Style::default().fg(DIM)
        } else if selected {
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let line = Line::from(vec![
            Span::styled(marker, style),
            Span::styled(format!("{name:<name_width$} "), style),
            Span::styled(
                cost,
                if usable {
                    Style::default().fg(Color::Gray)
                } else {
                    style
                },
            ),
        ]);
        let row_area = Rect {
            y: inner.y + row as u16,
            height: 1,
            ..inner
        };
        f.render_widget(Paragraph::new(line), row_area);
        app.ui.click_targets.push(
            row_area,
            crate::ui::click_targets::ClickTarget::CombatMove(i),
        );
    }
}

/// The player's current and queued moves.
fn render_action_queue(f: &mut Frame, area: Rect, app: &crate::App) {
    let mut lines = Vec::new();
    let queue = app
        .core
        .game
        .get_player_entity()
        .and_then(|p| app.core.game.world.get::<&ActionQueue>(p).ok());
    let queued = queue.as_ref().map(|q| q.actions.len()).unwrap_or(0);
    match queue.as_ref().and_then(|q| q.current_action.as_ref()) {
        Some(current) => lines.push(Line::from(vec![
            Span::styled(
                format!("> {}", action_label(app, current)),
                Style::default().fg(Color::Green),
            ),
            Span::styled(
                format!(" {}t", current.remaining_time_ticks),
                Style::default().fg(Color::Gray),
            ),
        ])),
        None => lines.push(Line::from("> Ready".fg(Color::Yellow))),
    }
    if let Some(queue) = &queue {
        for action in &queue.actions {
            lines.push(Line::from(
                format!("  {}", action_label(app, action)).fg(Color::Gray),
            ));
        }
    }
    let block = Block::default()
        .borders(Borders::TOP | Borders::LEFT)
        .border_style(Style::default().fg(DIM))
        .title(Line::from(
            format!(
                "Action Queue {}/{}",
                queued,
                lithicrivers_core::moves::MAX_QUEUED_ACTIONS
            )
            .fg(Color::Gray),
        ));
    f.render_widget(Paragraph::new(lines).block(block), area);
}

/// A player move with its target: "Melee → Feral dog".
fn action_label(app: &crate::App, action: &QueuedAction) -> String {
    match &action.action {
        CombatAction::PlayerMove {
            move_data,
            target_entity: Some(target),
            ..
        } => format!(
            "{} → {}",
            move_data.move_type.human_name(),
            enemy_name(app, *target)
        ),
        CombatAction::PlayerMove { move_data, .. } => move_data.move_type.human_name().to_string(),
        _ => "Enemy action".to_string(),
    }
}

fn render_controls(f: &mut Frame, area: Rect, many_enemies: bool, app: &crate::App) {
    let key = |category, action| {
        app.core
            .config_manager
            .get_printable_key_for_keybind(category, action)
    };
    let hint = |k: &str, what: &'static str| {
        [
            Span::styled(format!("[{k}]"), Style::default().fg(Color::Yellow)),
            Span::styled(format!(" {what} "), Style::default().fg(Color::Gray)),
        ]
    };
    let mut spans = Vec::new();
    if many_enemies {
        spans.extend(hint("←→", "Target"));
    }
    spans.extend(hint("↑↓", "Move"));
    spans.extend(hint(key("ui", "MENU_ACTIVATE"), "Use"));
    spans.extend(hint(key("combat", "CLEAR_MOVE_QUEUE"), "Clear"));
    spans.extend(hint(key("movement", "WAIT"), "Wait"));
    f.render_widget(
        Paragraph::new(Line::from(spans)).alignment(Alignment::Center),
        area,
    );
}

/// Total energy the player's current and queued moves will spend.
fn calculate_pending_energy_consumption(app: &crate::App) -> u32 {
    let Some(queue) = app
        .core
        .game
        .get_player_entity()
        .and_then(|p| app.core.game.world.get::<&ActionQueue>(p).ok())
    else {
        return 0;
    };
    queue
        .current_action
        .iter()
        .chain(queue.get_queued_actions().iter())
        .map(|a| match &a.action {
            CombatAction::PlayerMove { move_data, .. } => move_data.energy_cost,
            _ => 0,
        })
        .sum()
}

/// What a combatant is, for the combat panel: "Feral dog", "Nuclear slime".
fn enemy_name(app: &crate::App, entity: hecs::Entity) -> String {
    match app.core.game.world.get::<&EntityKind>(entity).map(|k| *k) {
        Ok(EntityKind::FeralDog) => "Feral dog",
        Ok(EntityKind::NuclearSlime) => "Nuclear slime",
        Ok(EntityKind::MiningBot) => "Mining bot",
        Ok(EntityKind::MilitaryAndroid) => "Military android",
        Ok(EntityKind::Sheep) => "Sheep",
        _ => "Enemy",
    }
    .to_string()
}

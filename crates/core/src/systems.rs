use crate::component_access::{ComponentAccess, ComponentUpdate, DamageResult};
use crate::components::{
    BattleDelay, BlocksMovement, Combat, Dead, DogAI, DroppedItem, Energy, EntityKind, FeralDog,
    GameEntity, Inventory, ItemKind, Player, Position, Sheep, SpriteRef, Stunned,
};
use crate::intent::PlayerAction;
use crate::moves::{ActionQueue, CombatAction, Move, MoveType, QueuedAction};
use crate::pathfinding::{DogBehavior, Pathfinder};
use crate::resources::Resources;
use hecs::World;
use tracing::info;

/// Check if an entity is stunned (has BattleDelay component)
fn is_stunned(world: &World, entity: hecs::Entity) -> bool {
    world.get::<&BattleDelay>(entity).is_ok()
}

/// ECS Helper Functions for Systems

/// Get the player entity using proper ECS query
pub fn get_player_entity(world: &World) -> Option<hecs::Entity> {
    world.query::<&Player>().iter().next().map(|(e, _)| e)
}

/// Get player position using ECS query  
pub fn get_player_position(world: &World) -> Option<Position> {
    world
        .query::<(&Player, &Position)>()
        .iter()
        .next()
        .map(|(_, (_, pos))| *pos)
}

/// Check if given entity is the player
fn is_player_entity(world: &World, entity: hecs::Entity) -> bool {
    world.get::<&Player>(entity).is_ok()
}

pub fn move_player_system(world: &mut World, res: &mut Resources) {
    if let Some(action) = res.player_state.intent.take() {
        if let Some(player_e) = get_player_entity(world) {
            match action {
                PlayerAction::Move { dx, dy, dz } => {
                    // Read current position and check for movement blocking
                    let (cx, cy, cz) = if let Ok(pos) = world.get::<&Position>(player_e) {
                        (pos.x, pos.y, pos.z)
                    } else {
                        return; // No position component
                    };

                    let nx = cx + dx;
                    let ny = cy + dy;
                    let nz = cz + dz;

                    // Check horizontal movement blocking
                    let mut blocked = false;
                    if dx != 0 || dy != 0 {
                        let t = res.world_state.world.get_tile_cached(nx, ny, cz);
                        blocked = !t.is_passable();
                        if !blocked {
                            for (_, (_, epos)) in
                                world.query::<(&BlocksMovement, &Position)>().iter()
                            {
                                if epos.x == nx && epos.y == ny && epos.z == cz {
                                    blocked = true;
                                    break;
                                }
                            }
                        }
                        if blocked {
                            info!("Blocked by {:?} at ({}, {})", t, nx, ny);
                            res.player_state.last_blocked_tile = Some((nx, ny));
                        }
                    }

                    // Apply movement if not blocked
                    if !blocked {
                        let new_position = Position {
                            x: nx,
                            y: ny,
                            z: nz,
                        };
                        let mut comp_access = ComponentAccess::new(world, res);
                        if let Err(e) = comp_access.atomic_update(
                            player_e,
                            ComponentUpdate::Move {
                                new_position,
                                remove_combat_delay: false,
                            },
                        ) {
                            tracing::warn!(target: "systems", "Failed to update player position: {}", e);
                        }
                    }
                }
                _ => {
                    // Other actions handled by different systems
                }
            }
        }
    }
}

/// Process mining intent: if the player requested mining, act on current tile.
/// Returns true if mining was successful (e.g. chopped a tree), false otherwise.
pub fn mining_system(world: &mut World, res: &mut Resources) -> bool {
    // Check if there's a mining action in the intent and get coordinates
    let (x, y, z) = match &res.player_state.intent.action {
        Some(PlayerAction::Mine) => {
            // Mine at player's current position
            let Some(player_e) = get_player_entity(world) else {
                return false;
            };
            let Ok(pos) = world.get::<&Position>(player_e) else {
                return false;
            };
            let coords = (pos.x, pos.y, pos.z);
            // End immutable borrow before mutating the world
            drop(pos);
            coords
        }
        Some(PlayerAction::MineAt { x, y, z }) => {
            // Mine at specified coordinates
            (*x, *y, *z)
        }
        _ => {
            return false;
        }
    };
    let t = res.world_state.world.get_tile_cached(x, y, z);
    use crate::tiles::TileKind;
    use rand::Rng;
    match t {
        TileKind::Tree => {
            // Chop tree: convert to Dirt and drop items
            res.world_state
                .world
                .set_tile_cached(x, y, z, TileKind::Dirt);

            let mut rng = rand::thread_rng();
            let wood_qty = rng.gen_range(2..=3);
            let acorn_qty = rng.gen_range(1..=3);

            // Spawn Wood
            world.spawn((
                Position { x, y, z },
                DroppedItem {
                    kind: ItemKind::Log,
                    qty: wood_qty,
                },
                SpriteRef::new("items", "log"),
            ));

            // Spawn Acorns
            world.spawn((
                Position { x, y, z },
                DroppedItem {
                    kind: ItemKind::Acorn,
                    qty: acorn_qty,
                },
                SpriteRef::new("items", "acorn"),
            ));

            res.events.interaction_event(
                format!(
                    "You chop the tree. (+{} Wood, +{} Acorn)",
                    wood_qty, acorn_qty
                ),
                res.time.tick,
            );
            true
        }
        _ => {
            // No-op for other tiles for now
            false
        }
    }
}

/// When the player is on the same tile as any DroppedItem, pick it up into Inventory.
pub fn pickup_system(world: &mut World, res: &mut Resources) {
    let Some(player_e) = get_player_entity(world) else {
        return;
    };

    // If Item Auto Pickup is toggled OFF, skip
    if let Ok(inv) = world.get::<&Inventory>(player_e) {
        if !inv.auto_pickup {
            // info!("ItemAutoPickup is toggled off, skipping pickup system");
            return;
        }
    }

    // Get player position and immediately drop the borrow
    let (px, py, pz) = {
        let Ok(ppos) = world.get::<&Position>(player_e) else {
            return;
        };
        (ppos.x, ppos.y, ppos.z)
    };

    // Collect item entities to pick up
    let mut pickups: Vec<hecs::Entity> = Vec::new();
    let mut items: Vec<DroppedItem> = Vec::new();

    // First pass: collect items and their entities
    for (e, (ipos, di)) in world.query::<(&Position, &DroppedItem)>().iter() {
        if ipos.x == px && ipos.y == py && ipos.z == pz {
            pickups.push(e);
            items.push(*di);
        }
    }

    // Early return if nothing to pick up
    if pickups.is_empty() {
        return;
    }

    // Process inventory updates using safe component access
    let mut comp_access = ComponentAccess::new(world, res);
    if let Ok(()) = comp_access.modify_inventory(player_e, |inv| {
        let mut total = 0u32;
        for di in &items {
            inv.add(di.kind, di.qty);
            total += di.qty;
        }
        if total > 0 {
            // This will be handled outside the closure
        }
    }) {
        let total: u32 = items.iter().map(|di| di.qty).sum();
        if total > 0 {
            res.events
                .interaction_event(format!("Picked up {} items", total), res.time.tick);
        }
    }

    // Remove the picked up items from the world
    for e in pickups {
        let _ = world.despawn(e);
    }
}

// Feral dogs with A* pathfinding that chase the player intelligently
pub fn feral_dog_system(world: &mut World, res: &mut Resources) {
    // Get player position if available
    let player_pos = match get_player_position(world) {
        Some(pos) => pos,
        None => {
            tracing::info!("No player found for dogs to chase");
            return; // No player to chase
        }
    };

    // Create pathfinder with 33% efficiency as requested
    let pathfinder = Pathfinder::new(33);

    // Collect dogs with their positions and AI state to avoid borrowing issues
    let mut dogs = Vec::new();
    for (e, (pos, _)) in world.query::<(&Position, &FeralDog)>().iter() {
        let ai_state = world.get::<&DogAI>(e).ok().map(|ai| *ai);
        dogs.push((e, *pos, ai_state));
    }

    tracing::info!(
        "Found {} dogs to process, player at {:?}",
        dogs.len(),
        player_pos
    );

    // Process each dog
    for (dog_entity, dog_pos, ai_state) in dogs {
        // Skip stunned, dead, or combat-engaged dogs - they cannot move
        if is_stunned(world, dog_entity)
            || world.get::<&Dead>(dog_entity).is_ok()
            || world
                .get::<&Combat>(dog_entity)
                .map(|c| c.triggered)
                .unwrap_or(false)
        {
            continue;
        }

        // Calculate distance to player
        let dx = player_pos.x - dog_pos.x;
        let dy = player_pos.y - dog_pos.y;
        let distance_sq = dx * dx + dy * dy;

        // Only chase if player is within 15 tiles (increased range for smarter dogs)
        if distance_sq > 225 {
            // 15^2
            tracing::info!(
                "Dog at {:?} too far from player (distance_sq: {})",
                dog_pos,
                distance_sq
            );
            continue;
        }

        tracing::info!(
            "Processing dog at {:?}, distance_sq: {}",
            dog_pos,
            distance_sq
        );

        // Initialize or get AI state
        let mut current_ai = ai_state.unwrap_or(DogAI {
            behavior: DogBehavior::Hunting,
            behavior_timer: 20, // Start with 20 tick behavior
            circle_center: None,
            steps_taken: 0,
        });

        // Update behavior timer
        current_ai.behavior_timer = current_ai.behavior_timer.saturating_sub(1);

        // Check if we should change behavior (timer expired or too many steps taken)
        if current_ai.behavior_timer == 0 || current_ai.steps_taken >= 12 {
            tracing::info!(
                "Dog behavior change: timer={}, steps_taken={}, old_behavior={:?}",
                current_ai.behavior_timer,
                current_ai.steps_taken,
                current_ai.behavior
            );
            current_ai.behavior = DogBehavior::next_behavior(
                current_ai.behavior,
                res.world_state.seed,
                res.time.tick,
                dog_pos,
            );
            // Set new behavior timer and reset steps
            current_ai.behavior_timer = match current_ai.behavior {
                DogBehavior::Hunting => 15 + ((res.time.tick + dog_pos.x as u64) % 10),
                DogBehavior::Circling => 8 + ((res.time.tick + dog_pos.y as u64) % 5),
                DogBehavior::Wandering => 5 + ((res.time.tick + dog_pos.x as u64) % 8),
            };
            current_ai.steps_taken = 0; // Reset step counter
        }

        // Rarely skip movement (5% chance for more active dogs)
        if ((res.world_state.seed + res.time.tick + dog_pos.x as u64 + dog_pos.y as u64) % 20) == 0
        {
            // Update AI state and continue
            if world.get::<&DogAI>(dog_entity).is_err() {
                world.insert_one(dog_entity, current_ai).ok();
            } else {
                if let Ok(mut ai) = world.get::<&mut DogAI>(dog_entity) {
                    *ai = current_ai;
                }
            }
            continue;
        }

        // Create passability checker
        let is_passable = |pos: Position| -> bool {
            // Check terrain
            if !res.world_state.world.is_passable(pos.x, pos.y, pos.z) {
                tracing::info!(
                    "Position {:?} blocked by terrain (tile: {:?})",
                    pos,
                    res.world_state.world.get_tile(pos.x, pos.y, pos.z)
                );
                return false;
            }

            // Check for blocking entities
            for (entity, (other_pos, _)) in world.query::<(&Position, &BlocksMovement)>().iter() {
                if *other_pos == pos {
                    tracing::info!(
                        "Position {:?} blocked by entity {:?} at same position",
                        pos,
                        entity
                    );
                    return false;
                }
            }
            true
        };

        // Determine next move based on behavior
        let next_pos = match current_ai.behavior {
            DogBehavior::Hunting => {
                // Use A* pathfinding to get adjacent to player (not on top of player)
                // Find the closest adjacent position to the player that's passable
                let adjacent_positions = [
                    Position {
                        x: player_pos.x - 1,
                        y: player_pos.y - 1,
                        z: player_pos.z,
                    },
                    Position {
                        x: player_pos.x - 1,
                        y: player_pos.y,
                        z: player_pos.z,
                    },
                    Position {
                        x: player_pos.x - 1,
                        y: player_pos.y + 1,
                        z: player_pos.z,
                    },
                    Position {
                        x: player_pos.x,
                        y: player_pos.y - 1,
                        z: player_pos.z,
                    },
                    Position {
                        x: player_pos.x,
                        y: player_pos.y + 1,
                        z: player_pos.z,
                    },
                    Position {
                        x: player_pos.x + 1,
                        y: player_pos.y - 1,
                        z: player_pos.z,
                    },
                    Position {
                        x: player_pos.x + 1,
                        y: player_pos.y,
                        z: player_pos.z,
                    },
                    Position {
                        x: player_pos.x + 1,
                        y: player_pos.y + 1,
                        z: player_pos.z,
                    },
                ];

                // Find the closest adjacent position to the dog
                let mut best_target = None;
                let mut best_distance = i32::MAX;

                for &adj_pos in &adjacent_positions {
                    if is_passable(adj_pos) {
                        let distance =
                            (adj_pos.x - dog_pos.x).abs() + (adj_pos.y - dog_pos.y).abs();
                        if distance < best_distance {
                            best_distance = distance;
                            best_target = Some(adj_pos);
                        }
                    }
                }

                if let Some(target) = best_target {
                    tracing::info!(
                        "Dog at {:?} hunting toward adjacent target {:?} near player {:?}",
                        dog_pos,
                        target,
                        player_pos
                    );
                    pathfinder.find_next_step(
                        dog_pos,
                        target,
                        is_passable,
                        res.world_state.seed,
                        res.time.tick,
                    )
                } else {
                    tracing::info!("Dog at {:?} no adjacent targets passable, trying to reach player {:?} directly", dog_pos, player_pos);
                    // No adjacent position is passable, fall back to trying to reach player directly
                    pathfinder.find_next_step(
                        dog_pos,
                        player_pos,
                        is_passable,
                        res.world_state.seed,
                        res.time.tick,
                    )
                }
            }
            DogBehavior::Circling => {
                // Set circle center if not set
                if current_ai.circle_center.is_none() {
                    current_ai.circle_center = Some(player_pos);
                }

                if let Some(center) = current_ai.circle_center {
                    // Generate circular movement around the center
                    let target = pathfinder.circular_move(
                        dog_pos,
                        center,
                        res.world_state.seed,
                        res.time.tick,
                    );
                    if let Some(target_pos) = target {
                        // Try to path to the circular target
                        pathfinder.find_next_step(
                            dog_pos,
                            target_pos,
                            is_passable,
                            res.world_state.seed,
                            res.time.tick,
                        )
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            DogBehavior::Wandering => {
                // Just wander randomly
                let directions = [
                    (-1, -1),
                    (-1, 0),
                    (-1, 1),
                    (0, -1),
                    (0, 1),
                    (1, -1),
                    (1, 0),
                    (1, 1),
                ];
                let random_index = ((res.world_state.seed + res.time.tick + dog_pos.x as u64)
                    % directions.len() as u64) as usize;
                let (dx, dy) = directions[random_index];

                let wander_pos = Position {
                    x: dog_pos.x + dx,
                    y: dog_pos.y + dy,
                    z: dog_pos.z,
                };

                if is_passable(wander_pos) {
                    Some(wander_pos)
                } else {
                    None
                }
            }
        };

        // Move the dog if we have a valid next position
        if let Some(new_pos) = next_pos {
            tracing::info!(
                "Moving dog from {:?} to {:?} (behavior: {:?}, steps_taken: {})",
                dog_pos,
                new_pos,
                current_ai.behavior,
                current_ai.steps_taken
            );
            if let Ok(mut pos) = world.get::<&mut Position>(dog_entity) {
                *pos = new_pos;
                current_ai.steps_taken += 1; // Increment step counter after successful move
            }
        } else {
            tracing::info!(
                "Dog at {:?} found no valid move (behavior: {:?}, steps_taken: {})",
                dog_pos,
                current_ai.behavior,
                current_ai.steps_taken
            );
        }

        // Update or insert AI state
        if world.get::<&DogAI>(dog_entity).is_err() {
            world.insert_one(dog_entity, current_ai).ok();
        } else {
            if let Ok(mut ai) = world.get::<&mut DogAI>(dog_entity) {
                *ai = current_ai;
            }
        }
    }
}

/// Very simple deterministic-ish wander: each sheep picks a random 4-neighbor step.
/// Uses the global RNG for determinism relative to the initial seed and tick order.
pub fn stumbling_sheep_system(world: &mut World, res: &mut Resources) {
    use rand::Rng;
    // Collect sheep entities to avoid borrow conflicts and to impose a stable order
    let mut entities: Vec<(hecs::Entity, Position)> = Vec::new();
    for (e, (pos, _s)) in world.query::<(&Position, &Sheep)>().iter() {
        entities.push((e, *pos));
    }
    // Sort by position then entity bits for a stable iteration order
    entities.sort_by_key(|(e, p)| (p.x, p.y, p.z, e.to_bits()));
    for (e, pos) in entities {
        // Skip stunned sheep - they cannot move
        if is_stunned(world, e) {
            continue;
        }

        // 25% chance to stay, else pick one of 4 directions
        let r: u32 = res.world_state.rng.gen_range(0..5);
        let (dx, dy) = match r {
            0 => (0, 0),
            1 => (1, 0),
            2 => (-1, 0),
            3 => (0, 1),
            _ => (0, -1),
        };
        if dx == 0 && dy == 0 {
            continue;
        }
        let nx = pos.x + dx;
        let ny = pos.y + dy;
        let nz = pos.z; // Sheep stay on the same Z-level

        // Check if the target tile is passable
        let t = res.world_state.world.get_tile(nx, ny, nz);
        if !t.is_passable() {
            continue;
        }

        // Avoid stepping into another blocking entity
        let mut occupied = false;
        for (other_e, (other_pos, _)) in world.query::<(&Position, &BlocksMovement)>().iter() {
            if other_e != e && other_pos.x == nx && other_pos.y == ny && other_pos.z == nz {
                occupied = true;
                break;
            }
        }
        if occupied {
            continue;
        }
        if let Ok(mut mypos) = world.get::<&mut Position>(e) {
            mypos.x = nx;
            mypos.y = ny;
        }
    }
}

/// System to count down battle delay timers and remove expired ones
pub fn battle_delay_timer_system(world: &mut World, _res: &mut Resources) {
    let mut entities_to_remove_delay: Vec<hecs::Entity> = Vec::new();

    // Collect entities with BattleDelay and GameEntity marker, and decrement their timers
    for (e, (delay, _)) in world.query::<(&mut BattleDelay, &GameEntity)>().iter() {
        if delay.remaining_ticks > 0 {
            delay.remaining_ticks -= 1;
        }

        if delay.remaining_ticks == 0 {
            entities_to_remove_delay.push(e);
        }
    }

    // Remove expired BattleDelay components
    for e in entities_to_remove_delay {
        let _ = world.remove_one::<BattleDelay>(e);
    }
}

/// System to process action queues and execute completed actions
pub fn action_queue_system(world: &mut World, res: &mut Resources) {
    const DELTA_TICKS: u64 = 1; // Each game tick advances by 1

    let mut entities_with_queues = Vec::new();

    // Collect entities that have action queues
    for (entity, _queue) in world.query::<&mut ActionQueue>().iter() {
        entities_with_queues.push(entity);
    }

    // Phase 1: Update all timers first (no entity state changes)
    for entity in &entities_with_queues {
        if let Ok(mut queue) = world.get::<&mut ActionQueue>(*entity) {
            queue.advance_timers(DELTA_TICKS);
        }
    }

    // Phase 2: Collect all completed actions
    let mut completed_actions = Vec::new();
    for entity in &entities_with_queues {
        if let Ok(mut queue) = world.get::<&mut ActionQueue>(*entity) {
            if let Some(completed_action) = queue.pop_completed_action() {
                completed_actions.push(completed_action);
            }
        }
    }

    // Phase 3: Execute all completed actions (entities may be killed here)
    for action in completed_actions {
        execute_combat_action(world, res, &action);
    }

    // Check if combat should end and clear queues if needed
    check_combat_end_conditions(world, res);
}

/// Check for combat end conditions and clear action queues if combat ends
fn check_combat_end_conditions(world: &mut World, res: &mut Resources) {
    // Check if player is dead
    let player_dead = if let Some(player_entity) = get_player_entity(world) {
        world.get::<&Dead>(player_entity).is_ok()
    } else {
        true // No player entity means dead
    };

    // Check if all enemies are dead
    let mut has_living_enemies = false;
    for (entity, (_, combat, _)) in world.query::<(&Position, &Combat, &GameEntity)>().iter() {
        if !is_player_entity(world, entity) && combat.triggered {
            if world.get::<&Dead>(entity).is_err() {
                has_living_enemies = true;
                break;
            }
        }
    }

    // Check if combat should end and handle the end conditions
    let combat_should_end = player_dead || !has_living_enemies;

    if combat_should_end && res.player_state.combat_active {
        // Only end combat if it was actually active
        res.player_state.combat_active = false;
        res.player_state.last_combat_end_tick = res.time.tick;
        res.player_state.combat_ended_this_tick = true;

        if player_dead {
            res.events.combat_event(
                "Combat ended: Player defeated - all action queues cleared",
                res.time.tick,
            );
        } else if !has_living_enemies {
            res.events.combat_event(
                "Combat ended: All enemies defeated - all action queues cleared",
                res.time.tick,
            );
        }

        // Clear all action queues
        let mut entities_to_clear = Vec::new();
        for (entity, _) in world.query::<&ActionQueue>().iter() {
            entities_to_clear.push(entity);
        }

        for entity in entities_to_clear {
            if let Ok(mut queue) = world.get::<&mut ActionQueue>(entity) {
                queue.clear();
            }
        }

        // TODO: Implement delayed entity cleanup - don't remove dead entities immediately
        // This allows UI and tests to see dead entities before they're cleaned up
        // let mut dead_entities = Vec::new();
        // for (entity, _) in world.query::<&Dead>().iter() {
        //     // Don't remove the player entity - let the game handle player death
        //     if !is_player_entity(world, entity) {
        //         dead_entities.push(entity);
        //     }
        // }
        //
        // // Remove dead entities from the world
        // for entity in dead_entities {
        //     if let Err(e) = world.despawn(entity) {
        //         tracing::warn!(target: "combat", "Failed to despawn dead entity {:?}: {}", entity, e);
        //     } else {
        //         tracing::debug!(target: "combat", "Removed dead entity {:?} from world", entity);
        //     }
        // }
    } else {
        // Reset combat end flag if combat is still active
        res.player_state.combat_ended_this_tick = false;
    }
}

/// Execute a completed combat action
fn execute_combat_action(world: &mut World, res: &mut Resources, action: &QueuedAction) {
    match &action.action {
        CombatAction::PlayerMove {
            move_data,
            target_entity,
            target_position,
        } => {
            // Validate target is still alive if action requires a target
            if let Some(target) = target_entity {
                if world.get::<&Dead>(*target).is_ok() {
                    res.events.combat_event(
                        "Target is already dead, move wasted!".to_string(),
                        res.time.tick,
                    );
                    return;
                }
            }

            execute_player_move(
                world,
                res,
                action.entity,
                move_data,
                *target_entity,
                *target_position,
            );
        }
        CombatAction::EnemyAttack {
            target_entity,
            damage,
        } => {
            // Validate target is still alive before attacking
            if world.get::<&Dead>(*target_entity).is_ok() {
                res.events.combat_event(
                    "Enemy attacks a dead target, attack wasted!".to_string(),
                    res.time.tick,
                );
                return;
            }

            execute_enemy_attack(world, res, action.entity, *target_entity, *damage);
        }
    }
}

/// Execute a player move action
fn execute_player_move(
    world: &mut World,
    res: &mut Resources,
    player_entity: hecs::Entity,
    move_data: &Move,
    target_entity: Option<hecs::Entity>,
    _target_position: Option<Position>,
) {
    use rand::Rng;

    // Get player position
    let player_pos = match world.get::<&Position>(player_entity) {
        Ok(pos) => *pos,
        Err(_) => return,
    };

    // Update player components
    let energy_consumed = {
        if let Ok(mut energy) = world.get::<&mut Energy>(player_entity) {
            if energy.consume(move_data.energy_cost) {
                true
            } else {
                res.events.combat_event("Not enough energy!", res.time.tick);
                false
            }
        } else {
            false
        }
    };

    if !energy_consumed {
        return;
    }

    // Note: Cooldowns are now handled by the action queue system through execution_time_ticks

    // Execute move effects based on type
    match move_data.move_type {
        MoveType::Melee => {
            if let Some(target) = target_entity {
                apply_damage(world, res, target, move_data.damage, "melee attack");
            }
        }
        MoveType::Fireball => {
            if let Some(target) = target_entity {
                apply_damage(world, res, target, move_data.damage, "fireball");
                //TODO: Get all entities in a radius and apply splash damage
                let target_xyz = if let Ok(pos) = world.get::<&Position>(target) {
                    *pos
                } else {
                    player_pos // Fallback to player position if we can't get target position
                };
                let splash_radius = move_data
                    .splash_radius
                    .unwrap_or(panic!("Fireball missing splash radius"));
                if splash_radius > 0 {
                    for (entity, (pos, combat, _)) in
                        world.query::<(&Position, &Combat, &GameEntity)>().iter()
                    {
                        if entity != target && entity != player_entity && combat.triggered {
                            // Check if enemy is dead (skip dead enemies)
                            if world.get::<&Dead>(entity).is_ok() {
                                continue;
                            }

                            let dx = pos.x - target_xyz.x;
                            let dy = pos.y - target_xyz.y;
                            let distance_sq = dx * dx + dy * dy;

                            if distance_sq <= (splash_radius * splash_radius) as i32 {
                                apply_damage(
                                    world,
                                    res,
                                    entity,
                                    move_data.damage,
                                    "fireball splash",
                                );
                            }
                        }
                    }
                }
            }
        }
        MoveType::Tackle => {
            if let Some(target) = target_entity {
                apply_damage(world, res, target, move_data.damage, "tackle");

                // Store original enemy position and push enemy back 2 spaces
                let original_enemy_pos =
                    if let Ok(mut target_pos) = world.get::<&mut Position>(target) {
                        let original_pos = *target_pos;
                        let pushed_pos =
                            crate::moves::calculate_push_position(player_pos, original_pos, 2);
                        target_pos.x = pushed_pos.x;
                        target_pos.y = pushed_pos.y;
                        target_pos.z = pushed_pos.z;
                        original_pos
                    } else {
                        player_pos // Fallback to player position if we can't get target position
                    };

                // Move player into enemy's original space
                if let Ok(mut player_position) = world.get::<&mut Position>(player_entity) {
                    player_position.x = original_enemy_pos.x;
                    player_position.y = original_enemy_pos.y;
                    player_position.z = original_enemy_pos.z;
                    res.log(format!(
                        "You tackle and move to ({}, {})",
                        original_enemy_pos.x, original_enemy_pos.y
                    ));
                }

                // 50% chance to stun
                if res.world_state.rng.gen_bool(0.5) {
                    world.insert_one(target, Stunned::new(600)).ok();
                    res.log("Target is stunned!".to_string());
                }

                // Check if there are any enemies adjacent to the player after the tackle
                let new_player_pos = original_enemy_pos; // Player is now at enemy's original position
                let mut has_adjacent_enemies = false;

                for (entity, (pos, combat, _)) in
                    world.query::<(&Position, &Combat, &GameEntity)>().iter()
                {
                    if entity != player_entity && combat.triggered {
                        // Check if enemy is dead (skip dead enemies)
                        if world.get::<&Dead>(entity).is_ok() {
                            continue;
                        }

                        let dx = (pos.x - new_player_pos.x).abs();
                        let dy = (pos.y - new_player_pos.y).abs();
                        let distance_sq = dx * dx + dy * dy;

                        if distance_sq <= 1 {
                            // Adjacent (including diagonally)
                            has_adjacent_enemies = true;
                            break;
                        }
                    }
                }

                // If no enemies are adjacent after tackle, end combat
                if !has_adjacent_enemies {
                    res.log("No enemies adjacent after tackle - combat ends!".to_string());
                    end_combat_for_nearby_enemies(world, new_player_pos);

                    // Clear the player's action queue since combat is ending
                    if let Ok(mut queue) = world.get::<&mut ActionQueue>(player_entity) {
                        queue.clear();
                        res.log("Action queue cleared after tackle combat end");
                    }

                    // Set combat end flag
                    res.player_state.combat_active = false;
                    res.player_state.combat_ended_this_tick = true;
                    res.player_state.last_combat_end_tick = res.time.tick;
                }
            }
        }
        MoveType::Escape => {
            // Add battle delay to player to prevent immediate re-engagement
            world.insert_one(player_entity, BattleDelay::new(1000)).ok();
            res.log("You attempt to escape from combat!".to_string());

            // End combat for all nearby enemies
            end_combat_for_nearby_enemies(world, player_pos);

            // Use centralized combat state manager to end combat
            crate::combat_state_manager::CombatStateManager::force_end_combat(world, res);
        }
        MoveType::DebugInstantKill => {
            if let Some(target) = target_entity {
                // Instantly kill the target for debugging purposes
                handle_entity_death(world, res, target);
                res.log("DEBUG: Target instantly killed!".to_string());
            }
        }
        MoveType::Heal => {
            // TODO: Implement healing logic
            res.log("Heal move used (not yet implemented)".to_string());
        }
        MoveType::Shield => {
            // TODO: Implement shield/block logic
            res.log("Shield move used (not yet implemented)".to_string());
        }
        MoveType::LightningBolt => {
            if let Some(target) = target_entity {
                apply_damage(world, res, target, move_data.damage, "lightning bolt");
            }
        }
        MoveType::PowerStrike => {
            if let Some(target) = target_entity {
                apply_damage(world, res, target, move_data.damage, "power strike");
            }
        }
    }

    res.log(format!("Used {}", move_data.name));
}

/// Execute an enemy attack action
fn execute_enemy_attack(
    world: &mut World,
    res: &mut Resources,
    _attacker_entity: hecs::Entity,
    target_entity: hecs::Entity,
    damage: u32,
) {
    apply_damage(world, res, target_entity, damage, "enemy attack");
}

/// Centralized function to clean up all action queues targeting a dead entity
/// OPTIMIZED: O(1) lookup using target tracker, with O(n) fallback for robustness
pub fn cleanup_actions_targeting_dead_entity(
    world: &mut World,
    res: &mut Resources,
    dead_entity: hecs::Entity,
) {
    // Fast O(1) lookup: find all entities that have actions targeting the dead entity
    let entities_targeting_dead = res.target_tracker.get_entities_targeting(dead_entity);

    if !entities_targeting_dead.is_empty() {
        // Optimized path: only check entities we know are targeting the dead entity
        for queue_entity in entities_targeting_dead {
            if let Ok(mut queue) = world.get::<&mut ActionQueue>(queue_entity) {
                queue.remove_actions_targeting(dead_entity);
                res.target_tracker
                    .remove_targeting(queue_entity, dead_entity);
            }
        }
    } else {
        // Fallback path: target tracker might be empty (e.g., in tests or during transition)
        // Fall back to scanning all entities - still works but slower
        let mut entities_with_queues = Vec::new();
        for (queue_entity, _) in world.query::<&ActionQueue>().iter() {
            entities_with_queues.push(queue_entity);
        }

        for queue_entity in entities_with_queues {
            if let Ok(mut queue) = world.get::<&mut ActionQueue>(queue_entity) {
                queue.remove_actions_targeting(dead_entity);
            }
        }
    }

    // Clear any action queues on the dead entity itself and remove from tracker
    if let Ok(mut queue) = world.get::<&mut ActionQueue>(dead_entity) {
        queue.clear_with_tracker(dead_entity, &mut res.target_tracker);
    }
}

/// Apply damage to an entity using safe component access
fn apply_damage(
    world: &mut World,
    res: &mut Resources,
    target_entity: hecs::Entity,
    damage: u32,
    source: &str,
) {
    let mut comp_access = ComponentAccess::new(world, res);

    match comp_access.apply_damage(target_entity, damage) {
        Ok(DamageResult::Damaged {
            old_health,
            new_health,
            damage_applied,
        }) => {
            res.events.combat_event(
                format!(
                    "Target takes {} damage from {} ({}/{} HP)",
                    damage_applied, source, new_health, old_health
                ),
                res.time.tick,
            );
        }
        Ok(DamageResult::Killed {
            old_health,
            damage_applied,
        }) => {
            res.events.combat_event(
                format!(
                    "Target takes {} damage from {} and dies! ({} HP)",
                    damage_applied, source, old_health
                ),
                res.time.tick,
            );
            // Additional death handling is done by ComponentAccess
        }
        Ok(DamageResult::AlreadyDead) => {
            tracing::debug!(target: "combat", "Attempted to damage already dead entity {:?}", target_entity);
        }
        Ok(DamageResult::NoDamageSystem) => {
            tracing::warn!(target: "combat", "Entity {:?} has no damage system (Health or Body)", target_entity);
        }
        Err(e) => {
            tracing::error!(target: "combat", "Failed to apply damage to {:?}: {}", target_entity, e);
        }
    }
}

/// Handle entity death - add Dead marker and clean up
fn handle_entity_death(world: &mut World, res: &mut Resources, entity: hecs::Entity) {
    // Check if it's a feral dog and get its position for corpse spawning
    let is_feral_dog = world.get::<&FeralDog>(entity).is_ok();
    let entity_pos = world.get::<&Position>(entity).ok().map(|p| *p);
    let is_player = is_player_entity(world, entity);

    // If it's a feral dog, collect inventory items before entity cleanup
    let inventory_items = if is_feral_dog {
        if let Ok(inv) = world.get::<&Inventory>(entity) {
            inv.slots.clone()
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    // Add Dead marker
    world.insert_one(entity, Dead).ok();

    // Use centralized cleanup function
    cleanup_actions_targeting_dead_entity(world, res, entity);

    // Remove combat capability
    world.remove_one::<Combat>(entity).ok();

    // For FeralDog entities, create a corpse with inventory and remove the original entity
    if is_feral_dog {
        if let Some(pos) = entity_pos {
            // Create corpse inventory with the items
            let mut corpse_inventory = Inventory::default();
            for item_stack in inventory_items {
                corpse_inventory.add(item_stack.kind, item_stack.qty);
            }

            // Create a corpse entity
            world.spawn((
                pos,
                GameEntity,
                EntityKind::Corpse,
                SpriteRef::new("entities", "corpse"),
                corpse_inventory,
                // Note: Corpses don't block movement and aren't combatants
            ));
            res.log("A corpse remains...".to_string());

            // Remove the original FeralDog entity since it's been replaced by a corpse
            if let Err(e) = world.despawn(entity) {
                tracing::warn!(target: "combat", "Failed to despawn dead FeralDog entity {:?}: {}", entity, e);
            }
        }
    }

    // Log death
    if is_player {
        res.log("You have died!".to_string());
        // TODO: Trigger game over state
    } else {
        res.log("Enemy defeated!".to_string());
    }
}

/// End combat for all enemies near the given position
pub fn end_combat_for_nearby_enemies(world: &mut World, center_pos: Position) {
    let mut entities_to_update = Vec::new();

    for (entity, (pos, combat, _)) in world.query::<(&Position, &Combat, &GameEntity)>().iter() {
        let dx = center_pos.x - pos.x;
        let dy = center_pos.y - pos.y;
        let distance_sq = dx * dx + dy * dy;

        // End combat for enemies within 3 tiles
        if distance_sq <= 9 && combat.triggered {
            entities_to_update.push(entity);
        }
    }

    for entity in entities_to_update {
        if let Ok(mut combat) = world.get::<&mut Combat>(entity) {
            combat.triggered = false;
        }
    }
}

/// System to automatically generate enemy actions during combat
pub fn enemy_combat_ai_system(world: &mut World, res: &mut Resources) {
    // Find all combat entities that should act
    let mut enemy_entities = Vec::new();

    for (entity, (_, combat, _)) in world.query::<(&Position, &Combat, &GameEntity)>().iter() {
        if combat.triggered && !is_player_entity(world, entity) {
            // Skip if entity is dead, stunned, or already has actions queued
            if world.get::<&Dead>(entity).is_ok() || world.get::<&Stunned>(entity).is_ok() {
                continue;
            }

            // Check if entity has an empty or no action queue
            let needs_action = match world.get::<&ActionQueue>(entity) {
                Ok(queue) => queue.current_action.is_none() && queue.actions.is_empty(),
                Err(_) => true, // No queue component means it needs one
            };

            if needs_action {
                enemy_entities.push(entity);
            }
        }
    }

    // Generate actions for each enemy that needs them
    for enemy_entity in enemy_entities {
        generate_enemy_action(world, res, enemy_entity);
    }
}

/// Generate an action for an enemy entity
fn generate_enemy_action(world: &mut World, res: &mut Resources, enemy_entity: hecs::Entity) {
    use rand::Rng;

    // Ensure enemy has an action queue
    if world.get::<&ActionQueue>(enemy_entity).is_err() {
        world.insert_one(enemy_entity, ActionQueue::new()).ok();
    }

    // Find the player
    let player_entity = match get_player_entity(world) {
        Some(player) => player,
        None => return,
    };

    // Simple AI: attack the player with random damage and timing
    let damage = res.world_state.rng.gen_range(5..=15);
    let execution_time = res.world_state.rng.gen_range(100..=250); // 100-250 ticks (~1.5-4 seconds at 60 ticks/sec)

    let action = QueuedAction {
        entity: enemy_entity,
        action: CombatAction::EnemyAttack {
            target_entity: player_entity,
            damage,
        },
        execution_time_ticks: execution_time,
        remaining_time_ticks: execution_time,
    };

    if let Ok(mut queue) = world.get::<&mut ActionQueue>(enemy_entity) {
        queue.queue_action(action);
        queue.start_next_action();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{Energy, GameEntity, Health, Player};
    use crate::moves::{ActionQueue, CombatAction, MoveType, QueuedAction};
    use rand::Rng;

    #[test]
    fn test_queue_cleanup_on_enemy_death() {
        // Setup a minimal world with player and enemy
        let mut world = World::new();
        let mut res = Resources::new(12345);

        // Create player
        let player = world.spawn((
            Position { x: 0, y: 0, z: 0 },
            Player,
            GameEntity,
            Health::new(100),
            Energy::new(100),
            ActionQueue::new(),
        ));

        // Create enemy with low health (will die from one hit)
        let enemy = world.spawn((
            Position { x: 1, y: 0, z: 0 },
            GameEntity,
            Health::new(10), // Low health - will die from melee attack
            Combat::default(),
        ));

        // Create multiple player actions targeting the enemy
        let action1 = QueuedAction {
            entity: player,
            action: CombatAction::PlayerMove {
                move_data: Move::melee(),
                target_entity: Some(enemy),
                target_position: None,
            },
            execution_time_ticks: 150,
            remaining_time_ticks: 1, // Will complete after 1 tick (kills enemy)
        };

        let action2 = QueuedAction {
            entity: player,
            action: CombatAction::PlayerMove {
                move_data: Move::melee(),
                target_entity: Some(enemy),
                target_position: None,
            },
            execution_time_ticks: 150,
            remaining_time_ticks: 150, // Should be removed when enemy dies
        };

        let action3 = QueuedAction {
            entity: player,
            action: CombatAction::PlayerMove {
                move_data: Move::fireball(),
                target_entity: Some(enemy),
                target_position: None,
            },
            execution_time_ticks: 200,
            remaining_time_ticks: 200, // Should be removed when enemy dies
        };

        // Add actions to player's queue
        if let Ok(mut queue) = world.get::<&mut ActionQueue>(player) {
            queue.queue_action(action1);
            queue.queue_action(action2);
            queue.queue_action(action3);
            queue.start_next_action();
        }

        // Verify we have 3 actions initially (1 current, 2 queued)
        if let Ok(queue) = world.get::<&ActionQueue>(player) {
            assert!(queue.current_action.is_some(), "Should have current action");
            assert_eq!(queue.actions.len(), 2, "Should have 2 queued actions");
        }

        // Process one tick of the action queue system - this should kill the enemy
        action_queue_system(&mut world, &mut res);

        // Verify the enemy is now dead
        assert!(
            world.get::<&Dead>(enemy).is_ok(),
            "Enemy should be marked as Dead"
        );

        // Verify all remaining actions targeting the dead enemy are removed
        if let Ok(queue) = world.get::<&ActionQueue>(player) {
            // Current action should be None (or not targeting the dead enemy)
            if let Some(current) = &queue.current_action {
                if let CombatAction::PlayerMove { target_entity, .. } = &current.action {
                    assert!(
                        target_entity.map_or(true, |t| t != enemy),
                        "Current action should not target dead enemy"
                    );
                }
            }

            // No queued actions should target the dead enemy
            for action in &queue.actions {
                if let CombatAction::PlayerMove { target_entity, .. } = &action.action {
                    assert!(
                        target_entity.map_or(true, |t| t != enemy),
                        "Queued action should not target dead enemy"
                    );
                }
            }
        }

        // Verify player is still alive
        assert!(
            world.get::<&Dead>(player).is_err(),
            "Player should still be alive"
        );
    }

    #[test]
    fn test_centralized_cleanup_function() {
        // Test the centralized cleanup function directly
        let mut world = World::new();
        let mut res = Resources::new(12345);

        // Create player and enemy entities
        let player = world.spawn((
            Position { x: 0, y: 0, z: 0 },
            Player,
            GameEntity,
            ActionQueue::new(),
        ));

        let enemy1 = world.spawn((
            Position { x: 1, y: 0, z: 0 },
            GameEntity,
            ActionQueue::new(),
        ));

        let enemy2 = world.spawn((
            Position { x: 2, y: 0, z: 0 },
            GameEntity,
            ActionQueue::new(),
        ));

        // Create actions targeting enemy1 from multiple entities
        let player_action = QueuedAction {
            entity: player,
            action: CombatAction::PlayerMove {
                move_data: Move::melee(),
                target_entity: Some(enemy1),
                target_position: None,
            },
            execution_time_ticks: 150,
            remaining_time_ticks: 150,
        };

        let enemy_action = QueuedAction {
            entity: enemy2,
            action: CombatAction::EnemyAttack {
                target_entity: enemy1,
                damage: 10,
            },
            execution_time_ticks: 100,
            remaining_time_ticks: 100,
        };

        // Add actions to queues
        if let Ok(mut queue) = world.get::<&mut ActionQueue>(player) {
            queue.queue_action(player_action.clone());
            queue.start_next_action();
        }

        if let Ok(mut queue) = world.get::<&mut ActionQueue>(enemy2) {
            queue.queue_action(enemy_action.clone());
            queue.start_next_action();
        }

        // Add an action queue to enemy1 itself
        if let Ok(mut queue) = world.get::<&mut ActionQueue>(enemy1) {
            queue.queue_action(QueuedAction {
                entity: enemy1,
                action: CombatAction::EnemyAttack {
                    target_entity: player,
                    damage: 5,
                },
                execution_time_ticks: 80,
                remaining_time_ticks: 80,
            });
        }

        // Verify initial state - should have actions targeting enemy1
        assert!(world
            .get::<&ActionQueue>(player)
            .unwrap()
            .current_action
            .is_some());
        assert!(world
            .get::<&ActionQueue>(enemy2)
            .unwrap()
            .current_action
            .is_some());
        assert!(!world
            .get::<&ActionQueue>(enemy1)
            .unwrap()
            .actions
            .is_empty());

        // Call centralized cleanup for enemy1
        cleanup_actions_targeting_dead_entity(&mut world, &mut res, enemy1);

        // Verify cleanup worked
        // Player's current action targeting enemy1 should be removed
        if let Ok(queue) = world.get::<&ActionQueue>(player) {
            if let Some(current) = &queue.current_action {
                if let CombatAction::PlayerMove { target_entity, .. } = &current.action {
                    assert!(
                        target_entity.map_or(true, |t| t != enemy1),
                        "Player should not have current action targeting enemy1"
                    );
                }
            }
        }

        // Enemy2's current action targeting enemy1 should be removed
        if let Ok(queue) = world.get::<&ActionQueue>(enemy2) {
            if let Some(current) = &queue.current_action {
                if let CombatAction::EnemyAttack { target_entity, .. } = &current.action {
                    assert!(
                        *target_entity != enemy1,
                        "Enemy2 should not have current action targeting enemy1"
                    );
                }
            }
        }

        // Enemy1's own queue should be cleared
        assert!(world
            .get::<&ActionQueue>(enemy1)
            .unwrap()
            .actions
            .is_empty());
        assert!(world
            .get::<&ActionQueue>(enemy1)
            .unwrap()
            .current_action
            .is_none());
    }

    #[test]
    fn test_combat_system_enemy_death() {
        // Setup a minimal world with player and enemy
        let mut world = World::new();
        let mut res = Resources::new(12345);

        // Create player
        let player = world.spawn((
            Position { x: 0, y: 0, z: 0 },
            Player,
            GameEntity,
            Health::new(100),
            Energy::new(100),
            ActionQueue::new(),
        ));
        // Note: player_entity no longer needed - use ECS queries

        // Create enemy with low health (will die from one hit)
        let enemy = world.spawn((
            Position { x: 1, y: 0, z: 0 },
            GameEntity,
            Health::new(10), // Low health - will die from melee attack
            Combat::default(),
        ));

        // Create a player melee attack action that should kill the enemy
        let action = QueuedAction {
            entity: player,
            action: CombatAction::PlayerMove {
                move_data: Move::melee(),
                target_entity: Some(enemy),
                target_position: None,
            },
            execution_time_ticks: 150,
            remaining_time_ticks: 1, // Will complete after 1 tick
        };

        // Add action to player's queue and execute it
        if let Ok(mut queue) = world.get::<&mut ActionQueue>(player) {
            queue.queue_action(action);
            queue.start_next_action();
        }

        // Process one tick of the action queue system
        action_queue_system(&mut world, &mut res);

        // Verify the enemy is now dead
        assert!(
            world.get::<&Dead>(enemy).is_ok(),
            "Enemy should be marked as Dead"
        );

        // Verify the enemy's health is 0 or below
        if let Ok(health) = world.get::<&Health>(enemy) {
            assert!(!health.is_alive(), "Enemy should not be alive");
        }

        // Verify player is still alive
        assert!(
            world.get::<&Dead>(player).is_err(),
            "Player should still be alive"
        );
        if let Ok(health) = world.get::<&Health>(player) {
            assert!(health.is_alive(), "Player should still be alive");
        };
    }

    #[test]
    fn test_comprehensive_combat_simulation() {
        // Comprehensive combat simulation test that exercises:
        // 1. Combat triggering when enemy becomes adjacent
        // 2. Multiple enemies joining mid-combat
        // 3. Enemy death and cleanup
        // 4. debugInstantKill functionality
        // 5. Combat state transitions

        let mut world = World::new();
        let mut res = Resources::new(12345);

        // Create player at origin
        let player = world.spawn((
            Position { x: 5, y: 5, z: 0 },
            Player,
            GameEntity,
            Health::new(100),
            Energy::new(100),
            ActionQueue::new(),
        ));

        // Create first enemy adjacent to player (should trigger combat)
        let enemy1 = world.spawn((
            Position { x: 6, y: 5, z: 0 }, // Adjacent to player
            GameEntity,
            Health::new(50),
            Combat::default(),
        ));

        // Create second enemy nearby but not adjacent (should not trigger combat initially)
        let enemy2 = world.spawn((
            Position { x: 8, y: 5, z: 0 }, // 3 tiles away
            GameEntity,
            Health::new(30),
            Combat::default(),
        ));

        // Step 1: Update combat state - should trigger combat with enemy1
        let (new_state, tick_result) =
            crate::combat_state_manager::CombatStateManager::update_combat_state(
                &mut world, &mut res,
            );
        assert_eq!(new_state, crate::combat_state_manager::CombatState::Active);
        assert!(tick_result.contains(crate::game::GameTickResult::CombatTriggered));
        assert!(res.player_state.combat_active);

        // Verify enemy1 has triggered flag set
        assert!(world.get::<&Combat>(enemy1).unwrap().triggered);
        // Verify enemy2 does not have triggered flag set (not adjacent)
        assert!(!world.get::<&Combat>(enemy2).unwrap().triggered);

        // Step 2: Move enemy2 to be adjacent to player (simulate it walking up)
        if let Ok(mut pos) = world.get::<&mut Position>(enemy2) {
            pos.x = 4; // Now adjacent to player at (5,5)
            pos.y = 5;
        }

        // Update combat state again - should update participants
        let (new_state, _) = crate::combat_state_manager::CombatStateManager::update_combat_state(
            &mut world, &mut res,
        );
        assert_eq!(new_state, crate::combat_state_manager::CombatState::Active);

        // Now enemy2 should also be triggered (joined mid-combat)
        assert!(world.get::<&Combat>(enemy2).unwrap().triggered);

        // Step 3: Test debugInstantKill - kill enemy1
        if let Ok(mut health) = world.get::<&mut Health>(enemy1) {
            health.current = 0; // Simulate instant kill
        }
        world.insert_one(enemy1, Dead).ok();

        // Run cleanup for the dead enemy
        crate::systems::cleanup_actions_targeting_dead_entity(&mut world, &mut res, enemy1);

        // Verify enemy1 is dead
        assert!(world.get::<&Dead>(enemy1).is_ok());
        assert!(!world.get::<&Health>(enemy1).unwrap().is_alive());

        // Step 4: Combat should still be active because enemy2 is alive and adjacent
        let (new_state, _) = crate::combat_state_manager::CombatStateManager::update_combat_state(
            &mut world, &mut res,
        );
        assert_eq!(new_state, crate::combat_state_manager::CombatState::Active);
        assert!(res.player_state.combat_active);

        // Step 5: Move enemy2 away (simulate it fleeing)
        if let Ok(mut pos) = world.get::<&mut Position>(enemy2) {
            pos.x = 20; // Far away
            pos.y = 20;
        }

        // Now combat should end
        let (new_state, tick_result) =
            crate::combat_state_manager::CombatStateManager::update_combat_state(
                &mut world, &mut res,
            );
        assert_eq!(
            new_state,
            crate::combat_state_manager::CombatState::JustEnded
        );
        assert!(tick_result.contains(crate::game::GameTickResult::CombatEnded));
        assert!(!res.player_state.combat_active);
        assert!(res.player_state.combat_ended_this_tick);

        // Step 6: Next tick should transition to Idle
        let (new_state, _) = crate::combat_state_manager::CombatStateManager::update_combat_state(
            &mut world, &mut res,
        );
        assert_eq!(new_state, crate::combat_state_manager::CombatState::Idle);
        assert!(!res.player_state.combat_active);
        assert!(!res.player_state.combat_ended_this_tick);

        // Verify player is still alive
        assert!(world.get::<&Dead>(player).is_err());
        assert!(world.get::<&Health>(player).unwrap().is_alive());
    }

    #[test]
    fn test_world_simulation_stress() {
        // Stress test simulating many game ticks with various scenarios
        // Uses deterministic seeded RNG for reproducible tests
        let mut world = World::new();
        let mut res = Resources::new(12345); // Fixed seed for determinism

        // Create player
        let player = world.spawn((
            Position { x: 0, y: 0, z: 0 },
            Player,
            GameEntity,
            Health::new(1000), // High health for stress test
            Energy::new(1000),
            ActionQueue::new(),
        ));

        // Create multiple enemies at various distances
        let mut enemies = Vec::new();
        for i in 0..10 {
            let enemy = world.spawn((
                Position {
                    x: i as i32 * 2,
                    y: 0,
                    z: 0,
                }, // Spread out
                GameEntity,
                Health::new(20),
                Combat::default(),
            ));
            enemies.push(enemy);
        }

        let mut combat_triggers = 0;
        let mut combat_ends = 0;

        // Simulate 1000 ticks of game time
        for tick in 0..1000 {
            res.time.tick = tick;

            // Randomly move enemies closer/further
            if tick % 50 == 0 {
                // Every 50 ticks
                for &enemy in &enemies {
                    if world.get::<&Dead>(enemy).is_err() {
                        // Only move living enemies
                        if let Ok(mut pos) = world.get::<&mut Position>(enemy) {
                            // Sometimes move adjacent to player
                            if res.world_state.rng.gen_bool(0.3) {
                                pos.x = if res.world_state.rng.gen_bool(0.5) {
                                    -1
                                } else {
                                    1
                                };
                                pos.y = 0;
                            } else {
                                // Sometimes move far away
                                pos.x = res.world_state.rng.gen_range(-10..=10);
                                pos.y = res.world_state.rng.gen_range(-10..=10);
                            }
                        }
                    }
                }
            }

            // Update combat state
            let (_, tick_result) =
                crate::combat_state_manager::CombatStateManager::update_combat_state(
                    &mut world, &mut res,
                );

            if tick_result.contains(crate::game::GameTickResult::CombatTriggered) {
                combat_triggers += 1;
            }
            if tick_result.contains(crate::game::GameTickResult::CombatEnded) {
                combat_ends += 1;
            }

            // Occasionally kill an enemy (simulate debugInstantKill or combat)
            if tick % 100 == 0 && res.player_state.combat_active {
                for &enemy in &enemies {
                    if world.get::<&Dead>(enemy).is_err() && res.world_state.rng.gen_bool(0.2) {
                        if let Ok(mut health) = world.get::<&mut Health>(enemy) {
                            health.current = 0;
                        }
                        world.insert_one(enemy, Dead).ok();
                        crate::systems::cleanup_actions_targeting_dead_entity(
                            &mut world, &mut res, enemy,
                        );
                        break; // Only kill one per check
                    }
                }
            }
        }

        // Verify the simulation ran and had combat activity
        assert!(
            combat_triggers > 0,
            "Should have had some combat triggers during 1000 ticks"
        );

        // Player should still be alive after stress test
        assert!(world.get::<&Dead>(player).is_err());
        assert!(world.get::<&Health>(player).unwrap().is_alive());

        println!(
            "Stress test completed: {} combat triggers, {} combat ends over 1000 ticks",
            combat_triggers, combat_ends
        );
    }

    #[test]
    fn test_corpse_death_and_movement() {
        // Setup a minimal world with player and feral dog
        let mut world = World::new();
        let mut res = Resources::new(12345);

        // Create player at position (0, 0)
        let _player = world.spawn((
            Position { x: 0, y: 0, z: 0 },
            Player,
            GameEntity,
            Health::new(100),
            Energy::new(100),
            ActionQueue::new(),
            BlocksMovement,
        ));

        // Create a feral dog with inventory at position (1, 0)
        let mut dog_inventory = Inventory::default();
        dog_inventory.add(ItemKind::Leather, 2);
        dog_inventory.add(ItemKind::Meat, 3);

        let dog = world.spawn((
            Position { x: 1, y: 0, z: 0 },
            GameEntity,
            EntityKind::FeralDog,
            FeralDog,
            Health::new(10), // Low health - will die easily
            Combat::default(),
            BlocksMovement, // Dogs block movement when alive
            dog_inventory,
        ));

        // Kill the dog and verify corpse creation
        handle_entity_death(&mut world, &mut res, dog);

        // Verify the dog is gone (despawned)
        assert!(
            world.get::<&FeralDog>(dog).is_err(),
            "Dog should be despawned"
        );

        // Find the corpse entity
        let mut corpse_entity = None;
        for (entity, (pos, kind)) in world.query::<(&Position, &EntityKind)>().iter() {
            if *kind == EntityKind::Corpse && pos.x == 1 && pos.y == 0 && pos.z == 0 {
                corpse_entity = Some(entity);
                break;
            }
        }

        let corpse = corpse_entity.expect("Corpse should exist at dog's former position");

        // Verify corpse has the dog's inventory
        if let Ok(corpse_inv) = world.get::<&Inventory>(corpse) {
            let leather_count = corpse_inv
                .slots
                .iter()
                .find(|s| s.kind == ItemKind::Leather)
                .map(|s| s.qty)
                .unwrap_or(0);
            let meat_count = corpse_inv
                .slots
                .iter()
                .find(|s| s.kind == ItemKind::Meat)
                .map(|s| s.qty)
                .unwrap_or(0);

            assert_eq!(leather_count, 2, "Corpse should have 2 leather");
            assert_eq!(meat_count, 3, "Corpse should have 3 meat");
        } else {
            panic!("Corpse should have an inventory");
        }

        // Most importantly: Verify corpse doesn't block movement
        assert!(
            world.get::<&BlocksMovement>(corpse).is_err(),
            "Corpse should NOT have BlocksMovement component"
        );

        // Verify that movement system would not consider the corpse position blocked
        let corpse_pos = world.get::<&Position>(corpse).unwrap();
        let mut is_blocked_by_corpse = false;
        for (_, (_, other_pos)) in world.query::<(&BlocksMovement, &Position)>().iter() {
            if other_pos.x == corpse_pos.x
                && other_pos.y == corpse_pos.y
                && other_pos.z == corpse_pos.z
            {
                is_blocked_by_corpse = true;
                break;
            }
        }
        assert!(
            !is_blocked_by_corpse,
            "Corpse position should not be blocked by any BlocksMovement entity"
        );
    }
}

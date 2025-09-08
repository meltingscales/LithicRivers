use crate::component_access::{ComponentAccess, ComponentUpdate, DamageResult};
use crate::components::{
    BattleDelay, BlocksMovement, Combat, Dead, DroppedItem, Energy, FeralDog, GameEntity,
    Inventory, ItemKind, Player, Position, Sheep, SpriteRef, Stunned,
};
use crate::intent::PlayerAction;
use crate::moves::{ActionQueue, CombatAction, Move, MoveType, QueuedAction};
use crate::resources::Resources;
use hecs::World;
use tracing::info;

/// Check if an entity is stunned (has BattleDelay component)
fn is_stunned(world: &World, entity: hecs::Entity) -> bool {
    world.get::<&BattleDelay>(entity).is_ok()
}

/// ECS Helper Functions for Systems

/// Get the player entity using proper ECS query
fn get_player_entity(world: &World) -> Option<hecs::Entity> {
    world.query::<&Player>().iter().next().map(|(e, _)| e)
}

/// Get player position using ECS query  
fn get_player_position(world: &World) -> Option<Position> {
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
                        let mut comp_access = ComponentAccess::new(world);
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
    // Check if there's a mining action in the intent
    let has_mining_action = matches!(res.player_state.intent.action, Some(PlayerAction::Mine));
    if !has_mining_action {
        return false;
    }

    let Some(player_e) = get_player_entity(world) else {
        return false;
    };
    let Ok(pos) = world.get::<&Position>(player_e) else {
        return false;
    };
    let (x, y, z) = (pos.x, pos.y, pos.z);
    // End immutable borrow before mutating the world
    drop(pos);
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
    let mut comp_access = ComponentAccess::new(world);
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

pub enum CombatState {
    Idle,
    CombatStarted,
    CombatEnded,
}

pub fn combat_trigger_system(world: &mut World, res: &mut Resources) -> CombatState {
    // get player position
    let player_pos = match get_player_position(world) {
        Some(pos) => pos,
        None => return CombatState::Idle, // No player to chase
    };

    //if we're already in combat, check to see if we should exit combat
    //TODO

    // collect all entities with a combat component that aren't in battle delay
    let mut combat_entities = Vec::new();
    for (e, (pos, _, _)) in world.query::<(&Position, &Combat, &GameEntity)>().iter() {
        // Skip entities with BattleDelay - they can't re-engage in combat yet
        if world.get::<&BattleDelay>(e).is_ok() {
            continue;
        }
        combat_entities.push((e, *pos));
    }

    // Define the 8 adjacent positions around the player
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

    // process each combat entity
    for (e, pos) in combat_entities {
        // check if entity is in any of the 8 adjacent positions
        let is_adjacent = adjacent_positions
            .iter()
            .any(|&(adj_x, adj_y)| pos.x == adj_x && pos.y == adj_y && pos.z == player_pos.z);

        if !is_adjacent {
            continue;
        }

        // trigger combat
        let combat_triggered = if let Ok(mut combat) = world.get::<&mut Combat>(e) {
            combat.triggered = true;
            true
        } else {
            false
        };

        if combat_triggered {
            // Mark combat as active
            res.player_state.combat_active = true;

            // Ensure player has an ActionQueue component for combat
            if let Some(player_entity) = get_player_entity(world) {
                if world.get::<&ActionQueue>(player_entity).is_err() {
                    world.insert_one(player_entity, ActionQueue::new()).ok();
                }
            }

            return CombatState::CombatStarted;
        }
    }
    CombatState::Idle
}

// Feral dogs will chase the player if they get too close.
pub fn feral_dog_system(world: &mut World, res: &mut Resources) {
    use rand::Rng;

    // Get player position if available
    let player_pos = match get_player_position(world) {
        Some(pos) => pos,
        None => return, // No player to chase
    };

    // Collect dogs with their positions to avoid borrowing issues
    let mut dogs = Vec::new();
    for (e, (pos, _)) in world.query::<(&Position, &FeralDog)>().iter() {
        dogs.push((e, *pos));
    }

    // Process each dog
    for (dog_entity, dog_pos) in dogs {
        // Skip stunned or dead dogs - they cannot move
        if is_stunned(world, dog_entity) || world.get::<&Dead>(dog_entity).is_ok() {
            continue;
        }

        // Calculate distance to player
        let dx = player_pos.x - dog_pos.x;
        let dy = player_pos.y - dog_pos.y;
        let distance_sq = dx * dx + dy * dy;

        // Only chase if player is within 10 tiles
        if distance_sq > 100 {
            // 10^2
            continue;
        }

        // decide if dog should do nothing for a turn (50% chance)
        if res.world_state.rng.gen_range(0..2) == 0 {
            continue;
        }

        // Determine movement direction (signum gives -1, 0, or 1)
        let move_x = dx.signum();
        let move_y = dy.signum();

        // Calculate new position
        let new_x = dog_pos.x + move_x;
        let new_y = dog_pos.y + move_y;
        let new_z = dog_pos.z;

        // Check if new position is blocked by terrain
        if !res.world_state.world.is_passable(new_x, new_y, new_z) {
            continue;
        }

        // Check for blocking entities at new position
        let mut blocked = false;
        for (_, (other_pos, _)) in world.query::<(&Position, &BlocksMovement)>().iter() {
            if other_pos.x == new_x && other_pos.y == new_y && other_pos.z == new_z {
                blocked = true;
                break;
            }
        }

        // Move the dog if not blocked
        if !blocked {
            if let Ok(mut pos) = world.get::<&mut Position>(dog_entity) {
                pos.x = new_x;
                pos.y = new_y;
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

    let mut completed_actions = Vec::new();
    let mut entities_with_queues = Vec::new();

    // Collect entities that have action queues
    for (entity, _queue) in world.query::<&mut ActionQueue>().iter() {
        entities_with_queues.push(entity);
    }

    // Process each entity's action queue
    for entity in entities_with_queues {
        if let Ok(mut queue) = world.get::<&mut ActionQueue>(entity) {
            // Update timers and check for completed actions
            if let Some(completed_action) = queue.update_timers(DELTA_TICKS) {
                completed_actions.push(completed_action);
            }
        }
    }

    // Execute all completed actions
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
                // TODO: Add AoE damage to nearby enemies
            }
        }
        MoveType::Tackle => {
            if let Some(target) = target_entity {
                apply_damage(world, res, target, move_data.damage, "tackle");

                // Store original enemy position before pushing
                let original_enemy_pos = if let Ok(target_pos) = world.get::<&Position>(target) {
                    *target_pos
                } else {
                    player_pos // Fallback to player position if we can't get target position
                };

                // Push effect - push enemy back 2 spaces
                if let Ok(target_pos) = world.get::<&Position>(target) {
                    let pushed_pos =
                        crate::moves::calculate_push_position(player_pos, *target_pos, 2);
                    if let Ok(mut pos) = world.get::<&mut Position>(target) {
                        pos.x = pushed_pos.x;
                        pos.y = pushed_pos.y;
                        pos.z = pushed_pos.z;
                    }
                }

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

            // Mark combat as inactive
            res.player_state.combat_active = false;

            // Clear the player's action queue since combat is ending
            if let Ok(mut queue) = world.get::<&mut ActionQueue>(player_entity) {
                queue.clear();
                res.log("Action queue cleared after escape");
            }
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

/// Apply damage to an entity using safe component access
fn apply_damage(
    world: &mut World,
    res: &mut Resources,
    target_entity: hecs::Entity,
    damage: u32,
    source: &str,
) {
    let mut comp_access = ComponentAccess::new(world);

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
    // Add Dead marker
    world.insert_one(entity, Dead).ok();

    // Clear any action queues
    if let Ok(mut queue) = world.get::<&mut ActionQueue>(entity) {
        queue.clear();
    }

    // Remove combat capability
    world.remove_one::<Combat>(entity).ok();

    // Log death
    if is_player_entity(world, entity) {
        res.log("You have died!".to_string());
        // TODO: Trigger game over state
    } else {
        res.log("Enemy defeated!".to_string());
    }
}

/// End combat for all enemies near the given position
fn end_combat_for_nearby_enemies(world: &mut World, center_pos: Position) {
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
}

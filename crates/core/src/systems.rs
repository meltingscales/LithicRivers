use crate::components::{
    BattleDelay, BlocksMovement, Combat, Dead, DroppedItem, Energy, FeralDog, GameEntity, Health,
    Inventory, ItemKind, Position, Sheep, SpriteRef, Stunned,
};
use crate::moves::{ActionQueue, CombatAction, Move, MoveType, QueuedAction};
use crate::resources::Resources;
use hecs::World;
use tracing::info;

/// Check if an entity is stunned (has BattleDelay component)
fn is_stunned(world: &World, entity: hecs::Entity) -> bool {
    world.get::<&BattleDelay>(entity).is_ok()
}

pub fn move_player_system(world: &mut World, res: &mut Resources) {
    if let Some((dx, dy)) = res.player_move_intent.take() {
        if let Some(player_e) = res.player_entity {
            // 1) Read current position immutably to avoid aliasing with queries below
            if let Ok(pos) = world.get::<&Position>(player_e) {
                let (cx, cy, cz) = (pos.x, pos.y, pos.z);
                // Drop immutable borrow explicitly (not strictly necessary but clarifies intent)
                drop(pos);

                let nx = cx + dx;
                let ny = cy + dy;
                let nz = cz; // Keep same Z-level for now
                let t = res.world.get_tile_cached(nx, ny, nz);
                // Check tile passability and blocking entities
                let mut blocked = !t.is_passable();
                if !blocked {
                    for (_, (_, epos)) in world.query::<(&BlocksMovement, &Position)>().iter() {
                        if epos.x == nx && epos.y == ny && epos.z == nz {
                            blocked = true;
                            break;
                        }
                    }
                }
                if blocked {
                    info!("Blocked by {:?} at ({}, {})", t, nx, ny);
                    // Set resource for last blocked tile (only x,y for now)
                    res.last_blocked_tile = Some((nx, ny));
                } else {
                    // 2) Now borrow mutably to write the new position
                    if let Ok(mut pos_mut) = world.get::<&mut Position>(player_e) {
                        pos_mut.x = nx;
                        pos_mut.y = ny;
                        pos_mut.z = nz;
                    }
                }
            }
        }
    }

    // Handle vertical movement intent separately (no terrain checks yet)
    if let Some(dz) = res.player_move_intent_z.take() {
        if let Some(player_e) = res.player_entity {
            if let Ok(mut pos_mut) = world.get::<&mut Position>(player_e) {
                pos_mut.z = pos_mut.z.saturating_add(dz);
            }
        }
    }
}

/// Process mining intent: if the player requested mining, act on current tile.
/// Returns true if mining was successful (e.g. chopped a tree), false otherwise.
pub fn mining_system(world: &mut World, res: &mut Resources) -> bool {
    if !res.mining_intent {
        return false;
    }
    res.mining_intent = false;
    let Some(player_e) = res.player_entity else {
        return false;
    };
    let Ok(pos) = world.get::<&Position>(player_e) else {
        return false;
    };
    let (x, y, z) = (pos.x, pos.y, pos.z);
    // End immutable borrow before mutating the world
    drop(pos);
    let t = res.world.get_tile_cached(x, y, z);
    use crate::tiles::TileKind;
    use rand::Rng;
    match t {
        TileKind::Tree => {
            // Chop tree: convert to Dirt and drop items
            res.world.set_tile_cached(x, y, z, TileKind::Dirt);

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

            res.log(format!(
                "You chop the tree. (+{} Wood, +{} Acorn)",
                wood_qty, acorn_qty
            ));
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
    let Some(player_e) = res.player_entity else {
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

    // Process inventory updates
    if let Ok(mut inv) = world.get::<&mut Inventory>(player_e) {
        let mut total = 0u32;
        for di in &items {
            inv.add(di.kind, di.qty);
            total += di.qty;
        }
        if total > 0 {
            res.log(format!("Picked up {} items", total));
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
    let player_pos = match res
        .player_entity
        .and_then(|e| world.get::<&Position>(e).ok())
    {
        Some(pos) => *pos,
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

    // process each combat entity
    for (e, pos) in combat_entities {
        // calculate distance to player
        let dx = player_pos.x - pos.x;
        let dy = player_pos.y - pos.y;
        let distance_sq = dx * dx + dy * dy;

        // only trigger combat if player is touching on any of the 8 cardinal directions
        if distance_sq > 1 {
            // 1^2
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
            // Ensure player has an ActionQueue component for combat
            if let Some(player_entity) = res.player_entity {
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
    let player_pos = match res
        .player_entity
        .and_then(|e| world.get::<&Position>(e).ok())
    {
        Some(pos) => *pos,
        None => return, // No player to chase
    };

    // Collect dogs with their positions to avoid borrowing issues
    let mut dogs = Vec::new();
    for (e, (pos, _)) in world.query::<(&Position, &FeralDog)>().iter() {
        dogs.push((e, *pos));
    }

    // Process each dog
    for (dog_entity, dog_pos) in dogs {
        // Skip stunned dogs - they cannot move
        if is_stunned(world, dog_entity) {
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
        if res.rng.gen_range(0..2) == 0 {
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
        if !res.world.is_passable(new_x, new_y, new_z) {
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
        let r: u32 = res.rng.gen_range(0..5);
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
        let t = res.world.get_tile(nx, ny, nz);
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
    let player_dead = if let Some(player_entity) = res.player_entity {
        world.get::<&Dead>(player_entity).is_ok()
    } else {
        true // No player entity means dead
    };

    // Check if all enemies are dead
    let mut has_living_enemies = false;
    for (entity, (_, combat, _)) in world.query::<(&Position, &Combat, &GameEntity)>().iter() {
        if Some(entity) != res.player_entity && combat.triggered {
            if world.get::<&Dead>(entity).is_err() {
                has_living_enemies = true;
                break;
            }
        }
    }

    // Clear all action queues if combat should end
    if player_dead || !has_living_enemies {
        let mut entities_to_clear = Vec::new();
        for (entity, _) in world.query::<&ActionQueue>().iter() {
            entities_to_clear.push(entity);
        }

        for entity in entities_to_clear {
            if let Ok(mut queue) = world.get::<&mut ActionQueue>(entity) {
                queue.clear();
            }
        }

        if player_dead {
            res.log("Combat ended: Player defeated - all action queues cleared".to_string());
        } else if !has_living_enemies {
            res.log("Combat ended: All enemies defeated - all action queues cleared".to_string());
        }
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
                res.log("Not enough energy!".to_string());
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

                // Push effect
                if let Ok(target_pos) = world.get::<&Position>(target) {
                    let pushed_pos =
                        crate::moves::calculate_push_position(player_pos, *target_pos, 2);
                    if let Ok(mut pos) = world.get::<&mut Position>(target) {
                        pos.x = pushed_pos.x;
                        pos.y = pushed_pos.y;
                        pos.z = pushed_pos.z;
                    }
                }

                // 50% chance to stun
                if res.rng.gen_bool(0.5) {
                    world.insert_one(target, Stunned::new(600)).ok();
                    res.log("Target is stunned!".to_string());
                }
            }
        }
        MoveType::Escape => {
            // Add battle delay to player to prevent immediate re-engagement
            world.insert_one(player_entity, BattleDelay::new(1000)).ok();
            res.log("You attempt to escape from combat!".to_string());

            // End combat for all nearby enemies
            end_combat_for_nearby_enemies(world, player_pos);

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

/// Apply damage to an entity, handling both Health and Body systems
fn apply_damage(
    world: &mut World,
    res: &mut Resources,
    target_entity: hecs::Entity,
    damage: u32,
    source: &str,
) {
    use crate::model::body::Body;

    // Check if target is already dead
    if world.get::<&Dead>(target_entity).is_ok() {
        return;
    }

    // Try to apply damage to Health component first
    let should_handle_death = if let Ok(mut health) = world.get::<&mut Health>(target_entity) {
        health.damage(damage);
        res.log(format!(
            "Target takes {} damage from {} ({}/{} HP)",
            damage, source, health.current, health.max
        ));

        // Check for death
        !health.is_alive()
    } else {
        false
    };

    if should_handle_death {
        handle_entity_death(world, res, target_entity);
        return;
    }

    // If no Health component, try to damage body parts (for robots)
    let should_handle_body_death = if let Ok(mut body) = world.get::<&mut Body>(target_entity) {
        if let Some(damaged_part) =
            crate::moves::damage_random_body_part(&mut body, res.world.seed, res.gametick, damage)
        {
            res.log(format!(
                "Target's {:?} is damaged by {}",
                damaged_part, source
            ));

            // Check if body integrity is too low (death condition for robots)
            let integrity = crate::moves::calculate_body_integrity(&body);
            integrity < 0.1 // 10% integrity threshold
        } else {
            false
        }
    } else {
        false
    };

    if should_handle_body_death {
        handle_entity_death(world, res, target_entity);
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
    if Some(entity) == res.player_entity {
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
        if combat.triggered && entity != res.player_entity.unwrap_or(hecs::Entity::DANGLING) {
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
    let player_entity = match res.player_entity {
        Some(player) => player,
        None => return,
    };

    // Simple AI: attack the player with random damage and timing
    let damage = res.rng.gen_range(5..=15);
    let execution_time = res.rng.gen_range(100..=250); // 100-250 ticks (~1.5-4 seconds at 60 ticks/sec)

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
        res.player_entity = Some(player);

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
            remaining_time_ticks: 0, // Ready to execute immediately
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

use crate::component_access::{ComponentAccess, DamageResult};
use crate::components::{
    BattleDelay, Combat, Dead, Energy, EntityKind, FeralDog, GameEntity, Inventory, Player,
    Position, SpriteRef, Stunned,
};
use crate::moves::{ActionQueue, CombatAction, Move, MoveType, QueuedAction};
use crate::resources::Resources;
use crate::systems::ecs_utils::{get_player_entity, is_player_entity};
use hecs::World;

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
pub fn execute_combat_action(world: &mut World, res: &mut Resources, action: &QueuedAction) {
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
pub fn execute_player_move(
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
pub fn execute_enemy_attack(
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
pub fn handle_entity_death(world: &mut World, res: &mut Resources, entity: hecs::Entity) {
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

use crate::components::{
    BlocksMovement, Combat, Dead, DogAI, FeralDog, GameEntity, Position, Sheep, Stunned,
};
use crate::moves::{ActionQueue, CombatAction, QueuedAction};
use crate::pathfinding::{DogBehavior, Pathfinder};
use crate::resources::Resources;
use crate::systems::ecs_utils::{
    get_player_entity, get_player_position, is_player_entity, is_stunned,
};
use hecs::World;

// Feral dogs with A* pathfinding that chase the player intelligently
/// How far a feral dog notices the player by day, in tiles.
pub const DOG_NOTICE_DAY: i64 = 15;
/// How far a feral dog notices the player at night: they hunt bolder in the dark.
pub const DOG_NOTICE_NIGHT: i64 = 22;

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

        // Only chase a player they notice: farther away at night
        let notice = if res.time.is_night() {
            DOG_NOTICE_NIGHT
        } else {
            DOG_NOTICE_DAY
        };
        if distance_sq > notice * notice {
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
                let mut best_distance = i64::MAX;

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

/// How far a nuclear slime notices the player, in tiles on its level.
pub const SLIME_NOTICE: i64 = 6;

/// Nuclear slimes: once the player is within `SLIME_NOTICE` tiles on their level, they ooze
/// one tile toward them every other turn, without planning a path. Combat starts when one
/// is adjacent, as with dogs.
pub fn nuclear_slime_system(world: &mut World, res: &mut Resources) {
    use crate::components::{NuclearSlime, SlimeAI};
    let Some(player_pos) = get_player_position(world) else {
        return;
    };
    let mut slimes: Vec<(hecs::Entity, Position)> = world
        .query::<&Position>()
        .with::<&NuclearSlime>()
        .iter()
        .map(|(e, p)| (e, *p))
        .collect();
    slimes.sort_by_key(|(e, p)| (p.z, p.y, p.x, e.to_bits()));
    for (slime, pos) in slimes {
        let engaged = world.get::<&Combat>(slime).is_ok_and(|c| c.triggered);
        if is_stunned(world, slime) || world.get::<&Dead>(slime).is_ok() || engaged {
            continue;
        }
        let (dx, dy) = (player_pos.x - pos.x, player_pos.y - pos.y);
        let noticed = pos.z == player_pos.z && dx.abs().max(dy.abs()) <= SLIME_NOTICE;
        if !noticed {
            continue;
        }
        let turns = {
            let mut ai = world
                .get::<&mut SlimeAI>(slime)
                .expect("slimes have SlimeAI");
            ai.turns = ai.turns.wrapping_add(1);
            ai.turns
        };
        // Already beside the player, or resting this turn
        if dx.abs().max(dy.abs()) <= 1 || turns % 2 == 1 {
            continue;
        }
        let is_passable = |p: Position| {
            res.world_state.world.is_passable(p.x, p.y, p.z)
                && !world
                    .query::<&Position>()
                    .with::<&BlocksMovement>()
                    .iter()
                    .any(|(_, q)| *q == p)
        };
        // Slimes don't plan: straight toward the player, else sliding along a wall
        let (sx, sy) = (dx.signum(), dy.signum());
        let next = [(sx, sy), (sx, 0), (0, sy)]
            .into_iter()
            .filter(|&step| step != (0, 0))
            .map(|(mx, my)| Position {
                x: pos.x + mx,
                y: pos.y + my,
                z: pos.z,
            })
            .find(|&p| p != player_pos && is_passable(p));
        if let Some(next) = next {
            if let Ok(mut p) = world.get::<&mut Position>(slime) {
                *p = next;
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

/// System to automatically generate enemy actions during combat
pub fn enemy_combat_ai_system(world: &mut World, res: &mut Resources) {
    // Find all combat entities that should act
    let mut enemy_entities = Vec::new();

    tracing::debug!(target: "combat", "Enemy AI system running at tick {}", res.time.tick);

    for (entity, (_, combat, _)) in world.query::<(&Position, &Combat, &GameEntity)>().iter() {
        if combat.triggered && !is_player_entity(world, entity) {
            tracing::debug!(target: "combat", "Found combat entity {:?} with triggered combat", entity);

            // Skip if entity is dead, stunned, or already has actions queued
            if world.get::<&Dead>(entity).is_ok() {
                tracing::debug!(target: "combat", "Skipping dead entity {:?}", entity);
                continue;
            }
            if world.get::<&Stunned>(entity).is_ok() {
                tracing::debug!(target: "combat", "Skipping stunned entity {:?}", entity);
                continue;
            }

            // Check if entity has an empty or no action queue
            let needs_action = match world.get::<&ActionQueue>(entity) {
                Ok(queue) => {
                    let needs = queue.current_action.is_none() && queue.actions.is_empty();
                    tracing::debug!(target: "combat", "Entity {:?} has queue, needs_action: {}", entity, needs);
                    needs
                }
                Err(_) => {
                    tracing::debug!(target: "combat", "Entity {:?} has no queue, needs one", entity);
                    true
                }
            };

            if needs_action {
                enemy_entities.push(entity);
            }
        }
    }

    tracing::debug!(target: "combat", "Found {} enemies that need actions", enemy_entities.len());

    // Generate actions for each enemy that needs them
    for enemy_entity in enemy_entities {
        tracing::debug!(target: "combat", "Generating action for enemy {:?}", enemy_entity);
        generate_enemy_action(world, res, enemy_entity);
    }
}

/// Generate an action for an enemy entity
pub fn generate_enemy_action(world: &mut World, res: &mut Resources, enemy_entity: hecs::Entity) {
    use rand::Rng;

    tracing::debug!(target: "combat", "Generating action for enemy {:?}", enemy_entity);

    // Ensure enemy has an action queue
    if world.get::<&ActionQueue>(enemy_entity).is_err() {
        tracing::debug!(target: "combat", "Creating new ActionQueue for enemy {:?}", enemy_entity);
        world.insert_one(enemy_entity, ActionQueue::new()).ok();
    }

    // Find the player
    let player_entity = match get_player_entity(world) {
        Some(player) => {
            tracing::debug!(target: "combat", "Found player entity {:?} to target", player);
            player
        }
        None => {
            tracing::warn!(target: "combat", "No player entity found for enemy to target");
            return;
        }
    };

    // Simple AI: attack the player with random damage and timing. Slimes are slower and
    // hit harder than dogs
    let slime = world
        .get::<&crate::components::NuclearSlime>(enemy_entity)
        .is_ok();
    let (damage_range, time_range) = if slime {
        (10..=20, 14..=18)
    } else {
        (5..=15, 7..=10)
    };
    let damage = res.world_state.rng.gen_range(damage_range);
    let execution_time = res.world_state.rng.gen_range(time_range);

    tracing::debug!(target: "combat", "Enemy {:?} planning attack on player {:?} with {} damage in {} ticks", 
                   enemy_entity, player_entity, damage, execution_time);

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
        tracing::debug!(target: "combat", "Queueing attack action for enemy {:?}", enemy_entity);
        queue.queue_action(action);
        queue.start_next_action();
        tracing::debug!(target: "combat", "Enemy {:?} action queue now has {} actions, current_action: {}", 
                       enemy_entity, queue.actions.len(), queue.current_action.is_some());
    } else {
        tracing::error!(target: "combat", "Failed to get mutable ActionQueue for enemy {:?}", enemy_entity);
    }
}

#[cfg(test)]
mod tests {
    use crate::components::Position;
    use crate::intent::PlayerIntent;
    use crate::test_support::TestWorld;
    use crate::tiles::TileKind;

    /// Where a feral dog 20 tiles east of the player is after a few turns at `hour`:00.
    fn dog_after_turns(hour: u64) -> Position {
        let mut world = TestWorld::new(7).fill((-5, -3, 0), (30, 3, 0), TileKind::Grass);
        let dog = world.feral_dog(20, 0, 0);
        let mut game = world.build();
        game.game.res.time.tick = game.game.res.time.ticks_until(hour, 0);
        for _ in 0..3 {
            game.game.res.player_state.intent = PlayerIntent::movement(0, 0, 0, 200);
            game.tick();
        }
        game.pos(dog).unwrap()
    }

    #[test]
    fn dogs_notice_the_player_from_farther_away_at_night() {
        assert_eq!(dog_after_turns(12).x, 20, "at noon 20 tiles is too far");
        assert!(dog_after_turns(0).x < 20, "at midnight the dog gives chase");
    }
}

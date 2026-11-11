use crate::{
    components::{BattleDelay, Combat, Dead, GameEntity, Position},
    game::GameTickResult,
    moves::ActionQueue,
    resources::Resources,
    systems::{get_player_entity, get_player_position},
};
use hecs::World;

/// Centralized combat state management to ensure consistency
/// across all systems and prevent state desynchronization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CombatState {
    /// Not in combat, no enemies nearby
    #[default]
    Idle,
    /// Combat is active, enemies are nearby
    Active,
    /// Combat just ended this tick (transitional state)
    JustEnded,
}

/// Centralized manager for all combat state transitions
pub struct CombatStateManager;

impl CombatStateManager {
    /// Main function to update combat state each tick
    /// Returns the new state and any tick results that should be communicated
    pub fn update_combat_state(
        world: &mut World,
        res: &mut Resources,
    ) -> (CombatState, GameTickResult) {
        let current_state = Self::get_current_state(res);
        let should_be_in_combat = Self::should_be_in_combat(world, res);

        match (current_state, should_be_in_combat) {
            // Not in combat, enemies appear -> start combat
            (CombatState::Idle, true) => {
                Self::start_combat(world, res);
                (CombatState::Active, GameTickResult::CombatTriggered)
            }
            // In combat, enemies still present -> update triggered flags for new enemies
            (CombatState::Active, true) => {
                Self::update_combat_participants(world, res);
                (CombatState::Active, GameTickResult::empty())
            }
            // In combat, no enemies -> end combat
            (CombatState::Active, false) => {
                Self::end_combat(world, res);
                (CombatState::JustEnded, GameTickResult::CombatEnded)
            }
            // Combat just ended -> return to idle
            (CombatState::JustEnded, _) => {
                res.player_state.combat_ended_this_tick = false;
                (CombatState::Idle, GameTickResult::empty())
            }
            // No state change needed
            (state, _) => (state, GameTickResult::empty()),
        }
    }

    /// Get the current combat state from player state
    fn get_current_state(res: &Resources) -> CombatState {
        if res.player_state.combat_ended_this_tick {
            CombatState::JustEnded
        } else if res.player_state.combat_active {
            CombatState::Active
        } else {
            CombatState::Idle
        }
    }

    /// Determine if combat should be active based on nearby enemies
    fn should_be_in_combat(world: &World, res: &Resources) -> bool {
        // Get player position
        let player_pos = match get_player_position(world) {
            Some(pos) => pos,
            None => return false, // No player
        };

        // If already in combat, check if it should end
        if res.player_state.combat_active {
            return Self::should_continue_combat(world, player_pos);
        }

        // Not in combat, check if it should start
        !Self::engaging(world, res, player_pos).is_empty()
    }

    /// Living combatants without a battle delay that fight the player now: beside them,
    /// or able to shoot them (`ranged::engages`)
    fn engaging(world: &World, res: &Resources, player_pos: Position) -> Vec<hecs::Entity> {
        world
            .query::<(&Position, &Combat, &GameEntity)>()
            .iter()
            .filter(|&(entity, (pos, _, _))| {
                world.get::<&Dead>(entity).is_err()
                    && world.get::<&BattleDelay>(entity).is_err()
                    && crate::systems::ranged::engages(world, res, entity, *pos, player_pos)
            })
            .map(|(entity, _)| entity)
            .collect()
    }

    /// Mark every combatant that fights the player as in combat
    fn trigger_engaging(world: &mut World, res: &Resources, player_pos: Position) {
        for entity in Self::engaging(world, res, player_pos) {
            if let Ok(mut combat) = world.get::<&mut Combat>(entity) {
                combat.triggered = true;
            }
        }
    }

    /// Check if combat should continue (any living enemies nearby)
    fn should_continue_combat(world: &World, player_pos: Position) -> bool {
        const COMBAT_END_DISTANCE: f32 = 8.0; // Slightly larger than trigger distance

        for (entity, (pos, _, _)) in world.query::<(&Position, &Combat, &GameEntity)>().iter() {
            // Skip dead entities
            if world.get::<&Dead>(entity).is_ok() {
                continue;
            }

            let distance = ((pos.x as f32 - player_pos.x as f32).powi(2)
                + (pos.y as f32 - player_pos.y as f32).powi(2)
                + (pos.z as f32 - player_pos.z as f32).powi(2))
            .sqrt();

            if distance <= COMBAT_END_DISTANCE {
                return true;
            }
        }

        false
    }

    /// Update combat participants - ensure all adjacent enemies have triggered = true
    /// This is called every tick while in combat to handle enemies joining mid-combat
    fn update_combat_participants(world: &mut World, res: &mut Resources) {
        if let Some(player_pos) = get_player_position(world) {
            Self::trigger_engaging(world, res, player_pos);
        }
    }

    /// Start combat - set all necessary state and components
    fn start_combat(world: &mut World, res: &mut Resources) {
        res.player_state.combat_active = true;
        res.player_state.combat_ended_this_tick = false;

        let Some(player_pos) = get_player_position(world) else {
            return; // No player to start combat with
        };
        Self::trigger_engaging(world, res, player_pos);

        // Ensure player has an ActionQueue component for combat
        if let Some(player_entity) = get_player_entity(world) {
            if world.get::<&ActionQueue>(player_entity).is_err() {
                world.insert_one(player_entity, ActionQueue::new()).ok();
            }
        }
    }

    /// End combat - clean up state and components
    fn end_combat(world: &mut World, res: &mut Resources) {
        res.player_state.combat_active = false;
        res.player_state.combat_ended_this_tick = true;
        res.player_state.last_combat_end_tick = res.time.tick;

        // Clear player's action queue
        if let Some(player_entity) = get_player_entity(world) {
            if let Ok(mut queue) = world.get::<&mut ActionQueue>(player_entity) {
                queue.clear();
            }
        }
    }

    /// Force end combat (for escape, death, etc.)
    pub fn force_end_combat(world: &mut World, res: &mut Resources) {
        Self::end_combat(world, res);
    }

    /// The living enemies in the fight, nearest first: the combat panel's cards and the
    /// targets the player picks from, in the same order
    pub fn enemies(world: &World) -> Vec<hecs::Entity> {
        let Some(player_pos) = get_player_position(world) else {
            return vec![];
        };
        let mut enemies: Vec<(i64, u64, hecs::Entity)> = world
            .query::<(&Position, &Combat, &GameEntity)>()
            .iter()
            .filter(|&(entity, (_, combat, _))| {
                combat.triggered
                    && world.get::<&Dead>(entity).is_err()
                    && world.get::<&crate::components::Player>(entity).is_err()
            })
            .map(|(entity, (pos, _, _))| {
                let d = crate::systems::ranged::tile_distance(*pos, player_pos);
                (d, entity.to_bits().get(), entity)
            })
            .collect();
        enemies.sort();
        enemies.into_iter().map(|(_, _, e)| e).collect()
    }

    /// Check if currently in combat
    pub fn is_in_combat(res: &Resources) -> bool {
        res.player_state.combat_active
    }

    /// Check if combat just ended this tick
    pub fn combat_just_ended(res: &Resources) -> bool {
        res.player_state.combat_ended_this_tick
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{Health, Position};
    use crate::game::GameTickResult;
    use crate::intent::PlayerAction;
    use crate::test_support::TestWorld;
    use crate::tiles::TileKind;

    fn arena() -> TestWorld {
        TestWorld::new(7).fill((-6, -6, 0), (6, 6, 0), TileKind::Grass)
    }

    #[test]
    fn walking_up_to_a_feral_dog_starts_combat() {
        let mut world = arena();
        let dog = world.feral_dog(2, 0, 0);
        let mut game = world.build();
        assert!(!CombatStateManager::is_in_combat(&game.game.res));

        game.act(PlayerAction::Move {
            dx: 1,
            dy: 0,
            dz: 0,
        });
        let result = game.tick();

        assert_eq!(game.player_pos(), Position { x: 1, y: 0, z: 0 });
        assert!(result.contains(GameTickResult::CombatTriggered));
        assert!(CombatStateManager::is_in_combat(&game.game.res));
        assert!(game.pos(dog).is_some());
    }

    #[test]
    fn combat_ends_when_the_dog_reaches_zero_health() {
        let mut world = arena();
        let dog = world.feral_dog(1, 0, 0);
        let mut game = world.build();
        assert!(game.tick().contains(GameTickResult::CombatTriggered));

        // Same path as a killing blow in combat_actions: health to zero, then the death handler
        let dog = game.entity(dog);
        game.game.world.get::<&mut Health>(dog).unwrap().current = 0;
        crate::systems::handle_entity_death(&mut game.game.world, &mut game.game.res, dog);
        // The next tick sees no living enemy nearby and ends combat; it stays over,
        // since the corpse is not a combatant
        game.tick();
        assert!(!CombatStateManager::is_in_combat(&game.game.res));
        game.ticks(3);
        assert!(!CombatStateManager::is_in_combat(&game.game.res));
        let corpses = game
            .game
            .world
            .query::<&crate::components::EntityKind>()
            .iter()
            .filter(|(_, k)| **k == crate::components::EntityKind::Corpse)
            .count();
        assert_eq!(corpses, 1);
    }

    #[test]
    fn walking_away_reports_combat_ended() {
        let mut world = arena();
        world.feral_dog(1, 0, 0);
        let mut game = world.build();
        assert!(game.tick().contains(GameTickResult::CombatTriggered));

        // Step well outside the combat end distance
        let player = game.game.get_player_entity().unwrap();
        *game.game.world.get::<&mut Position>(player).unwrap() = Position { x: 40, y: 40, z: 0 };
        let result = game.tick();

        assert!(!CombatStateManager::is_in_combat(&game.game.res));
        assert!(
            result.contains(GameTickResult::CombatEnded),
            "the tick that ends combat must report CombatEnded so the client closes the panel"
        );
        // Reported once, not every tick after
        assert!(!game.tick().contains(GameTickResult::CombatEnded));
    }
}

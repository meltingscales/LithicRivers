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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatState {
    /// Not in combat, no enemies nearby
    Idle,
    /// Combat is active, enemies are nearby
    Active,
    /// Combat just ended this tick (transitional state)
    JustEnded,
}

impl Default for CombatState {
    fn default() -> Self {
        Self::Idle
    }
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
        Self::should_start_combat(world, player_pos)
    }

    /// Check if combat should start (enemies nearby)
    /// Uses exact adjacency logic like the original system for compatibility
    fn should_start_combat(world: &World, player_pos: Position) -> bool {
        // Define the 8 adjacent positions around the player (original logic)
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

        for (entity, (pos, _, _)) in world.query::<(&Position, &Combat, &GameEntity)>().iter() {
            // Skip dead entities
            if world.get::<&Dead>(entity).is_ok() {
                continue;
            }

            // Skip entities with BattleDelay
            if world.get::<&BattleDelay>(entity).is_ok() {
                continue;
            }

            // Check if entity is in any of the 8 adjacent positions
            let is_adjacent = adjacent_positions
                .iter()
                .any(|&(adj_x, adj_y)| pos.x == adj_x && pos.y == adj_y && pos.z == player_pos.z);

            if is_adjacent {
                return true;
            }
        }

        false
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

    /// Start combat - set all necessary state and components
    fn start_combat(world: &mut World, res: &mut Resources) {
        res.player_state.combat_active = true;
        res.player_state.combat_ended_this_tick = false;

        // Get player position for adjacency checks
        let player_pos = match get_player_position(world) {
            Some(pos) => pos,
            None => return, // No player to start combat with
        };

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

        // Set combat.triggered = true for adjacent combat entities (original behavior)
        for (entity, (pos, combat, _)) in world
            .query::<(&Position, &mut Combat, &GameEntity)>()
            .iter()
        {
            // Skip dead entities
            if world.get::<&Dead>(entity).is_ok() {
                continue;
            }

            // Skip entities with BattleDelay
            if world.get::<&BattleDelay>(entity).is_ok() {
                continue;
            }

            // Check if entity is in any of the 8 adjacent positions
            let is_adjacent = adjacent_positions
                .iter()
                .any(|&(adj_x, adj_y)| pos.x == adj_x && pos.y == adj_y && pos.z == player_pos.z);

            if is_adjacent {
                combat.triggered = true;
            }
        }

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

    /// Check if currently in combat
    pub fn is_in_combat(res: &Resources) -> bool {
        res.player_state.combat_active
    }

    /// Check if combat just ended this tick
    pub fn combat_just_ended(res: &Resources) -> bool {
        res.player_state.combat_ended_this_tick
    }
}

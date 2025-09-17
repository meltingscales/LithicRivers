use crate::components::{BattleDelay, Player, Position};
use hecs::World;

/// Check if an entity is stunned (has BattleDelay component)
pub fn is_stunned(world: &World, entity: hecs::Entity) -> bool {
    world.get::<&BattleDelay>(entity).is_ok()
}

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
pub fn is_player_entity(world: &World, entity: hecs::Entity) -> bool {
    world.get::<&Player>(entity).is_ok()
}

use crate::components::{FogOfWar, LightSource, Player, Position};
use hecs::World;

/// Update fog of war for the player based on their current position and light radius
pub fn update_fog_of_war(world: &mut World) {
    // Find the player entity with fog of war, light source, and position
    for (_entity, (fog_of_war, light_source, position, _player)) in world
        .query::<(&mut FogOfWar, &LightSource, &Position, &Player)>()
        .iter()
    {
        // Mark all tiles within light radius as visited
        let radius = light_source.radius as i64;

        for dx in -radius..=radius {
            for dy in -radius..=radius {
                // Use circular distance for more natural light falloff
                let distance_sq = dx * dx + dy * dy;
                if distance_sq <= radius * radius {
                    let tile_x = position.x + dx;
                    let tile_y = position.y + dy;
                    let tile_z = position.z;

                    fog_of_war.mark_visited(tile_x, tile_y, tile_z);
                }
            }
        }
    }
}

/// Check if a tile is illuminated by the player's current light source
pub fn is_tile_illuminated(world: &World, tile_x: i64, tile_y: i64, tile_z: i64) -> bool {
    // Find the player entity with light source and position
    for (_entity, (light_source, position, _player)) in
        world.query::<(&LightSource, &Position, &Player)>().iter()
    {
        let radius = light_source.radius as i64;
        let dx = tile_x - position.x;
        let dy = tile_y - position.y;
        let dz = tile_z - position.z;

        // Only illuminate tiles on the same Z level
        if dz != 0 {
            return false;
        }

        // Check if tile is within light radius using circular distance
        let distance_sq = dx * dx + dy * dy;
        if distance_sq <= radius * radius {
            return true;
        }
    }

    false
}

/// Check if a tile has been visited by the player
pub fn is_tile_visited(world: &World, tile_x: i64, tile_y: i64, tile_z: i64) -> bool {
    // Find the player entity with fog of war
    for (_entity, (fog_of_war, _player)) in world.query::<(&FogOfWar, &Player)>().iter() {
        return fog_of_war.is_visited(tile_x, tile_y, tile_z);
    }

    false
}

/// Get the fog of war state for a tile
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FogState {
    Unvisited,   // Black '?'
    Visited,     // Grayscale
    Illuminated, // Normal colors
}

pub fn get_fog_state(world: &World, tile_x: i64, tile_y: i64, tile_z: i64) -> FogState {
    // Disable fog of war at surface level and above (Z >= 0)
    if tile_z >= 0 {
        return FogState::Illuminated;
    }

    let illuminated = is_tile_illuminated(world, tile_x, tile_y, tile_z);
    let visited = is_tile_visited(world, tile_x, tile_y, tile_z);

    if illuminated {
        FogState::Illuminated
    } else if visited {
        FogState::Visited
    } else {
        FogState::Unvisited
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{GameEntity, Glyph, Inventory, ItemKind, SpriteRef};
    use hecs::World;

    fn create_test_world_with_player() -> (World, hecs::Entity) {
        let mut world = World::new();

        // Create a player with fog of war and light source components
        let mut inventory = Inventory::default();
        inventory.add(ItemKind::Torch, 2); // Add torches for testing

        let player_entity = world.spawn((
            Position {
                x: 10,
                y: 10,
                z: -1,
            }, // Underground so fog of war is active
            Player,
            GameEntity,
            Glyph('@'),
            SpriteRef::new("entities", "player"),
            FogOfWar::default(),
            LightSource::default(), // Default 2-tile radius, torch not equipped
            inventory,
        ));

        (world, player_entity)
    }

    #[test]
    fn test_fog_of_war_initial_state() {
        let (world, _player) = create_test_world_with_player();

        // Player is at (10, 10, -1)
        // Test tiles at various distances

        // Tile at player position should be illuminated (distance 0)
        assert_eq!(get_fog_state(&world, 10, 10, -1), FogState::Illuminated);

        // Tile within default light radius (2 tiles) should be illuminated
        assert_eq!(get_fog_state(&world, 11, 10, -1), FogState::Illuminated);
        assert_eq!(get_fog_state(&world, 10, 12, -1), FogState::Illuminated);

        // Tile exactly at edge of radius (distance = 2) should be illuminated
        assert_eq!(get_fog_state(&world, 12, 10, -1), FogState::Illuminated);

        // Tile outside light radius should be unvisited
        assert_eq!(get_fog_state(&world, 15, 10, -1), FogState::Unvisited);
        assert_eq!(get_fog_state(&world, 10, 15, -1), FogState::Unvisited);

        // Tile on different Z level should be unvisited
        assert_eq!(get_fog_state(&world, 10, 10, -2), FogState::Unvisited);
    }

    #[test]
    fn test_fog_of_war_after_movement() {
        let (mut world, player_entity) = create_test_world_with_player();

        // Initial update - player at (10, 10, -1)
        update_fog_of_war(&mut world);

        // Move player to (15, 10, -1)
        if let Ok(mut pos) = world.get::<&mut Position>(player_entity) {
            pos.x = 15;
        }

        // Update fog of war after movement
        update_fog_of_war(&mut world);

        // New position should be illuminated
        assert_eq!(get_fog_state(&world, 15, 10, -1), FogState::Illuminated);
        assert_eq!(get_fog_state(&world, 16, 10, -1), FogState::Illuminated);

        // Old position should be visited but not illuminated
        assert_eq!(get_fog_state(&world, 10, 10, -1), FogState::Visited);
        assert_eq!(get_fog_state(&world, 11, 10, -1), FogState::Visited);

        // Tiles never visited should remain unvisited
        assert_eq!(get_fog_state(&world, 5, 5, -1), FogState::Unvisited);
    }

    #[test]
    fn test_torch_toggle_functionality() {
        let (mut world, player_entity) = create_test_world_with_player();

        // Initially torch should not be equipped
        {
            let light_source = world.get::<&LightSource>(player_entity).unwrap();
            assert!(!light_source.torch_equipped);
            assert_eq!(light_source.radius, 2);
        }

        // Equip torch
        {
            let mut light_source = world.get::<&mut LightSource>(player_entity).unwrap();
            light_source.set_torch_equipped(true);
        }

        // Check torch is equipped and radius increased
        {
            let light_source = world.get::<&LightSource>(player_entity).unwrap();
            assert!(light_source.torch_equipped);
            assert_eq!(light_source.radius, 8);
        }

        // Test illumination with torch equipped
        update_fog_of_war(&mut world);

        // Player at (10, 10, -1) with torch radius 8
        assert_eq!(get_fog_state(&world, 10, 10, -1), FogState::Illuminated);
        assert_eq!(get_fog_state(&world, 18, 10, -1), FogState::Illuminated); // 8 tiles away
        assert_eq!(get_fog_state(&world, 19, 10, -1), FogState::Unvisited); // 9 tiles away (outside radius)

        // Test diagonal distance (should use circular distance)
        // Distance to (16, 16) from (10, 10) = sqrt(36 + 36) = sqrt(72) ≈ 8.49 > 8
        assert_eq!(get_fog_state(&world, 16, 16, -1), FogState::Unvisited);
        // Distance to (15, 15) from (10, 10) = sqrt(25 + 25) = sqrt(50) ≈ 7.07 < 8
        assert_eq!(get_fog_state(&world, 15, 15, -1), FogState::Illuminated);
    }

    #[test]
    fn test_torch_toggle_without_torch_in_inventory() {
        let mut world = World::new();

        // Create player without torch in inventory
        let empty_inventory = Inventory::default();
        let player_entity = world.spawn((
            Position {
                x: 10,
                y: 10,
                z: -1,
            },
            Player,
            GameEntity,
            Glyph('@'),
            SpriteRef::new("entities", "player"),
            FogOfWar::default(),
            LightSource::default(),
            empty_inventory,
        ));

        // Try to equip torch when none available
        let has_torch = {
            let inventory = world.get::<&Inventory>(player_entity).unwrap();
            inventory
                .slots
                .iter()
                .any(|stack| stack.kind == ItemKind::Torch && stack.qty > 0)
        };

        assert!(!has_torch, "Player should not have torch in inventory");

        // Light source should remain at default radius
        let light_source = world.get::<&LightSource>(player_entity).unwrap();
        assert_eq!(light_source.radius, 2);
        assert!(!light_source.torch_equipped);
    }

    #[test]
    fn test_circular_light_radius() {
        let (mut world, _player) = create_test_world_with_player();
        update_fog_of_war(&mut world);

        // Player at (10, 10, -1) with radius 2
        // Test that light follows circular pattern

        // Cardinal directions at exactly radius 2 should be illuminated
        assert_eq!(get_fog_state(&world, 12, 10, -1), FogState::Illuminated); // East
        assert_eq!(get_fog_state(&world, 8, 10, -1), FogState::Illuminated); // West
        assert_eq!(get_fog_state(&world, 10, 12, -1), FogState::Illuminated); // South
        assert_eq!(get_fog_state(&world, 10, 8, -1), FogState::Illuminated); // North

        // Diagonal corners at distance > 2 should not be illuminated
        // Distance to (12, 12) = sqrt(4 + 4) = sqrt(8) ≈ 2.83 > 2
        assert_eq!(get_fog_state(&world, 12, 12, -1), FogState::Unvisited);

        // Points closer than radius should be illuminated
        // Distance to (11, 11) = sqrt(1 + 1) = sqrt(2) ≈ 1.41 < 2
        assert_eq!(get_fog_state(&world, 11, 11, -1), FogState::Illuminated);
    }

    #[test]
    fn test_z_level_isolation() {
        let (mut world, player_entity) = create_test_world_with_player();

        // Move player to Z level -5 (underground)
        if let Ok(mut pos) = world.get::<&mut Position>(player_entity) {
            pos.z = -5;
        }

        update_fog_of_war(&mut world);

        // Player at (10, 10, -5) should illuminate tiles on same Z level
        assert_eq!(get_fog_state(&world, 10, 10, -5), FogState::Illuminated);
        assert_eq!(get_fog_state(&world, 11, 10, -5), FogState::Illuminated);

        // Tiles on different underground Z levels should not be illuminated
        assert_eq!(get_fog_state(&world, 10, 10, -4), FogState::Unvisited);
        assert_eq!(get_fog_state(&world, 10, 10, -6), FogState::Unvisited);
    }

    #[test]
    fn test_fog_of_war_disabled_at_surface_and_above() {
        let (world, _player) = create_test_world_with_player();

        // Test that fog of war is disabled at Z >= 0
        // All tiles at surface level (Z=0) and above should be illuminated regardless of distance
        assert_eq!(get_fog_state(&world, 0, 0, 0), FogState::Illuminated);
        assert_eq!(get_fog_state(&world, 100, 100, 0), FogState::Illuminated);
        assert_eq!(get_fog_state(&world, -100, -100, 0), FogState::Illuminated);

        // Test above ground levels (Z > 0)
        assert_eq!(get_fog_state(&world, 0, 0, 1), FogState::Illuminated);
        assert_eq!(get_fog_state(&world, 100, 100, 5), FogState::Illuminated);

        // Test underground levels still have fog of war
        assert_eq!(get_fog_state(&world, 100, 100, -1), FogState::Unvisited);
        assert_eq!(get_fog_state(&world, 100, 100, -5), FogState::Unvisited);
    }
}

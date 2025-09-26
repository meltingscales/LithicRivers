use crate::components::{FogOfWar, LightSource, Player, Position};
use crate::resources::Resources;
use hecs::World;

/// Bresenham line algorithm to get all points on a line from start to end
fn bresenham_line(start_x: i64, start_y: i64, end_x: i64, end_y: i64) -> Vec<(i64, i64)> {
    let mut points = Vec::new();

    let dx = (end_x - start_x).abs();
    let dy = (end_y - start_y).abs();
    let sx = if start_x < end_x { 1 } else { -1 };
    let sy = if start_y < end_y { 1 } else { -1 };
    let mut err = dx - dy;

    let mut x = start_x;
    let mut y = start_y;

    loop {
        points.push((x, y));

        if x == end_x && y == end_y {
            break;
        }

        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }

    points
}

/// Check if a tile blocks light using line-of-sight raycasting
fn has_line_of_sight(
    _world: &World,
    resources: &Resources,
    from_x: i64,
    from_y: i64,
    to_x: i64,
    to_y: i64,
    z: i64,
) -> bool {
    // Get all points on the line from source to target
    let line_points = bresenham_line(from_x, from_y, to_x, to_y);

    // Check each point along the line (excluding start and end points)
    // We want to be able to see the target tile even if it's a wall
    if line_points.len() > 2 {
        for (point_x, point_y) in line_points.iter().skip(1).take(line_points.len() - 2) {
            let tile = resources
                .world_state
                .world
                .get_tile_at_z_no_worldgen(*point_x, *point_y, z);

            // If we hit a non-passable tile (wall), light is blocked
            if !tile.is_passable() {
                return false;
            }
        }
    }

    true
}

/// Update fog of war for the player based on their current position and light radius with line-of-sight
pub fn update_fog_of_war(world: &mut World, resources: &Resources) {
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

                    // Check line-of-sight before marking as visited
                    if has_line_of_sight(
                        world, resources, position.x, position.y, tile_x, tile_y, tile_z,
                    ) {
                        fog_of_war.mark_visited(tile_x, tile_y, tile_z);
                    }
                }
            }
        }
    }
}

/// Check if a tile is illuminated by the player's current light source with line-of-sight
pub fn is_tile_illuminated(
    world: &World,
    resources: &Resources,
    tile_x: i64,
    tile_y: i64,
    tile_z: i64,
) -> bool {
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
            // Check line-of-sight from player to tile
            return has_line_of_sight(
                world, resources, position.x, position.y, tile_x, tile_y, tile_z,
            );
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

pub fn get_fog_state(
    world: &World,
    resources: &Resources,
    tile_x: i64,
    tile_y: i64,
    tile_z: i64,
) -> FogState {
    // Disable fog of war at surface level and above (Z >= 0)
    if tile_z >= 0 {
        return FogState::Illuminated;
    }

    let illuminated = is_tile_illuminated(world, resources, tile_x, tile_y, tile_z);
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
    use crate::resources::Resources;
    use crate::world::GameWorld;
    use hecs::World;

    fn create_mock_resources() -> Resources {
        // Create a simple test world with mostly passable tiles
        let mut game_world = GameWorld::new(100, 100, 42); // Use a test seed

        // Fill the test world with mostly passable tiles (dirt)
        for x in -50..50 {
            for y in -50..50 {
                for z in -10..10 {
                    game_world.set_tile_cached(x, y, z, crate::tile_registry::TileKind::Dirt);
                }
            }
        }

        let mut resources = Resources::new(42);

        // Override the world with our test world
        resources.world_state.world = game_world;

        resources
    }

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
        let resources = create_mock_resources();

        // Player is at (10, 10, -1)
        // Test tiles at various distances

        // Tile at player position should be illuminated (distance 0)
        assert_eq!(
            get_fog_state(&world, &resources, 10, 10, -1),
            FogState::Illuminated
        );

        // Tile within default light radius (2 tiles) should be illuminated
        assert_eq!(
            get_fog_state(&world, &resources, 11, 10, -1),
            FogState::Illuminated
        );
        assert_eq!(
            get_fog_state(&world, &resources, 10, 12, -1),
            FogState::Illuminated
        );

        // Tile exactly at edge of radius (distance = 2) should be illuminated
        assert_eq!(
            get_fog_state(&world, &resources, 12, 10, -1),
            FogState::Illuminated
        );

        // Tile outside light radius should be unvisited
        assert_eq!(
            get_fog_state(&world, &resources, 15, 10, -1),
            FogState::Unvisited
        );
        assert_eq!(
            get_fog_state(&world, &resources, 10, 15, -1),
            FogState::Unvisited
        );

        // Tile on different Z level should be unvisited
        assert_eq!(
            get_fog_state(&world, &resources, 10, 10, -2),
            FogState::Unvisited
        );
    }

    #[test]
    fn test_fog_of_war_after_movement() {
        let (mut world, player_entity) = create_test_world_with_player();

        // Create mock resources for testing
        let resources = create_mock_resources();

        // Initial update - player at (10, 10, -1)
        update_fog_of_war(&mut world, &resources);

        // Move player to (15, 10, -1)
        if let Ok(mut pos) = world.get::<&mut Position>(player_entity) {
            pos.x = 15;
        }

        // Update fog of war after movement
        update_fog_of_war(&mut world, &resources);

        // New position should be illuminated
        assert_eq!(
            get_fog_state(&world, &resources, 15, 10, -1),
            FogState::Illuminated
        );
        assert_eq!(
            get_fog_state(&world, &resources, 16, 10, -1),
            FogState::Illuminated
        );

        // Old position should be visited but not illuminated
        assert_eq!(
            get_fog_state(&world, &resources, 10, 10, -1),
            FogState::Visited
        );
        assert_eq!(
            get_fog_state(&world, &resources, 11, 10, -1),
            FogState::Visited
        );

        // Tiles never visited should remain unvisited
        assert_eq!(
            get_fog_state(&world, &resources, 5, 5, -1),
            FogState::Unvisited
        );
    }

    #[test]
    fn test_torch_toggle_functionality() {
        let (mut world, player_entity) = create_test_world_with_player();
        let resources = create_mock_resources();

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
        update_fog_of_war(&mut world, &resources);

        // Player at (10, 10, -1) with torch radius 8
        assert_eq!(
            get_fog_state(&world, &resources, 10, 10, -1),
            FogState::Illuminated
        );
        assert_eq!(
            get_fog_state(&world, &resources, 18, 10, -1),
            FogState::Illuminated
        ); // 8 tiles away
        assert_eq!(
            get_fog_state(&world, &resources, 19, 10, -1),
            FogState::Unvisited
        ); // 9 tiles away (outside radius)

        // Test diagonal distance (should use circular distance)
        // Distance to (16, 16) from (10, 10) = sqrt(36 + 36) = sqrt(72) ≈ 8.49 > 8
        assert_eq!(
            get_fog_state(&world, &resources, 16, 16, -1),
            FogState::Unvisited
        );
        // Distance to (15, 15) from (10, 10) = sqrt(25 + 25) = sqrt(50) ≈ 7.07 < 8
        assert_eq!(
            get_fog_state(&world, &resources, 15, 15, -1),
            FogState::Illuminated
        );
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
        let resources = create_mock_resources();
        update_fog_of_war(&mut world, &resources);

        // Player at (10, 10, -1) with radius 2
        // Test that light follows circular pattern

        // Cardinal directions at exactly radius 2 should be illuminated
        assert_eq!(
            get_fog_state(&world, &resources, 12, 10, -1),
            FogState::Illuminated
        ); // East
        assert_eq!(
            get_fog_state(&world, &resources, 8, 10, -1),
            FogState::Illuminated
        ); // West
        assert_eq!(
            get_fog_state(&world, &resources, 10, 12, -1),
            FogState::Illuminated
        ); // South
        assert_eq!(
            get_fog_state(&world, &resources, 10, 8, -1),
            FogState::Illuminated
        ); // North

        // Diagonal corners at distance > 2 should not be illuminated
        // Distance to (12, 12) = sqrt(4 + 4) = sqrt(8) ≈ 2.83 > 2
        assert_eq!(
            get_fog_state(&world, &resources, 12, 12, -1),
            FogState::Unvisited
        );

        // Points closer than radius should be illuminated
        // Distance to (11, 11) = sqrt(1 + 1) = sqrt(2) ≈ 1.41 < 2
        assert_eq!(
            get_fog_state(&world, &resources, 11, 11, -1),
            FogState::Illuminated
        );
    }

    #[test]
    fn test_z_level_isolation() {
        let (mut world, player_entity) = create_test_world_with_player();
        let resources = create_mock_resources();

        // Move player to Z level -5 (underground)
        if let Ok(mut pos) = world.get::<&mut Position>(player_entity) {
            pos.z = -5;
        }

        update_fog_of_war(&mut world, &resources);

        // Player at (10, 10, -5) should illuminate tiles on same Z level
        assert_eq!(
            get_fog_state(&world, &resources, 10, 10, -5),
            FogState::Illuminated
        );
        assert_eq!(
            get_fog_state(&world, &resources, 11, 10, -5),
            FogState::Illuminated
        );

        // Tiles on different underground Z levels should not be illuminated
        assert_eq!(
            get_fog_state(&world, &resources, 10, 10, -4),
            FogState::Unvisited
        );
        assert_eq!(
            get_fog_state(&world, &resources, 10, 10, -6),
            FogState::Unvisited
        );
    }

    #[test]
    fn test_fog_of_war_disabled_at_surface_and_above() {
        let (world, _player) = create_test_world_with_player();
        let resources = create_mock_resources();

        // Test that fog of war is disabled at Z >= 0
        // All tiles at surface level (Z=0) and above should be illuminated regardless of distance
        assert_eq!(
            get_fog_state(&world, &resources, 0, 0, 0),
            FogState::Illuminated
        );
        assert_eq!(
            get_fog_state(&world, &resources, 100, 100, 0),
            FogState::Illuminated
        );
        assert_eq!(
            get_fog_state(&world, &resources, -100, -100, 0),
            FogState::Illuminated
        );

        // Test above ground levels (Z > 0)
        assert_eq!(
            get_fog_state(&world, &resources, 0, 0, 1),
            FogState::Illuminated
        );
        assert_eq!(
            get_fog_state(&world, &resources, 100, 100, 5),
            FogState::Illuminated
        );

        // Test underground levels still have fog of war
        assert_eq!(
            get_fog_state(&world, &resources, 100, 100, -1),
            FogState::Unvisited
        );
        assert_eq!(
            get_fog_state(&world, &resources, 100, 100, -5),
            FogState::Unvisited
        );
    }

    #[test]
    fn test_raycasting_through_walls() {
        let (mut world, _player) = create_test_world_with_player();
        let mut resources = create_mock_resources();

        // Create a wall between player (10, 10, -1) and target (15, 10, -1)
        resources.world_state.world.set_tile_cached(
            12,
            10,
            -1,
            crate::tile_registry::TileKind::Rock,
        );

        update_fog_of_war(&mut world, &resources);

        // Player at (10, 10, -1) should illuminate tiles up to the wall
        assert_eq!(
            get_fog_state(&world, &resources, 10, 10, -1),
            FogState::Illuminated
        ); // Player position
        assert_eq!(
            get_fog_state(&world, &resources, 11, 10, -1),
            FogState::Illuminated
        ); // Before wall
        assert_eq!(
            get_fog_state(&world, &resources, 12, 10, -1),
            FogState::Illuminated
        ); // Wall itself (can see the wall)

        // Tiles behind the wall should not be illuminated (light blocked)
        assert_eq!(
            get_fog_state(&world, &resources, 13, 10, -1),
            FogState::Unvisited
        ); // Behind wall
        assert_eq!(
            get_fog_state(&world, &resources, 15, 10, -1),
            FogState::Unvisited
        ); // Further behind wall

        // Light should still work in other directions not blocked by walls
        assert_eq!(
            get_fog_state(&world, &resources, 10, 11, -1),
            FogState::Illuminated
        ); // South
        assert_eq!(
            get_fog_state(&world, &resources, 10, 9, -1),
            FogState::Illuminated
        ); // North
        assert_eq!(
            get_fog_state(&world, &resources, 9, 10, -1),
            FogState::Illuminated
        ); // West
    }
}

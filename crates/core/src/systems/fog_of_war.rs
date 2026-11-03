use crate::components::{FogOfWar, Inventory, ItemKind, LightSource, Player, Position};
use crate::factory::{FactoryBlock, FactoryKind};
use crate::model::body::Body;
use crate::resources::Resources;
use hecs::World;
use std::collections::HashSet;

/// How far a placed lantern's light reaches, in tiles (circular, like the player's light).
pub const LANTERN_RADIUS: i64 = 6;
/// Lanterns this close to the player (on each axis, on the player's layer) light tiles;
/// wider than any screen plus the lantern radius.
pub const LANTERN_GATHER_RANGE: i64 = 48;

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

/// Whether a tile stops sight and light: anything impassable except see-through blocks
/// (fences, gates and lanterns).
fn blocks_sight(world: &World, resources: &Resources, x: i64, y: i64, z: i64) -> bool {
    let tile = resources
        .world_state
        .world
        .get_tile_at_z_no_worldgen(x, y, z);
    if tile.is_passable() {
        return false;
    }
    let see_through = tile == crate::tiles::TileKind::FactoryBlock
        && resources
            .factory_index
            .get(&Position { x, y, z })
            .and_then(|&e| world.get::<&crate::factory::FactoryBlock>(e).ok())
            .is_some_and(|b| b.kind.see_through());
    !see_through
}

/// Whether nothing that stops sight stands between two tiles on a layer
pub fn has_line_of_sight(
    world: &World,
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
            if blocks_sight(world, resources, *point_x, *point_y, z) {
                return false;
            }
        }
    }

    true
}

/// Put out the player's torch once they no longer carry one (dropped, stored or crafted
/// away), so the light never stays lit without a torch.
pub fn put_out_missing_torch(world: &mut World, res: &mut Resources) {
    for (_, (light, inventory, _)) in world.query_mut::<(&mut LightSource, &Inventory, &Player)>() {
        let has_torch = inventory
            .slots
            .iter()
            .any(|stack| stack.kind == ItemKind::Torch && stack.qty > 0);
        if light.torch_equipped && !has_torch {
            light.set_torch_equipped(false);
            res.log("Your last torch is gone. Light radius reduced to 2.");
        }
    }
}

/// The radius a light source reaches, dimmed by a damaged or missing head.
pub fn light_radius(light_source: &LightSource, body: Option<&Body>) -> u32 {
    body.map(|b| b.effects())
        .unwrap_or_default()
        .light_radius(light_source.radius)
}

/// The light of the placed lanterns near a point, on that point's layer.
#[derive(Default)]
pub struct LanternLight {
    /// The tiles they light
    pub lit: HashSet<(i64, i64)>,
}

impl LanternLight {
    /// Gather the lanterns within `LANTERN_GATHER_RANGE` of `center` on its layer, and trace
    /// each one's light once.
    pub fn around(world: &World, resources: &Resources, center: Position) -> Self {
        let lanterns: Vec<(i64, i64)> = world
            .query::<(&Position, &FactoryBlock)>()
            .iter()
            .filter(|(_, (pos, block))| {
                block.kind == FactoryKind::ScrapLantern
                    && pos.z == center.z
                    && (pos.x - center.x).abs() <= LANTERN_GATHER_RANGE
                    && (pos.y - center.y).abs() <= LANTERN_GATHER_RANGE
            })
            .map(|(_, (pos, _))| (pos.x, pos.y))
            .collect();
        let mut lit = HashSet::new();
        for &(lx, ly) in &lanterns {
            for dx in -LANTERN_RADIUS..=LANTERN_RADIUS {
                for dy in -LANTERN_RADIUS..=LANTERN_RADIUS {
                    let tile = (lx + dx, ly + dy);
                    if dx * dx + dy * dy > LANTERN_RADIUS * LANTERN_RADIUS || lit.contains(&tile) {
                        continue;
                    }
                    #[cfg(any(test, feature = "test-support"))]
                    LANTERN_RAYS_CAST.with(|n| n.set(n.get() + 1));
                    if has_line_of_sight(world, resources, lx, ly, tile.0, tile.1, center.z) {
                        lit.insert(tile);
                    }
                }
            }
        }
        Self { lit }
    }
}

/// Update fog of war for the player based on their current position and light radius with line-of-sight
pub fn update_fog_of_war(world: &mut World, resources: &Resources) {
    let mut seen = lit_tiles(world, resources);
    // Lantern-lit tiles are remembered like tiles the player lights
    let center = world
        .query::<(&Position, &Player)>()
        .iter()
        .next()
        .map(|(_, (pos, _))| *pos);
    if let Some(center) = center {
        let placed = LanternLight::around(world, resources, center);
        seen.extend(placed.lit.iter().map(|&(x, y)| (x, y, center.z)));
    }
    for (_, (fog_of_war, _)) in world.query_mut::<(&mut FogOfWar, &Player)>() {
        for &(x, y, z) in &seen {
            fog_of_war.mark_visited(x, y, z);
        }
    }
}

/// The tiles the player's light reaches with line of sight.
fn lit_tiles(world: &World, resources: &Resources) -> Vec<(i64, i64, i64)> {
    let mut seen = Vec::new();
    for (_entity, (_fog, light_source, position, _player, body)) in world
        .query::<(&FogOfWar, &LightSource, &Position, &Player, Option<&Body>)>()
        .iter()
    {
        // Mark all tiles within light radius as visited
        let radius = light_radius(light_source, body) as i64;

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
                        seen.push((tile_x, tile_y, tile_z));
                    }
                }
            }
        }
    }
    seen
}

#[cfg(any(test, feature = "test-support"))]
thread_local! {
    /// How many fog lookups were built on this thread.
    static FOG_LOOKUPS_BUILT: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

#[cfg(any(test, feature = "test-support"))]
thread_local! {
    /// How many rays lanterns have cast on this thread.
    static LANTERN_RAYS_CAST: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// How many rays lanterns have cast on this thread, so tests can check that a lookup traces
/// each lantern's light once rather than once per tile drawn.
#[cfg(any(test, feature = "test-support"))]
pub fn lantern_rays_cast() -> u64 {
    LANTERN_RAYS_CAST.with(|n| n.get())
}

/// How many fog lookups have been built on this thread, so tests can check that drawing a
/// frame builds one rather than one per tile.
#[cfg(any(test, feature = "test-support"))]
pub fn fog_lookups_built() -> u64 {
    FOG_LOOKUPS_BUILT.with(|n| n.get())
}

/// What fog of war needs to know about the player, gathered once so many tiles (a whole
/// frame) can be checked without querying the world for each one.
pub struct FogLookup<'a> {
    world: &'a World,
    /// The player's position and light radius, if they carry a light
    light: Option<(Position, i64)>,
    /// The layer the player is on, and the lantern light around them there
    placed: Option<(i64, LanternLight)>,
    /// The tiles the player has seen
    memory: Option<hecs::Ref<'a, FogOfWar>>,
}

impl<'a> FogLookup<'a> {
    pub fn new(world: &'a World, resources: &Resources) -> Self {
        #[cfg(any(test, feature = "test-support"))]
        FOG_LOOKUPS_BUILT.with(|n| n.set(n.get() + 1));
        let light = world
            .query::<(&LightSource, &Position, &Player, Option<&Body>)>()
            .iter()
            .next()
            .map(|(_, (light, pos, _, body))| (*pos, light_radius(light, body) as i64));
        let memory = world
            .query::<(&FogOfWar, &Player)>()
            .iter()
            .next()
            .map(|(entity, _)| entity)
            .and_then(|entity| world.get::<&FogOfWar>(entity).ok());
        let placed = world
            .query::<(&Position, &Player)>()
            .iter()
            .next()
            .map(|(_, (pos, _))| (pos.z, LanternLight::around(world, resources, *pos)));
        Self {
            world,
            light,
            placed,
            memory,
        }
    }

    /// Whether the player's light reaches a tile, with line of sight.
    pub fn is_illuminated(
        &self,
        resources: &Resources,
        tile_x: i64,
        tile_y: i64,
        tile_z: i64,
    ) -> bool {
        let Some((z, placed)) = &self.placed else {
            return false;
        };
        // Only illuminate tiles on the player's Z level
        if tile_z != *z {
            return false;
        }
        if placed.lit.contains(&(tile_x, tile_y)) {
            return true;
        }
        let Some((position, radius)) = self.light else {
            return false;
        };
        // Check if tile is within light radius using circular distance
        let (dx, dy) = (tile_x - position.x, tile_y - position.y);
        dx * dx + dy * dy <= radius * radius
            && has_line_of_sight(
                self.world, resources, position.x, position.y, tile_x, tile_y, tile_z,
            )
    }

    /// Whether the player has seen a tile.
    pub fn is_visited(&self, tile_x: i64, tile_y: i64, tile_z: i64) -> bool {
        self.memory
            .as_ref()
            .is_some_and(|fog| fog.is_visited(tile_x, tile_y, tile_z))
    }

    /// The fog of war state for a tile.
    pub fn state(&self, resources: &Resources, tile_x: i64, tile_y: i64, tile_z: i64) -> FogState {
        // The surface (Z >= 0) is lit except at night; underground is always dark
        if (tile_z >= 0 && !resources.time.is_night())
            || self.is_illuminated(resources, tile_x, tile_y, tile_z)
        {
            FogState::Illuminated
        } else if self.is_visited(tile_x, tile_y, tile_z) {
            FogState::Visited
        } else {
            FogState::Unvisited
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
    FogLookup::new(world, resources).is_illuminated(resources, tile_x, tile_y, tile_z)
}

/// Check if a tile has been visited by the player
pub fn is_tile_visited(world: &World, tile_x: i64, tile_y: i64, tile_z: i64) -> bool {
    world
        .query::<(&FogOfWar, &Player)>()
        .iter()
        .next()
        .is_some_and(|(_, (fog_of_war, _))| fog_of_war.is_visited(tile_x, tile_y, tile_z))
}

/// Get the fog of war state for a tile
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FogState {
    Unvisited,   // Black '?'
    Visited,     // Grayscale
    Illuminated, // Normal colors
}

/// The fog of war state for one tile. To check many tiles, build a `FogLookup` once.
pub fn get_fog_state(
    world: &World,
    resources: &Resources,
    tile_x: i64,
    tile_y: i64,
    tile_z: i64,
) -> FogState {
    FogLookup::new(world, resources).state(resources, tile_x, tile_y, tile_z)
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
    fn a_damaged_head_dims_the_light_with_and_without_a_torch() {
        use crate::model::body::{BodyPartState, BodyPartType};
        let (mut world, player_entity) = create_test_world_with_player();
        let resources = create_mock_resources();
        let mut body = Body::default();
        body.parts.get_mut(&BodyPartType::Head).unwrap().state = BodyPartState::Damaged;
        world.insert_one(player_entity, body).unwrap();

        // Without a torch the radius of 2 drops to 1
        assert_eq!(
            get_fog_state(&world, &resources, 11, 10, -1),
            FogState::Illuminated
        );
        assert_eq!(
            get_fog_state(&world, &resources, 12, 10, -1),
            FogState::Unvisited
        );

        // With a torch the radius of 8 drops to 7
        world
            .get::<&mut LightSource>(player_entity)
            .unwrap()
            .set_torch_equipped(true);
        assert_eq!(
            get_fog_state(&world, &resources, 17, 10, -1),
            FogState::Illuminated
        );
        assert_ne!(
            get_fog_state(&world, &resources, 18, 10, -1),
            FogState::Illuminated
        );

        // With no head only the radius of 1 is left
        let mut body = world.get::<&mut Body>(player_entity).unwrap();
        body.parts.get_mut(&BodyPartType::Head).unwrap().state = BodyPartState::Missing;
        drop(body);
        assert_eq!(
            get_fog_state(&world, &resources, 12, 10, -1),
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
    fn night_darkens_the_surface_but_not_the_underground() {
        let (mut world, player) = create_test_world_with_player();
        let mut resources = create_mock_resources();
        let at_surface = |world: &mut World| {
            *world.get::<&mut Position>(player).unwrap() = Position { x: 10, y: 10, z: 0 };
        };
        at_surface(&mut world);
        let noon = resources.time.ticks_until(12, 0);
        resources.time.tick = noon;
        assert_eq!(
            get_fog_state(&world, &resources, 100, 100, 0),
            FogState::Illuminated
        );
        let underground_at_noon = get_fog_state(&world, &resources, 10, 30, -1);

        // Midnight: only the light radius (2 tiles without a torch) is lit
        resources.time.tick = noon + resources.time.ticks_until(0, 0);
        assert!(resources.time.is_night());
        assert_eq!(
            get_fog_state(&world, &resources, 12, 10, 0),
            FogState::Illuminated
        );
        assert_eq!(
            get_fog_state(&world, &resources, 13, 10, 0),
            FogState::Unvisited
        );
        update_fog_of_war(&mut world, &resources);
        *world.get::<&mut Position>(player).unwrap() = Position { x: 20, y: 10, z: 0 };
        assert_eq!(
            get_fog_state(&world, &resources, 11, 10, 0),
            FogState::Visited,
            "seen tiles are remembered in grayscale"
        );
        assert_eq!(
            get_fog_state(&world, &resources, 10, 30, -1),
            underground_at_noon
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

#[cfg(test)]
mod lantern_tests {
    use super::*;
    use crate::components::ItemKind;
    use crate::factory::{self, Facing};
    use crate::game::Game;
    use crate::recipe_handler::RecipeHandler;
    use crate::test_support::TestWorld;
    use crate::tile_registry::TileKind;
    use std::collections::HashMap;

    const Z: i64 = -1;

    /// An open underground room around the player at (0, 0, -1).
    fn cave() -> TestWorld {
        TestWorld::new(3)
            .player_at(0, 0, Z)
            .fill((-60, -20, Z), (60, 20, Z), TileKind::Dirt)
    }

    fn lantern(game: &mut Game, x: i64, y: i64, z: i64) {
        factory::place(
            &mut game.world,
            &mut game.res,
            FactoryKind::ScrapLantern,
            Position { x, y, z },
            Facing::North,
        )
        .expect("placed");
    }

    fn lit(game: &Game, x: i64, y: i64, z: i64) -> bool {
        is_tile_illuminated(&game.world, &game.res, x, y, z)
    }

    #[test]
    fn a_lantern_is_crafted_from_a_torch_and_iron_scrap() {
        let recipes = RecipeHandler::new();
        let index = recipes
            .find_recipe_by_name("Scrap Lantern")
            .expect("recipe");
        let mut inventory = HashMap::from([(ItemKind::Torch, 1), (ItemKind::IronScrap, 1)]);
        assert!(recipes.craft(index, &mut inventory));
        assert_eq!(inventory.get(&ItemKind::ScrapLantern), Some(&1));
        assert_eq!(inventory.get(&ItemKind::Torch).copied().unwrap_or(0), 0);
    }

    #[test]
    fn placing_and_breaking_a_lantern() {
        let mut game = cave().inventory(&[]).build();
        lantern(&mut game.game, 10, 0, Z);
        assert!(lit(&game.game, 13, 0, Z));
        let player = game.game.get_player_entity().unwrap();
        let broken = factory::break_block(
            &mut game.game.world,
            &mut game.game.res,
            Position { x: 10, y: 0, z: Z },
            player,
        );
        assert_eq!(broken, Some(FactoryKind::ScrapLantern));
        assert_eq!(game.player_inventory(), vec![(ItemKind::ScrapLantern, 1)]);
        assert!(!lit(&game.game, 13, 0, Z));
    }

    #[test]
    fn a_lantern_lights_six_tiles_around_it() {
        let mut game = cave().build();
        lantern(&mut game.game, 10, 0, Z);
        assert!(lit(&game.game, 14, 0, Z));
        assert!(lit(&game.game, 16, 0, Z));
        assert!(!lit(&game.game, 17, 0, Z));
        // Circular: (14, 4) is about 5.7 tiles away, (15, 5) about 7.1
        assert!(lit(&game.game, 14, 4, Z));
        assert!(!lit(&game.game, 15, 5, Z));
    }

    #[test]
    fn walls_cast_shadows() {
        let mut game = cave().tile(12, 0, Z, TileKind::Rock).build();
        lantern(&mut game.game, 10, 0, Z);
        assert!(lit(&game.game, 12, 0, Z), "the wall itself is lit");
        assert!(!lit(&game.game, 13, 0, Z));
    }

    #[test]
    fn light_passes_over_lanterns() {
        let mut game = cave().build();
        lantern(&mut game.game, 10, 0, Z);
        lantern(&mut game.game, 11, 0, Z);
        // Beyond the second lantern, but within the first one's reach
        assert!(lit(&game.game, 16, 0, Z));
        // The player's own light passes over a lantern next to them
        lantern(&mut game.game, -1, 0, Z);
        assert!(lit(&game.game, -2, 0, Z));
    }

    #[test]
    fn other_layers_stay_dark() {
        let mut game = cave()
            .fill((-20, -20, Z + 1), (20, 20, Z + 1), TileKind::Dirt)
            .build();
        lantern(&mut game.game, 10, 0, Z + 1);
        assert!(!lit(&game.game, 10, 1, Z));
        assert!(
            !lit(&game.game, 10, 1, Z + 1),
            "the player isn't on that layer"
        );
    }

    #[test]
    fn lanterns_too_far_away_are_not_gathered() {
        let mut game = cave().build();
        lantern(&mut game.game, LANTERN_GATHER_RANGE + 1, 0, Z);
        assert!(!lit(&game.game, LANTERN_GATHER_RANGE + 1, 1, Z));
    }

    #[test]
    fn lit_tiles_are_remembered_after_the_player_leaves() {
        let mut game = cave().build();
        lantern(&mut game.game, 10, 0, Z);
        game.tick();
        // Far beyond the player's light, so only the lantern could have shown it
        assert!(is_tile_visited(&game.game.world, 14, 0, Z));
        let player = game.game.get_player_entity().unwrap();
        *game.game.world.get::<&mut Position>(player).unwrap() = Position { x: -55, y: 0, z: Z };
        game.tick();
        assert!(!lit(&game.game, 14, 0, Z));
        assert_eq!(
            get_fog_state(&game.game.world, &game.game.res, 14, 0, Z),
            FogState::Visited
        );
    }

    #[test]
    fn a_lantern_lights_the_surface_at_night() {
        let mut game = TestWorld::new(3)
            .fill((-20, -20, 0), (20, 20, 0), TileKind::Grass)
            .build();
        let res = &mut game.game.res;
        res.time.tick = res.time.ticks_until(0, 0);
        assert!(res.time.is_night());
        lantern(&mut game.game, 10, 0, 0);
        let state = |game: &Game, x| get_fog_state(&game.world, &game.res, x, 0, 0);
        assert_eq!(state(&game.game, 14), FogState::Illuminated);
        assert_ne!(state(&game.game, 18), FogState::Illuminated);
    }

    #[test]
    fn a_lookup_traces_each_lantern_once() {
        let mut game = cave().build();
        for x in [5, 15, 25] {
            lantern(&mut game.game, x, 0, Z);
        }
        let before = lantern_rays_cast();
        let fog = FogLookup::new(&game.game.world, &game.game.res);
        for x in -40..40 {
            for y in -12..12 {
                fog.state(&game.game.res, x, y, Z);
            }
        }
        // At most one ray per tile in each lantern's circle, however many tiles are asked
        let circle = (2 * LANTERN_RADIUS + 1).pow(2) as u64;
        assert!(lantern_rays_cast() - before <= 3 * circle);
    }
}

#[cfg(test)]
mod torch_tests {
    use super::*;
    use crate::test_support::TestWorld;

    #[test]
    fn the_torch_goes_out_when_the_last_one_leaves_the_inventory() {
        let mut game = TestWorld::new(3)
            .player_at(0, 0, -1)
            .inventory(&[(ItemKind::Torch, 2)])
            .build();
        let player = game.game.get_player_entity().unwrap();
        let light = |game: &crate::test_support::TestGame| {
            *game.game.world.get::<&LightSource>(player).unwrap()
        };
        game.game
            .world
            .get::<&mut LightSource>(player)
            .unwrap()
            .set_torch_equipped(true);
        let set_torches = |game: &mut crate::test_support::TestGame, qty: u32| {
            let mut inv = game.game.world.get::<&mut Inventory>(player).unwrap();
            inv.slots.clear();
            if qty > 0 {
                inv.add(ItemKind::Torch, qty);
            }
        };
        // One torch left: still lit
        set_torches(&mut game, 1);
        game.tick();
        assert!(light(&game).torch_equipped);
        // None left (dropped, stored or crafted away): put out
        set_torches(&mut game, 0);
        game.tick();
        assert!(!light(&game).torch_equipped);
        assert_eq!(light(&game).radius, 2);
    }
}

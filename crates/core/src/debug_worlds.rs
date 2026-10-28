//! Debug world types: what each one places after its terrain is generated. Terrain itself
//! (void, flat) is in `GameWorld::generate_chunk`.
//!
//! Everything here is deterministic from the world seed and fixed positions.

use std::collections::HashSet;

use hecs::World;
use rand::seq::SliceRandom;

use crate::components::{FeralDog, Position};
use crate::game::Game;
use crate::resources::{WorldSettings, WorldType};
use crate::tiles::TileKind;

/// Where the player starts in Void, Ore test and Factory showcase worlds.
pub const DEBUG_SPAWN: Position = Position { x: 0, y: 0, z: 0 };
/// The Void platform reaches this many tiles from spawn (a 5x5 square).
pub const VOID_PLATFORM_RADIUS: i64 = 2;
/// Tiles per ore row, and rows start this far north of spawn.
pub const ORE_ROW_LENGTH: i64 = 5;
pub const ORE_ROWS_START: i64 = 3;
/// Dogs in a Dog arena, and the arena's rings.
pub const ARENA_DOGS: usize = 100;
pub const ARENA_FIRST_RING: i64 = 8;
pub const ARENA_RING_STEP: i64 = 4;
pub const ARENA_DOGS_PER_RING: usize = 25;

impl Game {
    /// A new world of the settings' type. Normal is `Game::new`; Dog arena is a normal world
    /// with the player on the surface and dogs around them; the others start the player at
    /// `DEBUG_SPAWN` with only what the type places.
    pub fn new_with_settings(seed: u64, settings: WorldSettings) -> Self {
        let mut game = match settings.world_type {
            WorldType::Normal => Game::new(seed),
            WorldType::DogArena => {
                let mut game = Game::new(seed);
                game.res.world_state.world.world_type = WorldType::DogArena;
                build_dog_arena(&mut game);
                game
            }
            world_type => {
                let mut game = Game::new_empty(seed, DEBUG_SPAWN);
                game.res.world_state.world.world_type = world_type;
                match world_type {
                    WorldType::Void => build_void_platform(&mut game),
                    WorldType::OreTest => build_ore_rows(&mut game),
                    WorldType::FactoryShowcase => crate::showcase::build_showcase(&mut game),
                    WorldType::Normal | WorldType::DogArena => unreachable!(),
                }
                game.tick();
                game
            }
        };
        game.res.world_settings = settings;
        game
    }
}

/// Where the `structure:<name>` debug world puts the player: beside the structure, which
/// stands with its top-left corner at the origin.
pub const STRUCTURE_PREVIEW_PLAYER: Position = Position { x: -2, y: -2, z: 0 };

impl Game {
    /// The `structure:<name>` debug world: flat ground (the factory showcase's terrain,
    /// without its builds) with the structure's bottom layer at the origin and the layers
    /// above it going up, and fog of war off.
    pub fn new_structure_preview(
        seed: u64,
        mut settings: WorldSettings,
        name: &str,
    ) -> Result<Self, String> {
        let folder = crate::structure_lint::find_structure(name)?;
        let mut game = Game::new_empty(seed, STRUCTURE_PREVIEW_PLAYER);
        settings.world_type = WorldType::FactoryShowcase;
        game.res.world_state.world.world_type = settings.world_type;
        game.load_quest_structure(&folder, 0, 0, 0, false);
        game.res.player_state.fog_of_war_enabled = false;
        game.tick();
        game.res.world_settings = settings;
        Ok(game)
    }
}

fn set_tile(game: &mut Game, pos: Position, tile: TileKind) {
    game.res
        .world_state
        .world
        .set_tile_cached(pos.x, pos.y, pos.z, tile);
}

fn build_void_platform(game: &mut Game) {
    let r = VOID_PLATFORM_RADIUS;
    for dy in -r..=r {
        for dx in -r..=r {
            let pos = Position {
                x: DEBUG_SPAWN.x + dx,
                y: DEBUG_SPAWN.y + dy,
                z: DEBUG_SPAWN.z,
            };
            set_tile(game, pos, TileKind::Grass);
        }
    }
}

/// Every tile the mining drop table lists, in tile variant order.
pub fn ore_test_tiles() -> Vec<TileKind> {
    TileKind::all_variants()
        .iter()
        .copied()
        .filter(|&tile| crate::crafting::mining_drop(tile).is_some())
        .collect()
}

/// The tiles of ore row `index`: north of spawn, centered on it, 2 rows apart.
pub fn ore_row(index: usize) -> impl Iterator<Item = Position> {
    let y = DEBUG_SPAWN.y - ORE_ROWS_START - 2 * index as i64;
    (0..ORE_ROW_LENGTH).map(move |i| Position {
        x: DEBUG_SPAWN.x - ORE_ROW_LENGTH / 2 + i,
        y,
        z: DEBUG_SPAWN.z,
    })
}

fn build_ore_rows(game: &mut Game) {
    for (index, tile) in ore_test_tiles().into_iter().enumerate() {
        for pos in ore_row(index) {
            set_tile(game, pos, tile);
        }
    }
}

/// The tiles of the square ring `radius` tiles (Chebyshev) from `center`.
fn ring(center: Position, radius: i64) -> Vec<Position> {
    let mut tiles = Vec::new();
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            if dx.abs().max(dy.abs()) == radius {
                tiles.push(Position {
                    x: center.x + dx,
                    y: center.y + dy,
                    z: center.z,
                });
            }
        }
    }
    tiles
}

/// Move the player to the configured surface spawn and place `ARENA_DOGS` feral dogs in
/// rings around it, `ARENA_DOGS_PER_RING` per ring, on random free passable tiles of each
/// ring (seeded by the spawn position). Rings with too few free tiles leave their dogs to
/// the next ring, so the count is exact.
fn build_dog_arena(game: &mut Game) {
    let [x, y, z] = game.res.config.default_player_position;
    let center = Position { x, y, z };
    if let Some(player) = game.get_player_entity() {
        if let Ok(mut pos) = game.world.get::<&mut Position>(player) {
            *pos = center;
        }
    }
    let mut occupied: HashSet<Position> = game
        .world
        .query::<&Position>()
        .iter()
        .map(|(_, p)| *p)
        .collect();
    let mut rng = game
        .res
        .world_state
        .world
        .new_rng(center.x, center.y, center.z);
    let mut placed = 0;
    let mut radius = ARENA_FIRST_RING;
    while placed < ARENA_DOGS {
        let mut tiles = ring(center, radius);
        tiles.shuffle(&mut rng);
        let quota = (placed + ARENA_DOGS_PER_RING).min(ARENA_DOGS);
        for pos in tiles {
            if placed == quota {
                break;
            }
            if occupied.contains(&pos) || !game.factory_tile(pos).is_passable() {
                continue;
            }
            crate::spawn_utils::world_spawn_feraldog(
                &mut game.world,
                pos.x,
                pos.y,
                pos.z,
                &game.res.world_state.world,
            );
            occupied.insert(pos);
            placed += 1;
        }
        radius += ARENA_RING_STEP;
    }
}

/// Feral dogs that are alive (not corpses).
pub fn live_dogs(world: &World) -> Vec<Position> {
    world
        .query::<&Position>()
        .with::<&FeralDog>()
        .without::<&crate::components::Dead>()
        .iter()
        .map(|(_, p)| *p)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_config::GameConfig;

    fn settings(world_type: WorldType) -> WorldSettings {
        WorldSettings {
            world_type,
            ..WorldSettings::defaults(&GameConfig::new())
        }
    }

    fn new_world(world_type: WorldType, seed: u64) -> Game {
        Game::new_with_settings(seed, settings(world_type))
    }

    fn tile(game: &mut Game, x: i64, y: i64, z: i64) -> TileKind {
        game.res.world_state.world.get_tile_cached(x, y, z)
    }

    fn entity_positions(game: &Game) -> Vec<Position> {
        let mut found: Vec<Position> = game
            .world
            .query::<&Position>()
            .iter()
            .map(|(_, p)| *p)
            .collect();
        found.sort_by_key(|p| (p.z, p.y, p.x));
        found
    }

    #[test]
    fn world_type_is_saved_and_old_settings_load_as_normal() {
        let old = r#"{"cheats_allowed": true, "developer_mode": false, "show_tutorial": true}"#;
        let loaded: WorldSettings = serde_json::from_str(old).unwrap();
        assert_eq!(loaded.world_type, WorldType::Normal);

        let mut game = new_world(WorldType::Void, 3);
        let saved = crate::save_load::save_to_string(&game, &serde_json::Value::Null).unwrap();
        let mut reloaded = Game::new_empty(3, DEBUG_SPAWN);
        crate::save_load::load_from_str(&mut reloaded, &saved).unwrap();
        assert_eq!(reloaded.res.world_settings.world_type, WorldType::Void);
        // Chunks first generated after the load are still void
        assert_eq!(tile(&mut reloaded, 500, 500, 0), TileKind::Air);
        assert_eq!(tile(&mut game, 500, 500, 0), TileKind::Air);
    }

    #[test]
    fn void_is_a_platform_in_empty_space() {
        let mut game = new_world(WorldType::Void, 3);
        assert_eq!(game.get_player_position(), Some(DEBUG_SPAWN));
        assert_eq!(tile(&mut game, 0, 0, 0), TileKind::Grass);
        assert_eq!(tile(&mut game, 2, -2, 0), TileKind::Grass);
        for (x, y, z) in [(3, 0, 0), (0, -3, 0), (40, 40, 0), (0, 0, -1), (0, 0, 1)] {
            assert_eq!(tile(&mut game, x, y, z), TileKind::Air, "({x}, {y}, {z})");
        }
        assert_eq!(entity_positions(&game), [DEBUG_SPAWN], "only the player");
        assert!(game.res.pending_structures.is_empty());
        let mut again = new_world(WorldType::Void, 3);
        assert_eq!(tile(&mut again, 2, 2, 0), tile(&mut game, 2, 2, 0));
        assert_eq!(entity_positions(&again), entity_positions(&game));
    }

    #[test]
    fn ore_test_has_a_row_for_every_minable_tile() {
        let mut game = new_world(WorldType::OreTest, 3);
        let tiles = ore_test_tiles();
        assert!(tiles.contains(&TileKind::IronScrap));
        assert!(tiles.contains(&TileKind::ScrapElectronics));
        for (index, expected) in tiles.into_iter().enumerate() {
            for pos in ore_row(index) {
                assert_eq!(tile(&mut game, pos.x, pos.y, pos.z), expected, "{pos:?}");
            }
        }
        assert_eq!(
            tile(&mut game, 0, 0, 0),
            TileKind::Grass,
            "flat ground at spawn"
        );
        assert_eq!(tile(&mut game, 30, 30, 0), TileKind::Grass);
    }

    #[test]
    fn dog_arena_adds_100_dogs_in_rings() {
        let normal: HashSet<Position> = live_dogs(&Game::new(7).world).into_iter().collect();
        let arena = new_world(WorldType::DogArena, 7);
        let [x, y, z] = arena.res.config.default_player_position;
        let center = Position { x, y, z };
        assert_eq!(arena.get_player_position(), Some(center), "on the surface");
        let dogs = live_dogs(&arena.world);
        assert_eq!(dogs.len(), normal.len() + ARENA_DOGS);
        let added: Vec<Position> = dogs.into_iter().filter(|p| !normal.contains(p)).collect();
        assert_eq!(added.len(), ARENA_DOGS);
        for p in &added {
            let distance = (p.x - center.x).abs().max((p.y - center.y).abs());
            assert!(distance >= ARENA_FIRST_RING, "{p:?}");
        }
        let again = new_world(WorldType::DogArena, 7);
        let mut first = live_dogs(&arena.world);
        let mut second = live_dogs(&again.world);
        first.sort_by_key(|p| (p.z, p.y, p.x));
        second.sort_by_key(|p| (p.z, p.y, p.x));
        assert_eq!(first, second, "same seed, same dogs");
    }
}

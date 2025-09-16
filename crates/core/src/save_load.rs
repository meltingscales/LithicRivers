use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use crate::components::itemkind_sprite_name;
use anyhow::{Context, Result};
use hecs::World;
use serde::{Deserialize, Serialize};

use crate::components::{
    BlocksMovement, DogAI, DroppedItem, FeralDog, Glyph, Health, Inventory, ItemKind, Player,
    Position, Sheep, SpriteRef,
};
use crate::model::body::Body; // currently not persisted (MVP)
use crate::resources::Resources;
use crate::world::Chunk as TileChunk;
use crate::world::World as TileWorld;

pub const SAVE_VERSION: u32 = 3;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSave {
    pub pos: Position,
    pub inventory: Inventory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheepSave {
    pub pos: Position,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeralDogSave {
    pub pos: Position,
    pub health: Health,
    pub ai: Option<DogAI>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DroppedItemSave {
    pub pos: Position,
    pub kind: ItemKind,
    pub qty: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportSave {
    pub view_x: i32,
    pub view_y: i32,
    pub view_z: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    pub version: u32,
    pub seed: u64,
    pub gametick: u64,
    pub world: TileWorld,
    pub player: PlayerSave,
    pub sheep: Vec<SheepSave>,
    pub feral_dogs: Vec<FeralDogSave>,
    pub dropped_items: Vec<DroppedItemSave>,
    pub viewport: ViewportSave,
    pub chunk_generation_states:
        std::collections::HashMap<(i64, i64, i64), crate::resources::ChunkGenerationState>,
    pub structure_generation_states:
        std::collections::HashMap<String, crate::resources::StructureGenerationState>,
    pub pending_structures: Vec<crate::resources::PendingStructure>,
}

impl SaveData {
    pub fn from_game(game: &crate::Game) -> Result<Self> {
        Self::from_game_with_viewport(
            game,
            ViewportSave {
                view_x: 0,
                view_y: 0,
                view_z: 0,
            },
        )
    }

    pub fn from_game_with_viewport(game: &crate::Game, viewport: ViewportSave) -> Result<Self> {
        let mut player_save: Option<PlayerSave> = None;
        let mut sheep: Vec<SheepSave> = Vec::new();
        let mut feral_dogs: Vec<FeralDogSave> = Vec::new();
        let mut dropped_items: Vec<DroppedItemSave> = Vec::new();
        for (
            _e,
            (
                pos,
                maybe_player,
                maybe_inventory,
                maybe_sheep,
                maybe_feral_dog,
                maybe_health,
                maybe_dog_ai,
                maybe_drop,
            ),
        ) in game
            .world
            .query::<(
                &Position,
                Option<&Player>,
                Option<&Inventory>,
                Option<&Sheep>,
                Option<&FeralDog>,
                Option<&Health>,
                Option<&DogAI>,
                Option<&DroppedItem>,
            )>()
            .iter()
        {
            if maybe_player.is_some() {
                let inv = maybe_inventory.cloned().expect("Player missing inventory");
                player_save = Some(PlayerSave {
                    pos: *pos,
                    inventory: inv,
                });
            } else if maybe_sheep.is_some() {
                sheep.push(SheepSave { pos: *pos });
            } else if maybe_feral_dog.is_some() {
                let health = maybe_health.cloned().expect("FeralDog missing health");
                let ai = maybe_dog_ai.cloned(); // AI is optional
                feral_dogs.push(FeralDogSave {
                    pos: *pos,
                    health,
                    ai,
                });
            } else if let Some(di) = maybe_drop {
                dropped_items.push(DroppedItemSave {
                    pos: *pos,
                    kind: di.kind,
                    qty: di.qty,
                });
            }
        }
        let player = player_save.context("Player entity missing during save")?;
        Ok(SaveData {
            version: SAVE_VERSION,
            seed: game.res.world_state.seed,
            gametick: game.res.time.tick,
            world: game.res.world_state.world.clone(),
            player,
            sheep,
            feral_dogs,
            dropped_items,
            viewport,
            chunk_generation_states: game.res.chunk_generation_states.clone(),
            structure_generation_states: game.res.structure_generation_states.clone(),
            pending_structures: game.res.pending_structures.clone(),
        })
    }

    pub fn apply_to_game(self, game: &mut crate::Game) -> Result<()> {
        // Replace resources (except RNG; reconstruct from seed)
        game.res = Resources::new(self.seed);
        game.res.time.tick = self.gametick;
        game.res.world_state.world = self.world;

        // Restore state tracking data
        game.res.chunk_generation_states = self.chunk_generation_states;
        game.res.structure_generation_states = self.structure_generation_states;
        game.res.pending_structures = self.pending_structures;

        // Rebuild entity world
        game.world = World::new();
        // Player
        let _player_e = game.world.spawn((
            self.player.pos,
            Glyph('@'),
            Player,
            BlocksMovement,
            Body::default(),
            SpriteRef::new("entities", "player"),
            self.player.inventory,
        ));
        // Note: player_entity no longer needed - use ECS queries
        // Sheep
        for s in self.sheep.into_iter() {
            game.world.spawn((
                s.pos,
                Glyph('s'),
                Sheep,
                BlocksMovement,
                SpriteRef::new("entities", "sheep"),
            ));
        }

        // Feral Dogs
        for dog in self.feral_dogs.into_iter() {
            let mut entity_builder = hecs::EntityBuilder::new();
            entity_builder.add(dog.pos);
            entity_builder.add(Glyph('d'));
            entity_builder.add(FeralDog);
            entity_builder.add(dog.health);
            if let Some(ai) = dog.ai {
                entity_builder.add(ai);
            }
            entity_builder.add(BlocksMovement);
            entity_builder.add(SpriteRef::new("entities", "feral_dog"));
            game.world.spawn(entity_builder.build());
        }

        // Dropped items
        for d in self.dropped_items.into_iter() {
            let sprite_name = itemkind_sprite_name(d.kind);
            game.world.spawn((
                d.pos,
                DroppedItem {
                    kind: d.kind,
                    qty: d.qty,
                },
                SpriteRef::new("items", sprite_name),
            ));
        }
        Ok(())
    }
}

// JSON-friendly mirrors that encode maps as Vec entries
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorldJson {
    pub seed: u64,
    pub chunks: Vec<((i64, i64, i64), TileChunk)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SaveDataJson {
    pub version: u32,
    pub seed: u64,
    pub gametick: u64,
    pub world: WorldJson,
    pub player: PlayerSave,
    pub sheep: Vec<SheepSave>,
    pub feral_dogs: Vec<FeralDogSave>,
    pub dropped_items: Vec<DroppedItemSave>,
    pub viewport: ViewportSave,
}

impl From<TileWorld> for WorldJson {
    fn from(w: TileWorld) -> Self {
        // Preserve cached chunks rather than clearing.
        let chunks = w.chunks_to_vec();
        WorldJson {
            seed: w.seed,
            chunks,
        }
    }
}

impl From<WorldJson> for TileWorld {
    fn from(j: WorldJson) -> Self {
        let mut w = TileWorld::new(0, 0, j.seed);
        w.set_chunks_from_vec(j.chunks);
        w
    }
}

impl From<SaveData> for SaveDataJson {
    fn from(s: SaveData) -> Self {
        SaveDataJson {
            version: s.version,
            seed: s.seed,
            gametick: s.gametick,
            world: s.world.into(),
            player: s.player,
            sheep: s.sheep,
            feral_dogs: s.feral_dogs,
            dropped_items: s.dropped_items,
            viewport: s.viewport,
        }
    }
}

impl From<SaveDataJson> for SaveData {
    fn from(j: SaveDataJson) -> Self {
        SaveData {
            version: j.version,
            seed: j.seed,
            gametick: j.gametick,
            world: j.world.into(),
            player: j.player,
            sheep: j.sheep,
            feral_dogs: j.feral_dogs,
            dropped_items: j.dropped_items,
            viewport: j.viewport,
            chunk_generation_states: std::collections::HashMap::new(),
            structure_generation_states: std::collections::HashMap::new(),
            pending_structures: Vec::new(),
        }
    }
}

// MessagePack (rmp-serde)
pub fn save_game_msgpack<P: AsRef<Path>>(game: &crate::Game, path: P) -> Result<()> {
    let data = SaveData::from_game(game)?;
    let f = File::create(path.as_ref()).with_context(|| format!("create {:?}", path.as_ref()))?;
    let mut writer = BufWriter::new(f);
    rmp_serde::encode::write_named(&mut writer, &data).context("serialize msgpack")?;
    Ok(())
}

pub fn load_game_msgpack<P: AsRef<Path>>(game: &mut crate::Game, path: P) -> Result<()> {
    let f = File::open(path.as_ref()).with_context(|| format!("open {:?}", path.as_ref()))?;
    let reader = BufReader::new(f);
    let data: SaveData = rmp_serde::decode::from_read(reader).context("deserialize msgpack")?;
    if data.version != SAVE_VERSION {
        // For now, require exact match
        anyhow::bail!(
            "Unsupported save version: {} (expected {})",
            data.version,
            SAVE_VERSION
        );
    }
    data.apply_to_game(game)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tiles::TileKind;

    fn simulate_full_actions(game: &mut crate::Game) {
        // Move player to a non-origin chunk to avoid structure asset dependency in tests
        if let Some(e) = game.get_player_entity() {
            if let Ok(mut pos) = game.world.get::<&mut Position>(e) {
                pos.x = crate::world::CHUNK_SIZE + 2;
                pos.y = crate::world::CHUNK_SIZE + 2;
            }
        }

        // Also reposition any Sheep away from origin so their wandering doesn't touch (0,0) chunk
        let mut sheep_entities: Vec<(hecs::Entity, Position)> = Vec::new();
        for (e, pos) in game.world.query::<&Position>().with::<&Sheep>().iter() {
            sheep_entities.push((e, *pos));
        }
        for (e, _pos) in sheep_entities {
            if let Ok(mut mpos) = game.world.get::<&mut Position>(e) {
                mpos.x = crate::world::CHUNK_SIZE + 5;
                mpos.y = crate::world::CHUNK_SIZE + 5;
            }
        }

        // Move the player a few steps deterministically
        game.queue_player_move(1, 0);
        game.tick();
        game.queue_player_move(0, 1);
        game.tick();

        // A few more ticks to advance ai deterministically
        for _ in 0..3 {
            game.tick();
        }
    }

    fn simulate_movement_only(game: &mut crate::Game) {
        // Reposition away from origin and sheep too, but avoid any call that caches world chunks
        if let Some(e) = game.get_player_entity() {
            if let Ok(mut pos) = game.world.get::<&mut Position>(e) {
                pos.x = crate::world::CHUNK_SIZE + 2;
                pos.y = crate::world::CHUNK_SIZE + 2;
            }
        }
        let mut sheep_entities: Vec<(hecs::Entity, Position)> = Vec::new();
        for (e, pos) in game.world.query::<&Position>().with::<&Sheep>().iter() {
            sheep_entities.push((e, *pos));
        }
        for (e, _pos) in sheep_entities {
            if let Ok(mut mpos) = game.world.get::<&mut Position>(e) {
                mpos.x = crate::world::CHUNK_SIZE + 5;
                mpos.y = crate::world::CHUNK_SIZE + 5;
            }
        }
        // Just movement (uses non-cached tile reads)
        game.queue_player_move(1, 0);
        game.tick();
        game.queue_player_move(0, 1);
        game.tick();
    }

    fn wood_count(game: &crate::Game) -> u32 {
        if let Some(e) = game.get_player_entity() {
            if let Ok(inv) = game.world.get::<&Inventory>(e) {
                return inv
                    .slots
                    .iter()
                    .find(|s| s.kind == crate::components::ItemKind::Log)
                    .map(|s| s.qty)
                    .unwrap_or(0);
            }
        }
        0
    }

    #[test]
    fn save_load_json_roundtrip_after_actions() {
        let seed = 12345u64;
        let mut game = crate::Game::new(seed);
        simulate_movement_only(&mut game);

        // Mutate a tile in a cached chunk to ensure it persists through JSON
        if let Some(e) = game.get_player_entity() {
            if let Ok(pos) = game.world.get::<&Position>(e) {
                // Ensure chunk is cached and then set a unique tile
                let before = game
                    .res
                    .world_state
                    .world
                    .get_tile_cached(pos.x, pos.y, pos.z);
                let new_tile = if before == crate::tiles::TileKind::Rock {
                    crate::tiles::TileKind::Dirt
                } else {
                    crate::tiles::TileKind::Rock
                };
                game.res
                    .world_state
                    .world
                    .set_tile_cached(pos.x, pos.y, pos.z, new_tile);
            }
        }

        // Snapshot key expectations
        let player_pos_before = game.get_player_position().expect("no player entity");
        // JSON roundtrip in-memory (use JSON-friendly mirror)
        let data = SaveData::from_game(&game).expect("save");
        let data_json: SaveDataJson = data.into();
        let s = serde_json::to_string(&data_json).expect("to json");
        let decoded_json: SaveDataJson = serde_json::from_str(&s).expect("from json");
        let decoded: SaveData = decoded_json.into();

        let mut loaded = crate::Game::new(0);
        decoded.apply_to_game(&mut loaded).expect("apply");

        // Verify seed and tick
        assert_eq!(loaded.res.world_state.seed, seed);
        assert_eq!(loaded.res.time.tick, game.res.time.tick);

        // Verify player position
        let player_pos_after = loaded
            .get_player_position()
            .expect("no player entity after load");
        assert_eq!(player_pos_after, player_pos_before);

        // Verify the mutated tile persisted
        let tile_after = loaded.res.world_state.world.get_tile_cached(
            player_pos_after.x,
            player_pos_after.y,
            player_pos_after.z,
        );
        let tile_before = game.res.world_state.world.get_tile_cached(
            player_pos_before.x,
            player_pos_before.y,
            player_pos_before.z,
        );
        assert_eq!(tile_after, tile_before);

        // Basic invariants
        assert_eq!(loaded.res.world_state.seed, seed);
        assert_eq!(loaded.res.time.tick, game.res.time.tick);
    }

    #[test]
    fn save_load_msgpack_roundtrip_after_actions() {
        let seed = 999u64;
        let mut game = crate::Game::new(seed);
        simulate_full_actions(&mut game);

        let data = SaveData::from_game(&game).expect("save");
        let buf = rmp_serde::to_vec_named(&data).expect("to msgpack");
        let decoded: SaveData = rmp_serde::from_slice(&buf).expect("from msgpack");

        let mut loaded = crate::Game::new(0);
        decoded.apply_to_game(&mut loaded).expect("apply");

        // Core invariants
        assert_eq!(loaded.res.world_state.seed, seed);
        assert_eq!(loaded.res.time.tick, game.res.time.tick);
        // Fluids removed

        // Player position and wood
        let p_before = game.get_player_position().unwrap();
        let p_after = loaded.get_player_position().unwrap();
        assert_eq!(p_after, p_before);
        assert_eq!(wood_count(&loaded), wood_count(&game));
    }

    #[test]
    fn save_load_feral_dog_roundtrip() {
        use crate::pathfinding::DogBehavior;

        let seed = 12345u64;
        let mut game = crate::Game::new(seed);

        // Clear any existing FeralDogs that Game::new() might have created
        let existing_dogs: Vec<hecs::Entity> = game
            .world
            .query::<&FeralDog>()
            .iter()
            .map(|(e, _)| e)
            .collect();
        for dog in existing_dogs {
            let _ = game.world.despawn(dog);
        }

        // Test 1: FeralDog with AI (like what the AI system creates)
        let dog_pos_with_ai = Position { x: 10, y: 15, z: 0 };
        let dog_health_with_ai = Health {
            current: 80,
            max: 100,
        };
        let dog_ai = DogAI {
            behavior: DogBehavior::Hunting,
            behavior_timer: 50,
            circle_center: Some(Position { x: 12, y: 16, z: 0 }),
            steps_taken: 25,
        };

        let _dog_entity_with_ai = game.world.spawn((
            dog_pos_with_ai,
            Glyph('d'),
            FeralDog,
            dog_health_with_ai,
            dog_ai,
            BlocksMovement,
            SpriteRef::new("entities", "feral_dog"),
        ));

        // Test 2: FeralDog without AI (like what Game::new creates initially)
        let dog_pos_no_ai = Position { x: 20, y: 25, z: 0 };
        let dog_health_no_ai = Health {
            current: 90,
            max: 100,
        };

        let _dog_entity_no_ai = game.world.spawn((
            dog_pos_no_ai,
            Glyph('d'),
            FeralDog,
            dog_health_no_ai,
            BlocksMovement,
            SpriteRef::new("entities", "feral_dog"),
        ));

        // Save the game
        let data = SaveData::from_game(&game).expect("save");

        // Verify both dogs are in save data
        assert_eq!(data.feral_dogs.len(), 2);

        // Find the dog with AI
        let dog_with_ai = data
            .feral_dogs
            .iter()
            .find(|d| d.pos == dog_pos_with_ai)
            .expect("dog with AI not found");
        assert_eq!(dog_with_ai.pos, dog_pos_with_ai);
        assert_eq!(dog_with_ai.health.current, 80);
        assert_eq!(dog_with_ai.health.max, 100);
        assert!(dog_with_ai.ai.is_some());
        let ai = dog_with_ai.ai.as_ref().unwrap();
        assert_eq!(ai.behavior, DogBehavior::Hunting);
        assert_eq!(ai.behavior_timer, 50);
        assert_eq!(ai.steps_taken, 25);

        // Find the dog without AI
        let dog_without_ai = data
            .feral_dogs
            .iter()
            .find(|d| d.pos == dog_pos_no_ai)
            .expect("dog without AI not found");
        assert_eq!(dog_without_ai.pos, dog_pos_no_ai);
        assert_eq!(dog_without_ai.health.current, 90);
        assert_eq!(dog_without_ai.health.max, 100);
        assert!(dog_without_ai.ai.is_none());

        // Test JSON roundtrip
        let data_json: SaveDataJson = data.into();
        let s = serde_json::to_string(&data_json).expect("to json");
        let decoded_json: SaveDataJson = serde_json::from_str(&s).expect("from json");
        let decoded: SaveData = decoded_json.into();

        let mut loaded = crate::Game::new(0);
        decoded.apply_to_game(&mut loaded).expect("apply");

        // Verify both feral dogs were restored correctly
        let mut found_dogs = 0;
        let mut found_dog_with_ai = false;
        let mut found_dog_without_ai = false;

        for (e, (pos, _feral_dog, health)) in loaded
            .world
            .query::<(&Position, &FeralDog, &Health)>()
            .iter()
        {
            found_dogs += 1;

            if *pos == dog_pos_with_ai {
                found_dog_with_ai = true;
                assert_eq!(health.current, 80);
                assert_eq!(health.max, 100);
                // This dog should have AI
                assert!(
                    loaded.world.get::<&DogAI>(e).is_ok(),
                    "Dog with AI should have DogAI component"
                );
            } else if *pos == dog_pos_no_ai {
                found_dog_without_ai = true;
                assert_eq!(health.current, 90);
                assert_eq!(health.max, 100);
                // This dog should not have AI
                assert!(
                    loaded.world.get::<&DogAI>(e).is_err(),
                    "Dog without AI should not have DogAI component"
                );
            }
        }
        assert_eq!(found_dogs, 2, "Should find exactly 2 FeralDogs after load");
        assert!(found_dog_with_ai, "FeralDog with AI not found after load");
        assert!(
            found_dog_without_ai,
            "FeralDog without AI not found after load"
        );
    }

    #[test]
    fn save_load_viewport_roundtrip() {
        let seed = 42u64;
        let game = crate::Game::new(seed);

        // Test viewport save/load
        let original_viewport = ViewportSave {
            view_x: 100,
            view_y: 200,
            view_z: 5,
        };

        // Save with viewport
        let returned_viewport =
            save_game_json_with_viewport(&game, "test_viewport.json", original_viewport.clone())
                .expect("save with viewport");
        assert_eq!(returned_viewport.view_x, 100);
        assert_eq!(returned_viewport.view_y, 200);
        assert_eq!(returned_viewport.view_z, 5);

        // Load and verify viewport is restored
        let mut loaded_game = crate::Game::new(0);
        let loaded_viewport = load_game_json_with_viewport(&mut loaded_game, "test_viewport.json")
            .expect("load with viewport");

        assert_eq!(loaded_viewport.view_x, 100);
        assert_eq!(loaded_viewport.view_y, 200);
        assert_eq!(loaded_viewport.view_z, 5);

        // Clean up test file
        let _ = std::fs::remove_file("test_viewport.json");
    }
}

// JSON helpers for debug
pub fn save_game_json<P: AsRef<Path>>(game: &crate::Game, path: P) -> Result<()> {
    let data = SaveData::from_game(game)?;
    let data_json: SaveDataJson = data.into();
    let f = File::create(path.as_ref()).with_context(|| format!("create {:?}", path.as_ref()))?;
    let writer = BufWriter::new(f);
    serde_json::to_writer_pretty(writer, &data_json).context("serialize json")?;
    Ok(())
}

pub fn load_game_json<P: AsRef<Path>>(game: &mut crate::Game, path: P) -> Result<()> {
    let f = File::open(path.as_ref()).with_context(|| format!("open {:?}", path.as_ref()))?;
    let reader = BufReader::new(f);
    let data_json: SaveDataJson = serde_json::from_reader(reader).context("deserialize json")?;
    if data_json.version != SAVE_VERSION {
        anyhow::bail!(
            "Unsupported save version: {} (expected {})",
            data_json.version,
            SAVE_VERSION
        );
    }
    let data: SaveData = data_json.into();
    data.apply_to_game(game)
}

// Viewport-aware save/load functions for client use
pub fn save_game_json_with_viewport<P: AsRef<Path>>(
    game: &crate::Game,
    path: P,
    viewport: ViewportSave,
) -> Result<ViewportSave> {
    let data = SaveData::from_game_with_viewport(game, viewport)?;
    let data_json: SaveDataJson = data.into();
    let f = File::create(path.as_ref()).with_context(|| format!("create {:?}", path.as_ref()))?;
    let writer = BufWriter::new(f);
    serde_json::to_writer_pretty(writer, &data_json).context("serialize json")?;
    Ok(data_json.viewport)
}

pub fn load_game_json_with_viewport<P: AsRef<Path>>(
    game: &mut crate::Game,
    path: P,
) -> Result<ViewportSave> {
    let f = File::open(path.as_ref()).with_context(|| format!("open {:?}", path.as_ref()))?;
    let reader = BufReader::new(f);
    let data_json: SaveDataJson = serde_json::from_reader(reader).context("deserialize json")?;
    if data_json.version != SAVE_VERSION {
        anyhow::bail!(
            "Unsupported save version: {} (expected {})",
            data_json.version,
            SAVE_VERSION
        );
    }
    let viewport = data_json.viewport.clone();
    let data: SaveData = data_json.into();
    data.apply_to_game(game)?;
    Ok(viewport)
}

use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use crate::components::itemkind_sprite_name;
use anyhow::{Context, Result};
use hecs::World;
use serde::{Deserialize, Serialize};

use crate::components::{
    BlocksMovement, DroppedItem, Glyph, Inventory, ItemKind, Player, Position, Sheep, SpriteRef,
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
pub struct DroppedItemSave {
    pub pos: Position,
    pub kind: ItemKind,
    pub qty: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    pub version: u32,
    pub seed: u64,
    pub gametick: u64,
    pub world: TileWorld,
    pub player: PlayerSave,
    pub sheep: Vec<SheepSave>,
    pub dropped_items: Vec<DroppedItemSave>,
}

impl SaveData {
    pub fn from_game(game: &crate::Game) -> Result<Self> {
        let mut player_save: Option<PlayerSave> = None;
        let mut sheep: Vec<SheepSave> = Vec::new();
        let mut dropped_items: Vec<DroppedItemSave> = Vec::new();
        for (_e, (pos, maybe_player, maybe_inventory, maybe_sheep, maybe_drop)) in game
            .world
            .query::<(
                &Position,
                Option<&Player>,
                Option<&Inventory>,
                Option<&Sheep>,
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
            dropped_items,
        })
    }

    pub fn apply_to_game(self, game: &mut crate::Game) -> Result<()> {
        // Replace resources (except RNG; reconstruct from seed)
        game.res = Resources::new(self.seed);
        game.res.time.tick = self.gametick;
        game.res.world_state.world = self.world;

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
    pub gen_z: i32,
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
    pub dropped_items: Vec<DroppedItemSave>,
}

impl From<TileWorld> for WorldJson {
    fn from(w: TileWorld) -> Self {
        // Preserve cached chunks rather than clearing.
        let chunks = w.chunks_to_vec();
        WorldJson {
            seed: w.seed,
            gen_z: w.gen_z,
            chunks,
        }
    }
}

impl From<WorldJson> for TileWorld {
    fn from(j: WorldJson) -> Self {
        let mut w = TileWorld::new(0, 0, j.seed);
        // Set gen_z directly to avoid clearing inserted chunks
        w.gen_z = j.gen_z;
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
            dropped_items: s.dropped_items,
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
            dropped_items: j.dropped_items,
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
                    .unwrap_or(panic!("No wood in inventory"));
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

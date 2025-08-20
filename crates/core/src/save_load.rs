use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use anyhow::{Context, Result};
use hecs::World;
use serde::{Deserialize, Serialize};

use crate::components::{BlocksMovement, Glyph, Inventory, Player, Position, Sheep};
use crate::model::body::Body; // currently not persisted (MVP)
use crate::resources::fluids::Fluid;
use crate::resources::fluids::FluidManager;
use crate::resources::world::World as TileWorld;
use crate::resources::world::Chunk as TileChunk;
use crate::resources::Resources;

pub const SAVE_VERSION: u32 = 1;

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
pub struct SaveData {
    pub version: u32,
    pub seed: u64,
    pub gametick: u64,
    pub world: TileWorld,
    pub fluids: FluidManager,
    pub player: PlayerSave,
    pub sheep: Vec<SheepSave>,
}

impl SaveData {
    pub fn from_game(game: &crate::Game) -> Result<Self> {
        let mut player_save: Option<PlayerSave> = None;
        let mut sheep: Vec<SheepSave> = Vec::new();
        for (_e, (pos, maybe_player, maybe_inventory, maybe_sheep)) in game
            .world
            .query::<(
                &Position,
                Option<&Player>,
                Option<&Inventory>,
                Option<&Sheep>,
            )>()
            .iter()
        {
            if maybe_player.is_some() {
                // inventory may be missing if something went wrong; default it
                let inv = maybe_inventory.cloned().unwrap_or_default();
                player_save = Some(PlayerSave {
                    pos: *pos,
                    inventory: inv,
                });
            } else if maybe_sheep.is_some() {
                sheep.push(SheepSave { pos: *pos });
            }
        }
        let player = player_save.context("Player entity missing during save")?;
        Ok(SaveData {
            version: SAVE_VERSION,
            seed: game.res.seed,
            gametick: game.res.gametick,
            world: game.res.world.clone(),
            fluids: game.res.fluids.clone(),
            player,
            sheep,
        })
    }

    pub fn apply_to_game(self, game: &mut crate::Game) -> Result<()> {
        // Replace resources (except RNG; reconstruct from seed)
        game.res = Resources::new(self.seed);
        game.res.gametick = self.gametick;
        game.res.world = self.world;
        game.res.fluids = self.fluids;

        // Rebuild entity world
        game.world = World::new();
        // Player
        let player_e = game.world.spawn((
            self.player.pos,
            Glyph('@'),
            Player,
            BlocksMovement,
            Body::default(),
            self.player.inventory,
        ));
        game.res.player_entity = Some(player_e);
        // Sheep
        for s in self.sheep.into_iter() {
            game.world.spawn((s.pos, Glyph('s'), Sheep, BlocksMovement));
        }
        Ok(())
    }
}

// JSON-friendly mirrors that encode maps as Vec entries
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorldJson {
    pub seed: u64,
    pub gen_z: i32,
    pub chunks: Vec<((i64, i64), TileChunk)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SaveDataJson {
    pub version: u32,
    pub seed: u64,
    pub gametick: u64,
    pub world: WorldJson,
    pub fluids: Vec<(Position, Fluid)>,
    pub player: PlayerSave,
    pub sheep: Vec<SheepSave>,
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
    fn from(mut s: SaveData) -> Self {
        // Convert fluids map and world cached chunks to JSON-friendly forms
        let fluids_vec: Vec<(Position, Fluid)> = s.fluids.fluids.into_iter().collect();
        SaveDataJson {
            version: s.version,
            seed: s.seed,
            gametick: s.gametick,
            world: s.world.into(),
            fluids: fluids_vec,
            player: s.player,
            sheep: s.sheep,
        }
    }
}

impl From<SaveDataJson> for SaveData {
    fn from(j: SaveDataJson) -> Self {
        let fluids_map = j.fluids.into_iter().collect();
        SaveData {
            version: j.version,
            seed: j.seed,
            gametick: j.gametick,
            world: j.world.into(),
            fluids: FluidManager { fluids: fluids_map, seeded_lava_chunks: Default::default() },
            player: j.player,
            sheep: j.sheep,
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
        // Clear fluids to avoid chunk (0,0) generation via debug pools during tests
        game.res.fluids = Default::default();

        // Move player to a non-origin chunk to avoid structure asset dependency in tests
        if let Some(e) = game.res.player_entity {
            if let Ok(mut pos) = game.world.get::<&mut Position>(e) {
                pos.x = crate::resources::world::CHUNK_SIZE + 2;
                pos.y = crate::resources::world::CHUNK_SIZE + 2;
            }
        }

        // Also reposition any Sheep away from origin so their wandering doesn't touch (0,0) chunk
        let mut sheep_entities: Vec<(hecs::Entity, Position)> = Vec::new();
        for (e, pos) in game.world.query::<&Position>().with::<&Sheep>().iter() {
            sheep_entities.push((e, *pos));
        }
        for (e, mut pos) in sheep_entities {
            if let Ok(mut mpos) = game.world.get::<&mut Position>(e) {
                mpos.x = crate::resources::world::CHUNK_SIZE + 5;
                mpos.y = crate::resources::world::CHUNK_SIZE + 5;
                // keep z
            }
        }

        // Move the player a few steps deterministically
        game.queue_player_move(1, 0);
        game.tick();
        game.queue_player_move(0, 1);
        game.tick();

        // Ensure a tree underfoot, then mine it (should drop Wood and convert to Dirt)
        if let Some(e) = game.res.player_entity {
            if let Ok(pos) = game.world.get::<&Position>(e) {
                game.res.world.set_tile_cached(pos.x, pos.y, TileKind::Tree);
            }
        }
        game.queue_mine();
        game.tick();

        // A few more ticks to advance fluids/ai deterministically
        for _ in 0..3 {
            game.tick();
        }
    }

    fn simulate_movement_only(game: &mut crate::Game) {
        // Reposition away from origin and sheep too, but avoid any call that caches world chunks
        // Also clear fluids to avoid (0,0) chunk access during fluid processing
        game.res.fluids = Default::default();
        if let Some(e) = game.res.player_entity {
            if let Ok(mut pos) = game.world.get::<&mut Position>(e) {
                pos.x = crate::resources::world::CHUNK_SIZE + 2;
                pos.y = crate::resources::world::CHUNK_SIZE + 2;
            }
        }
        let mut sheep_entities: Vec<(hecs::Entity, Position)> = Vec::new();
        for (e, pos) in game.world.query::<&Position>().with::<&Sheep>().iter() {
            sheep_entities.push((e, *pos));
        }
        for (e, _pos) in sheep_entities {
            if let Ok(mut mpos) = game.world.get::<&mut Position>(e) {
                mpos.x = crate::resources::world::CHUNK_SIZE + 5;
                mpos.y = crate::resources::world::CHUNK_SIZE + 5;
            }
        }
        // Just movement (uses non-cached tile reads)
        game.queue_player_move(1, 0);
        game.tick();
        game.queue_player_move(0, 1);
        game.tick();
    }

    fn wood_count(game: &crate::Game) -> u32 {
        if let Some(e) = game.res.player_entity {
            if let Ok(inv) = game.world.get::<&Inventory>(e) {
                return inv
                    .slots
                    .iter()
                    .find(|s| s.kind == crate::components::ItemKind::Wood)
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
        if let Some(e) = game.res.player_entity {
            if let Ok(pos) = game.world.get::<&Position>(e) {
                // Ensure chunk is cached and then set a unique tile
                let before = game.res.world.get_tile_cached(pos.x, pos.y);
                let new_tile = if before == crate::tiles::TileKind::Rock {
                    crate::tiles::TileKind::Dirt
                } else {
                    crate::tiles::TileKind::Rock
                };
                game.res.world.set_tile_cached(pos.x, pos.y, new_tile);
            }
        }

        // Snapshot key expectations
        let player_pos_before = if let Some(e) = game.res.player_entity {
            *game.world.get::<&Position>(e).unwrap()
        } else {
            panic!("no player entity");
        };
        // JSON roundtrip in-memory (use JSON-friendly mirror)
        let data = SaveData::from_game(&game).expect("save");
        let data_json: SaveDataJson = data.into();
        let s = serde_json::to_string(&data_json).expect("to json");
        let decoded_json: SaveDataJson = serde_json::from_str(&s).expect("from json");
        let decoded: SaveData = decoded_json.into();

        let mut loaded = crate::Game::new(0);
        decoded.apply_to_game(&mut loaded).expect("apply");

        // Verify seed and tick
        assert_eq!(loaded.res.seed, seed);
        assert_eq!(loaded.res.gametick, game.res.gametick);

        // Verify player position
        let player_pos_after = if let Some(e) = loaded.res.player_entity {
            *loaded.world.get::<&Position>(e).unwrap()
        } else {
            panic!("no player entity after load");
        };
        assert_eq!(player_pos_after, player_pos_before);

        // Verify the mutated tile persisted
        let tile_after = loaded.res.world.get_tile_cached(player_pos_after.x, player_pos_after.y);
        let tile_before = game.res.world.get_tile_cached(player_pos_before.x, player_pos_before.y);
        assert_eq!(tile_after, tile_before);

        // Basic invariants
        assert_eq!(loaded.res.seed, seed);
        assert_eq!(loaded.res.gametick, game.res.gametick);
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
        assert_eq!(loaded.res.seed, seed);
        assert_eq!(loaded.res.gametick, game.res.gametick);
        assert_eq!(loaded.res.fluids.fluids.len(), game.res.fluids.fluids.len());

        // Player position and wood
        let p_before = {
            let e = game.res.player_entity.unwrap();
            *game.world.get::<&Position>(e).unwrap()
        };
        let p_after = {
            let e = loaded.res.player_entity.unwrap();
            *loaded.world.get::<&Position>(e).unwrap()
        };
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

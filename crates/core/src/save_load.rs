use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use anyhow::{Context, Result};
use hecs::World;
use serde::{Deserialize, Serialize};

use crate::components::{BlocksMovement, Glyph, Inventory, Player, Position, Sheep};
use crate::model::body::Body; // currently not persisted (MVP)
use crate::resources::{self, Resources};
use crate::resources::fluids::FluidManager;
use crate::resources::world::World as TileWorld;

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
            .query::<(&Position, Option<&Player>, Option<&Inventory>, Option<&Sheep>)>()
            .iter()
        {
            if maybe_player.is_some() {
                // inventory may be missing if something went wrong; default it
                let inv = maybe_inventory.cloned().unwrap_or_default();
                player_save = Some(PlayerSave { pos: *pos, inventory: inv });
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
        anyhow::bail!("Unsupported save version: {} (expected {})", data.version, SAVE_VERSION);
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

        // Snapshot key expectations
        let player_pos_before = if let Some(e) = game.res.player_entity {
            *game.world.get::<&Position>(e).unwrap()
        } else {
            panic!("no player entity");
        };
        // Avoid any operations that would cache world chunks, since JSON cannot encode tuple keys.

        // JSON roundtrip in-memory
        let data = SaveData::from_game(&game).expect("save");
        let s = serde_json::to_string(&data).expect("to json");
        let decoded: SaveData = serde_json::from_str(&s).expect("from json");

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
    let mut data = SaveData::from_game(game)?;
    // JSON cannot encode non-string HashMap keys.
    // Strip caches that use tuple/struct keys to make debug JSON workable.
    data.world.clear_cache();
    // Debug JSON: omit fluids (Position keys) to avoid non-string-key maps.
    data.fluids = Default::default();
    let f = File::create(path.as_ref()).with_context(|| format!("create {:?}", path.as_ref()))?;
    let writer = BufWriter::new(f);
    serde_json::to_writer_pretty(writer, &data).context("serialize json")?;
    Ok(())
}

pub fn load_game_json<P: AsRef<Path>>(game: &mut crate::Game, path: P) -> Result<()> {
    let f = File::open(path.as_ref()).with_context(|| format!("open {:?}", path.as_ref()))?;
    let reader = BufReader::new(f);
    let data: SaveData = serde_json::from_reader(reader).context("deserialize json")?;
    if data.version != SAVE_VERSION {
        anyhow::bail!("Unsupported save version: {} (expected {})", data.version, SAVE_VERSION);
    }
    data.apply_to_game(game)
}

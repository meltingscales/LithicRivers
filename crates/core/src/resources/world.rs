use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::tiles::TileKind;
use crate::structure::StructureDefinition;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    tiles: Vec<TileKind>, // size CHUNK_SIZE * CHUNK_SIZE
}

impl Chunk {
    fn new_filled(fill: TileKind) -> Self {
        Self { tiles: vec![fill; (CHUNK_SIZE as usize) * (CHUNK_SIZE as usize)] }
    }
    #[inline]
    fn idx(tx: i32, ty: i32) -> usize { (ty as usize) * (CHUNK_SIZE as usize) + (tx as usize) }
    #[inline]
    fn get(&self, tx: i32, ty: i32) -> TileKind { self.tiles[Self::idx(tx, ty)] }
    #[inline]
    fn set(&mut self, tx: i32, ty: i32, t: TileKind) { let i = Self::idx(tx, ty); self.tiles[i] = t; }
}

pub const CHUNK_SIZE: i32 = 64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct World {
    pub seed: u64,
    chunks: HashMap<(i64, i64), Chunk>,
}

impl World {
    pub fn new(_width: usize, _height: usize, seed: u64) -> Self {
        // Width/height kept for compatibility; world is effectively infinite.
        Self { seed, chunks: HashMap::new() }
    }

    fn generate_chunk(&self, cx: i64, cy: i64, chunk: &mut Chunk) {
        // Place fixed demo structures at/near spawn (0,0 chunk only)
        if cx == 0 && cy == 0 {
            let structure_names = [
                "giant_corpse.lrstructure",
                "small_ship.lrstructure",
                "small_temple.lrstructure",
                "starter_ship.lrstructure",
            ];
            let asset_root = PathBuf::from("crates/client/assets/structures");
            let offsets = [(8, 8), (20, 40), (40, 20), (32, 32)];
            for (name, &(ox, oy)) in structure_names.iter().zip(offsets.iter()) {
                let struct_dir = asset_root.join(name);
                let structure = StructureDefinition::load_from_directory(&struct_dir);
                Self::apply_structure(chunk, &structure, ox, oy);
            }
        }

        // Deterministic generation based on world seed and chunk coords
        let mut rng = ChaCha20Rng::seed_from_u64(self.mix_coords(cx, cy));
        // Simple sprinkle of rocks with ~5% density
        let scatter = ((CHUNK_SIZE as usize) * (CHUNK_SIZE as usize)) / 20;
        for _ in 0..scatter {
            let tx = rng.gen_range(0..CHUNK_SIZE as i32);
            let ty = rng.gen_range(0..CHUNK_SIZE as i32);
            chunk.set(tx, ty, TileKind::Rock);
        }

        //add a few trees
        let scatter = ((CHUNK_SIZE as usize) * (CHUNK_SIZE as usize)) / 20;
        for _ in 0..scatter {
            let tx = rng.gen_range(0..CHUNK_SIZE as i32);
            let ty = rng.gen_range(0..CHUNK_SIZE as i32);
            chunk.set(tx, ty, TileKind::Tree);
        }

        // Optional: add pseudo-caves or features later
    }

    #[inline]
    fn div_floor(a: i32, b: i32) -> i64 {
        // floor division for negatives
        let mut q = (a as i64) / (b as i64);
        let r = (a as i64) % (b as i64);
        if (r != 0) && ((r > 0) != (b as i64 > 0)) { q -= 1; }
        q
    }

    #[inline]
    fn mod_floor(a: i32, b: i32) -> i32 { let m = a % b; if m < 0 { m + b } else { m } }

    fn mix_coords(&self, cx: i64, cy: i64) -> u64 {
        // Zigzag encode signed to unsigned, then mix with seed
        fn zz(x: i64) -> u64 { ((x << 1) ^ (x >> 63)) as u64 }
        let mut v = self.seed.wrapping_mul(0x9E3779B185EBCA87);
        v ^= zz(cx).wrapping_mul(0x94D049BB133111EB);
        v = v.rotate_left(27) ^ zz(cy).wrapping_mul(0xD2B74407B1CE6E93);
        v ^ 0xC0FFEE
    }

    pub fn get_tile(&self, x: i32, y: i32) -> TileKind {
        let cx = Self::div_floor(x, CHUNK_SIZE) as i64;
        let cy = Self::div_floor(y, CHUNK_SIZE) as i64;
        let tx = Self::mod_floor(x, CHUNK_SIZE);
        let ty = Self::mod_floor(y, CHUNK_SIZE);
        if let Some(ch) = self.chunks.get(&(cx, cy)) {
            ch.get(tx, ty)
        } else {
            // Generate a local chunk for read-only purposes
            let mut chunk = Chunk::new_filled(TileKind::Dirt);
            self.generate_chunk(cx, cy, &mut chunk);
            chunk.get(tx, ty)
        }
    }

    fn apply_structure(chunk: &mut Chunk, structure: &StructureDefinition, ox: i32, oy: i32) {
        for (z, layer) in structure.layers.iter().enumerate() {
            for (y, line) in layer.lines().enumerate() {
                for (x, ch) in line.chars().enumerate() {
                    if ch == ' ' { continue; }
                    let symbol = ch.to_string();
                    if let Some(tile) = structure.get_tile_for_symbol(&symbol) {
                        let tx = ox + x as i32;
                        let ty = oy + y as i32;
                        if tx >= 0 && tx < CHUNK_SIZE && ty >= 0 && ty < CHUNK_SIZE {
                            chunk.set(tx, ty, tile);
                        }
                    }
                }
            }
        }
    }
}

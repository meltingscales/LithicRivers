use std::collections::HashMap;

use noise::{NoiseFn, Perlin};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};

use crate::structure::StructureDefinition;
use crate::tiles::TileKind;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    tiles: Vec<TileKind>, // size CHUNK_SIZE * CHUNK_SIZE
}

impl Chunk {
    fn new_filled(fill: TileKind) -> Self {
        Self {
            tiles: vec![fill; (CHUNK_SIZE as usize) * (CHUNK_SIZE as usize)],
        }
    }
    #[inline]
    fn idx(tx: i32, ty: i32) -> usize {
        (ty as usize) * (CHUNK_SIZE as usize) + (tx as usize)
    }
    #[inline]
    fn get(&self, tx: i32, ty: i32) -> TileKind {
        self.tiles[Self::idx(tx, ty)]
    }
    #[inline]
    fn set(&mut self, tx: i32, ty: i32, t: TileKind) {
        let i = Self::idx(tx, ty);
        self.tiles[i] = t;
    }
}

pub const CHUNK_SIZE: i32 = 64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct World {
    pub seed: u64,
    // Z slice to use for generation-time noise sampling
    pub gen_z: i32,
    chunks: HashMap<(i64, i64), Chunk>,
}

impl World {
    pub fn new(_width: usize, _height: usize, seed: u64) -> Self {
        // Width/height kept for compatibility; world is effectively infinite.
        Self {
            seed,
            gen_z: 0,
            chunks: HashMap::new(),
        }
    }

    /// Set the Z slice that generation should use when sampling 3D noise.
    /// Clears cached chunks when Z changes so slices regenerate with new noise.
    pub fn set_generation_z(&mut self, z: i32) {
        if self.gen_z != z {
            self.gen_z = z;
            self.clear_cache();
        }
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
            let offsets = [(8, 8), (20, 40), (40, 20), (32, 32)];
            for (name, &(ox, oy)) in structure_names.iter().zip(offsets.iter()) {
                let structure = StructureDefinition::load_from_embedded(name);
                Self::apply_structure(chunk, &structure, ox, oy);
            }
        }

        // Create Perlin noise generators with different seeds for different features
        let perlin = Perlin::new(self.seed as u32);
        let biome_noise = Perlin::new(self.seed as u32 % 0x10000);
        let feature_noise = Perlin::new(self.seed as u32 % 0x20000);

        // Generate terrain using Perlin noise. Incorporate Z to get vertical variation.
        let zf = self.gen_z as f64;
        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                // Calculate world coordinates
                let wx = (cx as f64 * CHUNK_SIZE as f64) + x as f64;
                let wy = (cy as f64 * CHUNK_SIZE as f64) + y as f64;

                // Generate base terrain height (0.0 to 1.0)
                let scale = 0.01; // Adjust this to change the scale of the terrain features
                let height = perlin.get([wx * scale, wy * scale, zf * scale]);
                let height = (height + 1.0) * 0.5; // Convert from [-1, 1] to [0, 1]

                // Generate biome value
                let biome_scale = 0.005; // Larger scale for biomes (bigger areas)
                let biome_value =
                    biome_noise.get([wx * biome_scale, wy * biome_scale, zf * biome_scale]);

                // Generate feature value
                let feature_scale = 0.05; // Smaller scale for features
                let feature_value =
                    feature_noise.get([wx * feature_scale, wy * feature_scale, zf * feature_scale]);

                // Determine base tile type based on height
                let base_tile = if height < 0.3 {
                    // Water or beach
                    if height < 0.28 {
                        TileKind::Air // Water (handled by fluid system)
                    } else {
                        TileKind::Dirt // Beach
                    }
                } else if height < 0.4 {
                    // Grassland or forest
                    if biome_value > 0.3 {
                        TileKind::Grass
                    } else {
                        TileKind::Dirt
                    }
                } else if height < 0.7 {
                    // Hills with some rocks
                    if feature_value > 0.5 {
                        TileKind::Rock
                    } else {
                        TileKind::Grass
                    }
                } else {
                    // Mountains
                    TileKind::Rock
                };

                // Add trees and other features
                let mut tile = base_tile;
                if base_tile == TileKind::Grass || base_tile == TileKind::Dirt {
                    // Only place trees on grass or dirt
                    if biome_value > 0.0 && feature_value > 0.7 && height > 0.35 && height < 0.8 {
                        tile = TileKind::Tree;
                    }
                    // Add some rocks on grass
                    else if feature_value < -0.7 && height > 0.4 && height < 0.9 {
                        tile = TileKind::Rock;
                    }
                }

                chunk.set(x, y, tile);
            }
        }

        // Add some rare resources
        let mut rng = ChaCha20Rng::seed_from_u64(self.mix_coords(cx, cy));
        let rare_resources = [
            (TileKind::IronScrap, 0.95),        // 5% chance per chunk
            (TileKind::ScrapElectronics, 0.98), // 2% chance per chunk
            (TileKind::PlasteelScrap, 0.99),    // 1% chance per chunk
        ];

        for (resource, threshold) in rare_resources.iter() {
            if rng.gen::<f64>() > *threshold {
                let x = rng.gen_range(0..CHUNK_SIZE as i32);
                let y = rng.gen_range(0..CHUNK_SIZE as i32);
                chunk.set(x, y, *resource);
            }
        }
    }

    #[inline]
    fn div_floor(a: i32, b: i32) -> i64 {
        // floor division for negatives
        let mut q = (a as i64) / (b as i64);
        let r = (a as i64) % (b as i64);
        if (r != 0) && ((r > 0) != (b as i64 > 0)) {
            q -= 1;
        }
        q
    }

    #[inline]
    fn mod_floor(a: i32, b: i32) -> i32 {
        let m = a % b;
        if m < 0 {
            m + b
        } else {
            m
        }
    }

    fn mix_coords(&self, cx: i64, cy: i64) -> u64 {
        // Zigzag encode signed to unsigned, then mix with seed
        fn zz(x: i64) -> u64 {
            ((x << 1) ^ (x >> 63)) as u64
        }
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

    // Cached variant: generate-if-absent and store in self.chunks, then return tile.
    pub fn get_tile_cached(&mut self, x: i32, y: i32) -> TileKind {
        let cx = Self::div_floor(x, CHUNK_SIZE) as i64;
        let cy = Self::div_floor(y, CHUNK_SIZE) as i64;
        let tx = Self::mod_floor(x, CHUNK_SIZE);
        let ty = Self::mod_floor(y, CHUNK_SIZE);
        self.ensure_chunk(cx, cy);
        self.chunks
            .get(&(cx, cy))
            .expect("chunk must exist after ensure_chunk")
            .get(tx, ty)
    }

    // Ensure a chunk exists in cache by generating and inserting if absent.
    pub fn ensure_chunk(&mut self, cx: i64, cy: i64) {
        if self.chunks.contains_key(&(cx, cy)) {
            return;
        }
        let mut chunk = Chunk::new_filled(TileKind::Dirt);
        self.generate_chunk(cx, cy, &mut chunk);
        self.chunks.insert((cx, cy), chunk);
    }

    /// Mutate a tile at world coordinates, generating and caching the chunk if needed.
    pub fn set_tile_cached(&mut self, x: i32, y: i32, t: TileKind) {
        let cx = Self::div_floor(x, CHUNK_SIZE) as i64;
        let cy = Self::div_floor(y, CHUNK_SIZE) as i64;
        let tx = Self::mod_floor(x, CHUNK_SIZE);
        let ty = Self::mod_floor(y, CHUNK_SIZE);
        self.ensure_chunk(cx, cy);
        if let Some(ch) = self.chunks.get_mut(&(cx, cy)) {
            ch.set(tx, ty, t);
        }
    }

    // Prefetch all chunks overlapping the given rect [left..=right] x [top..=bottom]
    pub fn prefetch_rect(&mut self, left: i32, top: i32, right: i32, bottom: i32) {
        let min_cx = Self::div_floor(left, CHUNK_SIZE) as i64;
        let max_cx = Self::div_floor(right, CHUNK_SIZE) as i64;
        let min_cy = Self::div_floor(top, CHUNK_SIZE) as i64;
        let max_cy = Self::div_floor(bottom, CHUNK_SIZE) as i64;
        for cy in min_cy..=max_cy {
            for cx in min_cx..=max_cx {
                self.ensure_chunk(cx, cy);
            }
        }
    }

    /// Clear any cached chunks. Useful before JSON serialization, since
    /// serde_json cannot encode non-string map keys like (i64, i64).
    pub fn clear_cache(&mut self) {
        self.chunks.clear();
    }

    fn apply_structure(chunk: &mut Chunk, structure: &StructureDefinition, ox: i32, oy: i32) {
        for (z, layer) in structure.layers.iter().enumerate() {
            for (y, line) in layer.lines().enumerate() {
                for (x, ch) in line.chars().enumerate() {
                    if ch == ' ' {
                        continue;
                    }
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

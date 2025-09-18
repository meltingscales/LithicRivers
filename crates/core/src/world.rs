use std::collections::{HashMap, HashSet};
use std::time::Instant;
use tracing::info;

use noise::{NoiseFn, Perlin};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};

use crate::structure::StructureDefinition;
use crate::tiles::TileKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BiomeBand {
    Plains,
    Forest,
    Rocky,
    LithicRivers, // underground lava/rare ore biome
}

impl World {
    #[inline]
    pub fn biome_for(&self, wx: f64, wy: f64, zf: f64) -> BiomeBand {
        // Depth rule: Lithic Rivers only below or equal to -5 depth levels.
        if zf <= -5.0 {
            return BiomeBand::LithicRivers;
        }
        // Smooth, large-scale 3D noise to decide surface biome; depends on X, Y and Z.
        let scale = 0.0015; // large features
        let noise = Perlin::new((self.seed as u32) ^ 0xB10E);
        let v = noise.get([wx * scale, wy * scale, zf * scale]); // [-1,1]
                                                                 // Map v into three overlapping ranges; choose the closest center
        let centers = [-0.75f64, 0.0, 0.75];
        let labels = [BiomeBand::Plains, BiomeBand::Forest, BiomeBand::Rocky];
        let mut best = 0usize;
        let mut best_d = f64::INFINITY;
        for (i, c) in centers.iter().enumerate() {
            let d = (v - *c).abs();
            if d < best_d {
                best_d = d;
                best = i;
            }
        }
        labels[best]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    tiles: Vec<TileKind>, // size CHUNK_SIZE * CHUNK_SIZE
}

impl World {
    /// Deterministic post-process that adds small clusters of trees (10-20 tiles)
    /// onto suitable ground (grass/dirt). Uses a seeded RNG derived from
    /// seed and chunk coords so results are deterministic.
    fn add_tree_clusters(&self, cx: i64, cy: i64, cz: i64, chunk: &mut Chunk) {
        // Distinct salt so RNG stream differs from other features
        let salt: u64 = 0x7B1E_CA11_u64 ^ (cz as u64).wrapping_mul(0x5EED);
        let mut rng = ChaCha20Rng::seed_from_u64(self.mix_coords(cx, cy) ^ salt);

        // 0-2 clusters per chunk, biased toward 0/1
        let cluster_count = match rng.gen_range(0..100) {
            0..=60 => 0,
            61..=90 => 1,
            _ => 2,
        };

        for _ in 0..cluster_count {
            // Pick a random starting point on acceptable ground
            let mut attempts = 0;
            let (sx, sy) = loop {
                attempts += 1;
                if attempts > 32 {
                    // Give up if we can't find a good start
                    break (rng.gen_range(0..CHUNK_SIZE), rng.gen_range(0..CHUNK_SIZE));
                }
                let x = rng.gen_range(0..CHUNK_SIZE);
                let y = rng.gen_range(0..CHUNK_SIZE);
                let base = chunk.get(x, y);
                if base == TileKind::Grass || base == TileKind::Dirt || base == TileKind::Tree {
                    break (x, y);
                }
            };

            // Grow a blob via random frontier expansion
            let target = rng.gen_range(10..=20);
            let mut placed = 0usize;
            let mut visited: HashSet<(i32, i32)> = HashSet::new();
            let mut frontier: Vec<(i32, i32)> = vec![(sx, sy)];
            visited.insert((sx, sy));

            while placed < target && !frontier.is_empty() {
                let idx = rng.gen_range(0..frontier.len());
                let (x, y) = frontier.swap_remove(idx);

                // Only place on acceptable tiles
                let t = chunk.get(x, y);
                if t == TileKind::Grass || t == TileKind::Dirt || t == TileKind::Tree {
                    chunk.set(x, y, TileKind::Tree);
                    placed += 1;
                }

                // Expand 4-neighborhood with a mild bias
                let neighbors = [(1, 0), (-1, 0), (0, 1), (0, -1)];
                for (dx, dy) in neighbors {
                    let nx = x + dx;
                    let ny = y + dy;
                    if nx >= 0 && nx < CHUNK_SIZE && ny >= 0 && ny < CHUNK_SIZE {
                        let key = (nx, ny);
                        if !visited.contains(&key) {
                            visited.insert(key);
                            // Probability controls blob compactness
                            if rng.gen::<f32>() < 0.7 {
                                frontier.push(key);
                            }
                        }
                    }
                }
            }
        }
    }
}

impl Chunk {
    pub fn new_filled(fill: TileKind) -> Self {
        Self {
            tiles: vec![fill; (CHUNK_SIZE as usize) * (CHUNK_SIZE as usize)],
        }
    }
    #[inline]
    pub fn idx(tx: i32, ty: i32) -> usize {
        (ty as usize) * (CHUNK_SIZE as usize) + (tx as usize)
    }
    #[inline]
    pub fn get(&self, tx: i32, ty: i32) -> TileKind {
        self.tiles[Self::idx(tx, ty)]
    }
    #[inline]
    pub fn set(&mut self, tx: i32, ty: i32, t: TileKind) {
        let i = Self::idx(tx, ty);
        self.tiles[i] = t;
    }
}

pub const CHUNK_SIZE: i32 = 64;
pub const CHUNK_SIZE_Z: i32 = 1; // z slices are 1-tile thick for distinct layers

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct World {
    pub seed: u64,
    chunks: HashMap<(i64, i64, i64), Chunk>,
    // Guard against re-entrant structure placement triggering recursive generation
    structure_placement_depth: u32,
}

impl World {
    pub fn new(_width: usize, _height: usize, seed: u64) -> Self {
        // Width/height kept for compatibility; world is effectively infinite.
        Self {
            seed,
            chunks: HashMap::new(),
            structure_placement_depth: 0,
        }
    }

    fn generate_chunk(&self, cx: i64, cy: i64, cz: i64, chunk: &mut Chunk) {
        // Create Perlin noise generators with different seeds for different features
        let perlin = Perlin::new(self.seed as u32);
        let biome_noise = Perlin::new(self.seed as u32 % 0x10000);
        let feature_noise = Perlin::new(self.seed as u32 % 0x20000);

        // Generate terrain using Perlin noise. Incorporate Z to get vertical variation and bands.
        // Use the center Z of this chunk slice as the sampled Z plane.
        let zf = (cz as f64) * (CHUNK_SIZE_Z as f64) + (CHUNK_SIZE_Z as f64 * 0.5);
        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                // Calculate world coordinates
                let wx = (cx as f64 * CHUNK_SIZE as f64) + x as f64;
                let wy = (cy as f64 * CHUNK_SIZE as f64) + y as f64;
                let band = self.biome_for(wx, wy, zf);

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

                // Determine base tile type based on height and biome band
                let base_tile = match band {
                    BiomeBand::Plains => {
                        if height < 0.30 {
                            if height < 0.27 {
                                TileKind::Air
                            } else {
                                TileKind::Dirt
                            }
                        } else if height < 0.55 {
                            if biome_value > 0.0 {
                                TileKind::Grass
                            } else {
                                TileKind::Dirt
                            }
                        } else if height < 0.8 {
                            if feature_value > 0.6 {
                                TileKind::Rock
                            } else {
                                TileKind::Grass
                            }
                        } else {
                            TileKind::Rock
                        }
                    }
                    BiomeBand::Forest => {
                        if height < 0.28 {
                            if height < 0.26 {
                                TileKind::Air
                            } else {
                                TileKind::Dirt
                            }
                        } else if height < 0.7 {
                            // more vegetated
                            TileKind::Grass
                        } else {
                            TileKind::Rock
                        }
                    }
                    BiomeBand::Rocky => {
                        if height < 0.25 {
                            TileKind::Dirt
                        } else {
                            TileKind::Rock
                        }
                    }
                    BiomeBand::LithicRivers => {
                        // Horizontal Lithic Rivers (left-right) with cave-like features.
                        // Build a horizontal stripe field by sampling along Y with an X/Z-dependent offset.
                        let meander = biome_noise.get([wx * 0.02, zf * 0.02, 17.0]); // [-1,1]
                        let y_adj = wy + meander * 5.0; // up to ~5 tiles vertical wobble
                        let period = 22.0; // tiles between river centerlines
                        let t = (y_adj / period).fract(); // [0,1)
                        let tri = (t - 0.5).abs(); // distance to center in [0,0.5]
                        let d_tiles = tri * period; // distance in tiles from nearest river center

                        // Cave noise to make smaller features and open spaces
                        let cave_coarse =
                            biome_noise.get([wx * 0.03 + 300.0, wy * 0.03 - 300.0, zf * 0.03]);
                        let cave_fine =
                            biome_noise.get([wx * 0.10 - 700.0, wy * 0.10 + 700.0, zf * 0.10]);
                        let cave_mix = 0.6 * cave_coarse + 0.4 * cave_fine; // [-1,1]

                        // Determine terrain by distance to river and cave field
                        if d_tiles < 2.0 {
                            // Core: 0-2 tiles from center -> clear channel for lava flow
                            TileKind::Air
                        } else if d_tiles < 3.0 {
                            // Banks: 2-3 tiles -> mostly air with occasional rock pillars
                            if feature_value > -0.35 {
                                TileKind::Air
                            } else {
                                TileKind::Rock
                            }
                        } else if d_tiles < 4.0 {
                            // Rim: rocky edge with rare bedrock
                            if feature_value < -0.93 {
                                TileKind::Bedrock
                            } else {
                                TileKind::Rock
                            }
                        } else {
                            // Away from rivers: mix of rock and caves producing Minecraft-like caverns
                            // Open where cave noise is high; keep some structure via feature_value
                            if cave_mix > 0.55 || (cave_mix > 0.35 && feature_value > 0.75) {
                                TileKind::Air
                            } else if feature_value < -0.97 {
                                TileKind::Bedrock
                            } else {
                                TileKind::Rock
                            }
                        }
                    }
                };

                // Add trees and other features depending on band
                let mut tile = base_tile;
                match band {
                    BiomeBand::Plains | BiomeBand::Forest => {
                        if base_tile == TileKind::Grass || base_tile == TileKind::Dirt {
                            if biome_value > 0.1
                                && feature_value > 0.65
                                && height > 0.33
                                && height < 0.85
                            {
                                tile = TileKind::Tree;
                            } else if feature_value < -0.75 && height > 0.4 && height < 0.9 {
                                tile = TileKind::Rock;
                            }
                        }
                    }
                    BiomeBand::Rocky => {
                        if base_tile == TileKind::Rock && feature_value > 0.8 {
                            tile = TileKind::IronScrap;
                        }
                    }
                    BiomeBand::LithicRivers => {
                        // Ore enrichment near rim based on horizontal stripe distance
                        let meander = biome_noise.get([wx * 0.02, zf * 0.02, 17.0]);
                        let y_adj = wy + meander * 5.0;
                        let period = 22.0;
                        let t = (y_adj / period).fract();
                        let tri = (t - 0.5).abs();
                        let d_tiles = tri * period; // distance in tiles to nearest river centerline

                        if (3.0..4.0).contains(&d_tiles) && tile == TileKind::Rock {
                            if feature_value > 0.88 {
                                tile = TileKind::IronScrap;
                            }
                            if feature_value < -0.92 {
                                tile = TileKind::PlasteelScrap;
                            }
                        }

                        // Additional small caves away from rivers based on cave noise
                        if d_tiles >= 4.0 {
                            let cave_coarse =
                                biome_noise.get([wx * 0.03 + 300.0, wy * 0.03 - 300.0, zf * 0.03]);
                            let cave_fine =
                                biome_noise.get([wx * 0.10 - 700.0, wy * 0.10 + 700.0, zf * 0.10]);
                            let cave_mix = 0.6 * cave_coarse + 0.4 * cave_fine;
                            if cave_mix > 0.60 && tile == TileKind::Rock {
                                tile = TileKind::Air;
                            }
                        }
                    }
                }

                chunk.set(x, y, tile);
            }
        }

        // Post-worldgen step: add small tree clusters (diffuse noise blobs)
        self.add_tree_clusters(cx, cy, cz, chunk);

        // Add some rare resources; boost frequencies in Lithic Rivers
        let mut rng = ChaCha20Rng::seed_from_u64(self.mix_coords(cx, cy));
        let (p_iron, p_elec, p_plasteel) = if cz <= -5 {
            (0.15, 0.07, 0.03)
        } else {
            (0.05, 0.02, 0.01)
        };
        let rare_resources = [
            (TileKind::IronScrap, p_iron),
            (TileKind::ScrapElectronics, p_elec),
            (TileKind::PlasteelScrap, p_plasteel),
        ];
        for (resource, p) in rare_resources.iter() {
            if rng.gen::<f64>() < *p {
                let x = rng.gen_range(0..CHUNK_SIZE as i32);
                let y = rng.gen_range(0..CHUNK_SIZE as i32);
                chunk.set(x, y, *resource);
            }
        }

        // Note: structure placement is handled in ensure_chunk() via world-level writes
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

    pub fn is_passable(&self, x: i32, y: i32, z: i32) -> bool {
        self.get_tile(x, y, z).is_passable()
    }

    pub fn get_tile(&self, x: i32, y: i32, z: i32) -> TileKind {
        let cx = Self::div_floor(x, CHUNK_SIZE) as i64;
        let cy = Self::div_floor(y, CHUNK_SIZE) as i64;
        let cz = Self::div_floor(z, CHUNK_SIZE_Z) as i64;
        let tx = Self::mod_floor(x, CHUNK_SIZE);
        let ty = Self::mod_floor(y, CHUNK_SIZE);
        let _tz = Self::mod_floor(z, CHUNK_SIZE_Z);
        if let Some(ch) = self.chunks.get(&(cx, cy, cz)) {
            ch.get(tx, ty)
        } else {
            // Generate a local chunk for read-only purposes
            let mut chunk = Chunk::new_filled(TileKind::Dirt);
            self.generate_chunk(cx, cy, cz, &mut chunk);
            chunk.get(tx, ty)
        }
    }

    // Cached variant: generate-if-absent and store in self.chunks, then return tile.
    pub fn get_tile_cached(&mut self, x: i32, y: i32, z: i32) -> TileKind {
        tracing::info!(target: "world", "get_tile_cached called for world pos ({}, {}, {})", x, y, z);
        let cx = Self::div_floor(x, CHUNK_SIZE) as i64;
        let cy = Self::div_floor(y, CHUNK_SIZE) as i64;
        let cz = Self::div_floor(z, CHUNK_SIZE_Z) as i64;
        let tx = Self::mod_floor(x, CHUNK_SIZE);
        let ty = Self::mod_floor(y, CHUNK_SIZE);
        let _tz = Self::mod_floor(z, CHUNK_SIZE_Z);

        // Only call ensure_chunk if the chunk doesn't already exist
        if !self.has_chunk(cx, cy, cz) {
            self.ensure_chunk(cx, cy, cz);
        }

        self.chunks
            .get(&(cx, cy, cz))
            .expect("chunk must exist after ensure_chunk")
            .get(tx, ty)
    }

    /// Check if a chunk exists in the cache
    pub fn has_chunk(&self, cx: i64, cy: i64, cz: i64) -> bool {
        self.chunks.contains_key(&(cx, cy, cz))
    }

    /// Get a tile without triggering world generation - for rendering only
    /// Returns a default tile (Stone) if the chunk doesn't exist
    pub fn get_tile_cached_no_worldgen(&self, x: i32, y: i32, z: i32) -> TileKind {
        let cx = Self::div_floor(x, CHUNK_SIZE) as i64;
        let cy = Self::div_floor(y, CHUNK_SIZE) as i64;
        let cz = Self::div_floor(z, CHUNK_SIZE_Z) as i64;
        let tx = Self::mod_floor(x, CHUNK_SIZE);
        let ty = Self::mod_floor(y, CHUNK_SIZE);
        let _tz = Self::mod_floor(z, CHUNK_SIZE_Z);

        // Only return tile if chunk already exists, otherwise return default
        if let Some(chunk) = self.chunks.get(&(cx, cy, cz)) {
            chunk.get(tx, ty)
        } else {
            // Return a default tile for ungenerated areas (visible in render but doesn't trigger generation)
            TileKind::Rock
        }
    }

    // Ensure a chunk exists in cache by generating and inserting if absent.
    pub fn ensure_chunk(&mut self, cx: i64, cy: i64, cz: i64) {
        tracing::info!(target: "world", "ensure_chunk called for ({}, {}, {}) - cache has {} chunks",
            cx, cy, cz, self.chunks.len());
        if self.chunks.contains_key(&(cx, cy, cz)) {
            tracing::info!(target: "world", "Chunk ({}, {}, {}) already exists in cache", cx, cy, cz);
            return;
        }
        tracing::info!(target: "world", "Generating new chunk ({}, {}, {})", cx, cy, cz);
        let start = Instant::now();
        let mut chunk = Chunk::new_filled(TileKind::Dirt);
        self.generate_chunk(cx, cy, cz, &mut chunk);
        let dur_ms = start.elapsed().as_millis();
        self.chunks.insert((cx, cy, cz), chunk);
        info!(
            target: "world",
            "chunk_generated cx={} cy={} cz={} size={}ms cache_size={}",
            cx,
            cy,
            cz,
            dur_ms,
            self.chunks.len()
        );

        // World-level structure placement so edits can cross chunk boundaries and z layers
        if self.structure_placement_depth == 0 {
            // 1) Fixed demo structures near spawn on (0,0) but only once on cz==0
            if cx == 0 && cy == 0 && cz == 0 {
                info!(target: "world", "Placing demo structures at chunk ({}, {})", cx, cy);
                let structure_names = [
                    "giant_corpse.lrstructure",
                    "small_ship.lrstructure",
                    "small_temple.lrstructure",
                    "starter_ship.lrstructure",
                ];
                let offsets = [(8, 8), (20, 40), (40, 20), (32, 32)];
                for (name, &(ox, oy)) in structure_names.iter().zip(offsets.iter()) {
                    let structure = StructureDefinition::load_from_embedded(name);
                    let world_z = (cz as i32) * CHUNK_SIZE_Z;
                    self.apply_structure_world(cx, cy, cz, &structure, ox, oy, world_z, false);
                    info!(target: "world", "Placed structure {} at ({}, {})", name, ox, oy);
                }
            }

            // 2) Per-biome structure with low probability
            let center_wx = (cx as f64 * CHUNK_SIZE as f64) + (CHUNK_SIZE as f64 * 0.5);
            let center_wy = (cy as f64 * CHUNK_SIZE as f64) + (CHUNK_SIZE as f64 * 0.5);
            let zf = (cz as f64) * (CHUNK_SIZE_Z as f64) + (CHUNK_SIZE_Z as f64 * 0.5);
            let band_for_chunk = self.biome_for(center_wx, center_wy, zf);
            let mut rng = ChaCha20Rng::seed_from_u64(self.mix_coords(cx, cy));
            if rng.gen::<f64>() < 0.05 {
                let (name, ox, oy) = match band_for_chunk {
                    BiomeBand::Plains => (
                        "small_temple.lrstructure",
                        rng.gen_range(0..CHUNK_SIZE) as i32,
                        rng.gen_range(0..CHUNK_SIZE) as i32,
                    ),
                    BiomeBand::Forest => (
                        "giant_corpse.lrstructure",
                        rng.gen_range(0..CHUNK_SIZE) as i32,
                        rng.gen_range(0..CHUNK_SIZE) as i32,
                    ),
                    BiomeBand::Rocky => (
                        "small_ship.lrstructure",
                        rng.gen_range(0..CHUNK_SIZE) as i32,
                        rng.gen_range(0..CHUNK_SIZE) as i32,
                    ),
                    BiomeBand::LithicRivers => (
                        "small_temple.lrstructure",
                        rng.gen_range(0..CHUNK_SIZE) as i32,
                        rng.gen_range(0..CHUNK_SIZE) as i32,
                    ),
                };
                let structure = StructureDefinition::load_from_embedded(name);
                let world_z = (cz as i32) * CHUNK_SIZE_Z;
                self.apply_structure_world(cx, cy, cz, &structure, ox, oy, world_z, false);
            }
        }
    }

    /// Mutate a tile at world coordinates, generating and caching the chunk if needed.
    pub fn set_tile_cached(&mut self, x: i32, y: i32, z: i32, t: TileKind) {
        let cx = Self::div_floor(x, CHUNK_SIZE) as i64;
        let cy = Self::div_floor(y, CHUNK_SIZE) as i64;
        let cz = Self::div_floor(z, CHUNK_SIZE_Z) as i64;
        let tx = Self::mod_floor(x, CHUNK_SIZE);
        let ty = Self::mod_floor(y, CHUNK_SIZE);
        let _tz = Self::mod_floor(z, CHUNK_SIZE_Z);
        self.ensure_chunk(cx, cy, cz);
        if let Some(ch) = self.chunks.get_mut(&(cx, cy, cz)) {
            ch.set(tx, ty, t);
        }
    }

    /// Apply a structure using WORLD coordinates so placement can cross chunk boundaries
    /// and span multiple vertical layers (Z). Each structure layer `li` is written to
    /// world Z = `cz * CHUNK_SIZE + li`.
    pub fn apply_structure_world(
        &mut self,
        cx: i64,
        cy: i64,
        cz: i64,
        structure: &StructureDefinition,
        ox: i32,
        oy: i32,
        world_z: i32,
        bury_structure: bool,
    ) {
        // Re-entrancy guard: signal that we are in structure placement so ensure_chunk()
        // will not schedule additional placements while we write tiles.
        self.structure_placement_depth = self.structure_placement_depth.saturating_add(1);

        // First, ensure all Z chunks that this structure will span are generated
        let (min_structure_z, max_structure_z) = if bury_structure {
            // For buried structures, top layer is at world_z, bottom is world_z - (layers - 1)
            let bottom_z = world_z - (structure.layers.len() as i32 - 1);
            (bottom_z, world_z)
        } else {
            // For normal structures, bottom layer is at chunk base, top is chunk base + layers - 1
            let wz0 = (cz as i32) * CHUNK_SIZE_Z;
            (wz0, wz0 + structure.layers.len() as i32 - 1)
        };

        let min_chunk_z = Self::div_floor(min_structure_z, CHUNK_SIZE_Z) as i64;
        let max_chunk_z = Self::div_floor(max_structure_z, CHUNK_SIZE_Z) as i64;

        tracing::info!(target: "world", "Structure '{}' spans Z levels {} to {}, chunks {} to {}",
            structure.name, min_structure_z, max_structure_z, min_chunk_z, max_chunk_z);

        // Ensure all required Z chunks exist before applying structure
        for required_cz in min_chunk_z..=max_chunk_z {
            if !self.has_chunk(cx, cy, required_cz) {
                tracing::info!(target: "world", "Pre-generating chunk ({}, {}, {}) for structure '{}'",
                    cx, cy, required_cz, structure.name);
                self.ensure_chunk(cx, cy, required_cz);
            }
        }
        let wx0 = (cx as i32) * CHUNK_SIZE;
        let wy0 = (cy as i32) * CHUNK_SIZE;

        let mut edits: usize = 0;
        for (li, layer) in structure.layers.iter().enumerate() {
            for (y, line) in layer.lines().enumerate() {
                for (x, ch) in line.chars().enumerate() {
                    let symbol = ch.to_string();
                    if let Some(tile) = structure.get_tile_for_symbol(&symbol) {
                        let tx = wx0 + ox + x as i32;
                        let ty = wy0 + oy + y as i32;
                        let tz = if bury_structure {
                            // For buried structures, spawn point is top of structure
                            // Layer 0 (bottom) goes at world_z - (total_layers - 1)
                            // Layer i goes at world_z - (total_layers - 1 - i)
                            world_z - (structure.layers.len() as i32 - 1 - li as i32)
                        } else {
                            // For normal structures, layer 0 is at chunk base, subsequent layers go up
                            let wz0 = (cz as i32) * CHUNK_SIZE_Z;
                            wz0 + li as i32
                        };

                        // if the structure wants to use existing worldgen,
                        // don't overwrite it
                        if tile == TileKind::ExistingWorldgen {
                            // do nothing
                        } else {
                            self.set_tile_cached(tx, ty, tz, tile);
                        }

                        // if tile == TileKind::EnemySpawn {
                        //     // Spawn an enemy at the location
                        //     panic!("TODO EnemySpawn at ({}, {}, {})", tx, ty, tz);
                        // }

                        // if tile == TileKind::TreasureCommon {
                        //     // Spawn a common treasure at the location
                        //     panic!("TODO TreasureCommon at ({}, {}, {})", tx, ty, tz);
                        // }

                        // if tile == TileKind::TreasureRare {
                        //     // Spawn a rare treasure at the location
                        //     panic!("TODO TreasureRare at ({}, {}, {})", tx, ty, tz);
                        // }

                        edits += 1;
                    }
                }
            }
            info!(
                target: "world",
                "apply_structure_world layer {} applied for '{}'",
                li,
                structure.name
            );
        }
        info!(
            target: "world",
            "apply_structure_world '{}' edits={} at chunk=({}, {}, {}) offsets=({}, {})",
            structure.name,
            edits,
            cx,
            cy,
            cz,
            ox,
            oy
        );
        self.structure_placement_depth = self.structure_placement_depth.saturating_sub(1);
    }

    /// Get tile at specific Z level - now simplified without gen_z complexity
    pub fn get_tile_at_z(&mut self, x: i32, y: i32, z: i32) -> TileKind {
        self.get_tile_cached(x, y, z)
    }

    /// Get a tile at a specific Z level without triggering world generation - for rendering only
    /// Returns a default tile if the chunk doesn't exist
    pub fn get_tile_at_z_no_worldgen(&self, x: i32, y: i32, z: i32) -> TileKind {
        // No need to change gen_z since we're not generating anything
        self.get_tile_cached_no_worldgen(x, y, z)
    }

    /// Prefetch chunks at specific Z level - simplified without gen_z complexity
    pub fn prefetch_rect_at_z(&mut self, left: i32, top: i32, right: i32, bottom: i32, z: i32) {
        self.prefetch_rect(left, top, right, bottom, z);
    }

    // Prefetch all chunks overlapping the given rect at a z-level [left..=right] x [top..=bottom]
    pub fn prefetch_rect(&mut self, left: i32, top: i32, right: i32, bottom: i32, z: i32) {
        let min_cx = Self::div_floor(left, CHUNK_SIZE) as i64;
        let max_cx = Self::div_floor(right, CHUNK_SIZE) as i64;
        let min_cy = Self::div_floor(top, CHUNK_SIZE) as i64;
        let max_cy = Self::div_floor(bottom, CHUNK_SIZE) as i64;
        let cz = Self::div_floor(z, CHUNK_SIZE_Z) as i64;
        for cy in min_cy..=max_cy {
            for cx in min_cx..=max_cx {
                self.ensure_chunk(cx, cy, cz);
            }
        }
    }

    /// Clear any cached chunks. Useful before JSON serialization, since
    /// serde_json cannot encode non-string map keys like (i64, i64).
    pub fn clear_cache(&mut self) {
        self.chunks.clear();
    }

    /// Export cached chunks as a vector of entries for JSON-friendly serialization.
    /// Each entry is ((cx, cy, cz), Chunk).
    pub fn chunks_to_vec(&self) -> Vec<((i64, i64, i64), Chunk)> {
        self.chunks.iter().map(|(k, v)| (*k, v.clone())).collect()
    }

    /// Replace cached chunks from a vector produced by `chunks_to_vec`.
    pub fn set_chunks_from_vec(&mut self, entries: Vec<((i64, i64, i64), Chunk)>) {
        self.chunks = entries.into_iter().collect();
    }
}

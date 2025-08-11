use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Tile {
    Floor,
    Wall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct World {
    pub width: usize,
    pub height: usize,
    pub depth: usize, // keep for future; for now we render z=0
    tiles: Vec<Tile>, // z-major single layer for now (z=0)
}

impl World {
    pub fn new(width: usize, height: usize, seed: u64) -> Self {
        let depth = 1;
        let mut w = Self {
            width,
            height,
            depth,
            tiles: vec![Tile::Floor; width * height],
        };
        w.generate(seed);
        w
    }

    fn idx(&self, x: usize, y: usize) -> usize { y * self.width + x }

    fn generate(&mut self, seed: u64) {
        // Simple deterministic map: border walls + a few seeded scatter walls
        for x in 0..self.width {
            self.set(x, 0, Tile::Wall);
            self.set(x, self.height - 1, Tile::Wall);
        }
        for y in 0..self.height {
            self.set(0, y, Tile::Wall);
            self.set(self.width - 1, y, Tile::Wall);
        }
        let mut rng = ChaCha20Rng::seed_from_u64(seed ^ 0xC0FFEE);
        let scatter = (self.width * self.height) / 20; // ~5%
        for _ in 0..scatter {
            let x = rng.gen_range(1..self.width - 1);
            let y = rng.gen_range(1..self.height - 1);
            self.set(x, y, Tile::Wall);
        }
    }

    pub fn get(&self, x: usize, y: usize) -> Tile { self.tiles[self.idx(x, y)] }
    pub fn set(&mut self, x: usize, y: usize, t: Tile) { let i = self.idx(x, y); self.tiles[i] = t; }
}

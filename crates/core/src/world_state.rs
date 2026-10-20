use crate::world::GameWorld;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct WorldState {
    pub seed: u64,
    pub rng: ChaCha20Rng,
    #[serde(with = "crate::world::chunks_as_pairs")]
    pub world: GameWorld,
}

impl WorldState {
    pub fn new(seed: u64) -> Self {
        let rng = ChaCha20Rng::seed_from_u64(seed);
        Self {
            seed,
            rng,
            world: GameWorld::new(80, 24, seed),
        }
    }
}

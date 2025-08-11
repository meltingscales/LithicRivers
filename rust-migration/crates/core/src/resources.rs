use hecs::Entity;
use rand::{SeedableRng};
use rand_chacha::ChaCha20Rng;

pub struct Resources {
    pub seed: u64,
    pub rng: ChaCha20Rng,
    pub gametick: u64,
    pub player_entity: Option<Entity>,
}

impl Resources {
    pub fn new(seed: u64) -> Self {
        let rng = ChaCha20Rng::seed_from_u64(seed);
        Self {
            seed,
            rng,
            gametick: 0,
            player_entity: None,
        }
    }
}

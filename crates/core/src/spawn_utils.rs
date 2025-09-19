/// Utilities for spawning items and enemies or NPCs.
use crate::components::{
    BlocksMovement, Combat, EntityKind, FeralDog, GameEntity, Glyph, Health, Inventory, ItemKind,
    Position, SpriteRef,
};
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;

/// Spawn a FeralDog at the specified position with randomized inventory
/// Returns the entity ID of the spawned dog
pub fn world_spawn_feraldog(
    world: &mut hecs::World,
    x: i32,
    y: i32,
    z: i32,
    seed: u64,
) -> hecs::Entity {
    // Create dog inventory with 0-2 Leather and 1-3 Meat using deterministic RNG
    let mut dog_rng = ChaCha20Rng::seed_from_u64(seed.wrapping_add((x + y + z) as u64));
    let mut dog_inventory = Inventory::default();
    dog_inventory.add(ItemKind::Leather, dog_rng.gen_range(0..=2));
    dog_inventory.add(ItemKind::Meat, dog_rng.gen_range(1..=3));

    let dog = world.spawn((
        Position { x, y, z },
        GameEntity,
        EntityKind::FeralDog,
        FeralDog,
        Health::new(80), // Feral dogs have 80 HP
        Glyph('d'),
        SpriteRef::new("entities", "feral_dog"),
        BlocksMovement,
        Combat::default(),
        dog_inventory,
    ));

    dog
}

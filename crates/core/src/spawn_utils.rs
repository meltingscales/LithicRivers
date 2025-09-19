/// Utilities for spawning items and enemies or NPCs.
use crate::components::{
    BlocksMovement, Combat, DroppedItem, EntityKind, FeralDog, GameEntity, Glyph, Health,
    Inventory, ItemKind, Position, SpriteRef,
};
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;

pub fn world_spawn_treasurerare(
    ecs_world: &mut hecs::World,
    block_x: i32,
    block_y: i32,
    block_z: i32,
    _seed: u64,
) {
    let _item = ecs_world.spawn((
        Position {
            x: block_x,
            y: block_y,
            z: block_z,
        },
        DroppedItem {
            kind: ItemKind::Acorn,
            qty: 10,
        },
        SpriteRef::new("items", "acorn"),
    ));
}

pub fn world_spawn_treasurecommon(
    ecs_world: &mut hecs::World,
    block_x: i32,
    block_y: i32,
    block_z: i32,
    _seed: u64,
) {
    let _item = ecs_world.spawn((
        Position {
            x: block_x,
            y: block_y,
            z: block_z,
        },
        DroppedItem {
            kind: ItemKind::Log,
            qty: 1,
        },
        SpriteRef::new("items", "log"),
    ));
}

/// Spawn a FeralDog at the specified position with randomized inventory
/// Returns the entity ID of the spawned dog
pub fn world_spawn_feraldog(
    ecs_world: &mut hecs::World,
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

    let dog = ecs_world.spawn((
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

/// Utilities for spawning items and enemies or NPCs.
use crate::{
    components::{
        BlocksMovement, Combat, DroppedItem, EntityKind, FeralDog, GameEntity, Glyph, Health,
        Inventory, ItemKind, Position, SpriteRef,
    },
    world::GameWorld,
};
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;

pub fn world_spawn_treasurerare(
    ecs_world: &mut hecs::World,
    block_x: i64,
    block_y: i64,
    block_z: i64,
    game_world: &GameWorld,
) {
    let mut rng = game_world.new_rng(block_x, block_y, block_z);
    match rng.gen::<f64>() {
        //high chance for an acorn stack
        0.05..=1.00 => {
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
        //low chance for nothing
        _ => {}
    }
}

pub fn world_spawn_treasurecommon(
    ecs_world: &mut hecs::World,
    block_x: i64,
    block_y: i64,
    block_z: i64,
    game_world: &GameWorld,
) {
    let mut rng = game_world.new_rng(block_x, block_y, block_z);
    let random_num = rng.gen::<f64>();

    match random_num {
        0.0..0.10 => {
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
        0.10..0.20 => {
            let _item = ecs_world.spawn((
                Position {
                    x: block_x,
                    y: block_y,
                    z: block_z,
                },
                DroppedItem {
                    kind: ItemKind::Leather,
                    qty: 1,
                },
                SpriteRef::new("items", "leather"),
            ));
        }
        0.20..0.70 => {
            let _item = ecs_world.spawn((
                Position {
                    x: block_x,
                    y: block_y,
                    z: block_z,
                },
                DroppedItem {
                    kind: ItemKind::Meat,
                    qty: 1,
                },
                SpriteRef::new("items", "meat"),
            ));
        }
        _ => {}
    }
}

/// Spawn a FeralDog at the specified position with randomized inventory
/// Returns the entity ID of the spawned dog
pub fn world_spawn_feraldog(
    ecs_world: &mut hecs::World,
    x: i64,
    y: i64,
    z: i64,
    game_world: &GameWorld,
) -> hecs::Entity {
    // Create dog inventory with 0-2 Leather and 1-3 Meat using deterministic RNG
    let mut dog_rng = ChaCha20Rng::seed_from_u64(game_world.seed.wrapping_add((x + y + z) as u64));
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

/// Utilities for spawning items and enemies or NPCs.
use crate::{
    components::{
        BlocksMovement, Combat, Dialogue, DroppedItem, EntityKind, FeralDog, GameEntity, Glyph,
        Health, Inventory, ItemKind, NPCMood, Position, QuestTutorialBrokenAndroid, Sheep,
        SpriteRef,
    },
    dialogue::DialogueNodeID,
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
    // High chance for an acorn stack, low chance for nothing
    if (0.05..=1.00).contains(&rng.gen::<f64>()) {
        ecs_world.spawn((
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
}

pub fn world_spawn_scraprare(
    ecs_world: &mut hecs::World,
    block_x: i64,
    block_y: i64,
    block_z: i64,
    game_world: &GameWorld,
) {
    let mut rng = game_world.new_rng(block_x, block_y, block_z);
    let random_num = rng.gen::<f64>();

    if let 0.0..0.90 = random_num {
        let _item = ecs_world.spawn((
            Position {
                x: block_x,
                y: block_y,
                z: block_z,
            },
            DroppedItem {
                kind: ItemKind::Diamond,
                qty: 10,
            },
            SpriteRef::new("items", "diamond"),
        ));
    }
}

pub fn world_spawn_scrapcommon(
    ecs_world: &mut hecs::World,
    block_x: i64,
    block_y: i64,
    block_z: i64,
    game_world: &GameWorld,
) {
    let mut rng = game_world.new_rng(block_x, block_y, block_z);
    let random_num = rng.gen::<f64>();

    if let 0.0..0.50 = random_num {
        let _item = ecs_world.spawn((
            Position {
                x: block_x,
                y: block_y,
                z: block_z,
            },
            DroppedItem {
                kind: ItemKind::ScrapElectronics,
                qty: 10,
            },
            SpriteRef::new("items", "scrap_electronics"),
        ));
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
        0.20..1.00 => {
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

    ecs_world.spawn((
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
    ))
}

/// Spawn a sheep at the specified position
pub fn world_spawn_sheep(ecs_world: &mut hecs::World, x: i64, y: i64, z: i64) -> hecs::Entity {
    ecs_world.spawn((
        Position { x, y, z },
        GameEntity,
        EntityKind::Sheep,
        Sheep,
        Health::new(50), // Sheep have 50 HP
        Glyph('s'),
        SpriteRef::new("entities", "sheep"),
        BlocksMovement,
    ))
}

/// Spawn the tutorial quest NPC (Broken SapienCorp Android) at the specified position
pub fn world_spawn_tutorial_android(
    ecs_world: &mut hecs::World,
    x: i64,
    y: i64,
    z: i64,
) -> hecs::Entity {
    ecs_world.spawn((
        Position { x, y, z },
        GameEntity,
        EntityKind::QuestTutorialBrokenAndroid,
        QuestTutorialBrokenAndroid,
        Health::new(100),
        Glyph('Q'),
        SpriteRef::new("entities", "quest_tutorial_broken_android"),
        BlocksMovement,
        Dialogue {
            current_mood: NPCMood::Happy,
            met_before: false,
            current_dialogue_id: Some(DialogueNodeID::Start), // Start with tutorial broken android dialogue tree
            name: "Broken SapienCorp Android".to_string(),
        },
    ))
}

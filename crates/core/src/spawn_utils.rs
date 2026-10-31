/// Utilities for spawning items and enemies or NPCs.
use crate::{
    components::{
        BlocksMovement, Combat, Dialogue, DroppedItem, EntityKind, FeralDog, GameEntity, Glyph,
        Health, Inventory, ItemKind, NPCMood, Position, QuestTutorialBrokenAndroid, Sheep,
        SpriteRef,
    },
    dialogue::DialogueNodeID,
    tiles::TileKind,
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
    for (item, qty) in crate::crafting::roll_loot(crate::crafting::FERAL_DOG_LOOT, &mut dog_rng) {
        dog_inventory.add(item, qty);
    }

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

/// A nuclear slime's health: more than a feral dog's 80.
pub const NUCLEAR_SLIME_HEALTH: u32 = 120;

/// Spawn a nuclear slime. Whether it carries nuclear waste is rolled from the world seed
/// and its spawn position, so the same slime always carries the same thing.
pub fn world_spawn_nuclear_slime(
    ecs_world: &mut hecs::World,
    x: i64,
    y: i64,
    z: i64,
    game_world: &GameWorld,
) -> hecs::Entity {
    let mut rng = ChaCha20Rng::seed_from_u64(game_world.mix_coords(x, y, z) ^ 0x51_1AE);
    let mut inventory = Inventory::default();
    for (item, qty) in crate::crafting::roll_loot(crate::crafting::NUCLEAR_SLIME_LOOT, &mut rng) {
        inventory.add(item, qty);
    }
    ecs_world.spawn((
        Position { x, y, z },
        GameEntity,
        EntityKind::NuclearSlime,
        crate::components::NuclearSlime,
        crate::components::SlimeAI::default(),
        Health::new(NUCLEAR_SLIME_HEALTH),
        Glyph('s'),
        SpriteRef::new("entities", "nuclear_slime"),
        BlocksMovement,
        Combat::default(),
        inventory,
    ))
}

/// Hostile creatures that can be spawned on demand (e.g. by the `spawn_enemy` cheat).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EnemyKind {
    FeralDog,
    NuclearSlime,
}

impl EnemyKind {
    pub const ALL: &'static [EnemyKind] = &[EnemyKind::FeralDog, EnemyKind::NuclearSlime];

    pub fn name(self) -> &'static str {
        match self {
            EnemyKind::FeralDog => "feral_dog",
            EnemyKind::NuclearSlime => "nuclear_slime",
        }
    }

    /// How the game names a group of them.
    pub fn plural(self) -> &'static str {
        match self {
            EnemyKind::FeralDog => "feral dogs",
            EnemyKind::NuclearSlime => "nuclear slimes",
        }
    }

    pub fn from_name(name: &str) -> Option<EnemyKind> {
        Self::ALL.iter().copied().find(|kind| kind.name() == name)
    }
}

/// Why an enemy could not be spawned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpawnError {
    /// The tile itself blocks movement (wall, rock, ...)
    Impassable(TileKind),
    /// A creature, NPC or the player already stands there
    Occupied,
}

impl std::fmt::Display for SpawnError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpawnError::Impassable(tile) => write!(f, "the tile is not passable ({tile:?})"),
            SpawnError::Occupied => write!(f, "the tile is occupied"),
        }
    }
}

/// Spawn an enemy at `pos`, using the same spawn code as the game itself, if the tile is
/// passable and no movement-blocking entity is already there.
pub fn spawn_enemy(
    ecs_world: &mut hecs::World,
    game_world: &mut GameWorld,
    kind: EnemyKind,
    pos: Position,
) -> Result<hecs::Entity, SpawnError> {
    let tile = game_world.get_tile_cached(pos.x, pos.y, pos.z);
    if !tile.is_passable() {
        return Err(SpawnError::Impassable(tile));
    }
    let occupied = ecs_world
        .query::<&Position>()
        .with::<&BlocksMovement>()
        .iter()
        .any(|(_, p)| *p == pos);
    if occupied {
        return Err(SpawnError::Occupied);
    }
    Ok(match kind {
        EnemyKind::FeralDog => world_spawn_feraldog(ecs_world, pos.x, pos.y, pos.z, game_world),
        EnemyKind::NuclearSlime => {
            world_spawn_nuclear_slime(ecs_world, pos.x, pos.y, pos.z, game_world)
        }
    })
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TestWorld;

    fn try_spawn(
        game: &mut crate::test_support::TestGame,
        x: i64,
        y: i64,
    ) -> Result<hecs::Entity, SpawnError> {
        let g = &mut game.game;
        spawn_enemy(
            &mut g.world,
            &mut g.res.world_state.world,
            EnemyKind::FeralDog,
            Position { x, y, z: 0 },
        )
    }

    fn arena(seed: u64) -> crate::test_support::TestGame {
        TestWorld::new(seed)
            .fill((-4, -4, 0), (4, 4, 0), TileKind::Grass)
            .tile(-3, 0, 0, TileKind::Rock)
            .build()
    }

    #[test]
    fn spawns_a_full_health_feral_dog() {
        let mut game = arena(5);
        let dog = try_spawn(&mut game, 3, 0).unwrap();
        let w = &game.game.world;
        assert_eq!(*w.get::<&EntityKind>(dog).unwrap(), EntityKind::FeralDog);
        assert_eq!(
            *w.get::<&Position>(dog).unwrap(),
            Position { x: 3, y: 0, z: 0 }
        );
        assert_eq!(w.get::<&Health>(dog).unwrap().current, 80);
    }

    #[test]
    fn refuses_occupied_and_impassable_tiles() {
        let mut game = arena(5);
        assert_eq!(
            try_spawn(&mut game, 0, 0),
            Err(SpawnError::Occupied),
            "player's tile"
        );
        try_spawn(&mut game, 2, 0).unwrap();
        assert_eq!(
            try_spawn(&mut game, 2, 0),
            Err(SpawnError::Occupied),
            "dog's tile"
        );
        assert_eq!(
            try_spawn(&mut game, -3, 0),
            Err(SpawnError::Impassable(TileKind::Rock))
        );
    }

    #[test]
    fn same_seed_and_position_give_the_same_dog() {
        let dog = |seed| {
            let mut game = arena(seed);
            let e = try_spawn(&mut game, 3, 0).unwrap();
            let w = &game.game.world;
            let inv = format!("{:?}", w.get::<&Inventory>(e).unwrap().slots);
            let hp = w.get::<&Health>(e).unwrap().current;
            (hp, inv)
        };
        assert_eq!(dog(9), dog(9));
    }

    #[test]
    fn kinds_round_trip_by_name() {
        for kind in EnemyKind::ALL {
            assert_eq!(EnemyKind::from_name(kind.name()), Some(*kind));
        }
        assert_eq!(EnemyKind::from_name("dragon"), None);
    }
}

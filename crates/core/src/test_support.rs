//! Test fixtures: build a minimal, deterministic game with only what a test places,
//! then drive it tick by tick.
//!
//! ```ignore
//! let mut world = TestWorld::new(1).player_at(0, 0, 0).fill((-5, -5, 0), (5, 5, 0), TileKind::Grass);
//! let dog = world.feral_dog(2, 0, 0);
//! let mut game = world.build();
//! game.act(PlayerAction::Move { dx: 1, dy: 0, dz: 0 });
//! let result = game.tick();
//! ```
//!
//! Compiled for core's own tests, and for other crates with the `test-support` feature.

use crate::components::{
    itemkind_sprite_name, DroppedItem, Health, Inventory, ItemKind, Player, Position, SpriteRef,
};
use crate::game::{Game, GameTickResult};
use crate::intent::{PlayerAction, PlayerIntent};
use crate::resources::WorldSettings;
use crate::spawn_utils;
use crate::tiles::TileKind;

/// Refers to an entity added to a `TestWorld`, usable on the built `TestGame`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Handle(usize);

#[derive(Debug, Clone)]
enum Placement {
    Sheep,
    FeralDog,
    TutorialAndroid,
    Item(ItemKind, u32),
}

/// Describes a scenario. Configure it, add entities, then `build()`.
#[derive(Debug, Clone)]
pub struct TestWorld {
    seed: u64,
    player: Position,
    inventory: Option<Vec<(ItemKind, u32)>>,
    auto_pickup: Option<bool>,
    settings: Option<WorldSettings>,
    tiles: Vec<(Position, TileKind)>,
    entities: Vec<(Position, Placement)>,
}

impl TestWorld {
    /// A scenario with only the player, at (0, 0, 0).
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            player: Position { x: 0, y: 0, z: 0 },
            inventory: None,
            auto_pickup: None,
            settings: None,
            tiles: Vec::new(),
            entities: Vec::new(),
        }
    }

    /// Per-world settings, instead of the defaults from config.
    pub fn settings(mut self, settings: WorldSettings) -> Self {
        self.settings = Some(settings);
        self
    }

    pub fn player_at(mut self, x: i64, y: i64, z: i64) -> Self {
        self.player = Position { x, y, z };
        self
    }

    /// Replace the player's starting inventory.
    pub fn inventory(mut self, items: &[(ItemKind, u32)]) -> Self {
        self.inventory = Some(items.to_vec());
        self
    }

    /// Set the player's auto-pickup flag (otherwise it comes from config).
    pub fn auto_pickup(mut self, enabled: bool) -> Self {
        self.auto_pickup = Some(enabled);
        self
    }

    pub fn tile(mut self, x: i64, y: i64, z: i64, kind: TileKind) -> Self {
        self.tiles.push((Position { x, y, z }, kind));
        self
    }

    /// Fill the box between two corners (inclusive) with one tile kind.
    pub fn fill(mut self, a: (i64, i64, i64), b: (i64, i64, i64), kind: TileKind) -> Self {
        for x in a.0.min(b.0)..=a.0.max(b.0) {
            for y in a.1.min(b.1)..=a.1.max(b.1) {
                for z in a.2.min(b.2)..=a.2.max(b.2) {
                    self.tiles.push((Position { x, y, z }, kind));
                }
            }
        }
        self
    }

    fn add(&mut self, x: i64, y: i64, z: i64, placement: Placement) -> Handle {
        self.entities.push((Position { x, y, z }, placement));
        Handle(self.entities.len() - 1)
    }

    pub fn sheep(&mut self, x: i64, y: i64, z: i64) -> Handle {
        self.add(x, y, z, Placement::Sheep)
    }

    pub fn feral_dog(&mut self, x: i64, y: i64, z: i64) -> Handle {
        self.add(x, y, z, Placement::FeralDog)
    }

    /// The tutorial quest NPC (Broken SapienCorp Android).
    pub fn npc(&mut self, x: i64, y: i64, z: i64) -> Handle {
        self.add(x, y, z, Placement::TutorialAndroid)
    }

    pub fn item(&mut self, x: i64, y: i64, z: i64, kind: ItemKind, qty: u32) -> Handle {
        self.add(x, y, z, Placement::Item(kind, qty))
    }

    pub fn build(self) -> TestGame {
        let mut game = Game::new_empty(self.seed, self.player);
        if let Some(settings) = self.settings {
            game.res.world_settings = settings;
        }

        {
            let player = game.get_player_entity().expect("player exists");
            let mut inv = game
                .world
                .get::<&mut Inventory>(player)
                .expect("player has an inventory");
            if let Some(items) = &self.inventory {
                inv.slots.clear();
                for &(kind, qty) in items {
                    inv.add(kind, qty);
                }
            }
            if let Some(enabled) = self.auto_pickup {
                inv.auto_pickup = enabled;
            }
        }

        // set_tile_cached generates the chunk first, so worldgen can't overwrite these later
        for (pos, kind) in &self.tiles {
            game.res
                .world_state
                .world
                .set_tile_cached(pos.x, pos.y, pos.z, *kind);
        }

        let mut entities = Vec::with_capacity(self.entities.len());
        for (pos, placement) in &self.entities {
            let Position { x, y, z } = *pos;
            let world = &mut game.world;
            let entity = match placement {
                Placement::Sheep => spawn_utils::world_spawn_sheep(world, x, y, z),
                Placement::FeralDog => {
                    spawn_utils::world_spawn_feraldog(world, x, y, z, &game.res.world_state.world)
                }
                Placement::TutorialAndroid => {
                    spawn_utils::world_spawn_tutorial_android(world, x, y, z)
                }
                Placement::Item(kind, qty) => world.spawn((
                    *pos,
                    DroppedItem {
                        kind: *kind,
                        qty: *qty,
                    },
                    SpriteRef::new("items", itemkind_sprite_name(*kind)),
                )),
            };
            entities.push(entity);
        }

        TestGame { game, entities }
    }
}

/// A built scenario: the real `Game` plus helpers to drive and inspect it.
pub struct TestGame {
    pub game: Game,
    entities: Vec<hecs::Entity>,
}

impl TestGame {
    /// Queue a player action for the next tick, with the same cost the client uses.
    pub fn act(&mut self, action: PlayerAction) {
        match action {
            PlayerAction::Move { dx, dy, dz: 0 } => self.game.queue_player_move(dx, dy),
            PlayerAction::Move { dz, .. } => self.game.queue_player_move_z(dz),
            PlayerAction::Mine => self.game.queue_mine(),
            PlayerAction::MineAt { x, y, z } => self.game.queue_mine_at(x, y, z),
            PlayerAction::Interact | PlayerAction::InteractWith { .. } => {
                self.game.res.player_state.intent = PlayerIntent {
                    action: Some(action),
                    tick_cost: 100,
                }
            }
        }
    }

    pub fn tick(&mut self) -> GameTickResult {
        self.game.tick()
    }

    /// Run `n` ticks and return each tick's result.
    pub fn ticks(&mut self, n: usize) -> Vec<GameTickResult> {
        (0..n).map(|_| self.game.tick()).collect()
    }

    pub fn entity(&self, handle: Handle) -> hecs::Entity {
        self.entities[handle.0]
    }

    /// Position of a placed entity, or `None` if it no longer exists.
    pub fn pos(&self, handle: Handle) -> Option<Position> {
        self.game
            .world
            .get::<&Position>(self.entity(handle))
            .ok()
            .map(|p| *p)
    }

    /// Current health of a placed entity, or `None` if it has no health or no longer exists.
    pub fn health(&self, handle: Handle) -> Option<u32> {
        self.game
            .world
            .get::<&Health>(self.entity(handle))
            .ok()
            .map(|h| h.current)
    }

    pub fn player_pos(&self) -> Position {
        self.game.get_player_position().expect("player exists")
    }

    /// The player's inventory as (item, total quantity) pairs.
    pub fn player_inventory(&self) -> Vec<(ItemKind, u32)> {
        let mut query = self.game.world.query::<(&Player, &Inventory)>();
        let (_, (_, inv)) = query.iter().next().expect("player has an inventory");
        let mut items: Vec<(ItemKind, u32)> = Vec::new();
        for slot in &inv.slots {
            match items.iter_mut().find(|(k, _)| *k == slot.kind) {
                Some((_, qty)) => *qty += slot.qty,
                None => items.push((slot.kind, slot.qty)),
            }
        }
        items
    }

    /// Total quantity of one item in the player's inventory.
    pub fn player_has(&self, kind: ItemKind) -> u32 {
        self.player_inventory()
            .iter()
            .filter(|(k, _)| *k == kind)
            .map(|(_, q)| q)
            .sum()
    }

    pub fn tile(&mut self, x: i64, y: i64, z: i64) -> TileKind {
        self.game.res.world_state.world.get_tile_cached(x, y, z)
    }

    /// Dropped items at a position, as (item, quantity).
    pub fn items_at(&self, x: i64, y: i64, z: i64) -> Vec<(ItemKind, u32)> {
        let mut items: Vec<(ItemKind, u32)> = self
            .game
            .world
            .query::<(&Position, &DroppedItem)>()
            .iter()
            .filter(|(_, (p, _))| (p.x, p.y, p.z) == (x, y, z))
            .map(|(_, (_, d))| (d.kind, d.qty))
            .collect();
        items.sort_by_key(|(k, q)| (format!("{k:?}"), *q));
        items
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::EntityKind;

    /// Kind, position, health and inventory of every entity, sorted.
    fn signature(game: &Game) -> Vec<String> {
        let mut sig: Vec<String> = game
            .world
            .query::<(
                &Position,
                Option<&EntityKind>,
                Option<&Health>,
                Option<&Inventory>,
            )>()
            .iter()
            .map(|(_, (p, kind, health, inv))| {
                format!(
                    "{kind:?} ({},{},{}) {:?} {:?}",
                    p.x,
                    p.y,
                    p.z,
                    health.map(|h| h.current),
                    inv.map(|i| &i.slots)
                )
            })
            .collect();
        sig.sort();
        sig
    }

    /// Ticking runs real worldgen around the player, which may place structure loot
    /// (dropped items) in nearby chunks, but never adds creatures or NPCs.
    #[test]
    fn ticking_adds_no_creatures() {
        let mut game = TestWorld::new(1).build();
        game.ticks(5);
        let mut query = game.game.world.query::<&EntityKind>();
        let kinds: Vec<EntityKind> = query.iter().map(|(_, k)| *k).collect();
        assert_eq!(kinds, vec![EntityKind::Player]);
        let non_items = game
            .game
            .world
            .query::<&Position>()
            .without::<&DroppedItem>()
            .iter()
            .count();
        assert_eq!(non_items, 1, "only the player and dropped items");
    }

    #[test]
    fn empty_scenario_has_only_the_player() {
        let game = TestWorld::new(1).build();
        assert_eq!(game.player_pos(), Position { x: 0, y: 0, z: 0 });
        assert_eq!(signature(&game.game).len(), 1);
        assert!(game
            .game
            .world
            .query::<&DroppedItem>()
            .iter()
            .next()
            .is_none());
    }

    #[test]
    fn fill_and_tile_override_worldgen() {
        let mut game = TestWorld::new(1)
            .fill((-5, -5, 0), (5, 5, 0), TileKind::Grass)
            .tile(1, 0, 0, TileKind::Tree)
            .build();
        assert_eq!(game.tile(1, 0, 0), TileKind::Tree);
        for (x, y) in [(-5, -5), (5, 5), (0, 0), (-1, 3)] {
            assert_eq!(game.tile(x, y, 0), TileKind::Grass, "at ({x},{y})");
        }
    }

    #[test]
    fn placed_dog_is_found_by_handle_at_full_health() {
        let mut world = TestWorld::new(1);
        let dog = world.feral_dog(2, 0, 0);
        let game = world.build();
        assert_eq!(game.pos(dog), Some(Position { x: 2, y: 0, z: 0 }));
        assert_eq!(game.health(dog), Some(80));
        assert!(game
            .game
            .world
            .get::<&crate::components::FeralDog>(game.entity(dog))
            .is_ok());
    }

    #[test]
    fn inventory_can_be_replaced_or_emptied() {
        let game = TestWorld::new(1).inventory(&[]).build();
        assert!(game.player_inventory().is_empty());
        let game = TestWorld::new(1).inventory(&[(ItemKind::Torch, 2)]).build();
        assert_eq!(game.player_inventory(), vec![(ItemKind::Torch, 2)]);
    }

    #[test]
    fn chopping_a_tree_drops_logs_and_acorns() {
        let mut game = TestWorld::new(1)
            .fill((-3, -3, 0), (3, 3, 0), TileKind::Grass)
            .tile(1, 0, 0, TileKind::Tree)
            .build();
        game.act(PlayerAction::MineAt { x: 1, y: 0, z: 0 });
        assert!(game.tick().contains(GameTickResult::MiningSuccess));
        assert_eq!(game.tile(1, 0, 0), TileKind::Dirt);
        let drops: Vec<ItemKind> = game.items_at(1, 0, 0).iter().map(|(k, _)| *k).collect();
        assert!(drops.contains(&ItemKind::Log) && drops.contains(&ItemKind::Acorn));
    }

    #[test]
    fn same_fixture_gives_same_state() {
        let run = || {
            let mut world = TestWorld::new(42).fill((-8, -8, 0), (8, 8, 0), TileKind::Grass);
            world.sheep(3, 3, 0);
            world.feral_dog(-4, 2, 0);
            world.npc(0, 5, 0);
            world.item(1, 1, 0, ItemKind::Log, 2);
            let mut game = world.build();
            for _ in 0..10 {
                game.act(PlayerAction::Move {
                    dx: 1,
                    dy: 0,
                    dz: 0,
                });
                game.tick();
            }
            signature(&game.game)
        };
        assert_eq!(run(), run());
    }
}

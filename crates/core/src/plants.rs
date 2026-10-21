//! Tree farming: acorns are planted as saplings that grow into trees over game time.
//!
//! A sapling is a `TileKind::Sapling` tile plus an entity carrying its `Growth`. Growth is
//! checked every factory step, so saplings grow wherever they are.

use hecs::{Entity, World};
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::components::{Inventory, ItemKind, Position};
use crate::factory::TICKS_PER_SECOND;
use crate::resources::Resources;
use crate::tiles::TileKind;

/// Shortest and longest time a sapling takes to grow, in game minutes.
pub const GROWTH_MINUTES: (u64, u64) = (3, 5);

/// A planted sapling's timer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Growth {
    /// Game tick at which the sapling becomes a tree
    pub grows_at_tick: u64,
    /// The tile it was planted on, restored if the sapling is broken
    pub planted_on: TileKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlantError {
    /// Acorns only grow in grass or dirt
    NeedsGrassOrDirt,
    /// Something already stands there
    Occupied,
}

impl std::fmt::Display for PlantError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlantError::NeedsGrassOrDirt => write!(f, "Acorns need grass or dirt"),
            PlantError::Occupied => write!(f, "Something is already there"),
        }
    }
}

/// Whether placing this item plants it instead of putting down a block.
pub fn is_seed(item: ItemKind) -> bool {
    item == ItemKind::Acorn
}

/// Game ticks from planting to growing for a sapling at `pos`: 3–5 game minutes, chosen by
/// the world seed and position, so the same sapling always takes the same time.
pub fn growth_ticks(res: &Resources, pos: Position) -> u64 {
    let (min, max) = GROWTH_MINUTES;
    let ticks_per_minute = 60 * TICKS_PER_SECOND;
    let mut rng = res.world_state.world.new_rng(pos.x, pos.y, pos.z);
    rng.gen_range(min * ticks_per_minute..=max * ticks_per_minute)
}

/// Plant a sapling at `pos` (grass or dirt with nothing on it). Does not touch any inventory.
pub fn plant(world: &mut World, res: &mut Resources, pos: Position) -> Result<Entity, PlantError> {
    let tile = res.world_state.world.get_tile_cached(pos.x, pos.y, pos.z);
    if !matches!(tile, TileKind::Grass | TileKind::Dirt) {
        return Err(PlantError::NeedsGrassOrDirt);
    }
    if world.query::<&Position>().iter().any(|(_, p)| *p == pos) {
        return Err(PlantError::Occupied);
    }
    res.world_state
        .world
        .set_tile_cached(pos.x, pos.y, pos.z, TileKind::Sapling);
    let growth = Growth {
        grows_at_tick: res.time.tick + growth_ticks(res, pos),
        planted_on: tile,
    };
    Ok(world.spawn((pos, growth)))
}

/// Grow every sapling whose time has come by `tick`. Runs first in each factory step.
pub fn grow_saplings(world: &mut World, res: &mut Resources, tick: u64) {
    let grown: Vec<(Entity, Position)> = world
        .query::<(&Position, &Growth)>()
        .iter()
        .filter(|(_, (_, g))| g.grows_at_tick <= tick)
        .map(|(e, (p, _))| (e, *p))
        .collect();
    for (entity, pos) in grown {
        res.world_state
            .world
            .set_tile_cached(pos.x, pos.y, pos.z, TileKind::Tree);
        let _ = world.despawn(entity);
    }
}

/// Break the sapling at `pos`: the acorn goes to `receiver`'s inventory and the tile goes
/// back to what it was planted on. Returns false if there's no sapling there.
pub fn break_sapling(
    world: &mut World,
    res: &mut Resources,
    pos: Position,
    receiver: Entity,
) -> bool {
    let found = world
        .query::<(&Position, &Growth)>()
        .iter()
        .find(|(_, (p, _))| **p == pos)
        .map(|(e, (_, g))| (e, *g));
    let Some((entity, growth)) = found else {
        return false;
    };
    let _ = world.despawn(entity);
    res.world_state
        .world
        .set_tile_cached(pos.x, pos.y, pos.z, growth.planted_on);
    if let Ok(mut inv) = world.get::<&mut Inventory>(receiver) {
        inv.add(ItemKind::Acorn, 1);
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intent::{PlayerAction, PlayerIntent};
    use crate::test_support::{TestGame, TestWorld};

    fn at(x: i64, y: i64) -> Position {
        Position { x, y, z: 0 }
    }

    fn garden() -> TestGame {
        TestWorld::new(9)
            .fill((-3, -3, 0), (3, 3, 0), TileKind::Grass)
            .tile(2, 0, 0, TileKind::Dirt)
            .tile(-2, 0, 0, TileKind::Rock)
            .inventory(&[])
            .build()
    }

    fn plant_at(game: &mut TestGame, pos: Position) -> Result<Entity, PlantError> {
        plant(&mut game.game.world, &mut game.game.res, pos)
    }

    /// Advance the game by `ticks` in one waiting action.
    fn wait(game: &mut TestGame, ticks: u64) {
        game.game.res.player_state.intent = PlayerIntent::movement(0, 0, 0, ticks);
        game.tick();
    }

    #[test]
    fn planting_needs_empty_grass_or_dirt() {
        let mut game = garden();
        game.game
            .world
            .spawn((at(0, 1), crate::components::BlocksMovement));
        assert!(plant_at(&mut game, at(1, 0)).is_ok(), "grass");
        assert!(plant_at(&mut game, at(2, 0)).is_ok(), "dirt");
        assert_eq!(game.tile(1, 0, 0), TileKind::Sapling);
        assert_eq!(
            plant_at(&mut game, at(-2, 0)),
            Err(PlantError::NeedsGrassOrDirt)
        );
        assert_eq!(
            plant_at(&mut game, at(1, 0)),
            Err(PlantError::NeedsGrassOrDirt),
            "already a sapling"
        );
        assert_eq!(plant_at(&mut game, at(0, 1)), Err(PlantError::Occupied));
        assert_eq!(
            PlantError::NeedsGrassOrDirt.to_string(),
            "Acorns need grass or dirt"
        );
    }

    #[test]
    fn saplings_grow_into_trees_at_their_tick() {
        let mut game = garden();
        let sapling = plant_at(&mut game, at(1, 0)).unwrap();
        let grows_at = game
            .game
            .world
            .get::<&Growth>(sapling)
            .unwrap()
            .grows_at_tick;
        let now = game.game.res.time.tick;
        let (min, max) = (3 * 6000, 5 * 6000);
        assert!(
            (now + min..=now + max).contains(&grows_at),
            "3 to 5 game minutes"
        );

        // Just before: still a sapling
        wait(&mut game, grows_at - now - 1);
        assert_eq!(game.tile(1, 0, 0), TileKind::Sapling);
        // Reaching the growth tick (and its factory step): a tree you can chop
        wait(&mut game, crate::factory::FACTORY_STEP_TICKS);
        assert_eq!(game.tile(1, 0, 0), TileKind::Tree);
        assert!(!game.game.world.contains(sapling));
        game.act(PlayerAction::MineAt { x: 1, y: 0, z: 0 });
        game.tick();
        assert_eq!(game.tile(1, 0, 0), TileKind::Dirt);
    }

    #[test]
    fn growth_time_depends_only_on_seed_and_position() {
        let grows_at = |seed: u64, pos: Position| {
            let mut game = TestWorld::new(seed)
                .fill((60, -2, 0), (66, 2, 0), TileKind::Grass)
                .build();
            let e = plant_at(&mut game, pos).unwrap();
            let g = *game.game.world.get::<&Growth>(e).unwrap();
            g.grows_at_tick
        };
        assert_eq!(grows_at(9, at(61, 0)), grows_at(9, at(61, 0)));
        let ticks: std::collections::BTreeSet<u64> =
            (60..=66).map(|x| grows_at(9, at(x, 1))).collect();
        assert!(ticks.len() > 1, "positions differ");
    }

    #[test]
    fn saplings_on_chunk_edges_grow_normally() {
        use crate::world::CHUNK_SIZE;
        let mut game = TestWorld::new(3)
            .player_at(0, 5, 0)
            .fill((-2, -1, 0), (2, 1, 0), TileKind::Grass)
            .fill((CHUNK_SIZE - 1, 0, 0), (CHUNK_SIZE, 0, 0), TileKind::Grass)
            .build();
        for pos in [
            at(-1, 0),
            at(0, 0),
            at(CHUNK_SIZE - 1, 0),
            at(CHUNK_SIZE, 0),
        ] {
            plant_at(&mut game, pos).unwrap();
        }
        wait(&mut game, 5 * 6000 + crate::factory::FACTORY_STEP_TICKS);
        for pos in [
            at(-1, 0),
            at(0, 0),
            at(CHUNK_SIZE - 1, 0),
            at(CHUNK_SIZE, 0),
        ] {
            assert_eq!(game.tile(pos.x, pos.y, 0), TileKind::Tree, "{pos:?}");
        }
    }

    #[test]
    fn breaking_a_sapling_returns_the_acorn() {
        let mut game = garden();
        plant_at(&mut game, at(2, 0)).unwrap();
        game.act(PlayerAction::MineAt { x: 2, y: 0, z: 0 });
        game.tick();
        assert_eq!(game.player_has(ItemKind::Acorn), 1);
        assert_eq!(
            game.tile(2, 0, 0),
            TileKind::Dirt,
            "back to what it was planted on"
        );
        assert_eq!(game.game.world.query::<&Growth>().iter().count(), 0);
    }
}

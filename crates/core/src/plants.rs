//! Farming: acorns grow into trees, and wheat, potatoes and carrots grow into crops.
//!
//! A sapling is a `TileKind::Sapling` tile plus an entity carrying its `Growth`. A crop is a
//! tile per stage (seedling, growing, ripe) plus an entity carrying its `CropGrowth` until it
//! ripens. Growth is checked every factory step, so plants grow wherever they are. Crops only
//! grow in daylight on the surface; ripe crops are harvested through the mining drop table
//! (`crafting::mining_drop`), by hand or by an auto-harvester.

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

/// A crop that grows from a seed item into a ripe tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Crop {
    Wheat,
    Potato,
    Carrot,
}

impl Crop {
    pub const ALL: [Crop; 3] = [Crop::Wheat, Crop::Potato, Crop::Carrot];

    /// The item planted to grow it: wheat seeds, or a potato or carrot itself.
    pub fn seed(self) -> ItemKind {
        match self {
            Crop::Wheat => ItemKind::WheatSeeds,
            Crop::Potato => ItemKind::Potato,
            Crop::Carrot => ItemKind::Carrot,
        }
    }

    pub fn from_seed(item: ItemKind) -> Option<Crop> {
        Crop::ALL.into_iter().find(|c| c.seed() == item)
    }

    /// Its tiles: seedling, growing, ripe.
    pub fn tiles(self) -> [TileKind; 3] {
        match self {
            Crop::Wheat => [
                TileKind::WheatSeedling,
                TileKind::WheatGrowing,
                TileKind::WheatRipe,
            ],
            Crop::Potato => [
                TileKind::PotatoSeedling,
                TileKind::PotatoGrowing,
                TileKind::PotatoRipe,
            ],
            Crop::Carrot => [
                TileKind::CarrotSeedling,
                TileKind::CarrotGrowing,
                TileKind::CarrotRipe,
            ],
        }
    }

    pub fn ripe_tile(self) -> TileKind {
        self.tiles()[2]
    }

    /// The crop growing on `tile`, and whether it is ripe.
    pub fn on_tile(tile: TileKind) -> Option<(Crop, bool)> {
        Crop::ALL.into_iter().find_map(|c| {
            let tiles = c.tiles();
            tiles
                .iter()
                .position(|&t| t == tile)
                .map(|stage| (c, stage == 2))
        })
    }

    /// Game ticks of daylight from planting to ripe: wheat 3 game minutes, potatoes 4,
    /// carrots 2.
    pub fn grow_ticks(self) -> u64 {
        let minutes = match self {
            Crop::Wheat => 3,
            Crop::Potato => 4,
            Crop::Carrot => 2,
        };
        minutes * 60 * TICKS_PER_SECOND
    }
}

/// A planted crop's progress, until it ripens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CropGrowth {
    pub crop: Crop,
    /// Daylight ticks it has grown so far
    pub grown_ticks: u64,
    /// The tile it was planted on, restored if it is broken before it ripens
    pub planted_on: TileKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlantError {
    /// Seeds only grow in grass or dirt
    NeedsGrassOrDirt,
    /// Something already stands there
    Occupied,
    /// The item isn't a seed
    NotASeed,
}

impl std::fmt::Display for PlantError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlantError::NeedsGrassOrDirt => write!(f, "Seeds need grass or dirt"),
            PlantError::Occupied => write!(f, "Something is already there"),
            PlantError::NotASeed => write!(f, "That doesn't grow"),
        }
    }
}

/// Whether placing this item plants it instead of putting down a block.
pub fn is_seed(item: ItemKind) -> bool {
    item == ItemKind::Acorn || Crop::from_seed(item).is_some()
}

/// Game ticks from planting to growing for a sapling at `pos`: 3–5 game minutes, chosen by
/// the world seed and position, so the same sapling always takes the same time.
pub fn growth_ticks(res: &Resources, pos: Position) -> u64 {
    let (min, max) = GROWTH_MINUTES;
    let ticks_per_minute = 60 * TICKS_PER_SECOND;
    let mut rng = res.world_state.world.new_rng(pos.x, pos.y, pos.z);
    rng.gen_range(min * ticks_per_minute..=max * ticks_per_minute)
}

/// Plant `seed` at `pos` (grass or dirt with nothing on it): an acorn becomes a sapling, a
/// crop's seed its seedling. Does not touch any inventory.
pub fn plant(
    world: &mut World,
    res: &mut Resources,
    pos: Position,
    seed: ItemKind,
) -> Result<Entity, PlantError> {
    if !is_seed(seed) {
        return Err(PlantError::NotASeed);
    }
    let tile = res.world_state.world.get_tile_cached(pos.x, pos.y, pos.z);
    if !matches!(tile, TileKind::Grass | TileKind::Dirt) {
        return Err(PlantError::NeedsGrassOrDirt);
    }
    if world.query::<&Position>().iter().any(|(_, p)| *p == pos) {
        return Err(PlantError::Occupied);
    }
    crate::factory_stats::record(res, seed, 1, crate::factory_stats::Source::Planting, false);
    let Some(crop) = Crop::from_seed(seed) else {
        res.world_state
            .world
            .set_tile_cached(pos.x, pos.y, pos.z, TileKind::Sapling);
        let growth = Growth {
            grows_at_tick: res.time.tick + growth_ticks(res, pos),
            planted_on: tile,
        };
        return Ok(world.spawn((pos, growth)));
    };
    res.world_state
        .world
        .set_tile_cached(pos.x, pos.y, pos.z, crop.tiles()[0]);
    let growth = CropGrowth {
        crop,
        grown_ticks: 0,
        planted_on: tile,
    };
    Ok(world.spawn((pos, growth)))
}

/// Whether crops at `pos` grow at `tick`: on the surface (or above) and not at night.
fn crops_grow(pos: Position, tick: u64) -> bool {
    pos.z >= 0 && !crate::game_time::GameTime { tick }.is_night()
}

/// Grow every sapling whose time has come by `tick`, and every crop by one factory step if
/// it is in daylight. Runs first in each factory step.
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
    let mut ripe = Vec::new();
    let mut staged = Vec::new();
    for (entity, (pos, growth)) in world.query_mut::<(&Position, &mut CropGrowth)>() {
        if !crops_grow(*pos, tick) {
            continue;
        }
        let total = growth.crop.grow_ticks();
        let half = total / 2;
        let before = growth.grown_ticks;
        growth.grown_ticks += crate::factory::FACTORY_STEP_TICKS;
        if growth.grown_ticks >= total {
            ripe.push((entity, *pos, growth.crop.ripe_tile()));
        } else if before < half && growth.grown_ticks >= half {
            staged.push((*pos, growth.crop.tiles()[1]));
        }
    }
    for (pos, tile) in staged {
        res.world_state
            .world
            .set_tile_cached(pos.x, pos.y, pos.z, tile);
    }
    for (entity, pos, tile) in ripe {
        res.world_state
            .world
            .set_tile_cached(pos.x, pos.y, pos.z, tile);
        let _ = world.despawn(entity);
    }
}

/// Break the unripe plant at `pos` (a sapling or a crop that hasn't ripened): its seed goes
/// to `receiver`'s inventory and the tile goes back to what it was planted on. Returns the
/// seed, or `None` if there's no unripe plant there.
pub fn break_plant(
    world: &mut World,
    res: &mut Resources,
    pos: Position,
    receiver: Entity,
) -> Option<ItemKind> {
    let sapling = world
        .query::<(&Position, &Growth)>()
        .iter()
        .find(|(_, (p, _))| **p == pos)
        .map(|(e, (_, g))| (e, g.planted_on, ItemKind::Acorn));
    let crop = || {
        world
            .query::<(&Position, &CropGrowth)>()
            .iter()
            .find(|(_, (p, _))| **p == pos)
            .map(|(e, (_, g))| (e, g.planted_on, g.crop.seed()))
    };
    let (entity, planted_on, seed) = sapling.or_else(crop)?;
    let _ = world.despawn(entity);
    res.world_state
        .world
        .set_tile_cached(pos.x, pos.y, pos.z, planted_on);
    if let Ok(mut inv) = world.get::<&mut Inventory>(receiver) {
        inv.add(seed, 1);
    }
    Some(seed)
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
        plant(
            &mut game.game.world,
            &mut game.game.res,
            pos,
            ItemKind::Acorn,
        )
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
            "Seeds need grass or dirt"
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

    const MINUTE: u64 = 60 * TICKS_PER_SECOND;

    fn plant_seed(game: &mut TestGame, pos: Position, seed: ItemKind) -> Entity {
        plant(&mut game.game.world, &mut game.game.res, pos, seed).expect("planted")
    }

    #[test]
    fn every_crop_has_a_seed_and_three_tiles() {
        for crop in Crop::ALL {
            assert_eq!(Crop::from_seed(crop.seed()), Some(crop));
            assert!(is_seed(crop.seed()));
            for (stage, tile) in crop.tiles().into_iter().enumerate() {
                assert_eq!(Crop::on_tile(tile), Some((crop, stage == 2)));
                assert!(tile.is_passable());
            }
        }
        assert!(!is_seed(ItemKind::Wheat), "wheat is eaten, not planted");
        assert_eq!(Crop::on_tile(TileKind::Grass), None);
    }

    #[test]
    fn crops_grow_through_their_stages_by_day() {
        let mut game = garden();
        plant_seed(&mut game, at(1, 0), ItemKind::WheatSeeds);
        plant_seed(&mut game, at(0, 1), ItemKind::Potato);
        plant_seed(&mut game, at(-1, 0), ItemKind::Carrot);
        assert_eq!(game.tile(1, 0, 0), TileKind::WheatSeedling);
        // Halfway through carrots' 2 minutes: growing
        wait(&mut game, MINUTE + crate::factory::FACTORY_STEP_TICKS);
        assert_eq!(game.tile(-1, 0, 0), TileKind::CarrotGrowing);
        assert_eq!(game.tile(1, 0, 0), TileKind::WheatSeedling);
        // Carrots ripe at 2 minutes, wheat at 3, potatoes at 4
        wait(&mut game, MINUTE);
        assert_eq!(game.tile(-1, 0, 0), TileKind::CarrotRipe);
        assert_eq!(game.tile(1, 0, 0), TileKind::WheatGrowing);
        wait(&mut game, MINUTE);
        assert_eq!(game.tile(1, 0, 0), TileKind::WheatRipe);
        assert_eq!(game.tile(0, 1, 0), TileKind::PotatoGrowing);
        wait(&mut game, MINUTE);
        assert_eq!(game.tile(0, 1, 0), TileKind::PotatoRipe);
        assert_eq!(
            game.game.world.query::<&CropGrowth>().iter().count(),
            0,
            "ripe crops need no timer"
        );
    }

    #[test]
    fn crops_wait_out_the_night_and_need_the_surface() {
        let mut game = TestWorld::new(9)
            .fill((-3, -3, 0), (3, 3, 0), TileKind::Grass)
            .fill((-3, -3, -1), (3, 3, -1), TileKind::Grass)
            .build();
        // 21:00, the start of five game minutes of night
        game.game.res.time.tick = 13 * crate::game_time::HOUR_TICKS;
        game.game.res.factory_clock.last_tick = game.game.res.time.tick;
        assert!(game.game.res.time.is_night());
        let surface = plant_seed(&mut game, at(1, 0), ItemKind::Carrot);
        let below = Position { x: 1, y: 0, z: -1 };
        let cave = plant_seed(&mut game, below, ItemKind::Carrot);
        wait(&mut game, 4 * MINUTE);
        let grown = |game: &TestGame, e| game.game.world.get::<&CropGrowth>(e).unwrap().grown_ticks;
        assert_eq!(grown(&game, surface), 0, "nothing grows at night");
        // Dawn comes, and the surface carrot ripens two minutes of daylight later
        wait(&mut game, 3 * MINUTE);
        assert_eq!(game.tile(1, 0, 0), TileKind::CarrotRipe);
        assert_eq!(grown(&game, cave), 0, "nothing grows underground");
        assert_eq!(game.tile(1, 0, -1), TileKind::CarrotSeedling);
    }

    #[test]
    fn harvesting_ripe_crops_gives_produce_and_seeds() {
        for (seed, ripe, produce, range) in [
            (
                ItemKind::WheatSeeds,
                TileKind::WheatRipe,
                ItemKind::Wheat,
                2..=2,
            ),
            (
                ItemKind::Potato,
                TileKind::PotatoRipe,
                ItemKind::Potato,
                4..=5,
            ),
            (
                ItemKind::Carrot,
                TileKind::CarrotRipe,
                ItemKind::Carrot,
                3..=4,
            ),
        ] {
            let mut game = TestWorld::new(9)
                .fill((-3, -3, 0), (3, 3, 0), TileKind::Grass)
                .tile(1, 0, 0, ripe)
                .inventory(&[])
                .auto_pickup(true)
                .build();
            game.act(PlayerAction::MineAt { x: 1, y: 0, z: 0 });
            game.tick();
            game.act(PlayerAction::Move {
                dx: 1,
                dy: 0,
                dz: 0,
            });
            game.tick();
            assert_eq!(game.tile(1, 0, 0), TileKind::Dirt, "{ripe:?}");
            assert!(range.contains(&game.player_has(produce)), "{ripe:?}");
            if seed != produce {
                assert!((1..=2).contains(&game.player_has(seed)), "{ripe:?}");
            }
        }
    }

    #[test]
    fn breaking_an_unripe_crop_returns_its_seed() {
        let mut game = garden();
        plant_seed(&mut game, at(2, 0), ItemKind::WheatSeeds);
        game.act(PlayerAction::MineAt { x: 2, y: 0, z: 0 });
        game.tick();
        assert_eq!(game.player_has(ItemKind::WheatSeeds), 1);
        assert_eq!(game.tile(2, 0, 0), TileKind::Dirt, "planted on dirt");
        assert_eq!(game.game.world.query::<&CropGrowth>().iter().count(), 0);
        assert_eq!(
            plant(
                &mut game.game.world,
                &mut game.game.res,
                at(1, 0),
                ItemKind::Wheat
            ),
            Err(PlantError::NotASeed)
        );
    }

    #[test]
    fn crop_growth_survives_save_and_reload() {
        let mut game = garden();
        plant_seed(&mut game, at(1, 0), ItemKind::WheatSeeds);
        wait(&mut game, 2 * MINUTE);
        let saved = crate::save_load::save_to_string(&game.game, &serde_json::Value::Null).unwrap();
        let mut loaded = TestWorld::new(1).build().game;
        crate::save_load::load_from_str(&mut loaded, &saved).unwrap();
        let growth = |g: &crate::Game| {
            g.world
                .query::<&CropGrowth>()
                .iter()
                .map(|(_, c)| *c)
                .next()
        };
        assert_eq!(growth(&loaded), growth(&game.game));
        loaded.res.player_state.intent = PlayerIntent::movement(0, 0, 0, MINUTE);
        loaded.tick();
        assert_eq!(
            loaded.res.world_state.world.get_tile_cached(1, 0, 0),
            TileKind::WheatRipe
        );
    }
}

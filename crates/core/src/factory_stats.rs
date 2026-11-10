//! Factory statistics: each power network's supply and demand over time, and how many of
//! each item were made and used, by source.
//!
//! - **Power history:** once per factory second (every `STEPS_PER_FACTORY_SECOND` steps),
//!   each network records its supply and demand in a ring of `POWER_SAMPLES` samples: the
//!   last 10 game minutes. Histories are keyed by network id (its lowest pole), so they
//!   survive rebuilds and loads. When networks merge, the merged network keeps the history
//!   of the lowest id; a history whose network is gone is kept until its id has been unused
//!   for `HISTORY_EXPIRY_TICKS`.
//! - **Item flow:** `record` counts items produced and consumed by item and `Source`, in
//!   one bucket per game minute, for the last `FLOW_MINUTES` minutes. It is called wherever
//!   items are made or used up: player mining and chopping, machines, crafting, burning fuel
//!   and planting.
//!
//! Both are saved with the world (they live on `Resources`).

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use crate::components::{ItemKind, Position};
use crate::factory::{FactoryKind, TICKS_PER_SECOND};
use crate::resources::Resources;

/// Game ticks in one game minute: the item flow's bucket size.
pub const MINUTE_TICKS: u64 = 60 * TICKS_PER_SECOND;
/// Power samples kept per network: 10 game minutes of factory seconds.
pub const POWER_SAMPLES: usize =
    (10 * MINUTE_TICKS / crate::factory::FACTORY_SECOND_TICKS) as usize;
/// Power samples in the short view: 60 game seconds.
pub const SHORT_SAMPLES: usize = (MINUTE_TICKS / crate::factory::FACTORY_SECOND_TICKS) as usize;
/// A gone network's history is dropped once its id has been unused this long.
pub const HISTORY_EXPIRY_TICKS: u64 = 10 * MINUTE_TICKS;
/// Minutes of item flow kept.
pub const FLOW_MINUTES: usize = 60;

/// Where items came from or went.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Source {
    /// Mined or chopped by the player, or a plant they pulled up
    PlayerMining,
    /// A machine of this kind made or used them
    Machine(FactoryKind),
    /// Crafted by the player
    Crafting,
    /// Burned for energy in a generator or furnace
    Fuel,
    /// Planted as seeds
    Planting,
}

impl Source {
    pub fn name(self) -> String {
        match self {
            Source::PlayerMining => "mining by hand".to_string(),
            Source::Machine(kind) => kind.name().to_string(),
            Source::Crafting => "crafting".to_string(),
            Source::Fuel => "burned as fuel".to_string(),
            Source::Planting => "planting".to_string(),
        }
    }
}

/// One network's power over time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkHistory {
    pub id: Position,
    /// Oldest first
    pub samples: VecDeque<PowerSample>,
    /// The last tick this id belonged to a network
    pub last_seen_tick: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PowerSample {
    pub supply_w: u64,
    pub demand_w: u64,
}

impl PowerSample {
    /// Whether the network couldn't meet its demand.
    pub fn short(&self) -> bool {
        self.supply_w < self.demand_w
    }
}

/// Counts for one item from one source in one minute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlowEntry {
    pub item: ItemKind,
    pub source: Source,
    pub produced: u64,
    pub consumed: u64,
}

/// Every item flow recorded in one game minute.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MinuteFlow {
    /// Game minute since tick 0
    pub minute: u64,
    pub entries: Vec<FlowEntry>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactoryStats {
    pub power: Vec<NetworkHistory>,
    /// Oldest first
    pub flow: VecDeque<MinuteFlow>,
}

/// Count `qty` of `item` made (`produced`) or used up by `source`, in this game minute.
pub fn record(res: &mut Resources, item: ItemKind, qty: u32, source: Source, produced: bool) {
    if qty == 0 {
        return;
    }
    let minute = res.time.tick / MINUTE_TICKS;
    let flow = &mut res.factory_stats.flow;
    if flow.back().is_none_or(|m| m.minute != minute) {
        flow.push_back(MinuteFlow {
            minute,
            entries: Vec::new(),
        });
        while flow.len() > FLOW_MINUTES {
            flow.pop_front();
        }
    }
    let bucket = flow.back_mut().expect("pushed above");
    let entry = match bucket
        .entries
        .iter_mut()
        .position(|e| e.item == item && e.source == source)
    {
        Some(i) => &mut bucket.entries[i],
        None => {
            bucket.entries.push(FlowEntry {
                item,
                source,
                produced: 0,
                consumed: 0,
            });
            bucket.entries.last_mut().expect("pushed above")
        }
    };
    if produced {
        entry.produced += qty as u64;
    } else {
        entry.consumed += qty as u64;
    }
}

/// Record one finished craft of `recipe` by the player.
pub fn record_craft(res: &mut Resources, recipe: &crate::recipe_handler::Recipe) {
    for &(item, qty) in recipe.ingredients {
        record(res, item, qty, Source::Crafting, false);
    }
    record(res, recipe.result, recipe.quantity, Source::Crafting, true);
}

/// After power is shared: on each factory second, record every network's figures and
/// drop histories that have been unused too long.
pub fn sample_power(res: &mut Resources, tick: u64) {
    if !res
        .factory_clock
        .steps
        .is_multiple_of(crate::power::STEPS_PER_FACTORY_SECOND)
    {
        return;
    }
    let now: Vec<(Position, PowerSample)> = res
        .power
        .networks
        .iter()
        .map(|n| {
            (
                n.id,
                PowerSample {
                    supply_w: n.supply_w,
                    demand_w: n.demand_w,
                },
            )
        })
        .collect();
    let histories = &mut res.factory_stats.power;
    for (id, sample) in now {
        let history = match histories.iter_mut().position(|h| h.id == id) {
            Some(i) => &mut histories[i],
            None => {
                histories.push(NetworkHistory {
                    id,
                    samples: VecDeque::new(),
                    last_seen_tick: tick,
                });
                histories.last_mut().expect("pushed above")
            }
        };
        history.samples.push_back(sample);
        while history.samples.len() > POWER_SAMPLES {
            history.samples.pop_front();
        }
        history.last_seen_tick = tick;
    }
    histories.retain(|h| tick.saturating_sub(h.last_seen_tick) < HISTORY_EXPIRY_TICKS);
    histories.sort_by_key(|h| (h.id.z, h.id.y, h.id.x));
}

/// The power history of the network with id `id`, oldest first.
pub fn history(res: &Resources, id: Position) -> Option<&NetworkHistory> {
    res.factory_stats.power.iter().find(|h| h.id == id)
}

/// An item's flow over a window: per-minute averages of what was made and used.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ItemRate {
    pub item: ItemKind,
    pub produced_per_min: f64,
    pub consumed_per_min: f64,
}

/// The minutes of flow inside the last `minutes` game minutes, counting this one.
fn window(res: &Resources, minutes: u64) -> impl Iterator<Item = &MinuteFlow> {
    let now = res.time.tick / MINUTE_TICKS;
    let first = (now + 1).saturating_sub(minutes);
    res.factory_stats
        .flow
        .iter()
        .filter(move |m| m.minute >= first && m.minute <= now)
}

/// Every item with any flow in the last `minutes` game minutes, with its average rates,
/// in item order.
pub fn rates(res: &Resources, minutes: u64) -> Vec<ItemRate> {
    let mut totals: Vec<(ItemKind, u64, u64)> = Vec::new();
    for m in window(res, minutes) {
        for e in &m.entries {
            match totals.iter_mut().find(|(i, _, _)| *i == e.item) {
                Some((_, p, c)) => {
                    *p += e.produced;
                    *c += e.consumed;
                }
                None => totals.push((e.item, e.produced, e.consumed)),
            }
        }
    }
    totals.sort_by_key(|&(item, _, _)| item);
    totals
        .into_iter()
        .map(|(item, p, c)| ItemRate {
            item,
            produced_per_min: p as f64 / minutes as f64,
            consumed_per_min: c as f64 / minutes as f64,
        })
        .collect()
}

/// Totals for `item` over the last `minutes` game minutes, by source in source order.
pub fn sources(res: &Resources, item: ItemKind, minutes: u64) -> Vec<(Source, u64, u64)> {
    let mut found: Vec<(Source, u64, u64)> = Vec::new();
    for m in window(res, minutes) {
        for e in m.entries.iter().filter(|e| e.item == item) {
            match found.iter_mut().find(|(s, _, _)| *s == e.source) {
                Some((_, p, c)) => {
                    *p += e.produced;
                    *c += e.consumed;
                }
                None => found.push((e.source, e.produced, e.consumed)),
            }
        }
    }
    found.sort_by_key(|&(s, _, _)| s);
    found
}

/// `item`'s production in each of the last `minutes` game minutes, oldest first (zero for
/// minutes with none).
pub fn produced_per_minute(res: &Resources, item: ItemKind, minutes: u64) -> Vec<u64> {
    let now = res.time.tick / MINUTE_TICKS;
    let first = (now + 1).saturating_sub(minutes);
    (first..=now)
        .map(|minute| {
            res.factory_stats
                .flow
                .iter()
                .find(|m| m.minute == minute)
                .map_or(0, |m| {
                    m.entries
                        .iter()
                        .filter(|e| e.item == item)
                        .map(|e| e.produced)
                        .sum()
                })
        })
        .collect()
}

/// Totals over everything kept, by (item, source): for checking that every place items
/// change is recorded.
pub fn totals(res: &Resources) -> Vec<FlowEntry> {
    let mut all: Vec<FlowEntry> = Vec::new();
    for m in &res.factory_stats.flow {
        for e in &m.entries {
            match all
                .iter_mut()
                .find(|a| a.item == e.item && a.source == e.source)
            {
                Some(a) => {
                    a.produced += e.produced;
                    a.consumed += e.consumed;
                }
                None => all.push(*e),
            }
        }
    }
    all.sort_by_key(|e| (e.item, e.source));
    all
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Inventory;
    use crate::factory::{self, Facing, FACTORY_SECOND_TICKS};
    use crate::intent::{PlayerAction, PlayerIntent};
    use crate::test_support::{TestGame, TestWorld};
    use crate::tiles::TileKind;

    fn at(x: i64, y: i64) -> Position {
        Position { x, y, z: 0 }
    }

    fn wait(game: &mut TestGame, ticks: u64) {
        game.game.res.player_state.intent = PlayerIntent::movement(0, 0, 0, ticks);
        game.tick();
    }

    fn put(game: &mut TestGame, kind: FactoryKind, pos: Position) -> hecs::Entity {
        factory::place(
            &mut game.game.world,
            &mut game.game.res,
            kind,
            pos,
            Facing::North,
        )
        .expect("placed")
    }

    fn give(game: &mut TestGame, e: hecs::Entity, item: ItemKind, qty: u32) {
        game.game
            .world
            .get::<&mut Inventory>(e)
            .unwrap()
            .add(item, qty);
    }

    fn held(game: &TestGame, e: hecs::Entity) -> Vec<(ItemKind, u32)> {
        let mut v: Vec<(ItemKind, u32)> = game
            .game
            .world
            .get::<&Inventory>(e)
            .unwrap()
            .slots
            .iter()
            .map(|s| (s.kind, s.qty))
            .collect();
        v.sort();
        v
    }

    fn total(game: &TestGame, source: Source, produced: bool) -> Vec<(ItemKind, u32)> {
        totals(&game.game.res)
            .into_iter()
            .filter(|e| e.source == source)
            .map(|e| {
                (
                    e.item,
                    if produced { e.produced } else { e.consumed } as u32,
                )
            })
            .filter(|&(_, n)| n > 0)
            .collect()
    }

    /// A generator with logs, a pole and a test consumer drawing 45 kW.
    fn grid(game: &mut TestGame, x: i64) -> hecs::Entity {
        let generator = put(game, FactoryKind::BurnerGenerator, at(x, 0));
        give(game, generator, ItemKind::Log, 3);
        put(game, FactoryKind::PowerPole, at(x + 1, 0));
        let consumer = put(game, FactoryKind::TestConsumer, at(x + 2, 0));
        game.game
            .world
            .get::<&mut crate::power::PowerConsumer>(consumer)
            .unwrap()
            .active = true;
        generator
    }

    #[test]
    fn networks_are_sampled_once_per_factory_second() {
        let mut game = TestWorld::new(1).player_at(0, 20, 0).build();
        grid(&mut game, 0);
        wait(&mut game, 10 * FACTORY_SECOND_TICKS);
        let id = game.game.res.power.networks[0].id;
        let h = history(&game.game.res, id).unwrap();
        assert_eq!(h.samples.len(), 10);
        let last = h.samples.back().unwrap();
        assert_eq!((last.supply_w, last.demand_w), (45_000, 45_000));
        assert!(!last.short());
        wait(&mut game, (POWER_SAMPLES as u64 + 5) * FACTORY_SECOND_TICKS);
        let h = history(&game.game.res, id).unwrap();
        assert_eq!(h.samples.len(), POWER_SAMPLES, "a ring of 10 game minutes");
    }

    #[test]
    fn a_short_network_is_marked_and_merged_networks_keep_the_lowest_history() {
        let mut game = TestWorld::new(1).player_at(0, 20, 0).build();
        grid(&mut game, 0);
        // A second network far enough away to be its own
        put(&mut game, FactoryKind::PowerPole, at(20, 0));
        let hungry = put(&mut game, FactoryKind::TestConsumer, at(21, 0));
        game.game
            .world
            .get::<&mut crate::power::PowerConsumer>(hungry)
            .unwrap()
            .active = true;
        wait(&mut game, 3 * FACTORY_SECOND_TICKS);
        let ids: Vec<Position> = game
            .game
            .res
            .factory_stats
            .power
            .iter()
            .map(|h| h.id)
            .collect();
        assert_eq!(ids, [at(1, 0), at(20, 0)]);
        let starved = history(&game.game.res, at(20, 0)).unwrap();
        assert!(starved.samples.iter().all(|s| s.short()), "no generator");
        // Bridge them: the merged network is id (1, 0), and it keeps that history
        for x in [8, 15] {
            put(&mut game, FactoryKind::PowerPole, at(x, 0));
        }
        wait(&mut game, 2 * FACTORY_SECOND_TICKS);
        assert_eq!(game.game.res.power.networks.len(), 1);
        assert_eq!(history(&game.game.res, at(1, 0)).unwrap().samples.len(), 5);
        assert!(
            history(&game.game.res, at(20, 0)).is_some(),
            "kept until unused for 10 minutes"
        );
        wait(&mut game, HISTORY_EXPIRY_TICKS);
        assert!(history(&game.game.res, at(20, 0)).is_none(), "expired");
    }

    #[test]
    fn recorded_totals_match_what_inventories_gained_and_lost() {
        let mut game = TestWorld::new(7)
            .player_at(0, 0, 0)
            .fill((-8, -8, 0), (8, 8, 0), TileKind::Grass)
            .tile(1, 0, 0, TileKind::Tree)
            .tile(-1, 0, 0, TileKind::Rock)
            .inventory(&[])
            .auto_pickup(true)
            .build();
        // Mining by hand: what the player picked up is what was recorded
        for x in [1, -1] {
            game.act(PlayerAction::MineAt { x, y: 0, z: 0 });
            game.tick();
        }
        for dx in [1, -1, -1, 1] {
            game.act(PlayerAction::Move { dx, dy: 0, dz: 0 });
            game.tick();
        }
        let player = game.game.get_player_entity().unwrap();
        assert_eq!(
            held(&game, player),
            total(&game, Source::PlayerMining, true)
        );
        // Planting a seed by hand uses it up
        crate::plants::plant(
            &mut game.game.world,
            &mut game.game.res,
            at(0, 2),
            ItemKind::Acorn,
        )
        .unwrap();
        assert_eq!(
            total(&game, Source::Planting, false),
            [(ItemKind::Acorn, 1)]
        );
        // Burning fuel: the logs gone from the generator
        let generator = grid(&mut game, -8);
        // A furnace smelting logs on the same grid
        let furnace = put(&mut game, FactoryKind::ElectricFurnace, at(-7, 1));
        {
            let mut f = game
                .game
                .world
                .get::<&mut crate::furnace::Furnace>(furnace)
                .unwrap();
            f.input = Some(crate::components::ItemStack::new(ItemKind::Log, 2));
        }
        wait(&mut game, 20 * FACTORY_SECOND_TICKS);
        let burned = 3 - held(&game, generator).first().map_or(0, |&(_, n)| n);
        assert!(burned > 0);
        assert_eq!(total(&game, Source::Fuel, false), [(ItemKind::Log, burned)]);
        let made = game
            .game
            .world
            .get::<&crate::furnace::Furnace>(furnace)
            .unwrap()
            .output
            .map_or(0, |s| s.qty);
        assert_eq!(made, 2);
        let source = Source::Machine(FactoryKind::ElectricFurnace);
        assert_eq!(total(&game, source, false), [(ItemKind::Log, 2)]);
        assert_eq!(total(&game, source, true), [(ItemKind::Biochar, 2)]);
    }

    #[test]
    fn rates_average_over_their_window_by_source() {
        let mut game = TestWorld::new(1).build();
        let res = &mut game.game.res;
        record(res, ItemKind::Log, 6, Source::PlayerMining, true);
        record(
            res,
            ItemKind::Log,
            4,
            Source::Machine(FactoryKind::AutoHarvester),
            true,
        );
        record(res, ItemKind::Log, 2, Source::Fuel, false);
        res.time.tick = 3 * MINUTE_TICKS;
        record(res, ItemKind::Log, 10, Source::PlayerMining, true);
        let one = rates(res, 1);
        assert_eq!(one.len(), 1);
        assert_eq!(one[0].produced_per_min, 10.0);
        let ten = rates(res, 10);
        assert_eq!(ten[0].produced_per_min, 2.0);
        assert_eq!(ten[0].consumed_per_min, 0.2);
        assert_eq!(
            sources(res, ItemKind::Log, 10),
            [
                (Source::PlayerMining, 16, 0),
                (Source::Machine(FactoryKind::AutoHarvester), 4, 0),
                (Source::Fuel, 0, 2),
            ]
        );
        assert_eq!(produced_per_minute(res, ItemKind::Log, 4), [10, 0, 0, 10]);
        res.time.tick = (FLOW_MINUTES as u64 + 10) * MINUTE_TICKS;
        record(res, ItemKind::Stone, 1, Source::PlayerMining, true);
        assert!(
            res.factory_stats.flow.len() <= FLOW_MINUTES,
            "old minutes drop off"
        );
        assert!(rates(res, 60).iter().all(|r| r.item == ItemKind::Stone));
    }

    #[test]
    fn stats_survive_save_and_reload() {
        let mut game = TestWorld::new(1).player_at(0, 20, 0).build();
        grid(&mut game, 0);
        wait(&mut game, 4 * FACTORY_SECOND_TICKS);
        let before = game.game.res.factory_stats.clone();
        assert!(!before.flow.is_empty() && !before.power.is_empty());
        let saved = crate::save_load::save_to_string(&game.game, &serde_json::Value::Null).unwrap();
        let mut loaded = TestWorld::new(2).build().game;
        crate::save_load::load_from_str(&mut loaded, &saved).unwrap();
        assert_eq!(loaded.res.factory_stats, before);
    }
}

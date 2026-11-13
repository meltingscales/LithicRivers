//! The Aperture, the game's goal: an 11-by-11 biomechanical gate. A core sits at the
//! center, frame blocks fill the border, and five input ports at fixed border positions
//! each take one kind of item. While complete, each game minute it charges by how well
//! every input kept up with its required rate; at `OPEN_CHARGE` it opens.
//!
//! Ports count what arrives; the core gathers their counts each factory step, keeps the
//! last ten minutes, and settles the charge once per game minute crossed (time jumps by
//! whole actions, so several minutes can settle in one step).

use crate::components::{ItemKind, Position};
use crate::factory::{FactoryBlock, FactoryKind};
use crate::factory_stats::MINUTE_TICKS;
use crate::resources::Resources;
use hecs::{Entity, World};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Half the Aperture's width: the border is this many tiles from the core.
pub const HALF: i64 = 5;
/// Charge, in milli-points, that opens it (120 points, two hours at full rates).
pub const OPEN_CHARGE: u64 = 120_000;
/// Minutes of arrivals kept for the flow window.
pub const HISTORY_MINUTES: usize = 10;
/// An input below half its rate this many minutes in a row drains the charge.
pub const SHORTAGE_MINUTES: u8 = 3;

/// What Interact on an open Aperture says.
pub const FAR_SIDE_NOT_READY: &str = "The far side is not ready for you yet.";

/// The core whose ring could include `pos` (within `HALF` of it on its layer).
pub fn core_near(world: &World, pos: Position) -> Option<Entity> {
    world
        .query::<&Position>()
        .with::<&Aperture>()
        .iter()
        .find(|(_, c)| c.z == pos.z && (c.x - pos.x).abs() <= HALF && (c.y - pos.y).abs() <= HALF)
        .map(|(e, _)| e)
}

/// One input: its port's offset from the core, its item and its required rate in
/// milli-items per game minute.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Input {
    pub offset: (i64, i64),
    pub item: ItemKind,
    pub milli_per_minute: u64,
}

/// The Aperture's inputs, in port order.
pub const INPUTS: [Input; 5] = [
    Input {
        offset: (-HALF, -HALF),
        item: ItemKind::BiomechEmbryoPerfect,
        milli_per_minute: 200,
    },
    Input {
        offset: (0, -HALF),
        item: ItemKind::MeatSlurry,
        milli_per_minute: 4_000,
    },
    Input {
        offset: (HALF, 0),
        item: ItemKind::Ethanol,
        milli_per_minute: 6_000,
    },
    Input {
        offset: (0, HALF),
        item: ItemKind::Mash,
        milli_per_minute: 3_000,
    },
    Input {
        offset: (-HALF, 0),
        item: ItemKind::Biochar,
        milli_per_minute: 8_000,
    },
];

/// A port: the input it takes while its Aperture is complete, and what arrived since the
/// core last gathered.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AperturePort {
    pub input: Option<ItemKind>,
    pub arrived: u32,
}

/// The core's state.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Aperture {
    pub complete: bool,
    /// The first border position with the wrong block, while incomplete
    pub missing: Option<Position>,
    pub charge_milli: u64,
    /// Arrivals per input in the minute under way
    pub current: [u32; 5],
    /// Arrivals per input in each finished minute, newest last
    pub history: VecDeque<[u32; 5]>,
    /// Minutes in a row each input has been below half its rate
    pub short_streak: [u8; 5],
    /// The game minute under way
    pub minute: u64,
    pub opened: bool,
    /// Whether the client has played the closing sequence for this opening
    #[serde(default)]
    pub ending_seen: bool,
}

/// Whether a port takes `item` now.
pub fn port_accepts(world: &World, port: Entity, item: ItemKind) -> bool {
    world
        .get::<&AperturePort>(port)
        .is_ok_and(|p| p.input == Some(item))
}

/// Count one item into a port (the caller checked `port_accepts`).
pub fn port_insert(port: &mut AperturePort) {
    port.arrived += 1;
}

/// The block each border offset needs: the port of an input there (with its item), else a
/// frame.
pub fn needed(offset: (i64, i64)) -> (FactoryKind, Option<ItemKind>) {
    INPUTS
        .iter()
        .find(|i| i.offset == offset)
        .map_or((FactoryKind::ApertureFrame, None), |i| {
            (FactoryKind::AperturePort, Some(i.item))
        })
}

/// The border offsets, in a fixed order (row by row).
fn border() -> impl Iterator<Item = (i64, i64)> {
    (-HALF..=HALF)
        .flat_map(|dy| (-HALF..=HALF).map(move |dx| (dx, dy)))
        .filter(|&(dx, dy)| dx.abs() == HALF || dy.abs() == HALF)
}

/// Check a core's border, give each port its input when complete (none otherwise), and
/// return the first missing position.
fn check(world: &mut World, res: &Resources, core: Position) -> Option<Position> {
    let mut missing = None;
    let mut ports = Vec::new();
    for offset in border() {
        let pos = Position {
            x: core.x + offset.0,
            y: core.y + offset.1,
            z: core.z,
        };
        let (kind, input) = needed(offset);
        let block = res
            .factory_index
            .get(&pos)
            .and_then(|&e| world.get::<&FactoryBlock>(e).ok().map(|b| (e, b.kind)));
        match block {
            Some((e, k)) if k == kind => {
                if let Some(item) = input {
                    ports.push((e, item));
                }
            }
            _ => {
                missing.get_or_insert(pos);
            }
        }
    }
    for (port, item) in ports {
        if let Ok(mut p) = world.get::<&mut AperturePort>(port) {
            p.input = missing.is_none().then_some(item);
        }
    }
    missing
}

/// Settle one finished minute's arrivals into the charge. Each input is judged over the
/// kept minutes (up to ten), so a slow input like embryos, a fraction of one a minute,
/// isn't short in every minute it happens not to arrive.
fn settle(a: &mut Aperture) {
    let arrived = std::mem::take(&mut a.current);
    a.history.push_back(arrived);
    while a.history.len() > HISTORY_MINUTES {
        a.history.pop_front();
    }
    let minutes = a.history.len() as u64;
    let mut lowest = 1000u64;
    let mut drained = false;
    for (i, input) in INPUTS.iter().enumerate() {
        let total: u64 = a.history.iter().map(|m| m[i] as u64).sum();
        let permille = (total * 1_000_000 / (input.milli_per_minute * minutes)).min(1000);
        lowest = lowest.min(permille);
        a.short_streak[i] = if permille < 500 {
            a.short_streak[i].saturating_add(1)
        } else {
            0
        };
        drained |= a.short_streak[i] >= SHORTAGE_MINUTES;
    }
    a.charge_milli = if drained {
        a.charge_milli.saturating_sub(1000)
    } else {
        (a.charge_milli + lowest).min(OPEN_CHARGE)
    };
    if a.charge_milli >= OPEN_CHARGE {
        a.opened = true;
    }
}

/// One factory step: check every core, gather its ports' arrivals, and settle each game
/// minute crossed while complete.
pub fn step(world: &mut World, res: &mut Resources, step_tick: u64) {
    let cores: Vec<(Entity, Position)> = world
        .query::<&Position>()
        .with::<&Aperture>()
        .iter()
        .map(|(e, p)| (e, *p))
        .collect();
    let now = step_tick / MINUTE_TICKS;
    for (core, pos) in cores {
        let missing = check(world, res, pos);
        let mut gathered = [0u32; 5];
        for (i, input) in INPUTS.iter().enumerate() {
            let at = Position {
                x: pos.x + input.offset.0,
                y: pos.y + input.offset.1,
                z: pos.z,
            };
            if let Some(&port) = res.factory_index.get(&at) {
                if let Ok(mut p) = world.get::<&mut AperturePort>(port) {
                    gathered[i] = std::mem::take(&mut p.arrived);
                }
            }
        }
        let Ok(mut a) = world.get::<&mut Aperture>(core) else {
            continue;
        };
        let was_open = a.opened;
        let was_complete = a.complete;
        a.complete = missing.is_none();
        a.missing = missing;
        // A newly finished frame starts counting from now, not from the world's start
        if a.complete && !was_complete {
            a.minute = now;
        }
        if a.opened {
            continue;
        }
        if !a.complete {
            a.minute = now;
            continue;
        }
        for (c, g) in a.current.iter_mut().zip(gathered) {
            *c += g;
        }
        while a.minute < now {
            settle(&mut a);
            a.minute += 1;
            if a.opened {
                break;
            }
        }
        if a.opened && !was_open {
            drop(a);
            show_open(world, res, core);
        }
    }
}

/// An opened Aperture whose closing sequence hasn't played yet.
pub fn ending_due(world: &World) -> Option<Entity> {
    world
        .query::<&Aperture>()
        .iter()
        .find(|(_, a)| a.opened && !a.ending_seen)
        .map(|(e, _)| e)
}

/// Open a complete Aperture at once (the `open_aperture` cheat and tests).
pub fn force_open(world: &mut World, res: &mut Resources, core: Entity) {
    if let Ok(mut a) = world.get::<&mut Aperture>(core) {
        a.charge_milli = OPEN_CHARGE;
        a.opened = true;
    }
    show_open(world, res, core);
}

fn show_open(world: &mut World, res: &mut Resources, core: Entity) {
    let open = crate::components::SpriteRef::new("factory", "aperture_core_open");
    let _ = world.insert_one(core, open);
    res.log("The Aperture opens.");
}

/// How an input is keeping up, for the flow window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flow {
    Green,
    Yellow,
    Red,
}

/// An input's flow over the kept history: its ten-minute rate against the requirement,
/// red when falling over the last three minutes.
pub fn flow(a: &Aperture, index: usize) -> (u64, Flow) {
    let input = INPUTS[index];
    if a.history.is_empty() {
        return (0, Flow::Red);
    }
    let total: u64 = a.history.iter().map(|m| m[index] as u64).sum();
    let percent = total * 100_000 / (input.milli_per_minute * a.history.len() as u64);
    let recent: Vec<u32> = a.history.iter().rev().take(3).map(|m| m[index]).collect();
    let falling = recent.len() == 3 && recent[0] < recent[1] && recent[1] < recent[2];
    let color = if percent < 95 || falling {
        Flow::Red
    } else if percent < 110 {
        Flow::Yellow
    } else {
        Flow::Green
    };
    (percent, color)
}

/// Game minutes left at the ten-minute rates, or `None` when it isn't gaining.
pub fn minutes_left(a: &Aperture) -> Option<u64> {
    if a.history.is_empty() {
        return None;
    }
    let lowest = (0..INPUTS.len())
        .map(|i| flow(a, i).0.min(100))
        .min()
        .unwrap_or(0);
    (lowest > 0).then(|| (OPEN_CHARGE - a.charge_milli).div_ceil(lowest * 10))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::{insert, Facing};
    use crate::game::Game;
    use crate::tiles::TileKind;

    fn at(x: i64, y: i64) -> Position {
        Position { x, y, z: 0 }
    }

    fn put(g: &mut Game, kind: FactoryKind, pos: Position) -> Entity {
        crate::factory::place(&mut g.world, &mut g.res, kind, pos, Facing::North).unwrap()
    }

    /// A complete Aperture with its core at the origin; returns the core.
    fn built() -> (Game, Entity) {
        let mut g = crate::test_support::TestWorld::new(9)
            .fill((-8, -8, 0), (8, 8, 0), TileKind::Grass)
            .player_at(0, 7, 0)
            .build()
            .game;
        let core = put(&mut g, FactoryKind::ApertureCore, at(0, 0));
        for offset in border() {
            let (kind, _) = needed(offset);
            put(&mut g, kind, at(offset.0, offset.1));
        }
        (g, core)
    }

    fn port(g: &Game, i: usize) -> Entity {
        let (dx, dy) = INPUTS[i].offset;
        g.res.factory_index[&at(dx, dy)]
    }

    fn step_once(g: &mut Game) {
        g.res.time.tick += crate::factory::FACTORY_STEP_TICKS;
        crate::factory::run_due_steps(&mut g.world, &mut g.res);
    }

    /// Feed each input `percent` of its rate (by `rates`) for one game minute.
    fn feed_minute(g: &mut Game, rates: [u64; 5]) {
        for (i, input) in INPUTS.iter().enumerate() {
            let n = input.milli_per_minute * rates[i] / 100_000;
            let p = port(g, i);
            for _ in 0..n {
                assert!(insert(&mut g.world, p, input.item));
            }
        }
        g.res.time.tick += MINUTE_TICKS;
        crate::factory::run_due_steps(&mut g.world, &mut g.res);
    }

    fn state(g: &Game, core: Entity) -> Aperture {
        (*g.world.get::<&Aperture>(core).unwrap()).clone()
    }

    /// Feeds embryos as a whole number by scaling the minute (the embryo rate is 0.2).
    fn full() -> [u64; 5] {
        [100; 5]
    }

    #[test]
    fn a_complete_aperture_is_complete_and_a_missing_frame_is_named() {
        let (mut g, core) = built();
        step_once(&mut g);
        assert!(state(&g, core).complete);
        let player = g.get_player_entity().unwrap();
        crate::factory::break_block(&mut g.world, &mut g.res, at(2, -HALF), player);
        step_once(&mut g);
        let a = state(&g, core);
        assert!(!a.complete);
        assert_eq!(a.missing, Some(at(2, -HALF)));
        assert!(
            !port_accepts(&g.world, port(&g, 1), ItemKind::MeatSlurry),
            "ports idle"
        );
    }

    #[test]
    fn a_frame_where_a_port_belongs_leaves_it_incomplete() {
        let (mut g, core) = built();
        let player = g.get_player_entity().unwrap();
        crate::factory::break_block(&mut g.world, &mut g.res, at(HALF, 0), player);
        put(&mut g, FactoryKind::ApertureFrame, at(HALF, 0));
        step_once(&mut g);
        assert_eq!(state(&g, core).missing, Some(at(HALF, 0)));
    }

    #[test]
    fn each_port_takes_only_its_item() {
        let (mut g, _) = built();
        step_once(&mut g);
        assert!(port_accepts(&g.world, port(&g, 2), ItemKind::Ethanol));
        assert!(!port_accepts(&g.world, port(&g, 2), ItemKind::Biochar));
    }

    #[test]
    fn full_rates_open_it_after_two_hours() {
        let (mut g, core) = built();
        step_once(&mut g);
        // The embryo rate is 0.2 a minute: one every 5 minutes is full rate
        for minute in 0..120u64 {
            let mut rates = full();
            rates[0] = if minute % 5 == 0 { 500 } else { 0 };
            feed_minute(&mut g, rates);
        }
        let a = state(&g, core);
        assert_eq!(a.charge_milli, OPEN_CHARGE);
        assert!(a.opened);
    }

    #[test]
    fn a_short_input_adds_only_its_fraction() {
        let (mut g, core) = built();
        step_once(&mut g);
        let mut rates = full();
        rates[0] = 500;
        rates[1] = 75;
        feed_minute(&mut g, rates);
        assert_eq!(state(&g, core).charge_milli, 750, "3 of 4 meat slurry");
    }

    #[test]
    fn a_long_shortage_drains_a_point_and_stops_at_zero() {
        let (mut g, core) = built();
        step_once(&mut g);
        let mut good = full();
        good[0] = 500;
        feed_minute(&mut g, good);
        assert_eq!(state(&g, core).charge_milli, 1000);
        // Biochar stops: over the window it falls below half within a minute, and three
        // such minutes in a row drain a point each from then on
        let mut short = good;
        short[4] = 0;
        let mut charges = Vec::new();
        for _ in 0..6 {
            feed_minute(&mut g, short);
            charges.push(state(&g, core).charge_milli);
        }
        assert!(charges[0] >= 1000, "{charges:?}");
        assert!(
            charges.windows(2).any(|w| w[1] < w[0]),
            "it drains: {charges:?}"
        );
        for _ in 0..20 {
            feed_minute(&mut g, short);
        }
        assert_eq!(state(&g, core).charge_milli, 0, "never below zero");
    }

    #[test]
    fn flow_colors_follow_the_rates() {
        let mut a = Aperture::default();
        for _ in 0..3 {
            a.history.push_back([1, 5, 6, 2, 8]);
        }
        // meat slurry 125%, ethanol 100%, mash 66%
        assert_eq!(flow(&a, 1).1, Flow::Green);
        assert_eq!(flow(&a, 2).1, Flow::Yellow);
        assert_eq!(flow(&a, 3).1, Flow::Red);
    }

    #[test]
    fn charging_replays_the_same() {
        let run = || {
            let (mut g, core) = built();
            step_once(&mut g);
            for m in 0..20u64 {
                let rates = [500, 100, (m * 7) % 130, 90, 100];
                feed_minute(&mut g, rates);
            }
            state(&g, core).charge_milli
        };
        assert_eq!(run(), run());
    }
}

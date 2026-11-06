//! Teleporters: factory blocks that move the player to another teleporter. Each has a
//! battery bank that charges from its power network, and a jump spends charge from the
//! source teleporter's bank, more the farther it goes.
//!
//! SapienCorp transit stations stand in the quest locations. Each is dormant until the
//! player first uses it; after that it reaches the other activated stations, paid from a
//! bank that the RTG beside it keeps charging, and player-built teleporters list it too.
//!
//! All energy is whole joules, so charging and jump costs stay exact and deterministic.

use hecs::{Entity, World};
use serde::{Deserialize, Serialize};

use crate::components::{BlocksMovement, Position};
use crate::factory::STEPS_PER_FACTORY_SECOND;
use crate::power::PowerConsumer;

/// The most a battery bank holds, in joules.
pub const CAPACITY_J: u64 = 50_000_000;
/// The fastest a battery bank charges, in watts (joules per factory second).
pub const CHARGE_W: u64 = 200_000;
/// What every jump costs, in joules, before distance.
pub const JUMP_BASE_J: u64 = 10_000_000;
/// What a jump costs per tile of distance, in joules.
pub const JUMP_PER_TILE_J: u64 = 5_000;
/// Tiles of distance a layer of difference counts as.
pub const LAYER_TILES: u64 = 100;
/// Default names are this followed by a number.
pub const DEFAULT_NAME_PREFIX: &str = "Teleporter ";
/// The longest name a teleporter takes, in characters.
pub const MAX_NAME_CHARS: usize = 24;

/// A teleporter block's state: its name and its battery bank's charge.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Teleporter {
    pub name: String,
    pub charge_j: u64,
}

/// A SapienCorp transit station, built into a quest location: it can't be broken or picked
/// up, and does nothing until the player activates it. Its battery bank charges from the
/// RTG beside it (a station from an older save starts full).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransitStation {
    pub name: String,
    pub activated: bool,
    #[serde(default = "full_bank")]
    pub charge_j: u64,
}

fn full_bank() -> u64 {
    CAPACITY_J
}

impl Default for TransitStation {
    /// A dormant, unnamed station with a full bank.
    fn default() -> Self {
        Self {
            name: String::new(),
            activated: false,
            charge_j: CAPACITY_J,
        }
    }
}

/// The charge in a teleporter's or station's battery bank.
pub fn bank_charge(world: &World, entity: Entity) -> Option<u64> {
    if let Ok(t) = world.get::<&Teleporter>(entity) {
        return Some(t.charge_j);
    }
    world
        .get::<&TransitStation>(entity)
        .ok()
        .map(|s| s.charge_j)
}

/// Spend joules from a teleporter's or station's bank (the caller checked it holds them).
fn spend(world: &mut World, entity: Entity, joules: u64) {
    if let Ok(mut t) = world.get::<&mut Teleporter>(entity) {
        t.charge_j -= joules;
    } else if let Ok(mut s) = world.get::<&mut TransitStation>(entity) {
        s.charge_j -= joules;
    }
}

/// What a station built into a structure is called; `structure` is its folder name, with
/// or without the `.lrstructure` suffix.
/// `inner` is a structure's second station, deeper in (the factory's, by the arm locker).
pub fn station_name(structure: &str, inner: bool) -> &'static str {
    let structure = structure.trim_end_matches(".lrstructure");
    if structure == crate::game::FACILITY_STRUCTURE.trim_end_matches(".lrstructure") {
        return if inner {
            "Locker Station"
        } else {
            "Factory Station"
        };
    }
    match (structure, inner) {
        ("sapiencorp-bunker", _) => "Bunker Station",
        (_, false) => "Transit Station",
        (_, true) => "Inner Transit Station",
    }
}

/// The message logged when a station is activated.
pub const ACTIVATION_MESSAGE: &str =
    "Handshake accepted. Transit node registered: {name}. Reserve power: nominal.";

/// The distance a jump covers: the larger of the x and y gaps, plus `LAYER_TILES` for each
/// layer between them.
pub fn distance(a: Position, b: Position) -> u64 {
    a.x.abs_diff(b.x).max(a.y.abs_diff(b.y)) + LAYER_TILES * a.z.abs_diff(b.z)
}

/// What a jump between two positions costs, in joules: never more than a full bank.
pub fn jump_cost(from: Position, to: Position) -> u64 {
    JUMP_BASE_J
        .saturating_add(JUMP_PER_TILE_J.saturating_mul(distance(from, to)))
        .min(CAPACITY_J)
}

/// Joules as megajoules for the player, with one decimal where it matters ("8 MJ",
/// "12.5 MJ"), rounded up so a "needs more" figure is never too small.
pub fn format_mj(joules: u64) -> String {
    let tenths = joules.div_ceil(100_000);
    if tenths % 10 == 0 {
        format!("{} MJ", tenths / 10)
    } else {
        format!("{}.{} MJ", tenths / 10, tenths % 10)
    }
}

/// A fresh default name: "Teleporter N", one higher than any number in use.
pub fn default_name(world: &World) -> String {
    let highest = world
        .query::<&Teleporter>()
        .iter()
        .filter_map(|(_, t)| {
            t.name
                .strip_prefix(DEFAULT_NAME_PREFIX)?
                .parse::<u64>()
                .ok()
        })
        .max()
        .unwrap_or(0);
    format!("{DEFAULT_NAME_PREFIX}{}", highest + 1)
}

/// Rename a teleporter. The name is trimmed and cut to `MAX_NAME_CHARS`; an empty name is
/// refused. Returns whether it was renamed.
pub fn rename(world: &mut World, teleporter: Entity, name: &str) -> bool {
    let name: String = name.trim().chars().take(MAX_NAME_CHARS).collect();
    if name.is_empty() {
        return false;
    }
    match world.get::<&mut Teleporter>(teleporter) {
        Ok(mut t) => {
            t.name = name;
            true
        }
        Err(_) => false,
    }
}

/// Before power is shared: a bank wants power only while it isn't full, and never more
/// than it has room for this step.
pub fn update_demand(world: &mut World) {
    let want = |charge_j: u64, consumer: &mut PowerConsumer| {
        let room_j = CAPACITY_J.saturating_sub(charge_j);
        consumer.demand_w = CHARGE_W.min(room_j * STEPS_PER_FACTORY_SECOND);
        consumer.active = consumer.demand_w > 0;
    };
    for (_, (t, consumer)) in world.query_mut::<(&Teleporter, &mut PowerConsumer)>() {
        want(t.charge_j, consumer);
    }
    for (_, (s, consumer)) in world.query_mut::<(&TransitStation, &mut PowerConsumer)>() {
        want(s.charge_j, consumer);
    }
}

/// Joules a bank takes from its network in one step at its consumer's satisfaction.
fn delivered_j(consumer: &PowerConsumer) -> u64 {
    if !consumer.active {
        return 0;
    }
    consumer.demand_w / STEPS_PER_FACTORY_SECOND * u64::from(consumer.satisfaction_permille) / 1000
}

/// What a bank is charging at right now, in watts.
pub fn charge_rate_w(consumer: &PowerConsumer) -> u64 {
    delivered_j(consumer) * STEPS_PER_FACTORY_SECOND
}

/// After power is shared: each bank stores what its network delivered.
pub fn step_charging(world: &mut World) {
    for (_, (teleporter, consumer)) in world.query_mut::<(&mut Teleporter, &PowerConsumer)>() {
        teleporter.charge_j = (teleporter.charge_j + delivered_j(consumer)).min(CAPACITY_J);
    }
    for (_, (station, consumer)) in world.query_mut::<(&mut TransitStation, &PowerConsumer)>() {
        station.charge_j = (station.charge_j + delivered_j(consumer)).min(CAPACITY_J);
    }
}

/// Activate a dormant station, lighting it up. Returns its name if it was dormant.
pub fn activate(world: &mut World, station: Entity) -> Option<String> {
    let name = match world.get::<&mut TransitStation>(station) {
        Ok(mut s) if !s.activated => {
            s.activated = true;
            s.name.clone()
        }
        _ => return None,
    };
    let _ = world.insert_one(
        station,
        crate::components::SpriteRef::new("factory", "transit_station_active"),
    );
    Some(name)
}

/// Register the bunker's station, where a new player wakes up, without a message.
pub fn register_spawn_station(world: &mut World) {
    let spawn = station_name("sapiencorp-bunker", false);
    let bunker: Vec<Entity> = world
        .query::<&TransitStation>()
        .iter()
        .filter(|(_, s)| s.name == spawn)
        .map(|(e, _)| e)
        .collect();
    for station in bunker {
        activate(world, station);
    }
}

/// Where a jump can be made from, and what it lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// A player-built teleporter: lists teleporters and activated stations, paid from its bank
    Teleporter,
    /// An activated transit station: lists the other activated stations, paid from its
    /// RTG-charged bank
    Station,
    /// The player carrying a mobile tele-access device: lists every teleporter and
    /// activated station, at the jump cost from where they stand, paid from carried cells
    Device,
}

/// Whether `entity` carries a mobile tele-access device.
pub fn carries_device(world: &World, entity: Entity) -> bool {
    world
        .get::<&crate::components::Inventory>(entity)
        .is_ok_and(|inv| {
            inv.slots
                .iter()
                .any(|s| s.kind == crate::components::ItemKind::MobileTeleAccessDevice)
        })
}

/// What kind of jump source an entity is, if any. A dormant station isn't one, nor is a
/// player without the device.
pub fn source(world: &World, entity: Entity) -> Option<Source> {
    if world.get::<&Teleporter>(entity).is_ok() {
        return Some(Source::Teleporter);
    }
    if world.get::<&crate::components::Player>(entity).is_ok() {
        return carries_device(world, entity).then_some(Source::Device);
    }
    match world.get::<&TransitStation>(entity) {
        Ok(station) if station.activated => Some(Source::Station),
        _ => None,
    }
}

/// Another teleporter or station as the window lists it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Destination {
    pub entity: Entity,
    pub name: String,
    pub position: Position,
    pub distance: u64,
    pub cost_j: u64,
}

/// Where `from` can jump to, nearest first, then by position: from a teleporter, every
/// other teleporter and every activated station, at the jump cost; from an activated
/// station, every other activated station, at the jump cost; from a player with the device, every
/// teleporter and activated station, at the jump cost. Dormant stations are never listed.
pub fn destinations(world: &World, from: Entity) -> Vec<Destination> {
    let (Ok(origin), Some(source)) = (
        world.get::<&Position>(from).map(|p| *p),
        source(world, from),
    ) else {
        return Vec::new();
    };
    let stations = world
        .query::<(&Position, &TransitStation)>()
        .iter()
        .filter(|(_, (_, s))| s.activated)
        .map(|(e, (pos, s))| (e, *pos, s.name.clone()))
        .collect::<Vec<_>>();
    let teleporters = match source {
        Source::Teleporter | Source::Device => world
            .query::<(&Position, &Teleporter)>()
            .iter()
            .map(|(e, (pos, t))| (e, *pos, t.name.clone()))
            .collect(),
        Source::Station => Vec::new(),
    };
    let mut found: Vec<Destination> = teleporters
        .into_iter()
        .chain(stations)
        .filter(|(e, _, _)| *e != from)
        .map(|(entity, pos, name)| Destination {
            entity,
            name,
            position: pos,
            distance: distance(origin, pos),
            cost_j: jump_cost(origin, pos),
        })
        .collect();
    found.sort_by_key(|d| (d.distance, d.position.z, d.position.y, d.position.x));
    found
}

/// The tiles a player may land on around a teleporter, in the order they are tried:
/// north, east, south, west, then the diagonals clockwise from north-east.
pub fn landing_order(pos: Position) -> [Position; 8] {
    [
        (0, -1),
        (1, 0),
        (0, 1),
        (-1, 0),
        (1, -1),
        (1, 1),
        (-1, 1),
        (-1, -1),
    ]
    .map(|(dx, dy)| Position {
        x: pos.x + dx,
        y: pos.y + dy,
        z: pos.z,
    })
}

/// Why a jump didn't happen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JumpError {
    /// `to` isn't in `from`'s destination list
    NotTeleporters,
    /// The source bank needs this many more joules
    ShortOfCharge { needed_j: u64 },
    /// Nowhere free to stand beside the destination
    NoRoom,
    /// The device's carried cells can't cover the jump
    NoCharge(crate::power_cells::NoCharge),
}

impl JumpError {
    /// The message log line for it.
    pub fn message(&self) -> String {
        match self {
            JumpError::NotTeleporters => "The teleporter can't find that destination.".to_string(),
            JumpError::ShortOfCharge { needed_j } => format!(
                "The battery bank needs {} more charge for that jump.",
                format_mj(*needed_j)
            ),
            JumpError::NoRoom => {
                "The destination teleporter is boxed in: there's nowhere to stand beside it."
                    .to_string()
            }
            JumpError::NoCharge(no_charge) => no_charge.message(),
        }
    }
}

impl crate::Game {
    /// Move the player from `from` (a teleporter, an activated station, or the player with
    /// the device) to beside `to`, one of its `destinations`, paying the jump's cost from
    /// `from`'s bank (a teleporter's or station's), or for the device from the fullest
    /// carried cell. The destination's chunk is generated first; the player lands on the first
    /// free walkable tile in `landing_order`, and the world around them is loaded at once.
    /// Passes no time. Returns where the player landed.
    pub fn teleport(&mut self, from: Entity, to: Entity) -> Result<Position, JumpError> {
        let Some(dest) = destinations(&self.world, from)
            .into_iter()
            .find(|d| d.entity == to)
        else {
            return Err(JumpError::NotTeleporters);
        };
        let (target, name, cost) = (dest.position, dest.name, dest.cost_j);
        let device = source(&self.world, from) == Some(Source::Device);
        if device {
            let fullest = crate::power_cells::fullest_charge(&self.world, from);
            if fullest < cost {
                return Err(JumpError::NoCharge(crate::power_cells::NoCharge {
                    short_j: cost - fullest,
                }));
            }
        }
        let charge = bank_charge(&self.world, from).unwrap_or(0);
        if !device && charge < cost {
            return Err(JumpError::ShortOfCharge {
                needed_j: cost - charge,
            });
        }
        let player = self.get_player_entity();
        let landing = landing_order(target).into_iter().find(|&pos| {
            self.factory_tile(pos);
            crate::logistics::walkable(&mut self.res, &self.world, pos)
                && !self
                    .world
                    .query::<(&BlocksMovement, &Position)>()
                    .iter()
                    .any(|(e, (_, p))| *p == pos && Some(e) != player)
        });
        let Some(landing) = landing else {
            return Err(JumpError::NoRoom);
        };
        if device {
            crate::power_cells::use_power(&mut self.world, from, cost)
                .map_err(JumpError::NoCharge)?;
        } else {
            spend(&mut self.world, from, cost);
        }
        if let Some(player) = player {
            if let Ok(mut pos) = self.world.get::<&mut Position>(player) {
                *pos = landing;
            }
        }
        self.load_around_player();
        self.res.log_green(format!(
            "The teleporter hums, and the world folds. You step out beside {name}."
        ));
        Ok(landing)
    }

    /// Activate a dormant station and log the handshake. Returns whether it was dormant.
    pub fn activate_station(&mut self, station: Entity) -> bool {
        let Some(name) = activate(&mut self.world, station) else {
            return false;
        };
        self.res
            .log_green(ACTIVATION_MESSAGE.replace("{name}", &name));
        true
    }

    /// Turn the transit-station tiles that structures placed into station blocks, named for
    /// their structure, and the RTG, item-charger and pole tiles beside them into their
    /// blocks: the RTG and charger built in, the pole an ordinary one.
    pub fn place_transit_stations(&mut self) {
        use crate::factory::FactoryKind;
        use crate::tiles::TileKind;
        for (x, y, z, tile) in std::mem::take(&mut self.res.world_state.world.structure_blocks) {
            let (kind, built_in) = match tile {
                TileKind::SapiencorpRtg => (FactoryKind::Rtg, true),
                TileKind::StationItemCharger => (FactoryKind::ItemCharger, true),
                _ => (FactoryKind::PowerPole, false),
            };
            // Something already stands there: the structure goes without
            if let Ok(entity) = crate::factory::place(
                &mut self.world,
                &mut self.res,
                kind,
                Position { x, y, z },
                crate::factory::Facing::North,
            ) {
                if built_in {
                    let _ = self.world.insert_one(entity, crate::factory::BuiltIn);
                }
            }
        }
        for (x, y, z, name) in std::mem::take(&mut self.res.world_state.world.transit_stations) {
            let pos = Position { x, y, z };
            // Something already stands there: the structure goes without
            if let Ok(entity) = crate::factory::place(
                &mut self.world,
                &mut self.res,
                crate::factory::FactoryKind::TransitStation,
                pos,
                crate::factory::Facing::North,
            ) {
                if let Ok(mut s) = self.world.get::<&mut TransitStation>(entity) {
                    s.name = name.to_string();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::{self, Facing, FactoryKind};
    use crate::test_support::TestWorld;
    use crate::tiles::TileKind;
    use crate::Game;

    fn at(x: i64, y: i64, z: i64) -> Position {
        Position { x, y, z }
    }

    fn game() -> Game {
        TestWorld::new(4)
            .fill((-12, -12, 0), (12, 12, 0), TileKind::Grass)
            .player_at(0, 0, 0)
            .build()
            .game
    }

    fn put(game: &mut Game, kind: FactoryKind, pos: Position) -> Entity {
        game.res
            .world_state
            .world
            .set_tile_cached(pos.x, pos.y, pos.z, TileKind::Grass);
        factory::place(&mut game.world, &mut game.res, kind, pos, Facing::North).expect("placed")
    }

    /// A generator with plenty of fuel and a pole covering both it and `pos`.
    fn power(game: &mut Game, pos: Position, generators: i64) {
        for i in 0..generators {
            let g = put(
                game,
                FactoryKind::BurnerGenerator,
                at(pos.x - 2 + i, pos.y + 2, pos.z),
            );
            game.world
                .get::<&mut crate::components::Inventory>(g)
                .unwrap()
                .add(crate::components::ItemKind::Biochar, 99);
        }
        put(game, FactoryKind::PowerPole, at(pos.x, pos.y + 1, pos.z));
    }

    fn pass_factory_seconds(game: &mut Game, seconds: u64) {
        for _ in 0..seconds {
            game.res.player_state.intent = crate::intent::PlayerIntent::movement(
                0,
                0,
                0,
                crate::factory::FACTORY_SECOND_TICKS,
            );
            game.tick();
        }
    }

    fn charge(game: &Game, teleporter: Entity) -> u64 {
        game.world.get::<&Teleporter>(teleporter).unwrap().charge_j
    }

    fn set_charge(game: &mut Game, teleporter: Entity, joules: u64) {
        game.world
            .get::<&mut Teleporter>(teleporter)
            .unwrap()
            .charge_j = joules;
    }

    fn name(game: &Game, teleporter: Entity) -> String {
        game.world
            .get::<&Teleporter>(teleporter)
            .unwrap()
            .name
            .clone()
    }

    // --- 1.1 charging -------------------------------------------------------------------

    #[test]
    fn an_empty_bank_charges_at_its_rate_on_a_network_with_spare_power() {
        let mut game = game();
        let tp = put(&mut game, FactoryKind::Teleporter, at(5, 0, 0));
        // Three burners give 270 kW, more than the bank's 200 kW
        power(&mut game, at(5, 0, 0), 3);
        pass_factory_seconds(&mut game, 60);
        assert_eq!(charge(&game, tp), CHARGE_W * 60);
    }

    #[test]
    fn a_bank_stops_at_its_capacity_and_a_full_bank_draws_nothing() {
        let mut game = game();
        let tp = put(&mut game, FactoryKind::Teleporter, at(5, 0, 0));
        power(&mut game, at(5, 0, 0), 3);
        set_charge(&mut game, tp, CAPACITY_J - 30_000);
        pass_factory_seconds(&mut game, 1);
        assert_eq!(charge(&game, tp), CAPACITY_J);
        pass_factory_seconds(&mut game, 1);
        let consumer = *game.world.get::<&PowerConsumer>(tp).unwrap();
        assert!(!consumer.active);
        assert_eq!(consumer.demand_w, 0);
        assert_eq!(charge_rate_w(&consumer), 0);
        let network = game.res.power.network_of(tp).expect("on a network");
        assert_eq!(network.demand_w, 0);
    }

    #[test]
    fn a_short_network_charges_in_proportion() {
        let mut game = game();
        let tp = put(&mut game, FactoryKind::Teleporter, at(5, 0, 0));
        // One burner: 90 kW of the bank's 200 kW, so 450 per mille
        power(&mut game, at(5, 0, 0), 1);
        pass_factory_seconds(&mut game, 10);
        assert_eq!(charge(&game, tp), 90_000 * 10);
        let consumer = *game.world.get::<&PowerConsumer>(tp).unwrap();
        assert_eq!(consumer.satisfaction_permille, 450);
        assert_eq!(charge_rate_w(&consumer), 90_000);
    }

    #[test]
    fn a_bank_off_the_grid_stays_empty() {
        let mut game = game();
        let tp = put(&mut game, FactoryKind::Teleporter, at(5, 0, 0));
        pass_factory_seconds(&mut game, 10);
        assert_eq!(charge(&game, tp), 0);
    }

    // --- 1.2 jumps ----------------------------------------------------------------------

    #[test]
    fn jump_costs_grow_with_distance_and_layers_and_stop_at_capacity() {
        let o = at(0, 0, 0);
        assert_eq!(jump_cost(o, at(1, 0, 0)), JUMP_BASE_J + 5_000);
        assert_eq!(jump_cost(o, at(-300, 200, 0)), JUMP_BASE_J + 300 * 5_000);
        // Two layers count as 200 tiles
        assert_eq!(jump_cost(o, at(10, 0, -2)), JUMP_BASE_J + 210 * 5_000);
        assert_eq!(jump_cost(o, at(1_000_000, 0, 0)), CAPACITY_J);
        assert!(jump_cost(o, at(500, 0, 0)) > jump_cost(o, at(100, 0, 0)));
    }

    #[test]
    fn a_jump_moves_the_player_beside_the_destination_and_spends_the_cost() {
        let mut game = game();
        let from = put(&mut game, FactoryKind::Teleporter, at(1, 0, 0));
        let to = put(&mut game, FactoryKind::Teleporter, at(10, 5, 0));
        rename(&mut game.world, to, "North Base");
        set_charge(&mut game, from, CAPACITY_J);
        let tick = game.res.time.tick;
        let landed = game.teleport(from, to).expect("jumped");
        // North of the destination is free, so the player lands there
        assert_eq!(landed, at(10, 4, 0));
        assert_eq!(game.get_player_position(), Some(at(10, 4, 0)));
        assert_eq!(
            charge(&game, from),
            CAPACITY_J - jump_cost(at(1, 0, 0), at(10, 5, 0))
        );
        assert_eq!(charge(&game, to), 0, "the destination pays nothing");
        assert_eq!(game.res.time.tick, tick, "a jump passes no time");
    }

    #[test]
    fn landing_tries_north_east_south_west_then_the_diagonals() {
        let mut game = game();
        let from = put(&mut game, FactoryKind::Teleporter, at(-5, 0, 0));
        let to = put(&mut game, FactoryKind::Teleporter, at(5, 5, 0));
        let order = landing_order(at(5, 5, 0));
        assert_eq!(
            order[..4],
            [at(5, 4, 0), at(6, 5, 0), at(5, 6, 0), at(4, 5, 0)]
        );
        for (i, &expected) in order.iter().enumerate() {
            set_charge(&mut game, from, CAPACITY_J);
            assert_eq!(game.teleport(from, to), Ok(expected), "landing {i}");
            // Wall it off so the next jump lands on the next tile
            game.res
                .world_state
                .world
                .set_tile_cached(expected.x, expected.y, 0, TileKind::Rock);
            let back = at(-5, 1, 0);
            let player = game.get_player_entity().unwrap();
            *game.world.get::<&mut Position>(player).unwrap() = back;
        }
        set_charge(&mut game, from, CAPACITY_J);
        assert_eq!(game.teleport(from, to), Err(JumpError::NoRoom));
        assert_eq!(
            charge(&game, from),
            CAPACITY_J,
            "a refused jump costs nothing"
        );
    }

    #[test]
    fn landing_skips_tiles_with_a_creature_or_block() {
        let mut game = game();
        let from = put(&mut game, FactoryKind::Teleporter, at(-5, 0, 0));
        let to = put(&mut game, FactoryKind::Teleporter, at(5, 5, 0));
        put(&mut game, FactoryKind::Chest, at(5, 4, 0));
        game.world.spawn((at(6, 5, 0), BlocksMovement));
        set_charge(&mut game, from, CAPACITY_J);
        assert_eq!(game.teleport(from, to), Ok(at(5, 6, 0)));
    }

    #[test]
    fn a_short_bank_refuses_the_jump_and_says_how_much_more_it_needs() {
        let mut game = game();
        let from = put(&mut game, FactoryKind::Teleporter, at(1, 0, 0));
        let to = put(&mut game, FactoryKind::Teleporter, at(10, 0, 0));
        // A 20 MJ jump from a bank holding 12 MJ
        let cost = jump_cost(at(1, 0, 0), at(10, 0, 0));
        set_charge(&mut game, from, cost - 8_000_000);
        let err = game.teleport(from, to).unwrap_err();
        assert_eq!(
            err,
            JumpError::ShortOfCharge {
                needed_j: 8_000_000
            }
        );
        assert_eq!(
            err.message(),
            "The battery bank needs 8 MJ more charge for that jump."
        );
        assert_eq!(game.get_player_position(), Some(at(0, 0, 0)));
        assert_eq!(charge(&game, from), cost - 8_000_000);
    }

    #[test]
    fn megajoules_round_up() {
        assert_eq!(format_mj(8_000_000), "8 MJ");
        assert_eq!(format_mj(12_500_000), "12.5 MJ");
        assert_eq!(format_mj(1), "0.1 MJ");
        assert_eq!(format_mj(0), "0 MJ");
    }

    #[test]
    fn a_jump_across_chunks_and_layers_generates_the_destination_first() {
        let mut game = game();
        let from = put(&mut game, FactoryKind::Teleporter, at(1, 0, 0));
        set_charge(&mut game, from, CAPACITY_J);
        // Far off, two layers down, in a chunk nothing has generated yet
        let far = at(900, -700, -2);
        let (cx, cy, cz) = Game::world_to_chunk_coords(far.x, far.y, far.z);
        assert!(!game.is_chunk_generated(cx, cy, cz));
        game.ensure_chunk_with_pending_structures(cx, cy, cz);
        // Clear a pocket so the test doesn't depend on what worldgen put there
        for dx in -1..=1 {
            for dy in -1..=1 {
                game.res.world_state.world.set_tile_cached(
                    far.x + dx,
                    far.y + dy,
                    far.z,
                    TileKind::Grass,
                );
            }
        }
        let to = factory::place(
            &mut game.world,
            &mut game.res,
            FactoryKind::Teleporter,
            far,
            Facing::North,
        )
        .unwrap();
        let landed = game.teleport(from, to).expect("jumped");
        assert_eq!(landed, at(900, -701, -2));
        assert_eq!(game.get_player_position(), Some(landed));
        assert_eq!(
            CAPACITY_J - charge(&game, from),
            JUMP_BASE_J + (899 + 200) * JUMP_PER_TILE_J
        );
        assert!(game.res.is_chunk_explored(cx, cy, cz));
    }

    #[test]
    fn a_jump_into_an_ungenerated_chunk_generates_it() {
        let mut game = game();
        let from = put(&mut game, FactoryKind::Teleporter, at(1, 0, 0));
        set_charge(&mut game, from, CAPACITY_J);
        // A destination whose landing tiles are in a chunk next door that isn't generated:
        // its own tile was set by hand, the way a load or a far block could leave it
        let far = at(-641, 300, 0);
        let north = at(far.x, far.y - 1, far.z);
        let (cx, cy, cz) = Game::world_to_chunk_coords(north.x, north.y, north.z);
        let to = game.world.spawn((
            far,
            Teleporter {
                name: "Far".to_string(),
                charge_j: 0,
            },
        ));
        assert!(!game.is_chunk_generated(cx, cy, cz));
        let _ = game.teleport(from, to);
        assert!(game.is_chunk_generated(cx, cy, cz));
    }

    #[test]
    fn destinations_list_every_other_teleporter_nearest_first() {
        let mut game = game();
        let a = put(&mut game, FactoryKind::Teleporter, at(0, 2, 0));
        let b = put(&mut game, FactoryKind::Teleporter, at(10, 2, 0));
        let c = put(&mut game, FactoryKind::Teleporter, at(3, 2, 0));
        let list = destinations(&game.world, a);
        assert_eq!(
            list.iter().map(|d| d.entity).collect::<Vec<_>>(),
            vec![c, b]
        );
        assert_eq!(list[0].name, "Teleporter 3");
        assert_eq!(list[0].distance, 3);
        assert_eq!(list[1].cost_j, jump_cost(at(0, 2, 0), at(10, 2, 0)));
    }

    // --- 1.3 names ----------------------------------------------------------------------

    #[test]
    fn new_teleporters_get_unique_numbered_names() {
        let mut game = game();
        let a = put(&mut game, FactoryKind::Teleporter, at(0, 2, 0));
        let b = put(&mut game, FactoryKind::Teleporter, at(2, 2, 0));
        assert_eq!(name(&game, a), "Teleporter 1");
        assert_eq!(name(&game, b), "Teleporter 2");
        // Renaming one keeps the numbers going up, past the highest still in use
        rename(&mut game.world, a, "Furnace Row");
        let c = put(&mut game, FactoryKind::Teleporter, at(4, 2, 0));
        assert_eq!(name(&game, c), "Teleporter 3");
        rename(&mut game.world, b, "Home");
        rename(&mut game.world, c, "Home 2");
        let d = put(&mut game, FactoryKind::Teleporter, at(6, 2, 0));
        assert_eq!(name(&game, d), "Teleporter 1");
    }

    #[test]
    fn renaming_shows_in_other_teleporters_lists() {
        let mut game = game();
        let a = put(&mut game, FactoryKind::Teleporter, at(0, 2, 0));
        let b = put(&mut game, FactoryKind::Teleporter, at(2, 2, 0));
        assert!(rename(&mut game.world, b, "  Furnace Row  "));
        assert_eq!(destinations(&game.world, a)[0].name, "Furnace Row");
        assert!(
            !rename(&mut game.world, b, "   "),
            "empty names are refused"
        );
        assert_eq!(name(&game, b), "Furnace Row");
        rename(&mut game.world, b, &"x".repeat(40));
        assert_eq!(name(&game, b).len(), MAX_NAME_CHARS);
    }

    #[test]
    fn names_and_charge_survive_a_save_and_reload() {
        let mut game = game();
        let a = put(&mut game, FactoryKind::Teleporter, at(0, 2, 0));
        put(&mut game, FactoryKind::Teleporter, at(2, 2, 0));
        rename(&mut game.world, a, "North Base");
        set_charge(&mut game, a, 1_234_567);
        let saved = crate::save_load::save_to_string(&game, &serde_json::Value::Null).unwrap();
        // Changes after the save are undone by loading it back
        rename(&mut game.world, a, "Changed");
        crate::save_load::load_from_str(&mut game, &saved).expect("loaded");
        let mut found: Vec<(String, u64)> = game
            .world
            .query::<&Teleporter>()
            .iter()
            .map(|(_, t)| (t.name.clone(), t.charge_j))
            .collect();
        found.sort();
        assert_eq!(
            found,
            vec![
                ("North Base".to_string(), 1_234_567),
                ("Teleporter 2".to_string(), 0)
            ]
        );
    }

    // --- 3. transit stations ------------------------------------------------------------

    fn station(game: &mut Game, pos: Position, name: &str) -> Entity {
        let e = put(game, FactoryKind::TransitStation, pos);
        game.world.get::<&mut TransitStation>(e).unwrap().name = name.to_string();
        e
    }

    fn listed(game: &Game, from: Entity) -> Vec<(String, u64)> {
        destinations(&game.world, from)
            .into_iter()
            .map(|d| (d.name, d.cost_j))
            .collect()
    }

    #[test]
    fn a_dormant_station_activates_on_first_use_and_says_so() {
        let mut game = game();
        let bunker = station(&mut game, at(2, 0, 0), "Bunker Station");
        let factory = station(&mut game, at(8, 0, 0), "Factory Station");
        game.activate_station(bunker);
        assert!(
            listed(&game, bunker).is_empty(),
            "dormant stations aren't listed"
        );
        assert!(
            listed(&game, factory).is_empty(),
            "a dormant station lists nothing"
        );

        assert!(game.activate_station(factory));
        let log = game
            .res
            .events
            .get_message_log()
            .messages
            .last()
            .unwrap()
            .text
            .clone();
        assert!(
            log.contains("Transit node registered: Factory Station"),
            "{log}"
        );
        assert_eq!(
            game.get_player_position(),
            Some(at(0, 0, 0)),
            "activating doesn't jump"
        );
        assert_eq!(
            listed(&game, bunker),
            vec![(
                "Factory Station".to_string(),
                jump_cost(at(2, 0, 0), at(8, 0, 0))
            )]
        );
        assert!(!game.activate_station(factory), "only once");
        let sprite = game
            .world
            .get::<&crate::components::SpriteRef>(factory)
            .unwrap()
            .name
            .clone();
        assert_eq!(sprite, "transit_station_active");
    }

    #[test]
    fn stations_jump_on_their_bank_and_refuse_when_short() {
        let mut game = game();
        let bunker = station(&mut game, at(2, 0, 0), "Bunker Station");
        let factory = station(&mut game, at(10, 8, 0), "Factory Station");
        activate(&mut game.world, bunker);
        activate(&mut game.world, factory);
        let charge_of = |g: &Game, e| bank_charge(&g.world, e).unwrap();
        assert_eq!(charge_of(&game, bunker), CAPACITY_J, "stations start full");
        let cost = jump_cost(at(2, 0, 0), at(10, 8, 0));
        assert_eq!(game.teleport(bunker, factory), Ok(at(10, 7, 0)));
        assert_eq!(game.get_player_position(), Some(at(10, 7, 0)));
        assert_eq!(charge_of(&game, bunker), CAPACITY_J - cost);
        game.world
            .get::<&mut TransitStation>(factory)
            .unwrap()
            .charge_j = cost - 1_000_000;
        assert_eq!(
            game.teleport(factory, bunker),
            Err(JumpError::ShortOfCharge {
                needed_j: 1_000_000
            })
        );
    }

    #[test]
    fn a_stations_rtg_refills_its_bank() {
        let mut game = game();
        let s = station(&mut game, at(2, 0, 0), "Bunker Station");
        put(&mut game, FactoryKind::Rtg, at(4, 0, 0));
        put(&mut game, FactoryKind::PowerPole, at(3, 1, 0));
        game.world.get::<&mut TransitStation>(s).unwrap().charge_j = 0;
        pass_factory_seconds(&mut game, 10);
        // The RTG gives 100 kW, under the bank's 200 kW rate
        assert_eq!(bank_charge(&game.world, s), Some(crate::power::RTG_W * 10));
    }

    #[test]
    fn a_built_in_rtg_or_charger_stays_but_a_crafted_one_breaks() {
        let mut game = game();
        let player = game.get_player_entity().unwrap();
        let built_in = put(&mut game, FactoryKind::Rtg, at(1, 0, 0));
        game.world.insert_one(built_in, factory::BuiltIn).unwrap();
        put(&mut game, FactoryKind::Rtg, at(2, 0, 0));
        let charger = put(&mut game, FactoryKind::ItemCharger, at(3, 0, 0));
        game.world.insert_one(charger, factory::BuiltIn).unwrap();
        let brk =
            |g: &mut Game, x| factory::break_block(&mut g.world, &mut g.res, at(x, 0, 0), player);
        assert_eq!(brk(&mut game, 1), None);
        assert_eq!(brk(&mut game, 2), Some(FactoryKind::Rtg));
        assert_eq!(brk(&mut game, 3), None);
    }

    #[test]
    fn new_world_stations_stand_beside_an_rtg_a_pole_and_a_charger() {
        let game = Game::new(12345);
        let stations: Vec<Position> = game
            .world
            .query::<&Position>()
            .with::<&TransitStation>()
            .iter()
            .map(|(_, p)| *p)
            .collect();
        assert_eq!(stations.len(), 3);
        for s in stations {
            let near = |kind: FactoryKind| {
                game.world
                    .query::<(&Position, &factory::FactoryBlock)>()
                    .iter()
                    .filter(|(_, (p, b))| b.kind == kind && p.z == s.z && distance(**p, s) <= 3)
                    .map(|(e, _)| e)
                    .collect::<Vec<_>>()
            };
            for kind in [FactoryKind::Rtg, FactoryKind::ItemCharger] {
                let found = near(kind);
                assert_eq!(found.len(), 1, "{kind:?} by {s:?}");
                assert!(
                    factory::is_fixed(&game.world, found[0]),
                    "{kind:?} built in"
                );
            }
            let pole = near(FactoryKind::PowerPole);
            assert_eq!(pole.len(), 1);
            assert!(!factory::is_fixed(&game.world, pole[0]), "an ordinary pole");
        }
    }

    #[test]
    fn each_source_lists_its_own_destinations_and_costs() {
        let mut game = game();
        let tp = put(&mut game, FactoryKind::Teleporter, at(0, 2, 0));
        let other = put(&mut game, FactoryKind::Teleporter, at(4, 2, 0));
        let active = station(&mut game, at(8, 2, 0), "Bunker Station");
        let dormant = station(&mut game, at(9, 2, 0), "Factory Station");
        activate(&mut game.world, active);
        // A teleporter: other teleporters and activated stations, at the jump cost
        assert_eq!(
            listed(&game, tp),
            vec![
                (
                    "Teleporter 2".to_string(),
                    jump_cost(at(0, 2, 0), at(4, 2, 0))
                ),
                (
                    "Bunker Station".to_string(),
                    jump_cost(at(0, 2, 0), at(8, 2, 0))
                ),
            ]
        );
        // A station: only other activated stations, free; the teleporters aren't listed
        activate(&mut game.world, dormant);
        assert_eq!(
            listed(&game, active),
            vec![(
                "Factory Station".to_string(),
                jump_cost(at(8, 2, 0), at(9, 2, 0))
            )]
        );
        // A jump from a teleporter to a station is paid from its bank
        set_charge(&mut game, tp, CAPACITY_J);
        game.teleport(tp, active).expect("jumped");
        assert_eq!(
            charge(&game, tp),
            CAPACITY_J - jump_cost(at(0, 2, 0), at(8, 2, 0))
        );
        // A station can't jump to a teleporter
        assert_eq!(game.teleport(active, other), Err(JumpError::NotTeleporters));
    }

    #[test]
    fn stations_cant_be_broken_picked_up_or_copied() {
        let mut game = game();
        let s = station(&mut game, at(1, 0, 0), "Bunker Station");
        let player = game.get_player_entity().unwrap();
        assert_eq!(
            factory::break_block(&mut game.world, &mut game.res, at(1, 0, 0), player),
            None
        );
        assert!(game.world.contains(s));
        assert_eq!(game.res.factory_index.get(&at(1, 0, 0)), Some(&s));
        assert_eq!(FactoryKind::TransitStation.item(), None);
        let blueprint = crate::blueprints::capture(
            &game.world,
            &mut game.res,
            at(0, 0, 0),
            at(2, 0, 0),
            "copy".into(),
        );
        assert!(blueprint.blocks.is_empty());
    }

    #[test]
    fn station_activation_survives_a_save_and_reload() {
        let mut game = game();
        let a = station(&mut game, at(2, 0, 0), "Bunker Station");
        station(&mut game, at(8, 0, 0), "Factory Station");
        activate(&mut game.world, a);
        let saved = crate::save_load::save_to_string(&game, &serde_json::Value::Null).unwrap();
        crate::save_load::load_from_str(&mut game, &saved).expect("loaded");
        let mut found: Vec<(String, bool)> = game
            .world
            .query::<&TransitStation>()
            .iter()
            .map(|(_, s)| (s.name.clone(), s.activated))
            .collect();
        found.sort();
        assert_eq!(
            found,
            vec![
                ("Bunker Station".to_string(), true),
                ("Factory Station".to_string(), false)
            ]
        );
    }

    #[test]
    fn a_new_world_has_a_registered_station_in_the_wake_up_room() {
        let game = Game::new(12345);
        let player = game.get_player_position().unwrap();
        let stations: Vec<(Position, TransitStation)> = game
            .world
            .query::<(&Position, &TransitStation)>()
            .iter()
            .map(|(_, (p, s))| (*p, s.clone()))
            .collect();
        let (pos, bunker) = stations
            .iter()
            .find(|(_, s)| s.name == "Bunker Station")
            .expect("a bunker station");
        assert!(bunker.activated, "registered from the start");
        assert_eq!(pos.z, player.z);
        assert!(
            distance(*pos, player) <= 3,
            "{pos:?} is in the room with {player:?}"
        );
        // The arm quest's factory has two, still dormant: one at its entrance and one by
        // the arm locker
        for name in ["Factory Station", "Locker Station"] {
            assert!(
                stations.iter().any(|(_, s)| s.name == name && !s.activated),
                "{name}"
            );
        }
        let locker = game
            .world
            .query::<&Position>()
            .with::<&crate::components::Inventory>()
            .iter()
            .find(|(e, _)| {
                game.world
                    .get::<&crate::components::Inventory>(*e)
                    .is_ok_and(|inv| {
                        inv.slots
                            .iter()
                            .any(|s| s.kind == crate::components::ItemKind::SalvagedArm)
                    })
            })
            .map(|(_, p)| *p)
            .expect("the arm locker");
        let (station, _) = stations
            .iter()
            .find(|(_, s)| s.name == "Locker Station")
            .unwrap();
        assert!(distance(*station, locker) <= 3, "{station:?} by {locker:?}");
    }

    // --- 4. the mobile tele-access device -----------------------------------------------

    fn carry(game: &mut Game, stack: crate::components::ItemStack) {
        let player = game.get_player_entity().unwrap();
        game.world
            .get::<&mut crate::components::Inventory>(player)
            .unwrap()
            .add_stack(stack);
    }

    fn standard_cell(charge_j: u64) -> crate::components::ItemStack {
        crate::components::ItemStack {
            charge_j: Some(charge_j),
            ..crate::components::ItemStack::new(crate::components::ItemKind::StandardPowerCell, 1)
        }
    }

    fn device_game() -> (Game, Entity, Entity, Entity) {
        let mut game = game();
        let tp = put(&mut game, FactoryKind::Teleporter, at(10, 0, 0));
        let bunker = station(&mut game, at(-10, 5, 0), "Bunker Station");
        station(&mut game, at(-10, -5, 0), "Factory Station");
        activate(&mut game.world, bunker);
        let player = game.get_player_entity().unwrap();
        carry(
            &mut game,
            crate::components::ItemStack::new(
                crate::components::ItemKind::MobileTeleAccessDevice,
                1,
            ),
        );
        (game, player, tp, bunker)
    }

    #[test]
    fn the_device_lists_teleporters_and_awake_stations_at_the_jump_cost() {
        let (game, player, _, _) = device_game();
        let names: Vec<(String, u64)> = listed(&game, player);
        assert_eq!(
            names,
            vec![
                (
                    "Teleporter 1".to_string(),
                    jump_cost(at(0, 0, 0), at(10, 0, 0))
                ),
                (
                    "Bunker Station".to_string(),
                    jump_cost(at(0, 0, 0), at(-10, 5, 0))
                ),
            ]
        );
        // Without the device the player is no source at all
        let p = game.get_player_entity().unwrap();
        game.world
            .get::<&mut crate::components::Inventory>(p)
            .unwrap()
            .slots
            .clear();
        assert!(listed(&game, player).is_empty());
    }

    #[test]
    fn a_device_jump_draws_from_the_fullest_cell() {
        let (mut game, player, _, bunker) = device_game();
        carry(&mut game, standard_cell(5_000_000));
        carry(&mut game, standard_cell(20_000_000));
        let cost = jump_cost(at(0, 0, 0), at(-10, 5, 0));
        assert_eq!(game.teleport(player, bunker), Ok(at(-10, 4, 0)));
        let charges: Vec<u64> = game
            .world
            .get::<&crate::components::Inventory>(player)
            .unwrap()
            .slots
            .iter()
            .filter_map(|s| s.charge_j)
            .collect();
        assert_eq!(charges, vec![5_000_000, 20_000_000 - cost]);
    }

    #[test]
    fn a_device_with_empty_or_short_cells_refuses_with_no_charge() {
        let (mut game, player, tp, _) = device_game();
        let cost = jump_cost(at(0, 0, 0), at(10, 0, 0));
        // No cell at all, then an empty one
        let err = game.teleport(player, tp).unwrap_err();
        assert_eq!(
            err,
            JumpError::NoCharge(crate::power_cells::NoCharge { short_j: cost })
        );
        carry(&mut game, standard_cell(0));
        assert!(game
            .teleport(player, tp)
            .unwrap_err()
            .message()
            .starts_with("No charge"));
        // A partly charged cell: still short, and nothing is drawn
        carry(&mut game, standard_cell(6_000_000));
        assert_eq!(
            game.teleport(player, tp),
            Err(JumpError::NoCharge(crate::power_cells::NoCharge {
                short_j: cost - 6_000_000
            }))
        );
        assert_eq!(game.get_player_position(), Some(at(0, 0, 0)));
        assert_eq!(
            crate::power_cells::fullest_charge(&game.world, player),
            6_000_000
        );
    }
}

//! The Factory showcase: example builds laid out left to right north of spawn, from simple
//! to complex. Every factory block kind must appear in at least one example (a test checks
//! this), so a change that adds a kind adds its example here.

use std::collections::BTreeSet;

use crate::components::{DroppedItem, Inventory, ItemKind, Position, SpriteRef};
use crate::factory::{self, Facing, FactoryBlock, FactoryKind};
use crate::game::Game;
use crate::tiles::TileKind;

/// One example build. `build` places it with its top-left corner at the given position.
pub struct Example {
    pub name: &'static str,
    /// Width and height in tiles
    pub size: (i64, i64),
    /// Whether it has something to work on, so it changes as time passes
    pub has_input: bool,
    pub build: fn(&mut Game, Position),
}

pub const EXAMPLES: &[Example] = &[
    Example {
        name: "Chest",
        size: (1, 1),
        has_input: false,
        build: chest,
    },
    Example {
        name: "Belt line into a chest",
        size: (5, 1),
        has_input: true,
        build: belt_line,
    },
    Example {
        name: "Extractor: chest to chest",
        size: (5, 1),
        has_input: true,
        build: extractor_line,
    },
    Example {
        name: "Belt router split three ways",
        size: (5, 5),
        has_input: true,
        build: router_split,
    },
    Example {
        name: "Belt-fed crude furnace",
        size: (5, 2),
        has_input: true,
        build: belt_fed_furnace,
    },
    Example {
        name: "Self-feeding tree farm",
        size: (7, 7),
        has_input: true,
        build: tree_farm,
    },
    Example {
        name: "Vertical conveyors: up a layer and back down",
        size: (4, 2),
        has_input: true,
        build: lift_up_and_down,
    },
    Example {
        name: "Powered electric furnace, chest to chest",
        size: (11, 2),
        has_input: true,
        build: electric_furnace_line,
    },
    Example {
        name: "Resurrection vats: crude, standard and advanced",
        size: (5, 2),
        has_input: false,
        build: resurrection_vats,
    },
    Example {
        name: "Spawner farm: a fed, powered spawner in a fenced pen with a gate",
        size: (9, 7),
        has_input: true,
        build: spawner_farm,
    },
];

/// The top-left corner of the first example; the rest follow eastward.
pub const SHOWCASE_ORIGIN: Position = Position {
    x: -12,
    y: -7,
    z: 0,
};
/// Empty tiles between examples.
pub const SHOWCASE_GAP: i64 = 3;

/// Each example with the top-left corner it is built at.
pub fn layout() -> Vec<(&'static Example, Position)> {
    let mut x = SHOWCASE_ORIGIN.x;
    EXAMPLES
        .iter()
        .map(|example| {
            let origin = Position {
                x,
                ..SHOWCASE_ORIGIN
            };
            x += example.size.0 + SHOWCASE_GAP;
            (example, origin)
        })
        .collect()
}

pub fn build_showcase(game: &mut Game) {
    for (example, origin) in layout() {
        (example.build)(game, origin);
    }
}

/// Kinds in `kinds` that no placed block in `game` has.
pub fn missing_kinds(
    game: &Game,
    kinds: impl IntoIterator<Item = FactoryKind>,
) -> Vec<FactoryKind> {
    let placed: Vec<FactoryKind> = game
        .world
        .query::<&FactoryBlock>()
        .iter()
        .map(|(_, b)| b.kind)
        .collect();
    kinds.into_iter().filter(|k| !placed.contains(k)).collect()
}

fn at(origin: Position, dx: i64, dy: i64) -> Position {
    Position {
        x: origin.x + dx,
        y: origin.y + dy,
        z: origin.z,
    }
}

fn place(game: &mut Game, kind: FactoryKind, pos: Position, facing: Facing) -> hecs::Entity {
    factory::place(&mut game.world, &mut game.res, kind, pos, facing)
        .unwrap_or_else(|e| panic!("showcase: can't place {} at {pos:?}: {e:?}", kind.name()))
}

fn fill(game: &mut Game, block: hecs::Entity, items: &[(ItemKind, u32)]) {
    if let Ok(mut inv) = game.world.get::<&mut Inventory>(block) {
        for &(item, qty) in items {
            inv.add(item, qty);
        }
    }
}

fn chest(game: &mut Game, o: Position) {
    let chest = place(game, FactoryKind::Chest, o, Facing::North);
    fill(game, chest, &[(ItemKind::Log, 10), (ItemKind::Stone, 5)]);
}

/// Four belts east into a chest, with logs lying on the first belt.
fn belt_line(game: &mut Game, o: Position) {
    for dx in 0..4 {
        place(game, FactoryKind::Belt, at(o, dx, 0), Facing::East);
    }
    place(game, FactoryKind::Chest, at(o, 4, 0), Facing::North);
    game.world.spawn((
        o,
        DroppedItem {
            kind: ItemKind::Log,
            qty: 6,
        },
        SpriteRef::new("items", "log"),
    ));
}

/// Logs ride a belt into an up conveyor, cross the layer above on a belt, come back down
/// through a down conveyor, and end in a chest. A stairway beside it lets the player climb
/// up to watch the upper belt.
///
/// ```text
///   z + 1:  . = v .     (= belt east, v down conveyor)
///           S . . .     (S stairs)
///   z:      = ^ = C     (^ up conveyor)
///           S . . .
/// ```
fn lift_up_and_down(game: &mut Game, o: Position) {
    let above = |dx| Position {
        z: o.z + 1,
        ..at(o, dx, 0)
    };
    place(game, FactoryKind::Belt, o, Facing::East);
    place(
        game,
        FactoryKind::VerticalConveyorUp,
        at(o, 1, 0),
        Facing::North,
    );
    place(game, FactoryKind::Belt, above(1), Facing::East);
    place(
        game,
        FactoryKind::VerticalConveyorDown,
        above(2),
        Facing::North,
    );
    place(game, FactoryKind::Belt, at(o, 2, 0), Facing::East);
    place(game, FactoryKind::Chest, at(o, 3, 0), Facing::North);
    for z in [o.z, o.z + 1] {
        game.res
            .world_state
            .world
            .set_tile_cached(o.x, o.y + 1, z, TileKind::Stairs);
    }
    game.world.spawn((
        o,
        DroppedItem {
            kind: ItemKind::Log,
            qty: 4,
        },
        SpriteRef::new("items", "log"),
    ));
}

/// Logs on a belt into a furnace's back, logs as fuel on a belt into its side, and
/// biochar out its front onto a belt into a chest.
///
/// ```text
///   . . v . .
///   = = F = C
/// ```
fn belt_fed_furnace(game: &mut Game, o: Position) {
    place(game, FactoryKind::Belt, at(o, 0, 1), Facing::East);
    place(game, FactoryKind::Belt, at(o, 1, 1), Facing::East);
    place(game, FactoryKind::CrudeFurnace, at(o, 2, 1), Facing::East);
    place(game, FactoryKind::Belt, at(o, 3, 1), Facing::East);
    place(game, FactoryKind::Chest, at(o, 4, 1), Facing::North);
    place(game, FactoryKind::Belt, at(o, 2, 0), Facing::South);
    for (pos, qty) in [(at(o, 0, 1), 4), (at(o, 2, 0), 2)] {
        game.world.spawn((
            pos,
            DroppedItem {
                kind: ItemKind::Log,
                qty,
            },
            SpriteRef::new("items", "log"),
        ));
    }
}

/// A chest of logs, an extractor pulling them onto a long belt into an electric furnace's
/// back, and its biochar riding another long belt into a chest. A generator burning biochar
/// and a pole power the furnace.
///
/// ```text
///   . . G . + . . . . . .      G generator, + pole
///   C > = = = E = = = = C      E electric furnace facing east
/// ```
fn electric_furnace_line(game: &mut Game, o: Position) {
    let generator = place(
        game,
        FactoryKind::BurnerGenerator,
        at(o, 2, 0),
        Facing::North,
    );
    fill(game, generator, &[(ItemKind::Biochar, 4)]);
    place(game, FactoryKind::PowerPole, at(o, 4, 0), Facing::North);
    let source = place(game, FactoryKind::Chest, at(o, 0, 1), Facing::North);
    fill(game, source, &[(ItemKind::Log, 40)]);
    place(game, FactoryKind::Extractor, at(o, 1, 1), Facing::East);
    for dx in [2, 3, 4, 6, 7, 8, 9] {
        place(game, FactoryKind::Belt, at(o, dx, 1), Facing::East);
    }
    place(
        game,
        FactoryKind::ElectricFurnace,
        at(o, 5, 1),
        Facing::East,
    );
    place(game, FactoryKind::Chest, at(o, 10, 1), Facing::North);
}

/// A spare body in each vat tier: a crude vat with no power, and standard and advanced vats
/// on three generators.
///
/// ```text
///   G G + G .      G generator, + pole
///   c . s . a      c crude, s standard, a advanced vat
/// ```
fn resurrection_vats(game: &mut Game, o: Position) {
    use crate::model::body::{graded_item, PartGrade, PART_ORDER};
    for dx in [0, 1, 3] {
        let generator = place(
            game,
            FactoryKind::BurnerGenerator,
            at(o, dx, 0),
            Facing::North,
        );
        fill(game, generator, &[(ItemKind::Biochar, 20)]);
    }
    place(game, FactoryKind::PowerPole, at(o, 2, 0), Facing::North);
    for (dx, kind, grade) in [
        (0, FactoryKind::CrudeVat, PartGrade::Crude),
        (2, FactoryKind::StandardVat, PartGrade::Standard),
        (4, FactoryKind::AdvancedVat, PartGrade::Advanced),
    ] {
        let vat = place(game, kind, at(o, dx, 1), Facing::North);
        for part in PART_ORDER {
            factory::insert(&mut game.world, vat, graded_item(part, grade));
        }
    }
}

/// A chest of logs, an extractor pulling them onto two belts, and a chest at the end.
///
/// ```text
///   C > = = C
/// ```
/// A fenced pen with a gate on its west side. Inside, a generator and pole power a spawner,
/// and an extractor and a belt feed it meat slurry and dog embryos from a chest. Dogs grow
/// while the player is near and stay in the pen; the player walks in through the gate.
fn spawner_farm(game: &mut Game, o: Position) {
    for dx in 0..9 {
        for dy in 0..7 {
            if dx == 0 || dx == 8 || dy == 0 || dy == 6 {
                let kind = if (dx, dy) == (0, 5) {
                    FactoryKind::Gate
                } else {
                    FactoryKind::Fence
                };
                place(game, kind, at(o, dx, dy), Facing::North);
            }
        }
    }
    let generator = place(
        game,
        FactoryKind::BurnerGenerator,
        at(o, 7, 1),
        Facing::North,
    );
    fill(game, generator, &[(ItemKind::Biochar, 8)]);
    // East of the spawner, out of reach of the vats' pole next door
    place(game, FactoryKind::PowerPole, at(o, 6, 1), Facing::North);
    let feed = place(game, FactoryKind::Chest, at(o, 1, 3), Facing::North);
    fill(
        game,
        feed,
        &[
            (ItemKind::MeatSlurry, 16),
            (ItemKind::FeralDogEmbryoPoor, 4),
        ],
    );
    place(game, FactoryKind::Extractor, at(o, 2, 3), Facing::East);
    place(game, FactoryKind::Belt, at(o, 3, 3), Facing::East);
    place(game, FactoryKind::Spawner, at(o, 4, 3), Facing::North);
}

fn extractor_line(game: &mut Game, o: Position) {
    let source = place(game, FactoryKind::Chest, o, Facing::North);
    fill(game, source, &[(ItemKind::Log, 6)]);
    place(game, FactoryKind::Extractor, at(o, 1, 0), Facing::East);
    place(game, FactoryKind::Belt, at(o, 2, 0), Facing::East);
    place(game, FactoryKind::Belt, at(o, 3, 0), Facing::East);
    place(game, FactoryKind::Chest, at(o, 4, 0), Facing::North);
}

/// Logs on a belt into a router that sends them left, straight and right in turn, each
/// output belt ending in a chest.
///
/// ```text
///   . . C . .
///   . . ^ . .
///   = = R = C
///   . . v . .
///   . . C . .
/// ```
fn router_split(game: &mut Game, o: Position) {
    place(game, FactoryKind::Belt, at(o, 0, 2), Facing::East);
    place(game, FactoryKind::Belt, at(o, 1, 2), Facing::East);
    place(game, FactoryKind::BeltRouter, at(o, 2, 2), Facing::East);
    place(game, FactoryKind::Belt, at(o, 2, 1), Facing::North);
    place(game, FactoryKind::Belt, at(o, 3, 2), Facing::East);
    place(game, FactoryKind::Belt, at(o, 2, 3), Facing::South);
    for (dx, dy) in [(2, 0), (4, 2), (2, 4)] {
        place(game, FactoryKind::Chest, at(o, dx, dy), Facing::North);
    }
    game.world.spawn((
        at(o, 0, 2),
        DroppedItem {
            kind: ItemKind::Log,
            qty: 6,
        },
        SpriteRef::new("items", "log"),
    ));
}

/// Two generators and a pole powering an auto-planter and an auto-harvester. The harvester fills
/// a chest; a filter extractor set to acorns feeds them back to the planter, so the farm
/// replants itself.
///
/// ```text
///   G . . . . . .
///   G + . . . . .
///   . . . p . . t
///   . . . ^ . . .      ^ filter extractor (acorns), facing the planter
///   . . . C H . .      H harvester facing west into the chest
///   . . . . . . .
///   . . t . . t t
/// ```
fn tree_farm(game: &mut Game, o: Position) {
    // Two generators (180 kW) of the 240 kW the planter and harvester draw together
    for dy in [0, 1] {
        let generator = place(
            game,
            FactoryKind::BurnerGenerator,
            at(o, 0, dy),
            Facing::North,
        );
        fill(game, generator, &[(ItemKind::Log, 5)]);
    }
    place(game, FactoryKind::PowerPole, at(o, 1, 1), Facing::North);
    let planter = place(game, FactoryKind::AutoPlanter, at(o, 3, 2), Facing::North);
    fill(game, planter, &[(ItemKind::Acorn, 4)]);
    let extractor = place(
        game,
        FactoryKind::FilterExtractor,
        at(o, 3, 3),
        Facing::North,
    );
    if let Ok(mut ex) = game
        .world
        .get::<&mut crate::logistics::Extractor>(extractor)
    {
        ex.filter = Some(ItemKind::Acorn);
    }
    place(game, FactoryKind::Chest, at(o, 3, 4), Facing::North);
    place(game, FactoryKind::AutoHarvester, at(o, 4, 4), Facing::West);
    for (dx, dy) in [(6, 2), (2, 6), (5, 6), (6, 6)] {
        let p = at(o, dx, dy);
        game.res
            .world_state
            .world
            .set_tile_cached(p.x, p.y, p.z, TileKind::Tree);
    }
}

/// Everything inside an example's area that time can change: tiles, block contents and
/// items lying there.
pub fn fingerprint(game: &mut Game, example: &Example, origin: Position) -> String {
    let mut parts = BTreeSet::new();
    for dy in 0..example.size.1 {
        for dx in 0..example.size.0 {
            let p = at(origin, dx, dy);
            let tile = game.res.world_state.world.get_tile_cached(p.x, p.y, p.z);
            parts.insert(format!("{dx},{dy} {tile:?}"));
            if let Some(&block) = game.res.factory_index.get(&p) {
                if let Ok(inv) = game.world.get::<&Inventory>(block) {
                    parts.insert(format!("{dx},{dy} holds {:?}", inv.slots));
                }
                if let Ok(belt) = game.world.get::<&factory::BeltSlot>(block) {
                    parts.insert(format!("{dx},{dy} belt {:?}", belt.item));
                }
            }
        }
    }
    let inside = |p: &Position| {
        (origin.x..origin.x + example.size.0).contains(&p.x)
            && (origin.y..origin.y + example.size.1).contains(&p.y)
            && p.z == origin.z
    };
    for (_, (p, d)) in game.world.query::<(&Position, &DroppedItem)>().iter() {
        if inside(p) {
            parts.insert(format!("{p:?} lying {:?} {}", d.kind, d.qty));
        }
    }
    parts.into_iter().collect::<Vec<_>>().join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::{WorldSettings, WorldType};
    use strum::IntoEnumIterator;

    fn showcase() -> Game {
        let mut settings = WorldSettings::defaults(&crate::game_config::GameConfig::new());
        settings.world_type = WorldType::FactoryShowcase;
        Game::new_with_settings(5, settings)
    }

    fn names(kinds: &[FactoryKind]) -> Vec<&'static str> {
        kinds.iter().map(|k| k.name()).collect()
    }

    #[test]
    fn every_factory_block_kind_has_an_example() {
        let game = showcase();
        let real = FactoryKind::iter().filter(|k| !k.is_test_only());
        let missing = missing_kinds(&game, real);
        assert!(
            missing.is_empty(),
            "add a showcase example (showcase::EXAMPLES) using: {:?}",
            names(&missing)
        );
        // The check does name a kind with no example
        let all = missing_kinds(&game, FactoryKind::iter());
        assert!(names(&all).contains(&"test arrow"), "{:?}", names(&all));
    }

    #[test]
    fn the_spawner_farm_grows_dogs_that_stay_in_the_pen() {
        let mut game = showcase();
        let (example, o) = layout()
            .into_iter()
            .find(|(e, _)| e.name.starts_with("Spawner farm"))
            .unwrap();
        // Stand just outside the pen, south of it
        let player = game.get_player_entity().unwrap();
        *game.world.get::<&mut Position>(player).unwrap() = at(o, 4, 8);
        for _ in 0..12 {
            game.pass_time();
        }
        let in_pen = |p: &Position| {
            (o.x + 1..o.x + example.size.0 - 1).contains(&p.x)
                && (o.y + 1..o.y + example.size.1 - 1).contains(&p.y)
        };
        let dogs: Vec<Position> = game
            .world
            .query::<&Position>()
            .with::<&crate::spawner::SpawnedBy>()
            .iter()
            .map(|(_, p)| *p)
            .collect();
        assert!(!dogs.is_empty(), "the farm grew dogs");
        assert!(dogs.iter().all(in_pen), "{dogs:?}");
    }

    #[test]
    fn examples_with_inputs_make_progress_in_60_seconds() {
        let mut game = showcase();
        let before: Vec<String> = layout()
            .into_iter()
            .map(|(e, o)| fingerprint(&mut game, e, o))
            .collect();
        for _ in 0..6 {
            game.pass_time();
        }
        for ((example, origin), before) in layout().into_iter().zip(before) {
            if example.has_input {
                assert_ne!(
                    fingerprint(&mut game, example, origin),
                    before,
                    "{} made no progress",
                    example.name
                );
            }
        }
    }

    #[test]
    fn the_lift_example_delivers_logs_over_the_layer_above() {
        let mut game = showcase();
        let (_, origin) = layout()
            .into_iter()
            .find(|(e, _)| e.name.starts_with("Vertical conveyors"))
            .unwrap();
        for _ in 0..3 {
            game.pass_time();
        }
        let chest = game.res.factory_index[&at(origin, 3, 0)];
        let logs: u32 = game
            .world
            .get::<&Inventory>(chest)
            .unwrap()
            .slots
            .iter()
            .filter(|s| s.kind == ItemKind::Log)
            .map(|s| s.qty)
            .sum();
        assert_eq!(logs, 4);
    }

    #[test]
    fn examples_do_not_overlap_spawn() {
        for (example, origin) in layout() {
            let spawn = crate::debug_worlds::DEBUG_SPAWN;
            let covers_spawn = (origin.x..origin.x + example.size.0).contains(&spawn.x)
                && (origin.y..origin.y + example.size.1).contains(&spawn.y);
            assert!(!covers_spawn, "{}", example.name);
        }
    }
}

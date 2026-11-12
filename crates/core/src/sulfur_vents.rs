//! Sulfur vents: fuming fissures in the deep that worldgen places near the lava rivers
//! (`GameWorld::place_sulfur_vent`). They can't be broken, and while the player is near
//! they grow sulfur crystal tiles on the open ground beside them, up to a cap. The tiles
//! are the whole state: a vent is a tile, and the cap counts the crystals around it.

use crate::components::{BlocksMovement, Position};
use crate::resources::Resources;
use crate::systems::ecs_utils::get_player_position;
use crate::tiles::TileKind;
use hecs::World;
use rand::Rng;

/// About one deep chunk in this many gets a vent (where lava is near).
pub const VENT_CHUNK_ODDS: u32 = 6;
/// Ticks between growths: 30 game seconds.
pub const VENT_GROWTH_TICKS: u64 = 30 * crate::factory::TICKS_PER_SECOND;
/// Vents this close to the player (on each axis, on the player's layer) grow.
pub const VENT_RANGE: i64 = 24;
/// The most crystals a vent keeps beside it.
pub const VENT_CRYSTAL_CAP: usize = 4;

/// The 8 tiles around a position on its layer.
fn neighbors(p: Position) -> impl Iterator<Item = Position> {
    (-1..=1)
        .flat_map(move |dy| (-1..=1).map(move |dx| (dx, dy)))
        .filter(|&d| d != (0, 0))
        .map(move |(dx, dy)| Position {
            x: p.x + dx,
            y: p.y + dy,
            z: p.z,
        })
}

/// Grow one crystal beside each vent near the player for every growth period crossed
/// (time jumps by whole actions), up to the cap.
pub fn sulfur_vent_system(world: &mut World, res: &mut Resources) {
    let period = res.time.tick / VENT_GROWTH_TICKS;
    if period <= res.vent_growth_period {
        return;
    }
    let crossed = (period - res.vent_growth_period).min(VENT_CRYSTAL_CAP as u64);
    res.vent_growth_period = period;
    let Some(player) = get_player_position(world) else {
        return;
    };
    let mut vents = Vec::new();
    for y in player.y - VENT_RANGE..=player.y + VENT_RANGE {
        for x in player.x - VENT_RANGE..=player.x + VENT_RANGE {
            let tile = res
                .world_state
                .world
                .get_tile_at_z_no_worldgen(x, y, player.z);
            if tile == TileKind::SulfurVent {
                vents.push(Position { x, y, z: player.z });
            }
        }
    }
    for vent in vents {
        for step in 0..crossed {
            grow(world, res, vent, player, period - step);
        }
    }
}

/// Grow one crystal beside `vent` if it has fewer than the cap, on an open, walkable tile
/// with nobody on it, picked from the world seed, the vent's position and the period.
fn grow(world: &World, res: &mut Resources, vent: Position, player: Position, period: u64) {
    let game_world = &mut res.world_state.world;
    let crystals = neighbors(vent)
        .filter(|p| game_world.get_tile_cached(p.x, p.y, p.z) == TileKind::SulfurCrystal)
        .count();
    if crystals >= VENT_CRYSTAL_CAP {
        return;
    }
    let occupied = |p: &Position| {
        *p == player
            || world
                .query::<&Position>()
                .with::<&BlocksMovement>()
                .iter()
                .any(|(_, q)| q == p)
    };
    let open: Vec<Position> = neighbors(vent)
        .filter(|p| game_world.get_tile_cached(p.x, p.y, p.z) == TileKind::Air && !occupied(p))
        .collect();
    if open.is_empty() {
        return;
    }
    let mut rng = game_world.new_rng(vent.x, vent.y, vent.z ^ period as i64);
    let at = open[rng.gen_range(0..open.len())];
    game_world.set_tile_cached(at.x, at.y, at.z, TileKind::SulfurCrystal);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intent::PlayerIntent;
    use crate::test_support::{TestGame, TestWorld};

    /// Open ground in the deep with the player at the origin and a vent 3 tiles east.
    fn vent_room() -> TestGame {
        TestWorld::new(7)
            .player_at(0, 0, -20)
            .fill((-6, -6, -20), (10, 6, -20), TileKind::Air)
            .tile(3, 0, -20, TileKind::SulfurVent)
            .build()
    }

    fn crystals(game: &mut TestGame) -> usize {
        neighbors(Position { x: 3, y: 0, z: -20 })
            .filter(|p| game.tile(p.x, p.y, p.z) == TileKind::SulfurCrystal)
            .count()
    }

    fn pass(game: &mut TestGame, periods: u64) {
        for _ in 0..periods {
            game.game.res.time.tick += VENT_GROWTH_TICKS;
            game.game.res.player_state.intent = PlayerIntent::movement(0, 0, 0, 1);
            game.tick();
        }
    }

    #[test]
    fn a_vent_grows_crystals_beside_it_up_to_the_cap() {
        let mut game = vent_room();
        pass(&mut game, 2);
        assert_eq!(crystals(&mut game), 2);
        pass(&mut game, 10);
        assert_eq!(
            crystals(&mut game),
            VENT_CRYSTAL_CAP,
            "growth stops at the cap"
        );
    }

    #[test]
    fn mining_a_crystal_gives_sulfur_and_frees_the_tile() {
        let mut game = vent_room();
        pass(&mut game, 6);
        let crystal = neighbors(Position { x: 3, y: 0, z: -20 })
            .find(|p| game.tile(p.x, p.y, p.z) == TileKind::SulfurCrystal)
            .unwrap();
        let gained = crate::crafting::mine_tile(&mut game.game.res, crystal).unwrap();
        assert!(gained
            .iter()
            .any(|(k, n)| *k == crate::components::ItemKind::Sulfur && *n >= 1));
        assert_eq!(game.tile(crystal.x, crystal.y, crystal.z), TileKind::Air);
        pass(&mut game, 1);
        assert_eq!(crystals(&mut game), VENT_CRYSTAL_CAP, "it grows back");
    }

    #[test]
    fn a_vent_cant_be_mined() {
        let mut game = vent_room();
        let at = Position { x: 3, y: 0, z: -20 };
        assert!(crate::crafting::mine_tile(&mut game.game.res, at).is_none());
        assert_eq!(game.tile(3, 0, -20), TileKind::SulfurVent);
    }

    #[test]
    fn far_vents_dont_grow() {
        let mut game = TestWorld::new(7)
            .player_at(0, 0, -20)
            .fill((-6, -6, -20), (40, 6, -20), TileKind::Air)
            .tile(35, 0, -20, TileKind::SulfurVent)
            .build();
        pass(&mut game, 3);
        let grown = neighbors(Position {
            x: 35,
            y: 0,
            z: -20,
        })
        .filter(|p| game.tile(p.x, p.y, p.z) == TileKind::SulfurCrystal)
        .count();
        assert_eq!(grown, 0);
    }

    #[test]
    fn growth_replays_the_same() {
        let run = || {
            let mut game = vent_room();
            pass(&mut game, 3);
            neighbors(Position { x: 3, y: 0, z: -20 })
                .map(|p| game.tile(p.x, p.y, p.z))
                .collect::<Vec<_>>()
        };
        assert_eq!(run(), run());
    }
}

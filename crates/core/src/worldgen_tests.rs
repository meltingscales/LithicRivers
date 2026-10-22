//! Worldgen by depth: the survey pins each band's tile mix for a fixed seed, so tuning
//! shows up in review.

use crate::tiles::TileKind;
use crate::world::{depth_band, DepthBand, GameWorld, UndergroundNoises, CHUNK_SIZE};

fn world() -> GameWorld {
    GameWorld::new(0, 0, 12345)
}

fn share(survey: &std::collections::BTreeMap<String, f64>, tile: &str) -> f64 {
    survey.get(tile).copied().unwrap_or(0.0)
}

fn open_share(survey: &std::collections::BTreeMap<String, f64>) -> f64 {
    TileKind::all_variants()
        .iter()
        .filter(|t| t.is_passable())
        .map(|t| share(survey, &format!("{t:?}")))
        .sum()
}

#[test]
fn print_the_survey() {
    let w = world();
    for z in [2, 1, 0, -1, -3, -5, -6, -10, -20] {
        let s = w.survey(z, 256);
        let parts: Vec<String> = s.iter().map(|(k, v)| format!("{k} {v:.2}%")).collect();
        println!(
            "z={z:>4} {:?} open {:.0}% | {}",
            depth_band(z),
            open_share(&s),
            parts.join(", ")
        );
    }
}

#[test]
fn the_sky_is_open_air() {
    let s = world().survey(1, 128);
    assert_eq!(share(&s, "Air"), 100.0);
}

#[test]
fn the_shallows_are_rock_with_tunnels_and_deposits() {
    let w = world();
    for z in -5..=-1 {
        let s = w.survey(z, 256);
        let open = open_share(&s);
        assert!((12.0..=40.0).contains(&open), "z={z}: open {open:.1}%");
        assert!(share(&s, "Grass") == 0.0, "no grass underground at z={z}");
        let iron = share(&s, "IronScrap");
        let electronics = share(&s, "ScrapElectronics");
        assert!((0.3..=3.0).contains(&iron), "z={z}: iron {iron:.2}%");
        assert!(
            (0.1..=2.0).contains(&electronics),
            "z={z}: electronics {electronics:.2}%"
        );
    }
}

#[test]
fn the_deep_has_every_deposit() {
    let s = world().survey(-20, 256);
    for tile in ["IronScrap", "ScrapElectronics", "PlasteelScrap"] {
        let v = share(&s, tile);
        assert!((0.1..=4.0).contains(&v), "{tile} {v:.2}%");
    }
}

#[test]
fn shallow_caves_mostly_line_up_between_layers() {
    let w = world();
    let noises = UndergroundNoises::new(12345);
    let (mut open_here, mut open_both) = (0, 0);
    for x in -200..200 {
        for y in -200..200 {
            if w.underground_tile(&noises, x, y, -3).is_passable() {
                open_here += 1;
                if w.underground_tile(&noises, x, y, -2).is_passable() {
                    open_both += 1;
                }
            }
        }
    }
    let overlap = open_both as f64 * 100.0 / open_here as f64;
    assert!((50.0..=90.0).contains(&overlap), "overlap {overlap:.0}%");
}

#[test]
fn natural_stairways_line_up_on_both_layers() {
    let mut w = world();
    let mut found = 0;
    for cx in -3..3 {
        for cy in -3..3 {
            for x in 0..CHUNK_SIZE {
                for y in 0..CHUNK_SIZE {
                    let (wx, wy) = (cx * CHUNK_SIZE + x, cy * CHUNK_SIZE + y);
                    if w.get_tile_cached(wx, wy, -3) == TileKind::Stairs {
                        found += 1;
                        let up = w.get_tile_cached(wx, wy, -2) == TileKind::Stairs;
                        let down = w.get_tile_cached(wx, wy, -4) == TileKind::Stairs;
                        assert!(up || down, "a lone stair at ({wx}, {wy}, -3)");
                    }
                }
            }
        }
    }
    assert!(found > 0, "some natural stairs at z = -3");
}

#[test]
fn every_generated_solid_tile_can_be_mined() {
    let w = world();
    for z in [0, -2, -5, -10, -20] {
        for name in w.survey(z, 128).keys() {
            let tile = *TileKind::all_variants()
                .iter()
                .find(|t| format!("{t:?}") == *name)
                .unwrap();
            assert!(
                tile.is_passable()
                    || tile == TileKind::Bedrock
                    || crate::crafting::mining_drop(tile).is_some(),
                "{name} at z={z} is solid and can't be mined"
            );
        }
    }
}

#[test]
fn bands_by_depth() {
    assert_eq!(depth_band(3), DepthBand::Sky);
    assert_eq!(depth_band(0), DepthBand::Surface);
    assert_eq!(depth_band(-1), DepthBand::Shallows);
    assert_eq!(depth_band(-5), DepthBand::Shallows);
    assert_eq!(depth_band(-6), DepthBand::Deep);
}

/// A hash of the surface tiles of 25 chunks around the origin.
fn surface_hash(w: &mut GameWorld) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for cy in -2..=2 {
        for cx in -2..=2 {
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let t = w.get_tile_cached(cx * CHUNK_SIZE + x, cy * CHUNK_SIZE + y, 0);
                    h = (h ^ t as u64).wrapping_mul(0x100000001b3);
                }
            }
        }
    }
    h
}

#[test]
fn the_surface_is_generated_as_before_depth_bands() {
    // The same hash the generator from before depth bands gives (checked on release)
    let hash = surface_hash(&mut world());
    assert_eq!(hash, SURFACE_HASH, "the surface changed: {hash:#x}");
}

const SURFACE_HASH: u64 = 0x998f_620a_e8af_b832;

#[test]
fn the_bunker_staircase_reaches_the_surface() {
    let mut game = crate::Game::new(12345);
    let world = &mut game.res.world_state.world;
    // The bunker's staircase: one column of stairs on every layer from its bottom up to
    // the surface (natural stairways only join two layers)
    let column = (-20..60)
        .flat_map(|x| (-20..60).map(move |y| (x, y)))
        .find(|&(x, y)| (-19..=0).all(|z| world.get_tile_cached(x, y, z) == TileKind::Stairs));
    assert!(column.is_some(), "no staircase from z = -19 to the surface");
}

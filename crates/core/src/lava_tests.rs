//! Lava: impassable, unmineable, see-through, and bridged by plain blocks.

use crate::components::{ItemKind, Position};
use crate::factory::{self, Facing, FactoryKind, PlaceError};
use crate::intent::PlayerAction;
use crate::systems::fog_of_war::has_line_of_sight;
use crate::test_support::{TestGame, TestWorld};
use crate::tiles::TileKind;

/// Grass around the player with a north-south river of lava at x = 1.
fn riverbank() -> TestGame {
    TestWorld::new(5)
        .fill((-4, -4, 0), (4, 4, 0), TileKind::Grass)
        .fill((1, -4, 0), (1, 4, 0), TileKind::Lava)
        .inventory(&[])
        .build()
}

#[test]
fn the_player_cant_walk_into_lava() {
    let mut test = riverbank();
    test.act(PlayerAction::Move {
        dx: 1,
        dy: 0,
        dz: 0,
    });
    test.tick();
    assert_eq!(test.player_pos(), Position { x: 0, y: 0, z: 0 });
}

#[test]
fn lava_cant_be_mined() {
    let mut test = riverbank();
    test.act(PlayerAction::MineAt { x: 1, y: 0, z: 0 });
    test.tick();
    assert_eq!(test.tile(1, 0, 0), TileKind::Lava);
    assert!(test.player_inventory().is_empty());
}

#[test]
fn sight_passes_over_lava() {
    let test = riverbank();
    assert!(has_line_of_sight(
        &test.game.world,
        &test.game.res,
        0,
        0,
        3,
        0,
        0
    ));
}

#[test]
fn a_block_on_lava_leaves_walkable_crust() {
    let mut test = riverbank();
    let pos = Position { x: 1, y: 0, z: 0 };
    let placed =
        factory::place_plain_block(&test.game.world, &mut test.game.res, TileKind::Rock, pos);
    assert_eq!(placed, Ok(TileKind::LavaCrust));
    assert_eq!(test.tile(1, 0, 0), TileKind::LavaCrust);
    test.act(PlayerAction::Move {
        dx: 1,
        dy: 0,
        dz: 0,
    });
    test.tick();
    assert_eq!(test.player_pos(), pos);
}

#[test]
fn machines_cant_go_on_lava() {
    let mut test = riverbank();
    let pos = Position { x: 1, y: 0, z: 0 };
    let placed = factory::place(
        &mut test.game.world,
        &mut test.game.res,
        FactoryKind::Chest,
        pos,
        Facing::North,
    );
    assert_eq!(placed.err(), Some(PlaceError::Occupied));
    assert_eq!(test.tile(1, 0, 0), TileKind::Lava);
}

#[test]
fn riverglass_mines_into_riverglass() {
    let mut test = TestWorld::new(5)
        .fill((-2, -2, 0), (2, 2, 0), TileKind::Grass)
        .tile(1, 0, 0, TileKind::Riverglass)
        .inventory(&[])
        .auto_pickup(true)
        .build();
    test.act(PlayerAction::MineAt { x: 1, y: 0, z: 0 });
    test.tick();
    assert_eq!(test.tile(1, 0, 0), TileKind::Air);
    let have = test.player_has(ItemKind::Riverglass)
        + test
            .items_at(1, 0, 0)
            .iter()
            .filter(|(k, _)| *k == ItemKind::Riverglass)
            .map(|(_, n)| n)
            .sum::<u32>();
    assert_eq!(have, 1);
}

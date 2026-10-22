//! Stairs: what the player can climb between layers, and building a stairway. A stairway
//! is two stair tiles, one above the other, so it always leads somewhere.

use crate::components::{
    itemkind_sprite_name, DroppedItem, Inventory, ItemKind, Position, SpriteRef,
};
use crate::resources::Resources;
use crate::tiles::TileKind;
use hecs::{Entity, World};

/// Stairs items a full stairway uses: one per stair tile.
pub const STAIRWAY_COST: u32 = 2;

/// Whether the player can climb up or down from `pos`: stairs, or a vertical conveyor
/// (climbed like a ladder, either way).
pub fn climbable(world: &World, res: &mut Resources, pos: Position) -> bool {
    if res.world_state.world.get_tile_cached(pos.x, pos.y, pos.z) == TileKind::Stairs {
        return true;
    }
    res.factory_index.get(&pos).is_some_and(|&e| {
        world
            .get::<&crate::factory::FactoryBlock>(e)
            .is_ok_and(|b| b.kind.lift().is_some())
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StairsError {
    /// Fewer stairs in the inventory than the stairway needs (this many)
    NotEnough(u32),
    /// Something is on one of the two tiles, or the near tile is solid
    Occupied,
    /// The tile on the other layer is solid and can't be dug out
    Undiggable(TileKind),
}

impl StairsError {
    pub fn message(self) -> String {
        match self {
            StairsError::NotEnough(need) => {
                format!("This stairway takes {need} stairs, one for each new stair")
            }
            StairsError::Occupied => "Can't build stairs there: something is in the way".into(),
            StairsError::Undiggable(tile) => {
                format!("Can't dig the stairway through the {}", tile.display_name())
            }
        }
    }
}

fn stairs_held(world: &World, player: Entity) -> u32 {
    world
        .get::<&Inventory>(player)
        .map(|inv| {
            inv.slots
                .iter()
                .filter(|s| s.kind == ItemKind::Stairs)
                .map(|s| s.qty)
                .sum()
        })
        .unwrap_or(0)
}

/// Whether a stairway from `pos` to the tile `dz` layers away could be built, ignoring the
/// inventory: the stairs it would cost (one per tile that isn't stairs yet), or why not.
/// `player` may stand on either tile.
pub fn check_stairway(
    world: &World,
    res: &mut Resources,
    pos: Position,
    dz: i64,
    player: Entity,
) -> Result<u32, StairsError> {
    let other = Position {
        z: pos.z + dz,
        ..pos
    };
    let mut cost = 0;
    for (p, near) in [(pos, true), (other, false)] {
        let tile = res.world_state.world.get_tile_cached(p.x, p.y, p.z);
        if tile == TileKind::Stairs {
            continue;
        }
        let blocked = world
            .query::<&Position>()
            .iter()
            .any(|(e, q)| *q == p && e != player);
        if blocked {
            return Err(StairsError::Occupied);
        }
        if !tile.is_passable() {
            if near {
                return Err(StairsError::Occupied);
            }
            if crate::crafting::mining_drop(tile).is_none() {
                return Err(StairsError::Undiggable(tile));
            }
        }
        cost += 1;
    }
    Ok(cost)
}

/// Build a stairway from `pos` to the tile `dz` layers away (1 up, -1 down), paid from
/// `player`'s inventory, one Stairs per tile that isn't stairs yet. A solid tile on the
/// other layer is dug out first, and what it drops lands on the new stairs.
pub fn build_stairway(
    world: &mut World,
    res: &mut Resources,
    pos: Position,
    dz: i64,
    player: Entity,
) -> Result<(), StairsError> {
    let cost = check_stairway(world, res, pos, dz, player)?;
    if stairs_held(world, player) < cost {
        return Err(StairsError::NotEnough(cost));
    }
    let other = Position {
        z: pos.z + dz,
        ..pos
    };
    let other_tile = res
        .world_state
        .world
        .get_tile_cached(other.x, other.y, other.z);
    let dug = if other_tile.is_passable() {
        Vec::new()
    } else {
        crate::crafting::mine_tile(res, other).unwrap_or_default()
    };
    for p in [pos, other] {
        res.world_state
            .world
            .set_tile_cached(p.x, p.y, p.z, TileKind::Stairs);
    }
    for (kind, qty) in dug {
        world.spawn((
            other,
            DroppedItem { kind, qty },
            SpriteRef::new("items", itemkind_sprite_name(kind)),
        ));
    }
    if let Ok(mut inv) = world.get::<&mut Inventory>(player) {
        let mut left = cost;
        for slot in inv.slots.iter_mut().filter(|s| s.kind == ItemKind::Stairs) {
            let n = slot.qty.min(left);
            slot.qty -= n;
            left -= n;
        }
        inv.slots.retain(|s| s.qty > 0);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::{Facing, FactoryKind};
    use crate::intent::PlayerAction;
    use crate::test_support::{TestGame, TestWorld};

    fn at(x: i64, y: i64, z: i64) -> Position {
        Position { x, y, z }
    }

    fn game(stairs: u32) -> TestGame {
        TestWorld::new(3)
            .fill((-3, -3, 0), (3, 3, 0), TileKind::Grass)
            .fill((-3, -3, 1), (3, 3, 1), TileKind::Rock)
            .fill((-3, -3, -1), (3, 3, -1), TileKind::Rock)
            .inventory(&[(ItemKind::Stairs, stairs)])
            .auto_pickup(true)
            .build()
    }

    fn build(t: &mut TestGame, pos: Position, dz: i64) -> Result<(), StairsError> {
        let player = t.game.get_player_entity().unwrap();
        build_stairway(&mut t.game.world, &mut t.game.res, pos, dz, player)
    }

    fn climb(t: &mut TestGame, dz: i64) {
        t.act(PlayerAction::Move { dx: 0, dy: 0, dz });
        t.tick();
    }

    #[test]
    fn a_stairway_digs_up_through_rock_and_can_be_climbed_both_ways() {
        let mut t = game(2);
        build(&mut t, at(1, 0, 0), 1).unwrap();
        assert_eq!(t.tile(1, 0, 0), TileKind::Stairs);
        assert_eq!(t.tile(1, 0, 1), TileKind::Stairs);
        assert_eq!(t.player_has(ItemKind::Stairs), 0);
        assert_eq!(
            t.items_at(1, 0, 1),
            vec![(ItemKind::Stone, 1)],
            "the dug rock drops"
        );
        t.act(PlayerAction::Move {
            dx: 1,
            dy: 0,
            dz: 0,
        });
        t.tick();
        climb(&mut t, 1);
        assert_eq!(t.player_pos(), at(1, 0, 1));
        assert_eq!(t.player_has(ItemKind::Stone), 1, "picked up on arrival");
        climb(&mut t, -1);
        assert_eq!(t.player_pos(), at(1, 0, 0));
    }

    #[test]
    fn a_stairway_goes_down_too() {
        let mut t = game(2);
        build(&mut t, at(0, 1, 0), -1).unwrap();
        assert_eq!(t.tile(0, 1, -1), TileKind::Stairs);
    }

    #[test]
    fn one_stairs_item_is_not_enough_and_nothing_changes() {
        let mut t = game(1);
        assert_eq!(
            build(&mut t, at(1, 0, 0), 1),
            Err(StairsError::NotEnough(2))
        );
        assert_eq!(t.tile(1, 0, 1), TileKind::Rock);
        assert_eq!(t.player_has(ItemKind::Stairs), 1);
    }

    #[test]
    fn breaking_a_stair_gives_one_back() {
        let mut t = game(2);
        build(&mut t, at(1, 0, 0), 1).unwrap();
        t.act(PlayerAction::MineAt { x: 1, y: 0, z: 0 });
        t.tick();
        assert_eq!(t.tile(1, 0, 0), TileKind::Air);
        assert_eq!(t.items_at(1, 0, 0), vec![(ItemKind::Stairs, 1)]);
    }

    #[test]
    fn stacked_vertical_conveyors_climb_like_stairs() {
        let mut t = game(0);
        t.game
            .res
            .world_state
            .world
            .set_tile_cached(1, 0, 1, TileKind::Air);
        for z in 0..2 {
            crate::factory::place(
                &mut t.game.world,
                &mut t.game.res,
                FactoryKind::VerticalConveyorUp,
                at(1, 0, z),
                Facing::North,
            )
            .unwrap();
        }
        t.act(PlayerAction::Move {
            dx: 1,
            dy: 0,
            dz: 0,
        });
        t.tick();
        assert_eq!(t.player_pos(), at(1, 0, 0), "standing on the conveyor");
        climb(&mut t, 1);
        assert_eq!(t.player_pos(), at(1, 0, 1));
        // Off the top of the stack into the open, and back down onto it
        climb(&mut t, 1);
        assert_eq!(
            t.player_pos(),
            at(1, 0, 2),
            "climbs off the top into the open"
        );
        climb(&mut t, -1);
        assert_eq!(t.player_pos(), at(1, 0, 1));
        climb(&mut t, -1);
        assert_eq!(t.player_pos(), at(1, 0, 0));
    }

    #[test]
    fn a_stairway_from_where_the_player_stands() {
        let mut t = game(2);
        build(&mut t, at(0, 0, 0), -1).unwrap();
        assert_eq!(t.tile(0, 0, 0), TileKind::Stairs);
        climb(&mut t, -1);
        assert_eq!(t.player_pos(), at(0, 0, -1));
    }

    #[test]
    fn extending_a_stairway_costs_only_the_new_stair() {
        let mut t = game(3);
        build(&mut t, at(0, 0, 0), 1).unwrap();
        let player = t.game.get_player_entity().unwrap();
        let game = &mut t.game;
        assert_eq!(
            check_stairway(&game.world, &mut game.res, at(0, 0, 0), -1, player),
            Ok(1)
        );
        build(&mut t, at(0, 0, 0), -1).unwrap();
        assert_eq!(t.player_has(ItemKind::Stairs), 0);
        assert_eq!(t.tile(0, 0, -1), TileKind::Stairs);
    }

    #[test]
    fn stairs_on_either_end_are_enough_to_climb() {
        // Stairs only below the player: step down onto them
        let mut t = game(0);
        t.game
            .res
            .world_state
            .world
            .set_tile_cached(0, 0, -1, TileKind::Stairs);
        climb(&mut t, -1);
        assert_eq!(t.player_pos(), at(0, 0, -1), "down onto stairs below");
        // Stairs only where the player stands: step up into the open tile above
        climb(&mut t, 1);
        assert_eq!(t.player_pos(), at(0, 0, 0), "back up off the stairs");
    }

    #[test]
    fn something_solid_at_the_other_level_still_blocks() {
        // Stairs here, rock above: no climbing into rock
        let mut t = game(0);
        t.game
            .res
            .world_state
            .world
            .set_tile_cached(0, 0, 0, TileKind::Stairs);
        climb(&mut t, 1);
        assert_eq!(t.player_pos(), at(0, 0, 0));
        // No stairs on either end: no climbing, even into the open
        let mut t = game(0);
        t.game
            .res
            .world_state
            .world
            .set_tile_cached(0, 0, 1, TileKind::Air);
        climb(&mut t, 1);
        assert_eq!(t.player_pos(), at(0, 0, 0));
    }
}

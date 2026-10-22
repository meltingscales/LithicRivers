//! Stairs: what the player can climb between layers, and building a stairway. A stairway
//! is two stair tiles, one above the other, so it always leads somewhere.

use crate::components::{
    itemkind_sprite_name, DroppedItem, Inventory, ItemKind, Position, SpriteRef,
};
use crate::resources::Resources;
use crate::tiles::TileKind;
use hecs::{Entity, World};

/// Stairs items a stairway uses: one per stair tile.
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
    /// Fewer than `STAIRWAY_COST` stairs in the inventory
    NotEnough,
    /// Something is on one of the two tiles, or the near tile is solid
    Occupied,
    /// The tile on the other layer is solid and can't be dug out
    Undiggable(TileKind),
}

impl StairsError {
    pub fn message(self) -> String {
        match self {
            StairsError::NotEnough => {
                format!("A stairway takes {STAIRWAY_COST} stairs, one for each layer")
            }
            StairsError::Occupied => "Can't build stairs there: something is in the way".into(),
            StairsError::Undiggable(tile) => {
                format!("Can't dig the stairway through the {}", tile.display_name())
            }
        }
    }
}

/// Build a stairway from `pos` to the tile `dz` layers away (1 up, -1 down), paid from
/// `player`'s inventory. A solid tile on the other layer is dug out first, and what it
/// drops lands on the new stairs.
pub fn build_stairway(
    world: &mut World,
    res: &mut Resources,
    pos: Position,
    dz: i64,
    player: Entity,
) -> Result<(), StairsError> {
    let have: u32 = world
        .get::<&Inventory>(player)
        .map(|inv| {
            inv.slots
                .iter()
                .filter(|s| s.kind == ItemKind::Stairs)
                .map(|s| s.qty)
                .sum()
        })
        .unwrap_or(0);
    if have < STAIRWAY_COST {
        return Err(StairsError::NotEnough);
    }
    let other = Position {
        z: pos.z + dz,
        ..pos
    };
    let entity_on = |p: Position| world.query::<&Position>().iter().any(|(_, q)| *q == p);
    if !crate::factory::can_place_at(world, res, pos) || entity_on(other) {
        return Err(StairsError::Occupied);
    }
    let other_tile = res
        .world_state
        .world
        .get_tile_cached(other.x, other.y, other.z);
    if !other_tile.is_passable() && crate::crafting::mining_drop(other_tile).is_none() {
        return Err(StairsError::Undiggable(other_tile));
    }
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
        let mut left = STAIRWAY_COST;
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
        assert_eq!(build(&mut t, at(1, 0, 0), 1), Err(StairsError::NotEnough));
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
        climb(&mut t, 1);
        assert_eq!(t.player_pos(), at(1, 0, 1), "nothing to climb onto above");
        climb(&mut t, -1);
        assert_eq!(t.player_pos(), at(1, 0, 0));
    }
}

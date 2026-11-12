//! Sealed doors: SapienCorp security doors built into ruins (the datacenter's server hall).
//! A door draws a little power all the time, and while its network powers it the player
//! (and only the player, as with gates) can walk through. Without power it is a wall.

use crate::components::SpriteRef;
use crate::factory::{FactoryBlock, FactoryKind};
use crate::power::PowerConsumer;
use hecs::{Entity, World};

/// What a sealed door draws, in watts.
pub const SEALED_DOOR_W: u64 = 10_000;

/// The door's sprite, open or sealed.
pub fn sprite(open: bool) -> SpriteRef {
    SpriteRef::new(
        "factory",
        if open {
            "sealed_door_open"
        } else {
            "sealed_door_sealed"
        },
    )
}

/// Whether `entity` is a sealed door its network powers now.
pub fn is_open(world: &World, entity: Entity) -> bool {
    world
        .get::<&FactoryBlock>(entity)
        .is_ok_and(|b| b.kind == FactoryKind::SealedDoor)
        && world
            .get::<&PowerConsumer>(entity)
            .is_ok_and(|c| c.active && c.satisfaction_permille > 0)
}

/// Before power is shared: a door always wants its power.
pub fn update_demand(world: &mut World) {
    for (_, (block, consumer)) in world.query_mut::<(&FactoryBlock, &mut PowerConsumer)>() {
        if block.kind == FactoryKind::SealedDoor {
            consumer.active = true;
        }
    }
}

/// After power is shared: show each door open or sealed.
pub fn step(world: &mut World) {
    let doors: Vec<(Entity, bool)> = world
        .query::<&FactoryBlock>()
        .iter()
        .filter(|(_, b)| b.kind == FactoryKind::SealedDoor)
        .map(|(e, _)| (e, is_open(world, e)))
        .collect();
    for (door, open) in doors {
        if let Ok(mut s) = world.get::<&mut SpriteRef>(door) {
            *s = sprite(open);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{Inventory, ItemKind, Position};
    use crate::factory::{Facing, FACTORY_STEP_TICKS};
    use crate::game::Game;
    use crate::tiles::TileKind;

    fn at(x: i64, y: i64) -> Position {
        Position { x, y, z: 0 }
    }

    /// A door at the origin on a pole beside a burner generator (fueled when `fuel` > 0).
    fn doorway(fuel: u32) -> (Game, Entity, Entity) {
        let mut g = crate::test_support::TestWorld::new(9)
            .fill((-8, -8, 0), (8, 8, 0), TileKind::Grass)
            .player_at(0, 5, 0)
            .build()
            .game;
        let door = crate::factory::place(
            &mut g.world,
            &mut g.res,
            FactoryKind::SealedDoor,
            at(0, 0),
            Facing::North,
        )
        .unwrap();
        let generator = crate::factory::place(
            &mut g.world,
            &mut g.res,
            FactoryKind::BurnerGenerator,
            at(-3, 1),
            Facing::North,
        )
        .unwrap();
        crate::factory::place(
            &mut g.world,
            &mut g.res,
            FactoryKind::PowerPole,
            at(-1, 1),
            Facing::North,
        )
        .unwrap();
        if fuel > 0 {
            g.world
                .get::<&mut Inventory>(generator)
                .unwrap()
                .add(ItemKind::Stick, fuel);
        }
        (g, door, generator)
    }

    fn steps(g: &mut Game, n: u64) {
        for _ in 0..n {
            g.res.time.tick += FACTORY_STEP_TICKS;
            crate::factory::run_due_steps(&mut g.world, &mut g.res);
        }
    }

    fn walkable(g: &mut Game) -> bool {
        crate::logistics::walkable(&mut g.res, &g.world, at(0, 0))
    }

    #[test]
    fn a_door_without_power_is_a_wall() {
        let (mut g, _, _) = doorway(0);
        steps(&mut g, 5);
        assert!(!walkable(&mut g));
    }

    #[test]
    fn a_powered_door_lets_the_player_through_and_shows_it() {
        let (mut g, door, _) = doorway(20);
        steps(&mut g, 5);
        assert!(walkable(&mut g));
        assert_eq!(
            g.world.get::<&SpriteRef>(door).unwrap().name,
            "sealed_door_open"
        );
    }

    #[test]
    fn the_door_closes_when_the_fuel_runs_out() {
        let (mut g, door, _) = doorway(1);
        steps(&mut g, 2);
        assert!(walkable(&mut g), "open while the stick burns");
        steps(&mut g, 400);
        assert!(!walkable(&mut g), "sealed again");
        assert_eq!(
            g.world.get::<&SpriteRef>(door).unwrap().name,
            "sealed_door_sealed"
        );
    }

    #[test]
    fn creatures_never_pass_even_when_open() {
        let (mut g, _, _) = doorway(20);
        steps(&mut g, 5);
        // Creatures path over tiles, and the door's tile is a factory block
        assert!(!g.res.world_state.world.is_passable(0, 0, 0));
    }
}

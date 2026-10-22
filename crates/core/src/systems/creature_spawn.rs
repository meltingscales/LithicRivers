//! Creatures that appear as the player explores: each underground chunk is rolled once,
//! when the player first comes within a chunk of it, seeded by the world seed and the
//! chunk, so the same world always gets the same creatures in the same places.

use crate::components::{BlocksMovement, Position};
use crate::resources::{Resources, WorldType};
use crate::systems::ecs_utils::get_player_position;
use crate::world::{CHUNK_SIZE, CHUNK_SIZE_Z};
use hecs::World;
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;

/// Chance, in percent, that an underground chunk gets a nuclear slime.
pub const SLIME_CHANCE_PERCENT: u32 = 20;
/// Tiles tried when looking for open ground for a slime.
const SLIME_TRIES: usize = 16;
/// Slimes never appear this close to the player, so none pops up in view.
pub const SPAWN_MIN_DISTANCE: i64 = 16;

pub fn creature_spawn_system(world: &mut World, res: &mut Resources) {
    if res.world_state.world.world_type != WorldType::Normal {
        return;
    }
    let Some(player) = get_player_position(world) else {
        return;
    };
    let (pcx, pcy, pcz) = (
        player.x.div_euclid(CHUNK_SIZE),
        player.y.div_euclid(CHUNK_SIZE),
        player.z.div_euclid(CHUNK_SIZE_Z),
    );
    // Only underground layers
    if pcz >= 0 {
        return;
    }
    for cy in pcy - 1..=pcy + 1 {
        for cx in pcx - 1..=pcx + 1 {
            if res.spawn_rolled_chunks.insert((cx, cy, pcz)) {
                roll_slime(world, res, (cx, cy, pcz), player);
            }
        }
    }
}

fn roll_slime(
    world: &mut World,
    res: &mut Resources,
    (cx, cy, cz): (i64, i64, i64),
    player: Position,
) {
    let seed = res.world_state.world.mix_coords(cx, cy, cz) ^ 0x5_1A4E_C4A7;
    let mut rng = ChaCha20Rng::seed_from_u64(seed);
    if rng.gen_range(0..100) >= SLIME_CHANCE_PERCENT {
        return;
    }
    for _ in 0..SLIME_TRIES {
        let pos = Position {
            x: cx * CHUNK_SIZE + rng.gen_range(0..CHUNK_SIZE),
            y: cy * CHUNK_SIZE + rng.gen_range(0..CHUNK_SIZE),
            z: cz,
        };
        let far = (pos.x - player.x).abs().max((pos.y - player.y).abs()) >= SPAWN_MIN_DISTANCE;
        let open = res
            .world_state
            .world
            .get_tile_cached(pos.x, pos.y, pos.z)
            .is_passable();
        let free = !world
            .query::<&Position>()
            .with::<&BlocksMovement>()
            .iter()
            .any(|(_, p)| *p == pos);
        if far && open && free {
            crate::spawn_utils::world_spawn_nuclear_slime(
                world,
                pos.x,
                pos.y,
                pos.z,
                &res.world_state.world,
            );
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{EntityKind, Inventory, ItemKind, NuclearSlime};
    use crate::intent::PlayerAction;
    use crate::test_support::TestWorld;
    use crate::tiles::TileKind;

    fn waste(world: &World, slime: hecs::Entity) -> u32 {
        world
            .get::<&Inventory>(slime)
            .map(|inv| {
                inv.slots
                    .iter()
                    .filter(|s| s.kind == ItemKind::NuclearWaste)
                    .map(|s| s.qty)
                    .sum()
            })
            .unwrap_or(0)
    }

    #[test]
    fn a_slime_is_tougher_than_a_dog() {
        let mut world = TestWorld::new(5);
        let slime = world.nuclear_slime(3, 0, 0);
        let dog = world.feral_dog(-3, 0, 0);
        let game = world.build();
        assert!(game.health(slime) > game.health(dog));
    }

    #[test]
    fn about_one_slime_in_twenty_carries_waste_and_always_the_same_one() {
        let game = TestWorld::new(5).build().game;
        let mut world = World::new();
        let mut carriers = 0;
        for i in 0..2000 {
            let slime = crate::spawn_utils::world_spawn_nuclear_slime(
                &mut world,
                i,
                i * 7,
                -20,
                &game.res.world_state.world,
            );
            carriers += u32::from(waste(&world, slime) > 0);
        }
        let percent = carriers as f64 / 20.0;
        assert!((3.0..=7.0).contains(&percent), "{percent}% carry waste");
        // Same seed, same spot, same slime
        let again = TestWorld::new(5).build().game;
        let mut other = World::new();
        for i in 0..200 {
            let a = crate::spawn_utils::world_spawn_nuclear_slime(
                &mut world,
                i,
                3,
                -20,
                &game.res.world_state.world,
            );
            let b = crate::spawn_utils::world_spawn_nuclear_slime(
                &mut other,
                i,
                3,
                -20,
                &again.res.world_state.world,
            );
            assert_eq!(waste(&world, a), waste(&other, b));
        }
    }

    #[test]
    fn a_slime_closes_in_slowly_then_fights() {
        let mut world = TestWorld::new(5).fill((-8, -8, 0), (8, 8, 0), TileKind::Grass);
        let slime = world.nuclear_slime(4, 0, 0);
        let mut t = world.build();
        for _ in 0..5 {
            t.act(PlayerAction::Move {
                dx: 0,
                dy: 0,
                dz: 0,
            });
            t.tick();
        }
        assert!(
            t.pos(slime).unwrap().x > 1,
            "not adjacent yet after 5 turns"
        );
        t.act(PlayerAction::Move {
            dx: 0,
            dy: 0,
            dz: 0,
        });
        t.tick();
        assert_eq!(t.pos(slime).unwrap().x, 1, "adjacent after 6 turns");
        t.act(PlayerAction::Move {
            dx: 0,
            dy: 0,
            dz: 0,
        });
        t.tick();
        assert!(t.game.res.player_state.combat_active);
    }

    #[test]
    fn a_slime_leaves_a_corpse_only_if_it_carries_waste() {
        let game = TestWorld::new(5).build().game;
        let mut world = World::new();
        let mut res = game.res;
        let mut carrier = None;
        let mut empty = None;
        for i in 0..400 {
            let slime = crate::spawn_utils::world_spawn_nuclear_slime(
                &mut world,
                i,
                0,
                -20,
                &res.world_state.world,
            );
            if waste(&world, slime) > 0 {
                carrier.get_or_insert(slime);
            } else {
                empty.get_or_insert(slime);
            }
        }
        let (carrier, empty) = (carrier.unwrap(), empty.unwrap());
        let corpses = |world: &World| {
            world
                .query::<&EntityKind>()
                .iter()
                .filter(|(_, k)| **k == EntityKind::Corpse)
                .count()
        };
        crate::systems::handle_entity_death(&mut world, &mut res, empty);
        assert_eq!(corpses(&world), 0);
        assert!(world.get::<&NuclearSlime>(empty).is_err(), "it dissolved");
        crate::systems::handle_entity_death(&mut world, &mut res, carrier);
        assert_eq!(corpses(&world), 1);
        let corpse = world
            .query::<(&EntityKind, &Inventory)>()
            .iter()
            .find(|(_, (k, _))| **k == EntityKind::Corpse)
            .map(|(_, (_, inv))| inv.slots.clone())
            .unwrap();
        assert_eq!(corpse[0].kind, ItemKind::NuclearWaste);
    }

    #[test]
    fn slimes_spawn_underground_and_never_on_the_surface() {
        let slimes = |z: i64| {
            let mut t = TestWorld::new(12345).player_at(0, 0, z).build();
            for x in (0..10).map(|i| i * CHUNK_SIZE * 3) {
                *t.game
                    .world
                    .get::<&mut Position>(t.game.get_player_entity().unwrap())
                    .unwrap() = Position { x, y: 0, z };
                creature_spawn_system(&mut t.game.world, &mut t.game.res);
            }
            let count = t.game.world.query::<&NuclearSlime>().iter().count();
            count
        };
        assert_eq!(slimes(0), 0);
        assert!(slimes(-20) > 0);
    }

    #[test]
    fn a_slime_fight_replays_the_same() {
        let fight = || {
            let mut world = TestWorld::new(5).fill((-8, -8, 0), (8, 8, 0), TileKind::Grass);
            world.nuclear_slime(2, 0, 0);
            let mut t = world.build();
            for _ in 0..40 {
                t.act(PlayerAction::Move {
                    dx: 0,
                    dy: 0,
                    dz: 0,
                });
                t.tick();
            }
            // The fight's log: who hit what, for how much, and when
            t.game
                .res
                .events
                .get_message_log()
                .messages
                .iter()
                .map(|m| m.text.clone())
                .collect::<Vec<String>>()
        };
        let log = fight();
        assert!(
            log.iter().any(|m| m.contains("from enemy attack")),
            "the slime attacked: {log:?}"
        );
        assert_eq!(fight(), log);
    }
}

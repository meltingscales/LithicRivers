//! The player's death: the old body is left as a wreck holding what the robot carried and
//! the body itself, and the robot can respawn at the bunker with the default body and the
//! starting kit. (A stand-in until a real backup body is designed.)

use crate::components::{
    BattleDelay, Dead, EntityKind, GameEntity, Inventory, Position, SpriteRef, Wreck,
};
use crate::model::body::Body;
use crate::resources::Resources;
use hecs::{Entity, World};

/// The player has died of `cause`: mark them Dead and leave the wreck. Safe to call more
/// than once for the same death.
pub fn player_died(world: &mut World, res: &mut Resources, player: Entity, cause: String) {
    world.insert_one(player, Dead).ok();
    if res.player_state.death_cause.is_some() {
        return;
    }
    let Ok(pos) = world.get::<&Position>(player).map(|p| *p) else {
        return;
    };
    let slots = world
        .get::<&mut Inventory>(player)
        .map(|mut inv| std::mem::take(&mut inv.slots))
        .unwrap_or_default();
    let body = world.get::<&Body>(player).map(|b| (*b).clone()).ok();
    let mut builder = hecs::EntityBuilder::new();
    builder.add(pos);
    builder.add(GameEntity);
    builder.add(EntityKind::Corpse);
    builder.add(Wreck);
    builder.add(SpriteRef::new("entities", "wreck"));
    builder.add(Inventory {
        max_slots: slots.len().max(Inventory::default().max_slots),
        slots,
        auto_pickup: false,
    });
    if let Some(body) = body {
        builder.add(body);
    }
    world.spawn(builder.build());
    res.player_state.combat_active = false;
    res.log_red(format!(
        "Your body was destroyed: {cause}. It lies here with everything you carried."
    ));
    res.player_state.death_cause = Some(cause);
}

/// Whether the player is dead and waiting to respawn.
pub fn is_dead(world: &World, player: Entity) -> bool {
    world.get::<&Dead>(player).is_ok()
}

impl crate::Game {
    /// Bring a dead player back at the respawn point with the default body and the starting
    /// kit. Returns where, or `None` if the player isn't dead.
    pub fn respawn(&mut self) -> Option<Position> {
        let player = self.get_player_entity()?;
        if !is_dead(&self.world, player) {
            return None;
        }
        let [x, y, z] = self.res.config.default_player_position;
        let point = self
            .res
            .player_state
            .respawn_point
            .unwrap_or(Position { x, y, z });
        let auto_pickup = self
            .world
            .get::<&Inventory>(player)
            .map(|inv| inv.auto_pickup)
            .unwrap_or(self.res.config.auto_pickup_enabled);
        let _ = self.world.remove_one::<Dead>(player);
        let _ = self.world.remove_one::<crate::moves::ActionQueue>(player);
        let _ = self.world.insert(
            player,
            (
                point,
                Body::default(),
                crate::game::starting_inventory(auto_pickup),
                // Whatever waits at the respawn point can't attack at once
                BattleDelay::new(1000),
            ),
        );
        if let Ok(mut energy) = self.world.get::<&mut crate::components::Energy>(player) {
            energy.current = energy.max;
        }
        self.res.player_state.death_cause = None;
        self.res.player_state.combat_active = false;
        self.res
            .log_green("Backup systems online. You come to at the bunker.");
        Some(point)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::ItemKind;
    use crate::model::body::BodyPartType;
    use crate::test_support::TestWorld;
    use crate::tiles::TileKind;

    /// A wreck: where, what it holds, and whether it kept the body.
    type WreckView = (Position, Vec<(ItemKind, u32)>, bool);

    fn wrecks(world: &World) -> Vec<WreckView> {
        world
            .query::<(&Position, &Inventory, Option<&Body>)>()
            .with::<&Wreck>()
            .iter()
            .map(|(_, (p, inv, body))| {
                (
                    *p,
                    inv.slots.iter().map(|s| (s.kind, s.qty)).collect(),
                    body.is_some(),
                )
            })
            .collect()
    }

    fn dying_game() -> crate::test_support::TestGame {
        let mut t = TestWorld::new(3)
            .fill((-5, -5, 0), (5, 5, 0), TileKind::Grass)
            .player_at(2, 2, 0)
            .inventory(&[(ItemKind::Log, 3), (ItemKind::Stone, 7)])
            .build();
        t.game.res.player_state.respawn_point = Some(Position { x: -3, y: -3, z: 0 });
        t
    }

    #[test]
    fn dying_leaves_a_wreck_with_everything_carried_and_the_body() {
        let mut t = dying_game();
        let player = t.game.get_player_entity().unwrap();
        // A damaged leg, to see the old body kept on the wreck
        t.game
            .world
            .get::<&mut Body>(player)
            .unwrap()
            .parts
            .get_mut(&BodyPartType::LeftLeg)
            .unwrap()
            .receive_damage(70, false);
        let game = &mut t.game;
        player_died(
            &mut game.world,
            &mut game.res,
            player,
            "Torso destroyed by a dog".into(),
        );
        player_died(&mut game.world, &mut game.res, player, "again".into());
        let w = wrecks(&t.game.world);
        assert_eq!(w.len(), 1, "one wreck, however often death is reported");
        assert_eq!(w[0].0, Position { x: 2, y: 2, z: 0 });
        assert_eq!(w[0].1, vec![(ItemKind::Log, 3), (ItemKind::Stone, 7)]);
        assert!(w[0].2, "the wreck keeps the body");
        assert!(t.player_inventory().is_empty(), "nothing is duplicated");
        assert!(is_dead(&t.game.world, player));
        assert_eq!(
            t.game.res.player_state.death_cause.as_deref(),
            Some("Torso destroyed by a dog")
        );
    }

    #[test]
    fn respawning_brings_back_the_default_body_and_kit_at_the_bunker() {
        let mut t = dying_game();
        let player = t.game.get_player_entity().unwrap();
        let game = &mut t.game;
        player_died(&mut game.world, &mut game.res, player, "shut down".into());
        assert_eq!(t.game.respawn(), Some(Position { x: -3, y: -3, z: 0 }));
        assert_eq!(t.player_pos(), Position { x: -3, y: -3, z: 0 });
        assert!(!is_dead(&t.game.world, player));
        assert!(t.game.res.player_state.death_cause.is_none());
        let kit: Vec<(ItemKind, u32)> = crate::game::starting_inventory(false)
            .slots
            .iter()
            .map(|s| (s.kind, s.qty))
            .collect();
        assert_eq!(t.player_inventory(), kit);
        let body = t.game.world.get::<&Body>(player).unwrap();
        let default = Body::default();
        for (part, p) in &default.parts {
            assert_eq!(body.parts[part].state, p.state, "{part:?}");
        }
        drop(body);
        assert_eq!(
            wrecks(&t.game.world).len(),
            1,
            "the wreck stays where it was"
        );
        assert_eq!(t.game.respawn(), None, "only the dead respawn");
        // Moving works again
        t.act(crate::intent::PlayerAction::Move {
            dx: 1,
            dy: 0,
            dz: 0,
        });
        t.tick();
        assert_eq!(t.player_pos(), Position { x: -2, y: -3, z: 0 });
    }

    #[test]
    fn a_new_game_remembers_the_bunker_spawn() {
        let game = crate::Game::new(12345);
        assert_eq!(
            game.res.player_state.respawn_point,
            game.get_player_position()
        );
    }
}

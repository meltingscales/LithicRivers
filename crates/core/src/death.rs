//! The player's death: the old body is left as a wreck holding what the robot carried and
//! the body itself, and the robot can respawn at the bunker with the default body and the
//! starting kit. (A stand-in until a real backup body is designed.)

use crate::components::{itemkind_name, ItemStack};
use crate::components::{
    BattleDelay, Dead, EntityKind, GameEntity, Inventory, Position, SpriteRef, Wreck,
};
use crate::model::body::Body;
use crate::resources::{QuestMarker, QuestMarkerType, Resources};
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
    let keep_inventory = res.world_settings.keep_inventory;
    let slots = if keep_inventory {
        Vec::new()
    } else {
        world
            .get::<&mut Inventory>(player)
            .map(|mut inv| std::mem::take(&mut inv.slots))
            .unwrap_or_default()
    };
    let description = wreck_description(&cause, &slots);
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
    res.player_state.deaths += 1;
    res.add_quest_marker(QuestMarker {
        name: format!("Wreck {}", res.player_state.deaths),
        description,
        x: pos.x,
        y: pos.y,
        z: pos.z,
        marker_type: QuestMarkerType::Death,
    });
    res.log_red(if keep_inventory {
        format!("Your body was destroyed: {cause}. You keep everything you carried.")
    } else {
        format!("Your body was destroyed: {cause}. It lies here with everything you carried.")
    });
    res.player_state.death_cause = Some(cause);
}

/// A death marker's description: what destroyed the body and the main things it carried.
fn wreck_description(cause: &str, slots: &[ItemStack]) -> String {
    const LISTED: usize = 3;
    let mut stacks: Vec<&ItemStack> = slots.iter().collect();
    // Biggest stacks first; the sort is stable, so ties keep inventory order
    stacks.sort_by_key(|s| std::cmp::Reverse(s.qty));
    let mut held: Vec<String> = stacks
        .iter()
        .take(LISTED)
        .map(|s| format!("{} x{}", itemkind_name(s.kind), s.qty))
        .collect();
    if stacks.len() > LISTED {
        held.push(format!("{} more", stacks.len() - LISTED));
    }
    if held.is_empty() {
        format!("Cause: {cause}. It carried nothing.")
    } else {
        format!("Cause: {cause}. Holds {}.", held.join(", "))
    }
}

/// Remove the death markers whose wreck has been looted empty or is gone.
pub fn prune_death_markers(world: &World, res: &mut Resources) {
    if !res
        .quest_markers
        .iter()
        .any(|m| m.marker_type == QuestMarkerType::Death)
    {
        return;
    }
    let full_wrecks: Vec<Position> = world
        .query::<(&Position, &Inventory)>()
        .with::<&Wreck>()
        .iter()
        .filter(|(_, (_, inv))| !inv.slots.is_empty())
        .map(|(_, (pos, _))| *pos)
        .collect();
    res.quest_markers.retain(|m| {
        m.marker_type != QuestMarkerType::Death
            || full_wrecks.contains(&Position {
                x: m.x,
                y: m.y,
                z: m.z,
            })
    });
}

/// Whether the player is dead and waiting to respawn.
pub fn is_dead(world: &World, player: Entity) -> bool {
    world.get::<&Dead>(player).is_ok()
}

impl crate::Game {
    /// Bring a dead player back at the respawn point with the default body and the starting
    /// kit, or with the repaired old body and the carried items when the world keeps them.
    /// Returns where, or `None` if the player isn't dead.
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
        let settings = self.res.world_settings;
        let body = match self.world.get::<&Body>(player) {
            Ok(old) if settings.keep_body => {
                let mut body = (*old).clone();
                body.parts.values_mut().for_each(|p| p.repair_fully());
                body
            }
            _ => Body::default(),
        };
        let _ = self.world.remove_one::<Dead>(player);
        let _ = self.world.remove_one::<crate::moves::ActionQueue>(player);
        let _ = self.world.insert(
            player,
            (
                point,
                body,
                // Whatever waits at the respawn point can't attack at once
                BattleDelay::new(1000),
            ),
        );
        if !settings.keep_inventory {
            let _ = self
                .world
                .insert_one(player, crate::game::starting_inventory(auto_pickup));
        }
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
    use crate::model::body::{BodyPartState, BodyPartType};
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

    fn death_markers(res: &Resources) -> Vec<(String, String, Position)> {
        res.quest_markers
            .iter()
            .filter(|m| m.marker_type == QuestMarkerType::Death)
            .map(|m| {
                let pos = Position {
                    x: m.x,
                    y: m.y,
                    z: m.z,
                };
                (m.name.clone(), m.description.clone(), pos)
            })
            .collect()
    }

    #[test]
    fn each_death_marks_its_wreck_until_it_is_looted() {
        let mut t = dying_game();
        let player = t.game.get_player_entity().unwrap();
        let game = &mut t.game;
        player_died(
            &mut game.world,
            &mut game.res,
            player,
            "destroyed by a dog".into(),
        );
        assert_eq!(
            death_markers(&t.game.res),
            vec![(
                "Wreck 1".to_string(),
                "Cause: destroyed by a dog. Holds Stone x7, Log x3.".to_string(),
                Position { x: 2, y: 2, z: 0 }
            )]
        );

        // A second death with nothing carried gets the next number
        t.game.respawn();
        t.game
            .world
            .get::<&mut Inventory>(player)
            .unwrap()
            .slots
            .clear();
        let game = &mut t.game;
        player_died(&mut game.world, &mut game.res, player, "shut down".into());
        t.tick();
        let markers = death_markers(&t.game.res);
        assert_eq!(markers.len(), 1, "an empty wreck isn't marked");
        assert_eq!(markers[0].0, "Wreck 1");
        assert_eq!(t.game.res.player_state.deaths, 2);

        // Looting the first wreck empty removes its marker
        for (_, inv) in t.game.world.query_mut::<&mut Inventory>().with::<&Wreck>() {
            inv.slots.clear();
        }
        t.tick();
        assert!(death_markers(&t.game.res).is_empty());
    }

    #[test]
    fn keep_inventory_keeps_what_was_carried_and_leaves_an_empty_wreck() {
        let mut t = dying_game();
        t.game.res.world_settings.keep_inventory = true;
        let player = t.game.get_player_entity().unwrap();
        let game = &mut t.game;
        player_died(&mut game.world, &mut game.res, player, "shut down".into());
        let w = wrecks(&t.game.world);
        assert_eq!(w.len(), 1, "the wreck still marks the place");
        assert!(w[0].1.is_empty(), "the wreck is empty");
        t.game.respawn();
        assert_eq!(
            t.player_inventory(),
            vec![(ItemKind::Log, 3), (ItemKind::Stone, 7)],
            "no starting kit on top"
        );
    }

    #[test]
    fn keep_body_repairs_the_old_body_and_a_missing_arm_stays_missing() {
        let mut t = dying_game();
        t.game.res.world_settings.keep_body = true;
        let player = t.game.get_player_entity().unwrap();
        {
            let mut body = t.game.world.get::<&mut Body>(player).unwrap();
            body.parts
                .get_mut(&BodyPartType::LeftLeg)
                .unwrap()
                .receive_damage(70, false);
            body.parts
                .get_mut(&BodyPartType::RightArm)
                .unwrap()
                .receive_damage(200, true);
        }
        let game = &mut t.game;
        player_died(&mut game.world, &mut game.res, player, "shut down".into());
        t.game.respawn();
        let body = t.game.world.get::<&Body>(player).unwrap();
        let leg = &body.parts[&BodyPartType::LeftLeg];
        assert_eq!((leg.state, leg.integrity), (BodyPartState::Functional, 100));
        assert_eq!(
            body.parts[&BodyPartType::RightArm].state,
            BodyPartState::Missing
        );
        // The default body's missing left arm stays missing too
        assert_eq!(
            body.parts[&BodyPartType::LeftArm].state,
            BodyPartState::Missing
        );
        let core = &body.parts[&BodyPartType::PowerSource];
        assert_eq!(core.integrity, 100, "damaged parts come back whole");
    }

    #[test]
    fn a_wreck_description_lists_the_biggest_stacks() {
        let slots: Vec<ItemStack> = [
            (ItemKind::Log, 3),
            (ItemKind::Stone, 7),
            (ItemKind::Stick, 1),
            (ItemKind::Acorn, 5),
        ]
        .into_iter()
        .map(|(kind, qty)| ItemStack { kind, qty })
        .collect();
        assert_eq!(
            wreck_description("Torso destroyed by a dog", &slots),
            "Cause: Torso destroyed by a dog. Holds Stone x7, Acorn x5, Log x3, 1 more."
        );
    }
}

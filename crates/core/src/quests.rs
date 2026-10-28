//! Quests the game advances on its own each tick: "A New Arm".

use crate::components::{itemkind_name, Inventory, ItemKind, Player, Position};
use crate::dialogue::{ActiveQuest, QuestObjective, QuestObjectiveType, QuestState, QuestType};
use crate::game::FACILITY_POS;
use crate::model::body::{Body, BodyPartState, BodyPartType};
use crate::resources::Resources;
use hecs::World;

/// The facility's entrance: the surface door of its top room.
pub const FACILITY_ENTRANCE: (i64, i64, i64) =
    (FACILITY_POS.0 + 6, FACILITY_POS.1 + 23, FACILITY_POS.2 - 1);
/// How close (in tiles, on the same level) counts as reaching a location.
pub const REACH_RADIUS: i64 = 6;

/// What the android says once the arm is in.
pub const ARM_QUEST_CLOSING_LINE: &str =
    "SC-M4X7 on the radio: \"Arm signal detected. Two hands again, user!\"";

fn objective(description: &str, objective_type: QuestObjectiveType) -> QuestObjective {
    QuestObjective {
        description: description.to_string(),
        completed: false,
        objective_type,
    }
}

/// "A New Arm", offered by the broken android once it's repaired.
pub fn replace_arm(npc_entity: Option<hecs::Entity>) -> ActiveQuest {
    let (x, y, z) = FACILITY_ENTRANCE;
    ActiveQuest::new(
        QuestType::ReplaceArm,
        "A New Arm".to_string(),
        "SapienCorp built arms like yours at the facility a mile north. Find a salvaged one and install it.".to_string(),
        vec![
            objective(
                "Reach the SapienCorp facility",
                QuestObjectiveType::GoToLocation { x, y, z },
            ),
            objective(
                "Find the salvaged arm on the assembly line",
                QuestObjectiveType::FetchItem {
                    item_name: itemkind_name(ItemKind::SalvagedArm).to_string(),
                    quantity: 1,
                    consumed: false,
                },
            ),
            objective(
                "Install the arm (Body tab)",
                QuestObjectiveType::InstallPart {
                    part: BodyPartType::LeftArm,
                },
            ),
        ],
        npc_entity,
    )
}

fn objective_met(
    objective: &QuestObjectiveType,
    pos: Position,
    inventory: Option<&Inventory>,
    body: Option<&Body>,
) -> bool {
    match objective {
        QuestObjectiveType::GoToLocation { x, y, z } => {
            pos.z == *z && (pos.x - x).abs() <= REACH_RADIUS && (pos.y - y).abs() <= REACH_RADIUS
        }
        QuestObjectiveType::FetchItem {
            item_name,
            quantity,
            ..
        } => inventory.is_some_and(|inv| {
            inv.slots
                .iter()
                .any(|s| itemkind_name(s.kind) == item_name && s.qty >= *quantity)
        }),
        QuestObjectiveType::InstallPart { part } => body.is_some_and(|b| {
            b.parts
                .get(part)
                .is_some_and(|p| p.state != BodyPartState::Missing)
        }),
        QuestObjectiveType::TalkToNPC { .. } => false,
    }
}

/// Advance ordered quests: the furthest objective that holds now completes, and every
/// objective before it with it, so steps done early (the arm installed before the quest
/// was even accepted) still count, and objectives never come undone.
pub fn advance(world: &World, res: &mut Resources) {
    let Some((player, &pos)) = world
        .query::<(&Player, &Position)>()
        .iter()
        .next()
        .map(|(e, (_, p))| (e, p))
    else {
        return;
    };
    let inventory = world.get::<&Inventory>(player).ok();
    let body = world.get::<&Body>(player).ok();

    let mut objectives_done = Vec::new();
    let mut quests_done = Vec::new();
    for quest in res
        .active_quests
        .iter_mut()
        .filter(|q| q.state == QuestState::Active && q.quest_type.ordered())
    {
        let furthest = quest.objectives.iter().rposition(|o| {
            objective_met(
                &o.objective_type,
                pos,
                inventory.as_deref(),
                body.as_deref(),
            )
        });
        let Some(furthest) = furthest else {
            continue;
        };
        for objective in &mut quest.objectives[..=furthest] {
            if !objective.completed {
                objective.completed = true;
                objectives_done.push(objective.description.clone());
            }
        }
        if quest.is_completed() {
            quests_done.push(quest.quest_type);
        }
    }
    drop(inventory);
    drop(body);

    for description in objectives_done {
        res.log_green(format!("Quest objective completed: {description}"));
    }
    for quest_type in quests_done {
        res.complete_quest(quest_type);
        if quest_type == QuestType::ReplaceArm {
            res.log_green(ARM_QUEST_CLOSING_LINE);
        }
    }
}

/// Places the `tp_quest` cheat can take the player, for debugging quests.
pub const TP_QUEST_TARGETS: [&str; 3] = ["android", "facility", "locker"];

/// A walkable tile next to `pos` (east, west, south, north), or `pos` itself.
fn beside(res: &mut Resources, pos: (i64, i64, i64)) -> Position {
    let (x, y, z) = pos;
    [(1, 0), (-1, 0), (0, 1), (0, -1)]
        .into_iter()
        .map(|(dx, dy)| (x + dx, y + dy))
        .find(|&(nx, ny)| {
            res.world_state
                .world
                .get_tile_cached(nx, ny, z)
                .is_passable()
        })
        .map_or(Position { x, y, z }, |(x, y)| Position { x, y, z })
}

/// Where `tp_quest <target>` goes: beside the broken android, outside the facility's
/// door, or beside the arm locker. With no target, wherever the arm quest needs the
/// player next (the android before the quest is accepted).
pub fn tp_quest_target(
    world: &World,
    res: &mut Resources,
    target: Option<&str>,
) -> Result<Position, String> {
    let target = match target {
        Some(t) => t,
        None => match res.get_active_quest(QuestType::ReplaceArm) {
            None => "android",
            Some(quest) if quest.state != QuestState::Active => {
                return Err("A New Arm is already done".to_string())
            }
            Some(quest) => match quest.objectives.iter().position(|o| !o.completed) {
                Some(0) => "facility",
                Some(1) => "locker",
                _ => {
                    return Err("Nothing to walk to: install the arm from the Body tab".to_string())
                }
            },
        },
    };
    match target {
        "android" => {
            let (_, pos) = world
                .query::<(&crate::components::QuestTutorialBrokenAndroid, &Position)>()
                .iter()
                .next()
                .map(|(e, (_, p))| (e, *p))
                .ok_or("No broken android in this world")?;
            Ok(beside(res, (pos.x, pos.y, pos.z)))
        }
        "facility" => {
            let (x, y, z) = FACILITY_ENTRANCE;
            Ok(beside(res, (x, y, z)))
        }
        "locker" => {
            let chest = *res
                .quest_chests_placed
                .iter()
                .next()
                .ok_or("The arm locker hasn't been placed yet")?;
            Ok(beside(res, chest))
        }
        other => Err(format!(
            "unknown quest target '{other}'; targets: {}",
            TP_QUEST_TARGETS.join(", ")
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialogue::DialogueNodeID;
    use crate::dialogue_database::DialogueDatabase;
    use crate::game::Game;

    fn reload(game: &mut Game) {
        let saved = crate::save_load::save_to_string(game, &serde_json::Value::Null).unwrap();
        crate::save_load::load_from_str(game, &saved).unwrap();
    }

    fn done(game: &Game) -> Vec<bool> {
        game.res
            .get_active_quest(QuestType::ReplaceArm)
            .unwrap()
            .objectives
            .iter()
            .map(|o| o.completed)
            .collect()
    }

    fn teleport(game: &mut Game, (x, y, z): (i64, i64, i64)) {
        *game.get_player_component_mut::<Position>().unwrap() = Position { x, y, z };
    }

    /// Take the arm out of the facility's chest into the player's inventory.
    fn take_arm(game: &mut Game) {
        let chest = game
            .world
            .query::<&Inventory>()
            .iter()
            .find(|(_, inv)| inv.slots.iter().any(|s| s.kind == ItemKind::SalvagedArm))
            .map(|(e, _)| e)
            .expect("the arm chest");
        game.world
            .get::<&mut Inventory>(chest)
            .unwrap()
            .slots
            .clear();
        let player = game.get_player_entity().unwrap();
        game.world
            .get::<&mut Inventory>(player)
            .unwrap()
            .add(ItemKind::SalvagedArm, 1);
    }

    fn finish_repair_quest(game: &mut Game) {
        game.res.start_quest(ActiveQuest::new(
            QuestType::RepairBrokenAndroid,
            "Repair the Broken Android".to_string(),
            String::new(),
            vec![],
            None,
        ));
        game.res.complete_quest(QuestType::RepairBrokenAndroid);
    }

    /// Whether the android's tree shows the choice leading to (or unlocking) something.
    fn offered(
        game: &Game,
        node: DialogueNodeID,
        pick: impl Fn(&crate::dialogue::DialogueChoice) -> bool,
    ) -> bool {
        let db = DialogueDatabase::new();
        let tree = db.get_tree("broken_android_tutorial").unwrap();
        tree.get_node(&node)
            .unwrap()
            .choices
            .iter()
            .filter(|c| pick(c))
            .any(|c| game.res.quest_requirements_met(c))
    }

    #[test]
    fn tp_quest_follows_the_arm_quest() {
        let mut game = Game::new(12345);
        let target = |game: &mut Game, name| tp_quest_target(&game.world, &mut game.res, name);

        // Before the quest: the android
        let android = target(&mut game, None).unwrap();
        assert_eq!(target(&mut game, Some("android")), Ok(android));

        game.res.start_quest(replace_arm(None));
        let facility = target(&mut game, None).unwrap();
        let (x, y, z) = FACILITY_ENTRANCE;
        assert_eq!(facility.z, z);
        assert!((facility.x - x).abs() <= 1 && (facility.y - y).abs() <= 1);

        teleport(&mut game, (facility.x, facility.y, facility.z));
        game.tick();
        let locker = target(&mut game, None).unwrap();
        let chest = *game.res.quest_chests_placed.iter().next().unwrap();
        assert_eq!(locker.z, chest.2);
        assert_eq!((locker.x - chest.0).abs() + (locker.y - chest.1).abs(), 1);
        assert!(game
            .res
            .world_state
            .world
            .get_tile_cached(locker.x, locker.y, locker.z)
            .is_passable());

        assert!(target(&mut game, Some("moon")).is_err());
    }

    #[test]
    fn the_android_offers_the_arm_quest_after_its_repair() {
        let mut game = Game::new(12345);
        let asks = |c: &crate::dialogue::DialogueChoice| c.leads_to == Some(DialogueNodeID::NewArm);
        let accepts =
            |c: &crate::dialogue::DialogueChoice| c.unlocks_quest == Some(QuestType::ReplaceArm);
        assert!(!offered(&game, DialogueNodeID::Start, asks));

        finish_repair_quest(&mut game);
        assert!(offered(&game, DialogueNodeID::Start, asks));
        assert!(offered(&game, DialogueNodeID::NewArm, accepts));

        game.res.start_quest(replace_arm(None));
        assert!(!offered(&game, DialogueNodeID::NewArm, accepts));
    }

    #[test]
    fn the_arm_quest_completes_in_order_across_reloads() {
        let mut game = Game::new(12345);
        finish_repair_quest(&mut game);
        game.res.start_quest(replace_arm(None));
        reload(&mut game);
        game.tick();
        assert_eq!(done(&game), [false, false, false]);

        let (x, y, z) = FACILITY_ENTRANCE;
        teleport(&mut game, (x + 3, y + 2, z));
        game.tick();
        assert_eq!(done(&game), [true, false, false]);
        reload(&mut game);

        take_arm(&mut game);
        game.tick();
        assert_eq!(done(&game), [true, true, false]);
        reload(&mut game);

        assert_eq!(
            game.install_part(ItemKind::SalvagedArm),
            Ok(BodyPartType::LeftArm)
        );
        assert_eq!(done(&game), [true, true, true]);
        assert!(game.res.is_quest_completed(QuestType::ReplaceArm));
        assert!(!game
            .res
            .quest_markers
            .iter()
            .any(|m| m.name == crate::game::FACILITY_MARKER));
        assert!(game
            .res
            .events
            .get_message_log()
            .messages
            .iter()
            .any(|m| m.text.ends_with(ARM_QUEST_CLOSING_LINE)));
        reload(&mut game);
        assert!(game.res.is_quest_completed(QuestType::ReplaceArm));

        // A second install is refused: the arm isn't missing
        let player = game.get_player_entity().unwrap();
        game.world
            .get::<&mut Inventory>(player)
            .unwrap()
            .add(ItemKind::SalvagedArm, 1);
        assert_eq!(
            game.install_part(ItemKind::SalvagedArm),
            Err(crate::model::body::InstallError::NotMissing)
        );
    }

    #[test]
    fn an_arm_installed_before_accepting_completes_the_quest_at_once() {
        let mut game = Game::new(12345);
        take_arm(&mut game);
        game.install_part(ItemKind::SalvagedArm).unwrap();
        finish_repair_quest(&mut game);
        game.res.start_quest(replace_arm(None));
        game.tick();
        assert!(game.res.is_quest_completed(QuestType::ReplaceArm));
    }
}

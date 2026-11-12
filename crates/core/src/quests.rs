//! Quests the game advances on its own each tick: "A New Arm".

use crate::components::{itemkind_name, Inventory, ItemKind, Player, Position};
use crate::dialogue::{ActiveQuest, QuestObjective, QuestObjectiveType, QuestState, QuestType};
use crate::game::{DATACENTER_MARKER, DATACENTER_POS, FACILITY_POS};
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

/// The datacenter's entrance: the street outside its doors.
pub const DATACENTER_ENTRANCE: (i64, i64, i64) = (
    DATACENTER_POS.0 + 10,
    DATACENTER_POS.1 + 16,
    DATACENTER_POS.2,
);
/// The sealed door into the datacenter's server hall.
pub const DATACENTER_DOOR: (i64, i64, i64) = (
    DATACENTER_POS.0 + 14,
    DATACENTER_POS.1 + 8,
    DATACENTER_POS.2,
);

/// The guns "Black Box" can give, with their ammo; the world seed picks one.
pub const BLACK_BOX_GUNS: [(ItemKind, ItemKind, u32); 4] = [
    (ItemKind::Rifle, ItemKind::BrassCartridge, 12),
    (ItemKind::BurstCarbine, ItemKind::BrassCartridge, 18),
    (ItemKind::SlugThrower, ItemKind::BrassSlug, 8),
    (ItemKind::Scattergun, ItemKind::ScatterShell, 10),
];

/// What SC-M4X7 says once the shard is handed over.
pub const BLACK_BOX_CLOSING_LINE: &str =
    "SC-M4X7: \"A sealed site under the river of fire. Keep looking, user.\"";

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
        "SapienCorp built arms like yours at the facility half a mile north. Find a salvaged one and install it.".to_string(),
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

/// "Black Box", offered by the repaired android once the arm is in.
pub fn black_box(npc_entity: Option<hecs::Entity>) -> ActiveQuest {
    let (x, y, z) = DATACENTER_ENTRANCE;
    let (dx, dy, dz) = DATACENTER_DOOR;
    ActiveQuest::new(
        QuestType::BlackBox,
        "Black Box".to_string(),
        "SapienCorp backed everything up to a datacenter in the ruined city to the west. Recover the reactor's last data and bring it back.".to_string(),
        vec![
            objective(
                "Reach the SapienCorp datacenter",
                QuestObjectiveType::GoToLocation { x, y, z },
            ),
            objective(
                "Restore the datacenter's power",
                QuestObjectiveType::PoweredAt {
                    x: dx,
                    y: dy,
                    z: dz,
                },
            ),
            objective(
                "Recover the data shard from the server hall",
                QuestObjectiveType::FetchItem {
                    item_name: itemkind_name(ItemKind::DataShard).to_string(),
                    quantity: 1,
                    consumed: false,
                },
            ),
            objective(
                "Bring the data shard to SC-M4X7",
                QuestObjectiveType::TalkToNPC {
                    npc_name: "SC-M4X7".to_string(),
                },
            ),
        ],
        npc_entity,
    )
}

/// Start "Black Box" and mark the datacenter on the map.
pub fn start_black_box(res: &mut Resources, npc_entity: Option<hecs::Entity>) {
    res.start_quest(black_box(npc_entity));
    let (x, y, z) = DATACENTER_ENTRANCE;
    res.add_quest_marker(crate::resources::QuestMarker {
        name: DATACENTER_MARKER.to_string(),
        description: "Recover the reactor's data".to_string(),
        x,
        y,
        z,
        marker_type: crate::resources::QuestMarkerType::MainQuest,
    });
    res.log_green("Quest marker added: SapienCorp Datacenter");
}

/// The gun and ammo "Black Box" gives in this world, picked by the world seed alone.
pub fn black_box_gun(seed: u64) -> (ItemKind, ItemKind, u32) {
    use rand::{Rng, SeedableRng};
    let mut rng = rand_chacha::ChaCha20Rng::seed_from_u64(seed ^ 0xB1AC_0B0C);
    BLACK_BOX_GUNS[rng.gen_range(0..BLACK_BOX_GUNS.len())]
}

/// Finish "Black Box" (the shard already handed over): complete it and give the reward, a
/// standard fusion core and the world's gun with its ammo, into the inventory or onto the
/// ground when it's full.
pub fn finish_black_box(world: &mut World, res: &mut Resources) {
    if !res.is_quest_active(QuestType::BlackBox) {
        return;
    }
    if let Some(quest) = res
        .active_quests
        .iter_mut()
        .find(|q| q.quest_type == QuestType::BlackBox)
    {
        for o in &mut quest.objectives {
            o.completed = true;
        }
    }
    res.complete_quest(QuestType::BlackBox);
    let (gun, ammo, n) = black_box_gun(res.world_state.seed);
    let Some(player) = crate::systems::get_player_entity(world) else {
        return;
    };
    let pos = world.get::<&Position>(player).map(|p| *p).ok();
    for (item, qty) in [(ItemKind::StandardFusionCore, 1), (gun, 1), (ammo, n)] {
        let left = world
            .get::<&mut Inventory>(player)
            .map(|mut inv| {
                let before = inv.count(item);
                inv.add(item, qty);
                qty - (inv.count(item) - before)
            })
            .unwrap_or(qty);
        if let (true, Some(pos)) = (left > 0, pos) {
            crate::factory::spawn_drop(world, pos, crate::components::ItemStack::new(item, left));
        }
        res.log_green(format!("Received {qty} {}", itemkind_name(item)));
    }
    res.log_green(BLACK_BOX_CLOSING_LINE);
}

/// Hand the data shard to the android: take it and finish the quest. False, changing
/// nothing, without the shard or the quest.
pub fn hand_in_black_box(world: &mut World, res: &mut Resources) -> bool {
    let Some(player) = crate::systems::get_player_entity(world) else {
        return false;
    };
    if !res.is_quest_active(QuestType::BlackBox) {
        return false;
    }
    let taken = world
        .get::<&mut Inventory>(player)
        .map(|mut inv| inv.remove(ItemKind::DataShard, 1))
        .unwrap_or(0);
    if taken == 0 {
        return false;
    }
    finish_black_box(world, res);
    true
}

fn objective_met(
    objective: &QuestObjectiveType,
    pos: Position,
    inventory: Option<&Inventory>,
    body: Option<&Body>,
    powered: &[Position],
) -> bool {
    match objective {
        QuestObjectiveType::PoweredAt { x, y, z } => powered.contains(&Position {
            x: *x,
            y: *y,
            z: *z,
        }),
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
    // Sealed doors with power now, for "power restored" objectives
    let powered: Vec<Position> = world
        .query::<&Position>()
        .with::<&crate::factory::FactoryBlock>()
        .iter()
        .filter(|&(e, _)| crate::sealed_door::is_open(world, e))
        .map(|(_, p)| *p)
        .collect();

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
                &powered,
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
pub const TP_QUEST_TARGETS: [&str; 4] = ["android", "facility", "locker", "datacenter"];

/// A walkable tile next to `pos` (east, west, south, north), or `pos` itself.
pub(crate) fn beside(res: &mut Resources, pos: (i64, i64, i64)) -> Position {
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
            // The quest chest nearest the facility (the datacenter has one too)
            let (fx, fy, _) = FACILITY_POS;
            let chest = *res
                .quest_chests_placed
                .iter()
                .min_by_key(|(x, y, _)| (x - fx).abs() + (y - fy).abs())
                .ok_or("The arm locker hasn't been placed yet")?;
            Ok(beside(res, chest))
        }
        "datacenter" => Ok(beside(res, DATACENTER_ENTRANCE)),
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
        // Before the repair: the glitchy greeting, with the repair offer and no arm talk
        assert_eq!(game.res.opening_dialogue_node(), DialogueNodeID::Start);
        assert!(!offered(&game, DialogueNodeID::Start, asks));

        // After it: the repaired greeting, which asks about the arm first
        finish_repair_quest(&mut game);
        assert_eq!(game.res.opening_dialogue_node(), DialogueNodeID::Repaired);
        let db = DialogueDatabase::new();
        let repaired = db
            .get_tree("broken_android_tutorial")
            .unwrap()
            .get_node(&DialogueNodeID::Repaired)
            .unwrap();
        assert!(asks(&repaired.choices[0]));
        assert!(repaired
            .choices
            .iter()
            .all(|c| c.unlocks_quest != Some(QuestType::RepairBrokenAndroid)
                && c.leads_to != Some(DialogueNodeID::CanIHelp)));
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

    /// A new world with the datacenter loaded and its blocks placed.
    fn with_datacenter(seed: u64) -> Game {
        let mut game = Game::new(seed);
        let (x, y, z) = DATACENTER_POS;
        for dy in [0, 16] {
            for dx in [0, 22] {
                game.ensure_chunk_with_pending_structures_worldcoords(x + dx, y + dy, z);
            }
        }
        game.place_structure_blocks();
        game
    }

    fn at_offset(dx: i64, dy: i64) -> Position {
        let (x, y, z) = DATACENTER_POS;
        Position {
            x: x + dx,
            y: y + dy,
            z,
        }
    }

    fn black_box_done(game: &Game) -> Vec<bool> {
        game.res
            .get_active_quest(QuestType::BlackBox)
            .unwrap()
            .objectives
            .iter()
            .map(|o| o.completed)
            .collect()
    }

    fn finish_arm_quest(game: &mut Game) {
        finish_repair_quest(game);
        game.res.start_quest(replace_arm(None));
        game.res.complete_quest(QuestType::ReplaceArm);
    }

    #[test]
    fn every_world_has_the_datacenter_in_a_city_with_its_shard_door_and_station() {
        let mut game = with_datacenter(12345);
        let (x, y, _) = DATACENTER_POS;
        assert!(game.res.world_state.world.is_city(x as f64, y as f64));
        let chest = game.res.factory_index[&at_offset(19, 2)];
        let inv = game.world.get::<&Inventory>(chest).unwrap();
        assert_eq!(
            inv.count(ItemKind::DataShard),
            1,
            "the shard waits in its chest"
        );
        drop(inv);
        let door = game.res.factory_index[&at_offset(14, 8)];
        assert_eq!(
            game.world
                .get::<&crate::factory::FactoryBlock>(door)
                .unwrap()
                .kind,
            crate::factory::FactoryKind::SealedDoor
        );
        let station = game.res.factory_index[&at_offset(2, 12)];
        assert_eq!(
            game.world
                .get::<&crate::teleporter::TransitStation>(station)
                .unwrap()
                .name,
            "Datacenter Station"
        );
        // The door starts sealed: the generator is cold
        game.tick();
        assert!(!crate::sealed_door::is_open(&game.world, door));
    }

    #[test]
    fn black_box_is_offered_only_after_the_arm() {
        let mut game = Game::new(12345);
        let asks =
            |c: &crate::dialogue::DialogueChoice| c.leads_to == Some(DialogueNodeID::BlackBoxOffer);
        let accepts =
            |c: &crate::dialogue::DialogueChoice| c.unlocks_quest == Some(QuestType::BlackBox);
        finish_repair_quest(&mut game);
        assert!(!offered(&game, DialogueNodeID::Repaired, asks), "too early");
        game.res.start_quest(replace_arm(None));
        game.res.complete_quest(QuestType::ReplaceArm);
        assert!(offered(&game, DialogueNodeID::Repaired, asks));
        assert!(offered(&game, DialogueNodeID::BlackBoxOffer, accepts));
        start_black_box(&mut game.res, None);
        assert!(!offered(&game, DialogueNodeID::BlackBoxOffer, accepts));
        assert!(game
            .res
            .quest_markers
            .iter()
            .any(|m| m.name == DATACENTER_MARKER));
    }

    #[test]
    fn black_box_runs_in_order_and_pays_out_on_hand_in() {
        let mut game = with_datacenter(12345);
        finish_arm_quest(&mut game);
        start_black_box(&mut game.res, None);
        game.tick();
        assert_eq!(black_box_done(&game), [false, false, false, false]);

        teleport(&mut game, DATACENTER_ENTRANCE);
        game.tick();
        assert_eq!(black_box_done(&game), [true, false, false, false]);

        // Fuel the cold generator: the door opens
        let generator = game.res.factory_index[&at_offset(5, 3)];
        game.world
            .get::<&mut Inventory>(generator)
            .unwrap()
            .add(ItemKind::Biochar, 10);
        for _ in 0..5 {
            game.res.time.tick += crate::factory::FACTORY_STEP_TICKS;
            crate::factory::run_due_steps(&mut game.world, &mut game.res);
        }
        game.tick();
        assert_eq!(black_box_done(&game), [true, true, false, false]);

        // Take the shard
        let player = game.get_player_entity().unwrap();
        game.world
            .get::<&mut Inventory>(player)
            .unwrap()
            .add(ItemKind::DataShard, 1);
        game.tick();
        assert_eq!(black_box_done(&game), [true, true, true, false]);

        // Hand it in
        let core_before = game
            .world
            .get::<&Inventory>(player)
            .unwrap()
            .count(ItemKind::StandardFusionCore);
        assert!(hand_in_black_box(&mut game.world, &mut game.res));
        assert!(game.res.is_quest_completed(QuestType::BlackBox));
        let inv = game.world.get::<&Inventory>(player).unwrap();
        assert_eq!(
            inv.count(ItemKind::DataShard),
            0,
            "the shard was handed over"
        );
        assert_eq!(inv.count(ItemKind::StandardFusionCore), core_before + 1);
        let (gun, ammo, n) = black_box_gun(12345);
        assert_eq!(inv.count(gun), 1);
        assert!(inv.count(ammo) >= n);
        drop(inv);
        assert!(!game
            .res
            .quest_markers
            .iter()
            .any(|m| m.name == DATACENTER_MARKER));
        assert!(
            !hand_in_black_box(&mut game.world, &mut game.res),
            "only once"
        );
    }

    #[test]
    fn steps_done_early_still_count() {
        let mut game = with_datacenter(12345);
        finish_arm_quest(&mut game);
        let player = game.get_player_entity().unwrap();
        game.world
            .get::<&mut Inventory>(player)
            .unwrap()
            .add(ItemKind::DataShard, 1);
        start_black_box(&mut game.res, None);
        game.tick();
        assert_eq!(black_box_done(&game), [true, true, true, false]);
    }

    #[test]
    fn the_reward_gun_follows_the_seed() {
        assert_eq!(black_box_gun(12345), black_box_gun(12345));
        let kinds: std::collections::BTreeSet<ItemKind> =
            (0..40u64).map(|s| black_box_gun(s * 7919).0).collect();
        assert!(kinds.len() > 1, "{kinds:?}");
    }

    #[test]
    fn tp_quest_reaches_the_datacenter() {
        let mut game = Game::new(12345);
        let p = tp_quest_target(&game.world, &mut game.res, Some("datacenter")).unwrap();
        let (x, y, z) = DATACENTER_ENTRANCE;
        assert_eq!(p.z, z);
        assert!((p.x - x).abs() <= 1 && (p.y - y).abs() <= 1);
    }
}

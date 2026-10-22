//! Saving and loading a game. A version 5 save holds every entity with every component
//! (through the registry below, keeping entity IDs), all of [`Resources`] except the config,
//! and an opaque section the client fills with its own play state.

mod v4;

use std::any::TypeId;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use anyhow::{Context, Result};
use hecs::serialize::row::{self, DeserializeContext, SerializeContext};
use hecs::{EntityBuilder, EntityRef, World};
use serde::de::{self, MapAccess};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::components::{
    BattleDelay, BlocksMovement, Combat, Dead, Dialogue, DogAI, DroppedItem, Energy, EntityKind,
    FeralDog, FogOfWar, GameEntity, Glyph, Health, Inventory, LightSource, Player, Position,
    QuestTesty, QuestTutorialBrokenAndroid, Sheep, SpriteRef, Stunned,
};
use crate::model::body::Body;
use crate::moves::ActionQueue;
use crate::resources::Resources;

pub const SAVE_VERSION: u32 = 5;

/// The client's view position, as version 4 saves stored it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportSave {
    pub view_x: i64,
    pub view_y: i64,
    pub view_z: i64,
}

/// Every component type that can be on an entity, with the stable key it is saved under.
/// A component missing here is dropped by saves; `every_component_is_registered` fails.
macro_rules! saved_components {
    ($($key:literal => $ty:ty,)*) => {
        struct SaveEntities;

        impl SerializeContext for SaveEntities {
            fn serialize_entity<S: SerializeMap>(
                &mut self,
                entity: EntityRef<'_>,
                mut map: S,
            ) -> Result<S::Ok, S::Error> {
                $(row::try_serialize::<$ty, _, _>(&entity, $key, &mut map)?;)*
                map.end()
            }
        }

        struct LoadEntities;

        impl DeserializeContext for LoadEntities {
            fn deserialize_entity<'de, M: MapAccess<'de>>(
                &mut self,
                mut map: M,
                entity: &mut EntityBuilder,
            ) -> Result<(), M::Error> {
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        $($key => {
                            entity.add(map.next_value::<$ty>()?);
                        })*
                        other => return Err(de::Error::unknown_field(other, COMPONENT_KEYS)),
                    }
                }
                Ok(())
            }
        }

        const COMPONENT_KEYS: &[&str] = &[$($key),*];

        /// Each saved component's type and key.
        fn saved_component_types() -> Vec<(TypeId, &'static str)> {
            vec![$((TypeId::of::<$ty>(), $key)),*]
        }
    };
}

saved_components! {
    "position" => Position,
    "entity_kind" => EntityKind,
    "game_entity" => GameEntity,
    "player" => Player,
    "sheep" => Sheep,
    "feral_dog" => FeralDog,
    "quest_tutorial_broken_android" => QuestTutorialBrokenAndroid,
    "quest_testy" => QuestTesty,
    "glyph" => Glyph,
    "sprite" => SpriteRef,
    "blocks_movement" => BlocksMovement,
    "health" => Health,
    "energy" => Energy,
    "body" => Body,
    "inventory" => Inventory,
    "dropped_item" => DroppedItem,
    "dialogue" => Dialogue,
    "dog_ai" => DogAI,
    "combat" => Combat,
    "action_queue" => ActionQueue,
    "battle_delay" => BattleDelay,
    "stunned" => Stunned,
    "dead" => Dead,
    "fog_of_war" => FogOfWar,
    "light_source" => LightSource,
    "factory_block" => crate::factory::FactoryBlock,
    "growth" => crate::plants::Growth,
    "belt_slot" => crate::factory::BeltSlot,
    "generator" => crate::power::Generator,
    "power_pole" => crate::power::PowerPole,
    "power_consumer" => crate::power::PowerConsumer,
    "machine" => crate::machines::Machine,
    "furnace" => crate::furnace::Furnace,
    "extractor" => crate::logistics::Extractor,
    "router" => crate::logistics::Router,
}

/// Entities that carry a component type the registry doesn't know, described by the
/// registered components they do have. Empty when every component would be saved.
pub fn unsaved_components(world: &World) -> Vec<String> {
    let registered = saved_component_types();
    let key_of = |id: TypeId| registered.iter().find(|(t, _)| *t == id).map(|(_, k)| *k);
    world
        .iter()
        .filter_map(|entity| {
            let types: Vec<TypeId> = entity.component_types().collect();
            let unknown = types.iter().filter(|t| key_of(**t).is_none()).count();
            (unknown > 0).then(|| {
                let known: Vec<&str> = types.iter().filter_map(|t| key_of(*t)).collect();
                format!(
                    "entity {:?} with [{}] has {unknown} component type(s) not registered \
                     in saved_components! (save_load/mod.rs)",
                    entity.entity(),
                    known.join(", ")
                )
            })
        })
        .collect()
}

fn serialize_entities<S: Serializer>(world: &&World, serializer: S) -> Result<S::Ok, S::Error> {
    row::serialize(world, &mut SaveEntities, serializer)
}

fn deserialize_entities<'de, D: Deserializer<'de>>(deserializer: D) -> Result<World, D::Error> {
    row::deserialize(&mut LoadEntities, deserializer)
}

#[derive(Serialize)]
struct SaveRef<'a> {
    version: u32,
    resources: &'a Resources,
    #[serde(serialize_with = "serialize_entities")]
    entities: &'a World,
    client: &'a serde_json::Value,
}

#[derive(Deserialize)]
struct SaveOwned {
    version: u32,
    resources: Resources,
    #[serde(deserialize_with = "deserialize_entities")]
    entities: World,
    #[serde(default)]
    client: serde_json::Value,
}

impl SaveOwned {
    fn apply(self, game: &mut crate::Game) -> Result<LoadedSave> {
        if self.version != SAVE_VERSION {
            anyhow::bail!("Unsupported save version: {}", self.version);
        }
        game.world = self.entities;
        game.res = self.resources;
        game.res.factory_index = crate::factory::rebuild_index(&game.world);
        Ok(LoadedSave {
            client: self.client,
            viewport: None,
        })
    }
}

/// What a load hands back to the client.
#[derive(Debug)]
pub struct LoadedSave {
    /// The client section, as given to the save (`Null` if there was none).
    pub client: serde_json::Value,
    /// The view position of a version 4 save, which has no client section.
    pub viewport: Option<ViewportSave>,
}

fn save_ref<'a>(game: &'a crate::Game, client: &'a serde_json::Value) -> SaveRef<'a> {
    SaveRef {
        version: SAVE_VERSION,
        resources: &game.res,
        entities: &game.world,
        client,
    }
}

/// The game as JSON text, with `client` as the client section.
pub fn save_to_string(game: &crate::Game, client: &serde_json::Value) -> Result<String> {
    serde_json::to_string(&save_ref(game, client)).context("serialize json")
}

/// Replace `game` with a JSON save (version 5, or version 4 with its old gaps).
pub fn load_from_str(game: &mut crate::Game, json: &str) -> Result<LoadedSave> {
    #[derive(Deserialize)]
    struct Version {
        version: u32,
    }
    let Version { version } = serde_json::from_str(json).context("read save version")?;
    match version {
        SAVE_VERSION => serde_json::from_str::<SaveOwned>(json)
            .context("deserialize json")?
            .apply(game),
        v4::VERSION => {
            let viewport = v4::load_json_str(game, json)?;
            game.res.log("Save upgraded from version 4");
            Ok(LoadedSave {
                client: serde_json::Value::Null,
                viewport: Some(viewport),
            })
        }
        other => anyhow::bail!(
            "Unsupported save version: {other} (expected {SAVE_VERSION} or {})",
            v4::VERSION
        ),
    }
}

pub fn save_to_file<P: AsRef<Path>>(
    game: &crate::Game,
    path: P,
    client: &serde_json::Value,
) -> Result<()> {
    let path = path.as_ref();
    let writer = BufWriter::new(File::create(path).with_context(|| format!("create {path:?}"))?);
    serde_json::to_writer_pretty(writer, &save_ref(game, client)).context("serialize json")
}

pub fn load_from_file<P: AsRef<Path>>(game: &mut crate::Game, path: P) -> Result<LoadedSave> {
    let path = path.as_ref();
    let json = std::fs::read_to_string(path).with_context(|| format!("read {path:?}"))?;
    load_from_str(game, &json)
}

// MessagePack (rmp-serde): version 5 only, no client section
pub fn save_game_msgpack<P: AsRef<Path>>(game: &crate::Game, path: P) -> Result<()> {
    let path = path.as_ref();
    let mut writer =
        BufWriter::new(File::create(path).with_context(|| format!("create {path:?}"))?);
    rmp_serde::encode::write_named(&mut writer, &save_ref(game, &serde_json::Value::Null))
        .context("serialize msgpack")
}

pub fn load_game_msgpack<P: AsRef<Path>>(game: &mut crate::Game, path: P) -> Result<()> {
    let path = path.as_ref();
    let reader = BufReader::new(File::open(path).with_context(|| format!("open {path:?}"))?);
    let save: SaveOwned = rmp_serde::decode::from_read(reader).context("deserialize msgpack")?;
    save.apply(game).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TestWorld;
    use crate::tiles::TileKind;
    use crate::Game;

    fn reload(game: &mut Game) {
        let saved = save_to_string(game, &serde_json::Value::Null).expect("save");
        load_from_str(game, &saved).expect("load");
    }

    /// Everything a save holds, as JSON: equal before and after a reload if nothing is lost.
    fn snapshot(game: &Game) -> serde_json::Value {
        serde_json::to_value(save_ref(game, &serde_json::Value::Null)).unwrap()
    }

    /// Paths where two JSON values differ, e.g. `entities.4294967296.health.current`.
    fn differences(
        a: &serde_json::Value,
        b: &serde_json::Value,
        path: &str,
        out: &mut Vec<String>,
    ) {
        use serde_json::Value;
        match (a, b) {
            (Value::Object(x), Value::Object(y)) => {
                for key in x.keys().chain(y.keys().filter(|k| !x.contains_key(*k))) {
                    let null = Value::Null;
                    let sub = format!("{path}.{key}");
                    differences(
                        x.get(key).unwrap_or(&null),
                        y.get(key).unwrap_or(&null),
                        &sub,
                        out,
                    );
                }
            }
            (Value::Array(x), Value::Array(y)) if x.len() == y.len() => {
                for (i, (u, v)) in x.iter().zip(y).enumerate() {
                    differences(u, v, &format!("{path}[{i}]"), out);
                }
            }
            _ if a != b => {
                let short = |v: &Value| v.to_string().chars().take(120).collect::<String>();
                out.push(format!("{path}: {} != {}", short(a), short(b)));
            }
            _ => {}
        }
    }

    fn assert_same(before: &serde_json::Value, after: &serde_json::Value) {
        let mut out = Vec::new();
        differences(before, after, "save", &mut out);
        assert!(
            out.is_empty(),
            "{} differences:\n{}",
            out.len(),
            out[..out.len().min(20)].join("\n")
        );
    }

    #[test]
    fn reload_keeps_every_entity_and_resource() {
        let mut game = Game::new(12345);
        for _ in 0..5 {
            game.queue_player_move(1, 0);
            game.tick();
        }
        let before = snapshot(&game);
        reload(&mut game);
        assert_same(&before, &snapshot(&game));
    }

    #[test]
    fn factory_blocks_survive_a_reload_with_their_index() {
        use crate::factory::{place, Facing, FactoryBlock, FactoryKind};
        let mut game = TestWorld::new(1).build().game;
        let pos = Position { x: 2, y: 0, z: 0 };
        let chest = place(
            &mut game.world,
            &mut game.res,
            FactoryKind::Chest,
            pos,
            Facing::West,
        )
        .unwrap();
        game.res.player_state.intent = crate::intent::PlayerIntent::movement(0, 0, 0, 130);
        game.tick();
        let clock = game.res.factory_clock.clone();
        let before = snapshot(&game);

        reload(&mut game);
        assert_same(&before, &snapshot(&game));
        assert_eq!(
            game.res.factory_index.get(&pos),
            Some(&chest),
            "index rebuilt"
        );
        assert_eq!(
            game.world.get::<&FactoryBlock>(chest).unwrap().facing,
            Facing::West
        );
        assert_eq!(game.res.factory_clock, clock);
    }

    #[test]
    fn msgpack_round_trip_keeps_everything() {
        let mut game = Game::new(999);
        game.tick();
        let before = snapshot(&game);
        let path = std::env::temp_dir().join(format!("lr-save-{}.msgpack", std::process::id()));
        save_game_msgpack(&game, &path).unwrap();
        let mut loaded = Game::new(0);
        load_game_msgpack(&mut loaded, &path).unwrap();
        let _ = std::fs::remove_file(&path);
        assert_same(&before, &snapshot(&loaded));
    }

    #[test]
    fn client_section_comes_back_unchanged() {
        let mut game = TestWorld::new(1).build().game;
        let client = serde_json::json!({"hotbar": [null, "Log"], "step": 3});
        let saved = save_to_string(&game, &client).unwrap();
        let loaded = load_from_str(&mut game, &saved).unwrap();
        assert_eq!(loaded.client, client);
        assert!(loaded.viewport.is_none());
    }

    #[test]
    fn version_4_save_still_loads_and_saves_as_version_5() {
        let mut game = Game::new(0);
        let loaded = load_from_str(&mut game, include_str!("../testdata/save_v4.json")).unwrap();

        let vp = loaded.viewport.expect("version 4 saves carry the viewport");
        assert_eq!((vp.view_x, vp.view_y, vp.view_z), (2, 3, 0));
        assert_eq!(game.res.world_state.seed, 4);
        assert_eq!(
            game.get_player_position(),
            Some(Position { x: 2, y: 2, z: 0 })
        );
        let kinds = |game: &Game| {
            let mut kinds: Vec<String> = game
                .world
                .query::<&Position>()
                .iter()
                .map(|(e, p)| {
                    let sheep = game.world.get::<&Sheep>(e).is_ok();
                    let dog = game.world.get::<&FeralDog>(e).is_ok();
                    let npc = game.world.get::<&Dialogue>(e).is_ok();
                    let item = game.world.get::<&DroppedItem>(e).map(|d| d.qty).ok();
                    format!("{sheep} {dog} {npc} {item:?} {},{}", p.x, p.y)
                })
                .collect();
            kinds.sort();
            kinds
        };
        assert_eq!(kinds(&game).len(), 5, "player, sheep, dog, npc, item");
        let log = &game.res.events.get_message_log().messages;
        assert!(log
            .last()
            .unwrap()
            .text
            .contains("Save upgraded from version 4"));

        let saved = save_to_string(&game, &serde_json::Value::Null).unwrap();
        assert!(saved.starts_with(r#"{"version":5,"#));
        let before = kinds(&game);
        load_from_str(&mut game, &saved).unwrap();
        assert_eq!(kinds(&game), before);
    }

    #[test]
    fn unknown_versions_are_refused() {
        let err = load_from_str(&mut Game::new(0), r#"{"version": 9}"#).unwrap_err();
        assert!(err.to_string().contains("9"), "{err}");
    }

    /// Worlds built through every spawn path: the starting world after some play, every
    /// creature kind, a fight that leaves a corpse, and combat-only components.
    fn worlds_from_every_spawn_path() -> Vec<Game> {
        let mut started = Game::new(12345);
        for _ in 0..20 {
            started.tick();
        }

        let mut world = TestWorld::new(7).fill((-6, -6, 0), (6, 6, 0), TileKind::Grass);
        world.sheep(-3, -3, 0);
        let dog = world.feral_dog(1, 0, 0);
        world.feral_dog(4, 4, 0);
        world.npc(-4, 3, 0);
        world.item(2, 2, 0, crate::components::ItemKind::Log, 1);
        let mut fight = world.build();
        let chest = crate::factory::place(
            &mut fight.game.world,
            &mut fight.game.res,
            crate::factory::FactoryKind::Chest,
            Position { x: -2, y: -2, z: 0 },
            crate::factory::Facing::East,
        )
        .unwrap();
        fight
            .game
            .world
            .get::<&mut Inventory>(chest)
            .unwrap()
            .add(crate::components::ItemKind::Log, 3);
        crate::factory::place(
            &mut fight.game.world,
            &mut fight.game.res,
            crate::factory::FactoryKind::Belt,
            Position { x: -2, y: 2, z: 0 },
            crate::factory::Facing::South,
        )
        .unwrap();
        for (kind, pos) in [
            (
                crate::factory::FactoryKind::PowerPole,
                Position { x: 4, y: -4, z: 0 },
            ),
            (
                crate::factory::FactoryKind::BurnerGenerator,
                Position { x: 5, y: -4, z: 0 },
            ),
            (
                crate::factory::FactoryKind::TestConsumer,
                Position { x: 3, y: -4, z: 0 },
            ),
            (
                crate::factory::FactoryKind::AutoPlanter,
                Position { x: 2, y: -4, z: 0 },
            ),
            (
                crate::factory::FactoryKind::AutoHarvester,
                Position { x: 6, y: -4, z: 0 },
            ),
            (
                crate::factory::FactoryKind::CrudeFurnace,
                Position { x: 7, y: -4, z: 0 },
            ),
            (
                crate::factory::FactoryKind::FilterExtractor,
                Position { x: 8, y: -4, z: 0 },
            ),
            (
                crate::factory::FactoryKind::BeltRouter,
                Position { x: 9, y: -4, z: 0 },
            ),
            (
                crate::factory::FactoryKind::VerticalConveyorUp,
                Position { x: 10, y: -4, z: 0 },
            ),
        ] {
            crate::factory::place(
                &mut fight.game.world,
                &mut fight.game.res,
                kind,
                pos,
                crate::factory::Facing::North,
            )
            .unwrap();
        }
        crate::plants::plant(
            &mut fight.game.world,
            &mut fight.game.res,
            Position { x: 3, y: -3, z: 0 },
        )
        .unwrap();
        fight.ticks(30);
        let dog = fight.entity(dog);
        crate::systems::handle_entity_death(&mut fight.game.world, &mut fight.game.res, dog);
        let player = fight.game.get_player_entity().unwrap();
        fight
            .game
            .world
            .insert(player, (Stunned::new(5), BattleDelay::new(10), Dead))
            .unwrap();
        vec![started, fight.game]
    }

    #[test]
    fn every_component_is_registered() {
        for game in worlds_from_every_spawn_path() {
            assert_eq!(unsaved_components(&game.world), Vec::<String>::new());
        }
    }

    #[test]
    fn an_unregistered_component_is_reported() {
        struct NotRegistered;
        let mut game = TestWorld::new(1).build().game;
        let player = game.get_player_entity().unwrap();
        game.world.insert_one(player, NotRegistered).unwrap();
        let report = unsaved_components(&game.world);
        assert_eq!(report.len(), 1);
        assert!(
            report[0].contains("player") && report[0].contains("1 component type"),
            "{report:?}"
        );
    }
}

#[cfg(test)]
mod scenario_tests {
    //! State that a save must keep, one scenario per spec requirement.
    use crate::components::{EntityKind, Health, Inventory, ItemKind, Player, Position};
    use crate::model::body::{Body, BodyPartState, BodyPartType};
    use crate::test_support::TestWorld;
    use crate::tiles::TileKind;
    use crate::Game;

    fn reload(game: &mut Game) {
        let saved = super::save_to_string(game, &serde_json::Value::Null).expect("save");
        super::load_from_str(game, &saved).expect("load");
    }

    fn arena() -> TestWorld {
        TestWorld::new(7).fill((-6, -6, 0), (6, 6, 0), TileKind::Grass)
    }

    fn player_body(game: &Game) -> Body {
        game.get_player_component::<Body>()
            .expect("player has a body")
    }

    fn items(inv: &Inventory) -> Vec<(ItemKind, u32)> {
        inv.slots.iter().map(|s| (s.kind, s.qty)).collect()
    }

    #[test]
    fn damaged_body_survives_a_reload() {
        let mut game = arena().build().game;
        {
            let mut body = game.get_player_component_mut::<Body>().unwrap();
            let leg = body.parts.get_mut(&BodyPartType::RightLeg).unwrap();
            leg.integrity = 30;
            leg.update_state_from_integrity();
            let arm = body.parts.get_mut(&BodyPartType::LeftArm).unwrap();
            arm.integrity = 0;
            arm.update_state_from_integrity();
        }
        let before = player_body(&game);

        reload(&mut game);
        let after = player_body(&game);
        let leg = &after.parts[&BodyPartType::RightLeg];
        assert_eq!(leg.integrity, 30);
        assert_eq!(leg.state, BodyPartState::Damaged);
        assert_eq!(
            after.parts[&BodyPartType::LeftArm].state,
            BodyPartState::Missing
        );
        assert_eq!(after.walk_speed_modifier(), before.walk_speed_modifier());
    }

    #[test]
    fn corpse_with_loot_survives_a_reload() {
        // Game::new kills a feral dog carrying leather and meat next to the android
        let mut game = Game::new(12345);
        let corpses = |game: &Game| -> Vec<(Position, Vec<(ItemKind, u32)>)> {
            game.world
                .query::<(&Position, &EntityKind, &Inventory)>()
                .iter()
                .filter(|(_, (_, kind, _))| **kind == EntityKind::Corpse)
                .map(|(_, (pos, _, inv))| (*pos, items(inv)))
                .collect()
        };
        let before = corpses(&game);
        assert_eq!(before.len(), 1);
        assert!(before[0].1.contains(&(ItemKind::Leather, 1)));

        reload(&mut game);
        assert_eq!(corpses(&game), before);
    }

    #[test]
    fn creatures_keep_their_state() {
        let mut world = arena();
        let sheep = world.sheep(-3, -3, 0);
        let dog = world.feral_dog(4, 4, 0);
        let mut test = world.build();
        let sheep_e = test.entity(sheep);
        test.game
            .world
            .get::<&mut Health>(sheep_e)
            .unwrap()
            .damage(20);
        let dog_e = test.entity(dog);
        let dog_inv = items(&test.game.world.get::<&Inventory>(dog_e).unwrap());
        assert!(!dog_inv.is_empty());

        reload(&mut test.game);
        let health = |e| test.game.world.get::<&Health>(e).map(|h| h.current).ok();
        assert_eq!(health(sheep_e), Some(30), "sheep keeps its wounds");
        assert_eq!(health(dog_e), Some(80));
        let inv = test
            .game
            .world
            .get::<&Inventory>(dog_e)
            .map(|inv| items(&inv))
            .ok();
        assert_eq!(inv, Some(dog_inv), "dog keeps its loot");
    }

    #[test]
    fn toggles_and_message_log_survive_a_reload() {
        let mut game = arena().build().game;
        game.res.player_state.noclip_enabled = true;
        game.res.player_state.fog_of_war_enabled = false;
        game.res.log("first");
        game.res.log("second");
        let log = |game: &Game| -> Vec<String> {
            game.res
                .events
                .get_message_log()
                .messages
                .iter()
                .map(|m| m.text.clone())
                .collect()
        };
        let before = log(&game);

        reload(&mut game);
        assert!(game.res.player_state.noclip_enabled);
        assert!(!game.res.player_state.fog_of_war_enabled);
        assert_eq!(log(&game), before);
        assert_eq!(
            game.world.query::<&Player>().iter().count(),
            1,
            "still exactly one player"
        );
    }
}

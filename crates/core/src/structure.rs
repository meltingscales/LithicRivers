use crate::world::GameWorld;
use hecs::World as ECSWorld;
use rust_embed::RustEmbed;
use serde::Deserialize;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::spawn_utils;
use crate::tiles::TileKind;

#[derive(Debug, Clone, Deserialize)]
pub struct StructureDefinition {
    pub name: String,
    pub blocks: HashMap<String, String>,
    pub layers: Vec<String>,
    pub gen_biomes: String,
    pub gen_chance: f32,
    pub y_layer_gen_range: Vec<i64>,
}

#[derive(RustEmbed)]
#[folder = "../client/assets/"]
struct EmbeddedAssets;

pub fn structures_list() -> Vec<String> {
    let existing_structures = [
        "small_ship.lrstructure".to_string(),
        "small_temple.lrstructure".to_string(),
        "giant_corpse.lrstructure".to_string(),
        "starter_ship.lrstructure".to_string(),
        "sapiencorp-factory.lrstructure".to_string(),
    ];

    existing_structures.to_vec()
}

/// Should this block spawn an entity?
pub fn block_will_spawn_entity(tile_kind: TileKind) -> bool {
    match tile_kind {
        TileKind::ExistingWorldgen => false,
        TileKind::TreasureCommon => true,
        TileKind::TreasureRare => true,
        TileKind::EnemySpawn => true,
        TileKind::ScrapCommon => true,
        TileKind::ScrapRare => true,
        TileKind::TreasureQuest1 => true,
        _ => false,
    }
}

/// Spawn an entity for a block in a structure.
/// For example, a rare treasure block will turn into a rare item.
pub fn spawn_entity_for_block(
    game_world: &mut GameWorld,
    ecs_world: &mut ECSWorld,
    _structure_name: &str,
    tile_kind: TileKind,
    block_x: i64,
    block_y: i64,
    block_z: i64,
) {
    if !block_will_spawn_entity(tile_kind) {
        return;
    }

    game_world.set_tile_cached(block_x, block_y, block_z, TileKind::Air);

    match tile_kind {
        TileKind::EnemySpawn => {
            //by default, enemyspawn just spawns a feral dog.
            //in the future, we can add more options here
            spawn_utils::world_spawn_feraldog(ecs_world, block_x, block_y, block_z, game_world);
        }
        TileKind::TreasureCommon => {
            spawn_utils::world_spawn_treasurecommon(
                ecs_world, block_x, block_y, block_z, game_world,
            );
        }
        TileKind::TreasureRare => {
            spawn_utils::world_spawn_treasurerare(ecs_world, block_x, block_y, block_z, game_world);
        }
        TileKind::ScrapCommon => {
            spawn_utils::world_spawn_scrapcommon(ecs_world, block_x, block_y, block_z, game_world);
        }
        TileKind::ScrapRare => {
            spawn_utils::world_spawn_scraprare(ecs_world, block_x, block_y, block_z, game_world);
        }
        TileKind::TreasureQuest1 => {
            // The game turns it into the quest chest, once per world
            game_world.quest_treasures.push((block_x, block_y, block_z));
        }
        _ => {
            panic!("Unknown tile kind for entity spawning {:?}", tile_kind);
        }
    }
}

impl StructureDefinition {
    /// Where the structure's tiles of one kind land when it is placed with its top-left
    /// corner at (x, y) and, buried, its top layer at z (else its bottom layer at z).
    pub fn tile_positions(
        &self,
        tile: TileKind,
        x: i64,
        y: i64,
        z: i64,
        bury: bool,
    ) -> Vec<(i64, i64, i64)> {
        let top = self.layers.len() as i64 - 1;
        let mut found = Vec::new();
        for (li, layer) in self.layers.iter().enumerate() {
            let tz = if bury {
                z - (top - li as i64)
            } else {
                z + li as i64
            };
            for (ly, line) in layer.lines().enumerate() {
                for (lx, ch) in line.chars().enumerate() {
                    if self.get_tile_for_symbol(&ch.to_string()) == Some(tile) {
                        found.push((x + lx as i64, y + ly as i64, tz));
                    }
                }
            }
        }
        found
    }

    /// Load from embedded assets (panics on failure). `structure_name` is the
    /// folder name such as "starter_ship.lrstructure" under assets/structures/.
    pub fn load_from_embedded(structure_name: &str) -> Self {
        let base = format!("structures/{}/", structure_name);

        let data_path = format!("{}data.json", base);
        let shape_path = format!("{}shape_layers.txt", base);

        let data_file = EmbeddedAssets::get(&data_path)
            .unwrap_or_else(|| panic!("Missing embedded {}", data_path));
        let shape_file = EmbeddedAssets::get(&shape_path)
            .unwrap_or_else(|| panic!("Missing embedded {}", shape_path));

        let data: serde_json::Value = serde_json::from_slice(match data_file.data {
            Cow::Borrowed(b) => b,
            Cow::Owned(ref v) => v.as_slice(),
        })
        .expect("Failed to parse embedded data.json");

        let shape_content = match shape_file.data {
            Cow::Borrowed(b) => String::from_utf8(b.to_vec()).expect("shape_layers not UTF-8"),
            Cow::Owned(v) => String::from_utf8(v).expect("shape_layers not UTF-8"),
        };

        let layers: Vec<String> = shape_content
            .split("~~~~~")
            .map(|layer| layer.trim().to_string())
            .filter(|layer| !layer.is_empty())
            .collect();

        let blocks: HashMap<String, String> =
            serde_json::from_value(data["blocks"].clone()).expect("blocks must be a map");
        for (symbol, tile_str) in &blocks {
            if TileKind::from_key(tile_str).is_none() {
                panic!(
                    "Unknown tile kind '{}' for symbol '{}' in structure '{}'. Valid: {}",
                    tile_str,
                    symbol,
                    structure_name,
                    crate::tiles::TILE_KIND_STRS.join(", ")
                );
            }
        }

        Self {
            name: structure_name.replace(".lrstructure", ""),
            blocks,
            layers,
            gen_biomes: data["gen_biomes"].as_str().unwrap_or("").to_string(),
            gen_chance: data["gen_chance"].as_f64().unwrap_or(1.0) as f32,
            y_layer_gen_range: data["y_layer_gen_range"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .map(|v| v.as_i64().unwrap_or(0))
                .collect(),
        }
    }
    pub fn load_from_directory<P: AsRef<Path>>(structure_dir: P) -> Self {
        let structure_dir = structure_dir.as_ref();
        let name = structure_dir
            .file_name()
            .expect("structure_dir has no filename")
            .to_string_lossy()
            .replace(".lrstructure", "");

        // Load data.json
        let data_file = structure_dir.join("data.json");
        if !data_file.exists() {
            panic!(
                "Missing data.json for structure '{}': {}",
                name,
                data_file.display()
            );
        }
        let data: serde_json::Value =
            serde_json::from_reader(fs::File::open(&data_file).expect("Failed to open data.json"))
                .expect("Failed to parse data.json");

        // Load shape_layers.txt
        let shape_file = structure_dir.join("shape_layers.txt");
        if !shape_file.exists() {
            panic!(
                "Missing shape_layers.txt for structure '{}': {}",
                name,
                shape_file.display()
            );
        }
        let shape_content =
            fs::read_to_string(&shape_file).expect("Failed to read shape_layers.txt");
        let layers: Vec<String> = shape_content
            .split("~~~~~")
            .map(|layer| layer.trim().to_string())
            .filter(|layer| !layer.is_empty())
            .collect();

        let blocks: HashMap<String, String> =
            serde_json::from_value(data["blocks"].clone()).expect("blocks must be a map");
        // Validate all tile kinds
        for (symbol, tile_str) in &blocks {
            if TileKind::from_key(tile_str).is_none() {
                panic!(
                    "Unknown tile kind '{}' for symbol '{}' in structure '{}'. Valid: {}",
                    tile_str,
                    symbol,
                    name,
                    crate::tiles::TILE_KIND_STRS.join(", ")
                );
            }
        }
        Self {
            name,
            blocks,
            layers,
            gen_biomes: data["gen_biomes"].as_str().unwrap_or("").to_string(),
            gen_chance: data["gen_chance"].as_f64().unwrap_or(1.0) as f32,
            y_layer_gen_range: data["y_layer_gen_range"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .map(|v| v.as_i64().unwrap_or(0))
                .collect(),
        }
    }

    pub fn get_tile_for_symbol(&self, symbol: &str) -> Option<TileKind> {
        self.blocks
            .get(symbol)
            .and_then(|tile_str| TileKind::from_key(tile_str))
    }
}

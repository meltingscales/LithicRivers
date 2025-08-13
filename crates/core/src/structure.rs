use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::tiles::TileKind;

#[derive(Debug, Clone, Deserialize)]
pub struct StructureDefinition {
    pub name: String,
    pub blocks: HashMap<String, String>,
    pub layers: Vec<String>,
    pub gen_biomes: String,
    pub gen_chance: f32,
    pub y_layer_gen_range: Vec<i32>,
}

impl StructureDefinition {
    pub fn load_from_directory<P: AsRef<Path>>(structure_dir: P) -> Self {
        let structure_dir = structure_dir.as_ref();
        let name = structure_dir.file_name()
            .expect("structure_dir has no filename")
            .to_string_lossy()
            .replace(".lrstructure", "");

        // Load data.json
        let data_file = structure_dir.join("data.json");
        if !data_file.exists() {
            panic!("Missing data.json for structure '{}': {}", name, data_file.display());
        }
        let data: serde_json::Value = serde_json::from_reader(
            fs::File::open(&data_file).expect("Failed to open data.json")
        ).expect("Failed to parse data.json");

        // Load shape_layers.txt
        let shape_file = structure_dir.join("shape_layers.txt");
        if !shape_file.exists() {
            panic!("Missing shape_layers.txt for structure '{}': {}", name, shape_file.display());
        }
        let shape_content = fs::read_to_string(&shape_file).expect("Failed to read shape_layers.txt");
        let layers: Vec<String> = shape_content.split("~~~~~")
            .map(|layer| layer.trim().to_string())
            .filter(|layer| !layer.is_empty())
            .collect();

        let blocks: HashMap<String, String> = serde_json::from_value(data["blocks"].clone()).expect("blocks must be a map");
        // Validate all tile kinds
        for (symbol, tile_str) in &blocks {
            if TileKind::from_str(tile_str).is_none() {
                panic!("Unknown tile kind '{}' for symbol '{}' in structure '{}'. Valid: {}", tile_str, symbol, name, crate::tiles::TILE_KIND_STRS.join(", "));
            }
        }
        Self {
            name,
            blocks,
            layers,
            gen_biomes: data["gen_biomes"].as_str().unwrap_or("").to_string(),
            gen_chance: data["gen_chance"].as_f64().unwrap_or(1.0) as f32,
            y_layer_gen_range: data["y_layer_gen_range"].as_array().unwrap_or(&vec![])
                .iter().map(|v| v.as_i64().unwrap_or(0) as i32).collect(),
        }
    }

    pub fn get_tile_for_symbol(&self, symbol: &str) -> Option<TileKind> {
        self.blocks.get(symbol).and_then(|tile_str| TileKind::from_str(tile_str))
    }
}

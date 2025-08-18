use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use lithicrivers_core::tiles::TileKind;
use ratatui::prelude::Color;
// All color data must come from data.json in each .lrsprite. No hardcoded fallbacks.

#[derive(Debug, Clone, Deserialize)]
pub struct SpriteMetadata {
    pub name: String,
    pub color: String,
    pub description: String,
    pub scales: Option<Vec<u32>>, // Not strictly needed for loading, but present in JSON
}

#[derive(Debug, Clone)]
pub struct SpriteData {
    pub name: String,
    pub color: String,
    pub description: String,
    pub sprites: Vec<String>, // Each entry is a sprite at a different scale
}

pub struct SpriteLoader {
    pub data_path: PathBuf,
    sprite_cache: HashMap<String, SpriteData>,
}

unsafe impl Send for SpriteLoader {}
unsafe impl Sync for SpriteLoader {}

impl SpriteLoader {
    pub fn new(data_path: Option<&Path>) -> Self {
        let default = PathBuf::from("crates/client/assets/sprites");
        Self {
            data_path: data_path.unwrap_or(default.as_path()).to_path_buf(),
            sprite_cache: HashMap::new(),
        }
    }

    pub fn load_sprite(&mut self, sprite_name: &str, category: &str) -> &SpriteData {
        let cache_key = format!("{}/{}", category, sprite_name);
        if self.sprite_cache.contains_key(&cache_key) {
            return self.sprite_cache.get(&cache_key).unwrap();
        }
        let sprite_path = self
            .data_path
            .join(category)
            .join(format!("{}.lrsprite", sprite_name));
        let data_file = sprite_path.join("data.json");
        let sprites_file = sprite_path.join("sprites.txt");
        // Check for missing files and throw a clear error
        if !data_file.exists() {
            panic!(
                "Missing data.json for sprite '{}' in category '{}': {}",
                sprite_name,
                category,
                data_file.display()
            );
        }
        if !sprites_file.exists() {
            panic!(
                "Missing sprites.txt for sprite '{}' in category '{}': {}",
                sprite_name,
                category,
                sprites_file.display()
            );
        }
        // Load metadata
        let metadata: SpriteMetadata = {
            let file = fs::File::open(&data_file).expect("Failed to open data.json");
            serde_json::from_reader(file).expect("Failed to parse data.json")
        };
        // Load sprite lines
        let content = fs::read_to_string(&sprites_file).expect("Failed to read sprites.txt");
        let lines: Vec<&str> = content.lines().collect();
        // Group lines into sprites by scale (same as Python logic)
        let sprites = if lines.len() >= 6 {
            vec![
                lines[0].to_string(),
                lines[1..3].join("\n"),
                lines[3..6].join("\n"),
            ]
        } else {
            lines.iter().map(|l| l.to_string()).collect()
        };
        // Validate sprites
        Self::validate_sprite_dimensions(&sprites, sprite_name, category);
        let sprite_data = SpriteData {
            name: metadata.name,
            color: metadata.color,
            description: metadata.description,
            sprites,
        };
        self.sprite_cache.insert(cache_key.clone(), sprite_data);
        self.sprite_cache.get(&cache_key).unwrap()
    }

    // Discover and preload all sprites under data_path, scanning categories and *.lrsprite folders.
    pub fn preload_all(&mut self) {
        if !self.data_path.exists() {
            panic!(
                "Sprite assets path does not exist: {}",
                self.data_path.display()
            );
        }
        let Ok(categories) = fs::read_dir(&self.data_path) else { return };
        for cat_entry in categories.flatten() {
            let cat_path = cat_entry.path();
            if !cat_path.is_dir() {
                continue;
            }
            let category = match cat_path.file_name().and_then(|s| s.to_str()) {
                Some(name) => name.to_string(),
                None => continue,
            };
            self.preload_category(&category);
        }
    }

    // Preload a single category by loading all <name>.lrsprite directories within it.
    pub fn preload_category(&mut self, category: &str) {
        let cat_dir = self.data_path.join(category);
        let Ok(entries) = fs::read_dir(&cat_dir) else { return };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let Some(fname) = path.file_name().and_then(|s| s.to_str()) else { continue };
            if let Some(name) = fname.strip_suffix(".lrsprite") {
                // Will fill cache or validate
                let _ = self.load_sprite(name, category);
            }
        }
    }

    fn validate_sprite_dimensions(sprites: &Vec<String>, sprite_name: &str, category: &str) {
        for (i, sprite) in sprites.iter().enumerate() {
            if sprite.is_empty() {
                panic!(
                    "Sprite '{}' in '{}' category has empty sprite at scale {}",
                    sprite_name,
                    category,
                    i + 1
                );
            }
            let lines: Vec<&str> = sprite.split('\n').collect();
            if lines.is_empty() {
                panic!(
                    "Sprite '{}' in '{}' category has no lines at scale {}",
                    sprite_name,
                    category,
                    i + 1
                );
            }
            let line_lengths: Vec<usize> = lines.iter().map(|l| l.len()).collect();
            if line_lengths.iter().any(|&len| len != line_lengths[0]) {
                panic!(
                    "Sprite '{}' in '{}' category has inconsistent line lengths at scale {}. All lines must have the same width for square sprites.",
                    sprite_name, category, i + 1
                );
            }
            let width = line_lengths[0];
            let height = lines.len();
            if width != height {
                panic!(
                    "Sprite '{}' in '{}' category is not square at scale {}. Width: {}, Height: {}. Sprites must be square.",
                    sprite_name, category, i + 1, width, height
                );
            }
        }
    }
}

fn first_sprite_char(sd: &SpriteData) -> char {
    if let Some(first) = sd.sprites.first() {
        first.chars().next().unwrap_or(' ')
    } else {
        ' '
    }
}

fn parse_color_string(s: &str) -> Option<Color> {
    // Support #RRGGBB
    let s = s.trim();
    if let Some(hex) = s.strip_prefix('#') {
        if hex.len() == 6 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&hex[0..2], 16),
                u8::from_str_radix(&hex[2..4], 16),
                u8::from_str_radix(&hex[4..6], 16),
            ) {
                return Some(Color::Rgb(r, g, b));
            }
        }
    }
    None
}

pub fn sprite_for_tile(loader: &mut SpriteLoader, kind: TileKind) -> Option<(char, Color)> {
    let (category, name) = ("tiles",
        match kind {
            TileKind::Rock => "rock",
            TileKind::Dirt => "dirt",
            TileKind::Grass => "grass",
            TileKind::Tree => "tree",
            TileKind::Air => "air",
            TileKind::BoneBlock => "bone_block",
            TileKind::IronScrap => "iron_scrap",
            TileKind::Door => "door",
            TileKind::Bedrock => "bedrock",
            TileKind::ScrapElectronics => "scrap_electronics",
            TileKind::PlasteelScrap => "plasteel_scrap",
            TileKind::Treasure => "treasure",
            _ => return None,
        }
    );
    let sd = loader.load_sprite(name, category);
    let ch = first_sprite_char(sd);
    let color = parse_color_string(&sd.color).unwrap_or_else(|| {
        panic!(
            "Missing or invalid RGB color in data.json for tile sprite '{}::{}' (expected #RRGGBB)",
            category, name
        )
    });
    Some((ch, color))
}

pub fn sprite_for_fluid(
    loader: &mut SpriteLoader,
    fluid_type: lithicrivers_core::resources::fluids::FluidType,
) -> Option<(char, Color)> {
    use lithicrivers_core::resources::fluids::FluidType;
    let (category, name) = ("fluids",
        match fluid_type {
            FluidType::Water => "water",
            FluidType::Oil => "oil",
            FluidType::Blood => "blood",
            FluidType::Acid => "acid",
            FluidType::Lava => "lava",
        }
    );
    let sd = loader.load_sprite(name, category);
    let ch = first_sprite_char(sd);
    let color = parse_color_string(&sd.color).unwrap_or_else(|| {
        panic!(
            "Missing or invalid RGB color in data.json for fluid sprite '{}::{}' (expected #RRGGBB)",
            category, name
        )
    });
    Some((ch, color))
}

pub fn color_for_entity(loader: &mut SpriteLoader, ch: char) -> Color {
    // Map entity glyphs to sprite names in assets/sprites/entities/<name>.lrsprite
    let (category, name) = ("entities",
        if ch == '@' {
            "player"
        } else if ch == 's' || ch == 'S' {
            "sheep"
        } else {
            "entity_generic"
        }
    );
    let sd = loader.load_sprite(name, category);
    parse_color_string(&sd.color).unwrap_or_else(|| {
        panic!(
            "Missing or invalid RGB color in data.json for entity sprite '{}::{}' (expected #RRGGBB)",
            category, name
        )
    })
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_load_water_sprite() {
        let asset_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/sprites");
        let mut loader = SpriteLoader::new(Some(&asset_path));
        let sprite = loader.load_sprite("water", "fluids");
        println!("Loaded sprite: {:?}", sprite);
        for (i, s) in sprite.sprites.iter().enumerate() {
            println!("Scale {}:\n{}", i + 1, s);
        }
    }
}

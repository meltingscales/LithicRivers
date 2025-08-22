use lithicrivers_core::tiles::TileKind;
use ratatui::prelude::Color;
use serde::Deserialize;
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use rust_embed::RustEmbed;
// All color data must come from data.json in each .lrsprite. No hardcoded fallbacks.
#[derive(RustEmbed)]
#[folder = "assets/"]
struct EmbeddedAssets;

#[derive(Debug, Clone, Copy)]
pub enum Scale {
    Small,  // 1x1
    Medium, // 2x2
    Large,  // 3x3
}

// Multi-scale: return the full sprite block string (may be multi-line) and color for a SpriteRef
pub fn sprite_block_for_spriteref(
    loader: &mut SpriteLoader,
    sr: &lithicrivers_core::components::SpriteRef,
    scale: Scale,
) -> (String, Color) {
    let sd = loader.load_by_spriteref(sr);
    let block = sprite_block_for_scale(sd, scale).to_string();
    let color = parse_color_string(&sd.color).unwrap_or_else(|| {
        panic!(
            "Missing or invalid RGB color in data.json for sprite '{}::{}' (expected #RRGGBB)",
            sr.category, sr.name
        )
    });
    (block, color)
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpriteMetadata {
    pub name: String,
    pub color: String,
    pub description: String,
    pub scales: Option<Vec<u32>>, // Not strictly needed for loading, but present in JSON
}

// Multi-scale: return the full sprite block string (may be multi-line) and color
pub fn sprite_block_for_tile(
    loader: &mut SpriteLoader,
    kind: TileKind,
    scale: Scale,
) -> Option<(String, Color)> {
    let (category, name) = (
        "tiles",
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
        },
    );
    let sd = loader.load_sprite(name, category);
    let block = sprite_block_for_scale(sd, scale).to_string();
    let color = parse_color_string(&sd.color).unwrap_or_else(|| {
        panic!(
            "Missing or invalid RGB color in data.json for tile sprite '{}::{}' (expected #RRGGBB)",
            category, name
        )
    });
    Some((block, color))
}

#[derive(Debug, Clone)]
pub struct SpriteData {
    pub name: String,
    pub color: String,
    pub description: String,

    pub sprites: Vec<String>, // Each entry is a sprite at a different scale
    // Example sprites content: ["x", "xx\nxx", "xxx\nxxx\nxxx"]
    pub art12x8_sprites: Vec<String>, // Required 12x8 (or similar) ASCII art for items.
                                      // Example: 12 lines of 8-char-long-lines each, separated by newlines.
                                      // Example: ["xxxxxx", "xxxxxx", "xxxxxx", "xxxxxx", "xxxxxx", "xxxxxx", "xxxxxx", "xxxxxx", "xxxxxx", "xxxxxx", "xxxxxx", "xxxxxx"]
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

    fn get_embedded_text(path: &str) -> String {
        let asset =
            EmbeddedAssets::get(path).unwrap_or_else(|| panic!("Missing embedded asset: {}", path));
        match asset.data {
            Cow::Borrowed(b) => String::from_utf8(b.to_vec()).expect("embedded text not UTF-8"),
            Cow::Owned(v) => String::from_utf8(v).expect("embedded text not UTF-8"),
        }
    }

    pub fn load_sprite(&mut self, sprite_name: &str, category: &str) -> &SpriteData {
        let cache_key = format!("{}/{}", category, sprite_name);
        if self.sprite_cache.contains_key(&cache_key) {
            return self.sprite_cache.get(&cache_key).unwrap();
        }
        // Embedded asset paths
        let base = format!("sprites/{}/{}.lrsprite/", category, sprite_name);
        let data_path = format!("{}data.json", base);
        let sprites_path = format!("{}sprites.txt", base);
        let art12x8_path = format!("{}art12x8.txt", base);

        // Load metadata from embedded assets
        let metadata: SpriteMetadata = serde_json::from_str(&Self::get_embedded_text(&data_path))
            .expect("Failed to parse embedded data.json");
        // Load sprite lines from embedded assets
        let content = Self::get_embedded_text(&sprites_path);
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
        // Load optional 12x8 art as a single multi-line string into a Vec<String>
        let art12x8_sprites: Vec<String> = if let Some(d) = EmbeddedAssets::get(&art12x8_path) {
            let text = match d.data {
                Cow::Borrowed(b) => String::from_utf8(b.to_vec()).expect("art12x8 not UTF-8"),
                Cow::Owned(v) => String::from_utf8(v).expect("art12x8 not UTF-8"),
            };
            vec![text]
        } else {
            panic!(
                "Missing required 12x8 art for sprite '{}::{}'",
                category, sprite_name
            );
        };

        let sprite_data = SpriteData {
            name: metadata.name,
            color: metadata.color,
            description: metadata.description,
            sprites,
            art12x8_sprites,
        };

        // Validate art12x8_sprites
        Self::validate_12x8sprite_dimensions(&sprite_data.art12x8_sprites, sprite_name, category);

        self.sprite_cache.insert(cache_key.clone(), sprite_data);
        self.sprite_cache.get(&cache_key).unwrap()
    }

    // Convenience: load by a SpriteRef component (category + name)
    pub fn load_by_spriteref(
        &mut self,
        sr: &lithicrivers_core::components::SpriteRef,
    ) -> &SpriteData {
        self.load_sprite(&sr.name, &sr.category)
    }

    // Discover and preload all sprites from embedded assets under sprites/<category>/<name>.lrsprite/
    pub fn preload_all(&mut self) {
        use std::collections::BTreeSet;
        let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
        for file in EmbeddedAssets::iter() {
            let path = file.as_ref();
            if !path.starts_with("sprites/") {
                continue;
            }
            // Expect: sprites/<category>/<name>.lrsprite/<file>
            let parts: Vec<&str> = path.split('/').collect();
            if parts.len() < 4 {
                continue;
            }
            let category = parts[1];
            let name_with_ext = parts[2];
            if let Some(name) = name_with_ext.strip_suffix(".lrsprite") {
                seen.insert((category.to_string(), name.to_string()));
            }
        }
        for (category, name) in seen {
            let _ = self.load_sprite(&name, &category);
        }
    }

    fn validate_12x8sprite_dimensions(
        sprite_blocks: &Vec<String>,
        sprite_name: &str,
        category: &str,
    ) {
        if sprite_blocks.is_empty() {
            // Optional for now; nothing to validate
            return;
        }
        for (i, block) in sprite_blocks.iter().enumerate() {
            if block.is_empty() {
                panic!(
                    "Sprite '{}::{}' 12x8 art is empty at entry {}",
                    category, sprite_name, i
                );
            }
            let lines: Vec<&str> = block.split('\n').collect();
            if lines.len() != 8 {
                panic!(
                    "Sprite '{}::{}' 12x8 art has {} lines at entry {}, expected 8",
                    category,
                    sprite_name,
                    lines.len(),
                    i
                );
            }
            let line_lengths: Vec<usize> = lines.iter().map(|l| l.len()).collect();
            let width = line_lengths[0];
            if line_lengths.iter().any(|&len| len != width) {
                panic!(
                    "Sprite '{}::{}' 12x8 art has inconsistent line widths at entry {}",
                    category, sprite_name, i
                );
            }
            if width != 12 {
                panic!(
                    "Sprite '{}::{}' 12x8 art must be 12 columns wide, got {} at entry {}",
                    category, sprite_name, width, i
                );
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

fn sprite_block_for_scale(sd: &SpriteData, scale: Scale) -> &str {
    match scale {
        Scale::Small => sd
            .sprites
            .get(0)
            .map(|s| s.as_str())
            .unwrap_or_else(|| panic!("Missing sprite for scale Small: {}", sd.name)),
        Scale::Medium => sd.sprites.get(1).map(|s| s.as_str()).unwrap_or_else(|| {
            sd.sprites
                .get(0)
                .map(|s| s.as_str())
                .unwrap_or_else(|| panic!("Missing sprite for scale Medium: {}", sd.name))
        }),
        Scale::Large => sd.sprites.get(2).map(|s| s.as_str()).unwrap_or_else(|| {
            sd.sprites.get(1).map(|s| s.as_str()).unwrap_or_else(|| {
                sd.sprites
                    .get(0)
                    .map(|s| s.as_str())
                    .unwrap_or_else(|| panic!("Missing sprite for scale Large: {}", sd.name))
            })
        }),
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

// Fluids removed

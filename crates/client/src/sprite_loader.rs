use lithicrivers_core::components::EntityKind;
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

impl Scale {
    #[allow(dead_code)]
    pub fn as_i64(self) -> i64 {
        match self {
            Scale::Small => 1,
            Scale::Medium => 2,
            Scale::Large => 3,
        }
    }
}

impl From<i64> for Scale {
    fn from(value: i64) -> Self {
        match value {
            1 => Scale::Small,
            2 => Scale::Medium,
            3 => Scale::Large,
            _ => Scale::Small, // Fallback to smallest scale
        }
    }
}

// Multi-scale: return the full sprite block string (may be multi-line) and color for a SpriteRef
#[allow(dead_code)]
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
    #[allow(dead_code)]
    pub scales: Option<Vec<u32>>, // Not strictly needed for loading, but present in JSON
    #[serde(default)]
    pub has_emotion_states: bool, // For NPCs that need mood-specific portraits
}

// Multi-scale: return the full sprite block string (may be multi-line) and color
// This code maps TileKind to sprite names which is eventually used for rendering.
pub fn sprite_block_for_tile(
    loader: &mut SpriteLoader,
    kind: TileKind,
    scale: Scale,
) -> Option<(String, Color)> {
    let (category, name) = (
        "tiles",
        lithicrivers_core::tiles::sprite_name_for_tile(kind),
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
    #[allow(dead_code)]
    pub data_path: PathBuf,
    sprite_cache: HashMap<String, SpriteData>,
}

unsafe impl Send for SpriteLoader {}
unsafe impl Sync for SpriteLoader {}

impl SpriteLoader {
    pub fn sprite_path_for_entitykind(&self, kind: EntityKind) -> (String, String) {
        match kind {
            EntityKind::Player => ("entities".to_string(), "player".to_string()),
            EntityKind::Sheep => ("entities".to_string(), "sheep".to_string()),
            EntityKind::FeralDog => ("entities".to_string(), "feral_dog".to_string()),
            EntityKind::Corpse => ("entities".to_string(), "corpse".to_string()),
            EntityKind::QuestTesty => ("entities".to_string(), "quest_testy".to_string()),
        }
    }
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
            .expect(
                format!(
                    "Failed to parse embedded data.json at {}",
                    data_path.as_str()
                )
                .as_str(),
            );
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
        // Also check for mood-specific portrait files
        let mut art12x8_sprites: Vec<String> = Vec::new();

        // Try to load the main art12x8.txt first
        if let Some(d) = EmbeddedAssets::get(&art12x8_path) {
            let text = match d.data {
                Cow::Borrowed(b) => String::from_utf8(b.to_vec()).expect("art12x8 not UTF-8"),
                Cow::Owned(v) => String::from_utf8(v).expect("art12x8 not UTF-8"),
            };
            art12x8_sprites.push(text);
        }

        // Try to load mood-specific portrait files for NPCs
        let mood_files = [
            "art12x8happy.txt",
            "art12x8sad.txt",
            "art12x8neutral.txt",
            "art12x8weird.txt",
        ];
        for mood_file in &mood_files {
            let mood_path = format!("{}{}", base, mood_file);
            if let Some(d) = EmbeddedAssets::get(&mood_path) {
                let text = match d.data {
                    Cow::Borrowed(b) => String::from_utf8(b.to_vec()).expect("mood art not UTF-8"),
                    Cow::Owned(v) => String::from_utf8(v).expect("mood art not UTF-8"),
                };
                art12x8_sprites.push(text);
            } else {
                // Panic if mood files are expected but missing
                if category == "npcs" || category == "entities" {
                    if metadata.has_emotion_states {
                        panic!(
                            "Missing required mood portrait '{}' for sprite '{}::{}'",
                            mood_file, category, sprite_name
                        );
                    }
                }
            }
        }

        // If no art was loaded at all, panic
        if art12x8_sprites.is_empty() {
            panic!(
                "Missing required 12x8 art for sprite '{}::{}'",
                category, sprite_name
            );
        }

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

    // Get mood-specific portrait for NPCs
    pub fn get_mood_portrait(
        &mut self,
        sprite_ref: &lithicrivers_core::components::SpriteRef,
        mood: lithicrivers_core::components::NPCMood,
    ) -> Option<String> {
        let sprite_data = self.load_by_spriteref(sprite_ref);
        let mood_index = match mood {
            lithicrivers_core::components::NPCMood::Happy => 1, // art12x8happy.txt
            lithicrivers_core::components::NPCMood::Sad => 2,   // art12x8sad.txt
            lithicrivers_core::components::NPCMood::Neutral => 3, // art12x8neutral.txt
            lithicrivers_core::components::NPCMood::Weird => 4, // art12x8weird.txt
        };

        if mood_index < sprite_data.art12x8_sprites.len() {
            Some(sprite_data.art12x8_sprites[mood_index].clone())
        } else {
            // Fallback to main portrait if mood-specific one doesn't exist
            sprite_data.art12x8_sprites.get(0).cloned()
        }
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
            let lines: Vec<&str> = block.lines().collect();
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
            let lines: Vec<&str> = sprite.lines().collect();
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

pub fn parse_color_string(s: &str) -> Option<Color> {
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

// Re-export from the centralized tile registry
pub use crate::tile_registry::*;

// Legacy compatibility - use const array instead of function call
pub static TILE_KIND_STRS: &[&str] = &[
    "rock",
    "dirt",
    "grass",
    "tree",
    "air",
    "bone_block",
    "iron_scrap",
    "door",
    "bedrock",
    "scrap_electronics",
    "plasteel_scrap",
    "treasure",
    "plank_block",
    "stairs",
    "existing_worldgen",
    "enemy_spawn",
];

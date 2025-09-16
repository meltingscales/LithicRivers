use crate::palettekey::PaletteKey;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TileKind {
    Rock,
    Dirt,
    Grass,
    Tree,
    Air,
    BoneBlock,
    IronScrap,
    Door,
    Bedrock,
    ScrapElectronics,
    PlasteelScrap,
    Treasure,
    PlankBlock,
    Stairs,

    //special tile that gets replaced by existing worldgen
    ExistingWorldgen,

    // tiles for enemy spawns
    EnemySpawn,
}

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

impl TileKind {
    pub fn sprite_key(self) -> &'static str {
        match self {
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
            TileKind::PlankBlock => "plank_block",
            TileKind::Stairs => "stairs",
            TileKind::ExistingWorldgen => "existing_worldgen",
            TileKind::EnemySpawn => "enemy_spawn",
        }
    }
    pub fn palette_key(self) -> PaletteKey {
        match self {
            TileKind::Rock => PaletteKey::Rock,
            TileKind::Dirt => PaletteKey::Dirt,
            TileKind::Grass => PaletteKey::Grass,
            TileKind::Tree => PaletteKey::Tree,
            TileKind::Air => PaletteKey::Air,
            TileKind::BoneBlock => PaletteKey::BoneBlock,
            TileKind::IronScrap => PaletteKey::IronScrap,
            TileKind::Door => PaletteKey::Door,
            TileKind::Bedrock => PaletteKey::Bedrock,
            TileKind::ScrapElectronics => PaletteKey::ScrapElectronics,
            TileKind::PlasteelScrap => PaletteKey::PlasteelScrap,
            TileKind::Treasure => PaletteKey::Treasure,
            TileKind::PlankBlock => PaletteKey::PlankBlock,
            TileKind::Stairs => PaletteKey::Stairs,
            TileKind::ExistingWorldgen => PaletteKey::ExistingWorldgen,
            TileKind::EnemySpawn => PaletteKey::EnemySpawn,
        }
    }
    pub fn from_str(s: &str) -> Option<TileKind> {
        match s {
            "rock" => Some(TileKind::Rock),
            "dirt" => Some(TileKind::Dirt),
            "grass" => Some(TileKind::Grass),
            "tree" => Some(TileKind::Tree),
            "air" => Some(TileKind::Air),
            "bone_block" => Some(TileKind::BoneBlock),
            "iron_scrap" => Some(TileKind::IronScrap),
            "door" => Some(TileKind::Door),
            "bedrock" => Some(TileKind::Bedrock),
            "scrap_electronics" => Some(TileKind::ScrapElectronics),
            "plasteel_scrap" => Some(TileKind::PlasteelScrap),
            "treasure" => Some(TileKind::Treasure),
            "plank_block" => Some(TileKind::PlankBlock),
            "stairs" => Some(TileKind::Stairs),
            "existing_worldgen" => Some(TileKind::ExistingWorldgen),
            "enemy_spawn" => Some(TileKind::EnemySpawn),
            _ => None,
        }
    }

    pub fn is_passable(self) -> bool {
        match self {
            TileKind::Rock => false,
            TileKind::Dirt => true,
            TileKind::Grass => true,
            TileKind::Tree => true,
            TileKind::Air => true,
            TileKind::BoneBlock => false,
            TileKind::IronScrap => true,
            TileKind::Door => false,
            TileKind::Bedrock => false,
            TileKind::ScrapElectronics => false,
            TileKind::PlasteelScrap => true,
            TileKind::Treasure => false,
            TileKind::PlankBlock => false,
            TileKind::Stairs => true, // Stairs are passable for movement but special for Z-level transitions
            TileKind::ExistingWorldgen => false,
            TileKind::EnemySpawn => false,
        }
    }

    /// Convert an ItemKind to a TileKind for placing blocks
    /// Returns Some(TileKind) if the item can be placed as a block, None otherwise
    pub fn from_item_kind(item: crate::components::ItemKind) -> Option<TileKind> {
        use crate::components::ItemKind;
        match item {
            // Exact name matches between ItemKind and TileKind
            ItemKind::PlankBlock => Some(TileKind::PlankBlock),
            ItemKind::Stone => Some(TileKind::Rock), // Stone item places as Rock tile
            _ => None,                               // Other items are not placeable as blocks
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_roundtrip() {
        let kinds = [
            TileKind::Rock,
            TileKind::Dirt,
            TileKind::Grass,
            TileKind::Tree,
            TileKind::Air,
            TileKind::BoneBlock,
            TileKind::IronScrap,
            TileKind::Door,
            TileKind::Bedrock,
            TileKind::ScrapElectronics,
            TileKind::PlasteelScrap,
            TileKind::Treasure,
            TileKind::PlankBlock,
            TileKind::ExistingWorldgen,
            TileKind::EnemySpawn,
        ];
        for &k in &kinds {
            let s = serde_json::to_string(&k).expect("serialize");
            let k2: TileKind = serde_json::from_str(&s).expect("deserialize");
            assert_eq!(k, k2);
        }
    }
}

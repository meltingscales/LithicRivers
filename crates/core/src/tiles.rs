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
}

pub static TILE_KIND_STRS: &[&str] = &["rock", "dirt", "grass", "tree", "air", "bone_block", "iron_scrap", "door", "bedrock", "scrap_electronics", "plasteel_scrap", "treasure"];

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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_roundtrip() {
        let kinds = [TileKind::Rock, TileKind::Dirt, TileKind::Grass, TileKind::Tree, TileKind::Air, TileKind::BoneBlock, TileKind::IronScrap, TileKind::Door, TileKind::Bedrock, TileKind::ScrapElectronics, TileKind::PlasteelScrap, TileKind::Treasure];
        for &k in &kinds {
            let s = serde_json::to_string(&k).expect("serialize");
            let k2: TileKind = serde_json::from_str(&s).expect("deserialize");
            assert_eq!(k, k2);
        }
    }
}

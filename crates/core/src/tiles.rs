use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TileKind {
    Rock,
    Dirt,
    Grass,
    Tree,
    Air,
}

pub static TILE_KIND_STRS: &[&str] = &["rock", "dirt", "grass", "tree", "air"];

impl TileKind {
    pub fn sprite_key(self) -> &'static str {
        match self {
            TileKind::Rock => "rock",
            TileKind::Dirt => "dirt",
            TileKind::Grass => "grass",
            TileKind::Tree => "tree",
            TileKind::Air => "air",
        }
    }
    pub fn from_str(s: &str) -> Option<TileKind> {
        match s {
            "rock" => Some(TileKind::Rock),
            "dirt" => Some(TileKind::Dirt),
            "grass" => Some(TileKind::Grass),
            "tree" => Some(TileKind::Tree),
            "air" => Some(TileKind::Air),
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_roundtrip() {
        let kinds = [TileKind::Rock, TileKind::Dirt, TileKind::Grass, TileKind::Tree, TileKind::Air];
        for &k in &kinds {
            let s = serde_json::to_string(&k).expect("serialize");
            let k2: TileKind = serde_json::from_str(&s).expect("deserialize");
            assert_eq!(k, k2);
        }
    }
}

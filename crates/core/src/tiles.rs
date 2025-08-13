use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TileKind {
    Rock,
    Floor,
    Grass,
    Tree,
    Air,
}

impl TileKind {
    pub fn glyph(self) -> char {
        match self {
            TileKind::Rock => '#',
            TileKind::Floor => '.',
            TileKind::Grass => ',',
            TileKind::Tree => 't',
            TileKind::Air => ' ',
        }
    }
    pub fn is_passable(self) -> bool {
        match self {
            TileKind::Rock => false,
            TileKind::Floor => true,
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
    fn glyph_mapping_is_stable() {
        assert_eq!(TileKind::Rock.glyph(), '#');
        assert_eq!(TileKind::Floor.glyph(), '.');
        assert_eq!(TileKind::Grass.glyph(), ',');
        assert_eq!(TileKind::Tree.glyph(), 't');
        assert_eq!(TileKind::Air.glyph(), ' ');
    }

    #[test]
    fn serde_roundtrip() {
        let kinds = [TileKind::Rock, TileKind::Floor, TileKind::Grass, TileKind::Tree, TileKind::Air];
        for &k in &kinds {
            let s = serde_json::to_string(&k).expect("serialize");
            let k2: TileKind = serde_json::from_str(&s).expect("deserialize");
            assert_eq!(k, k2);
        }
    }
}

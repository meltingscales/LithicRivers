use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TileKind {
    Rock,
    Water,
    Floor,
    Grass,
}

impl TileKind {
    pub fn glyph(self) -> char {
        match self {
            TileKind::Rock => '#',
            TileKind::Water => '~',
            TileKind::Floor => '.',
            TileKind::Grass => ',',
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glyph_mapping_is_stable() {
        assert_eq!(TileKind::Rock.glyph(), '#');
        assert_eq!(TileKind::Water.glyph(), '~');
        assert_eq!(TileKind::Floor.glyph(), '.');
        assert_eq!(TileKind::Grass.glyph(), ',');
    }

    #[test]
    fn serde_roundtrip() {
        let kinds = [TileKind::Rock, TileKind::Water, TileKind::Floor, TileKind::Grass];
        for &k in &kinds {
            let s = serde_json::to_string(&k).expect("serialize");
            let k2: TileKind = serde_json::from_str(&s).expect("deserialize");
            assert_eq!(k, k2);
        }
    }
}

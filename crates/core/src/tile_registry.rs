/// Single source of truth for all tile definitions
/// Format: (VariantName, "string_key", passable)
macro_rules! define_tiles {
    ($($variant:ident, $key:literal, $passable:expr),* $(,)?) => {
        use crate::palettekey::PaletteKey;
        use serde::{Deserialize, Serialize};

        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub enum TileKind {
            $($variant,)*
        }

        impl TileKind {
            pub fn sprite_key(self) -> &'static str {
                match self {
                    $(TileKind::$variant => $key,)*
                }
            }

            pub fn palette_key(self) -> PaletteKey {
                match self {
                    $(TileKind::$variant => PaletteKey::$variant,)*
                }
            }

            pub fn from_str(s: &str) -> Option<TileKind> {
                match s {
                    $($key => Some(TileKind::$variant),)*
                    _ => None,
                }
            }

            pub fn is_passable(self) -> bool {
                match self {
                    $(TileKind::$variant => $passable,)*
                }
            }

            pub fn all_variants() -> &'static [TileKind] {
                &[$(TileKind::$variant,)*]
            }

            pub fn all_keys() -> &'static [&'static str] {
                &[$($key,)*]
            }
        }

        // Generate sprite mapping for client code
        pub fn sprite_name_for_tile(kind: TileKind) -> &'static str {
            match kind {
                $(TileKind::$variant => $key,)*
            }
        }
    };
}

// Single source of truth - add/remove/modify tiles here
define_tiles! {
    Rock, "rock", false,
    Dirt, "dirt", true,
    Grass, "grass", true,
    Tree, "tree", true,
    Air, "air", true,
    BoneBlock, "bone_block", false,
    IronScrap, "iron_scrap", true,
    Door, "door", false,
    Bedrock, "bedrock", false,
    ScrapElectronics, "scrap_electronics", false,
    PlasteelScrap, "plasteel_scrap", true,
    Treasure, "treasure", false,
    PlankBlock, "plank_block", false,
    Stairs, "stairs", true,
    ExistingWorldgen, "existing_worldgen", false,
    EnemySpawn, "enemy_spawn", false,
}

impl TileKind {
    /// Convert an ItemKind to a TileKind for placing blocks
    pub fn from_item_kind(item: crate::components::ItemKind) -> Option<TileKind> {
        use crate::components::ItemKind;
        match item {
            ItemKind::PlankBlock => Some(TileKind::PlankBlock),
            ItemKind::Stone => Some(TileKind::Rock),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_roundtrip() {
        for &kind in TileKind::all_variants() {
            let s = serde_json::to_string(&kind).expect("serialize");
            let k2: TileKind = serde_json::from_str(&s).expect("deserialize");
            assert_eq!(kind, k2);
        }
    }

    #[test]
    fn string_conversion_roundtrip() {
        for &kind in TileKind::all_variants() {
            let key = kind.sprite_key();
            let parsed = TileKind::from_str(key).expect("from_str should work");
            assert_eq!(kind, parsed);
        }
    }
}

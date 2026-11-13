/// Single source of truth for all tile definitions
/// Format: (VariantName, "string_key", "display_name", passable)
macro_rules! define_tiles {
    ($($variant:ident, $key:literal, $name:literal, $passable:expr),* $(,)?) => {
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

            pub fn from_key(s: &str) -> Option<TileKind> {
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

            pub fn display_name(self) -> &'static str {
                match self {
                    $(TileKind::$variant => $name,)*
                }
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
    Rock, "rock", "rock", false,
    Dirt, "dirt", "dirt", true,
    Grass, "grass", "grass", true,
    Tree, "tree", "tree", true,
    Air, "air", "air", true,
    BoneBlock, "bone_block", "bone block", false,
    IronScrap, "iron_scrap", "iron scrap", false,
    Door, "door", "door", false,
    DoorOpen, "door_open", "open door", true,
    Bedrock, "bedrock", "bedrock", false,
    ScrapElectronics, "scrap_electronics", "scrap electronics", false,
    PlasteelScrap, "plasteel_scrap", "plasteel scrap", false,
    TreasureCommon, "treasure_common", "common treasure", false,
    TreasureRare, "treasure_rare", "rare treasure", false,
    ScrapCommon, "scrap_common", "common scrap", true,
    ScrapRare, "scrap_rare", "rare scrap", true,
    TreasureQuest1, "treasure_quest_1", "quest treasure", false,
    PlankBlock, "plank_block", "plank block", false,
    Stairs, "stairs", "stairs", true,
    ExistingWorldgen, "existing_worldgen", "existing worldgen", false,
    EnemySpawn, "enemy_spawn", "enemy spawn", false,
    WildSpawner, "wild_spawner", "wild spawner", false,
    TeleporterStation, "teleporter_station", "transit station", false,
    TeleporterStationInner, "teleporter_station_inner", "inner transit station", false,
    SapiencorpRtg, "sapiencorp_rtg", "SapienCorp RTG", false,
    StationItemCharger, "station_item_charger", "station item charger", false,
    PowerPole, "power_pole", "power pole", false,
    SpecialPlayerSpawn, "special_player_spawn", "player spawn", false,
    FactoryBlock, "factory_block", "factory block", false,
    Sapling, "sapling", "sapling", true,
    StoneBrick, "stone_brick", "stone brick", false,
    Lava, "lava", "lava", false,
    LavaCrust, "lava_crust", "lava crust", true,
    Riverglass, "riverglass", "riverglass", false,
    WheatSeedling, "wheat_seedling", "wheat seedling", true,
    WheatGrowing, "wheat_growing", "growing wheat", true,
    WheatRipe, "wheat_ripe", "ripe wheat", true,
    PotatoSeedling, "potato_seedling", "potato seedling", true,
    PotatoGrowing, "potato_growing", "growing potato", true,
    PotatoRipe, "potato_ripe", "ripe potato", true,
    CarrotSeedling, "carrot_seedling", "carrot seedling", true,
    CarrotGrowing, "carrot_growing", "growing carrot", true,
    CarrotRipe, "carrot_ripe", "ripe carrot", true,
    AndroidSpawn, "android_spawn", "android spawn", false,
    Nitre, "nitre", "nitre", false,
    Sulfur, "sulfur", "sulfur", false,
    BrassScrap, "brass_scrap", "brass scrap", false,
    SulfurVent, "sulfur_vent", "sulfur vent", false,
    SulfurCrystal, "sulfur_crystal", "sulfur crystal", false,
    Ash, "ash", "ash", true,
    CrackedRoad, "cracked_road", "cracked road", true,
    Rubble, "rubble", "rubble", false,
    Concrete, "concrete", "concrete", false,
    TreasureQuest2, "treasure_quest_2", "quest treasure", false,
    SealedDoorSite, "sealed_door_site", "sealed door", false,
    BurnerGeneratorSite, "burner_generator_site", "burner generator", false,
    ReactorCasing, "reactor_casing", "reactor casing", false,
    NuclearWasteDeposit, "nuclear_waste_deposit", "nuclear waste", false,
    UraniumDeposit, "uranium_deposit", "uranium deposit", false,
    TreasureQuest3, "treasure_quest_3", "quest treasure", false,
}

impl TileKind {
    /// Convert an ItemKind to a TileKind for placing blocks
    pub fn from_item_kind(item: crate::components::ItemKind) -> Option<TileKind> {
        use crate::components::ItemKind;
        match item {
            ItemKind::PlankBlock => Some(TileKind::PlankBlock),
            ItemKind::StoneBrick => Some(TileKind::StoneBrick),
            ItemKind::Stone => Some(TileKind::Rock),
            // Placed as a stairway of two; see `stairs::build_stairway`
            ItemKind::Stairs => Some(TileKind::Stairs),
            // Planted rather than placed; see `plants::plant`
            ItemKind::Acorn => Some(TileKind::Sapling),
            ItemKind::WheatSeeds => Some(TileKind::WheatSeedling),
            ItemKind::Potato => Some(TileKind::PotatoSeedling),
            ItemKind::Carrot => Some(TileKind::CarrotSeedling),
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
            let parsed = TileKind::from_key(key).expect("from_key should work");
            assert_eq!(kind, parsed);
        }
    }
}

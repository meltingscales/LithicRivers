use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PaletteKey {
    // Tiles
    Rock,
    Dirt,
    Grass,
    Tree,
    Air,
    BoneBlock,
    IronScrap,
    Bedrock,
    ScrapElectronics,
    PlasteelScrap,
    PlankBlock,

    // Doors/stairs
    Door,
    DoorOpen,
    Stairs,

    // Fluids
    Water,
    Oil,
    Blood,
    Acid,
    Lava,

    // Entities
    Player,
    EntityGeneric,
    Sheep,
    // special tile that gets replaced by existing worldgen
    ExistingWorldgen,
    // special tile that spawns enemies
    EnemySpawn,
    // special tiles that spawn treasure
    ScrapCommon,
    ScrapRare,
    TreasureCommon,
    TreasureRare,
    TreasureQuest1,
}

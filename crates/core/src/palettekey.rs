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
    Door,
    Bedrock,
    ScrapElectronics,
    PlasteelScrap,
    Treasure,
    PlankBlock,
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
}

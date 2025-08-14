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
    // Fluids
    Water,
}

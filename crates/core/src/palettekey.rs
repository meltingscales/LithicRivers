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
    StoneBrick,

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
    // special tile that spawns a military android
    AndroidSpawn,
    // special tile that becomes a wild spawner block
    WildSpawner,
    // special tile that becomes a SapienCorp transit station block
    TeleporterStation,
    TeleporterStationInner,
    // special tiles that become a station's built-in RTG and item charger, and a pole
    SapiencorpRtg,
    StationItemCharger,
    PowerPole,
    // special tiles that spawn treasure
    ScrapCommon,
    ScrapRare,
    TreasureCommon,
    TreasureRare,
    TreasureQuest1,

    //other special tiles
    SpecialPlayerSpawn,

    // Marker under a factory block entity
    FactoryBlock,
    Sapling,
    LavaCrust,
    Riverglass,
    Nitre,
    Sulfur,
    BrassScrap,
    SulfurVent,
    SulfurCrystal,
    Ash,
    CrackedRoad,
    Rubble,
    Concrete,
    TreasureQuest2,
    SealedDoorSite,
    BurnerGeneratorSite,
    WheatSeedling,
    WheatGrowing,
    WheatRipe,
    PotatoSeedling,
    PotatoGrowing,
    PotatoRipe,
    CarrotSeedling,
    CarrotGrowing,
    CarrotRipe,
}

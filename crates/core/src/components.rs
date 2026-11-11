use crate::dialogue::DialogueNodeID;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

use std::fmt;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub struct Position {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Player;

/// Marker for all in-world game entities (parents Player, Sheep, etc.)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GameEntity;

/// Renderable glyph for ASCII views
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Glyph(pub char);

/// Marker for a simple wandering sheep
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Sheep;

/// Marker for a simple wandering feral dog
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FeralDog;

/// The player's old body, left where they died: a corpse holding what they carried and the
/// body itself
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Wreck;

/// Marker for nuclear slimes: slow, sturdy, glowing underground creatures
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct NuclearSlime;

/// Marker for mining bots: SapienCorp diggers in the deep that drill toward the player
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MiningBot;

/// Marker for rusty military androids: SapienCorp security units that shoot from a
/// distance while their ammo lasts (`ranged`)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MilitaryAndroid;

/// A military android's pace and memory: it acts on every other turn, and walks to where
/// it last saw the player when it loses sight of them
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AndroidAI {
    pub turns: u8,
    pub last_seen: Option<Position>,
}

/// A creature that can shoot. Its ammo is the `ammo` items in its inventory, so a wreck
/// keeps what it didn't fire. Hit chance falls off with distance (`ranged::hit_chance`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RangedAttack {
    /// Farthest a shot reaches, in tiles
    pub range: i64,
    /// Percent chance to hit from 1 tile away
    pub accuracy_at_1: u32,
    /// Percent lost per tile beyond the first
    pub falloff: u32,
    pub ammo: ItemKind,
}

/// A nuclear slime's pace: it moves on every other turn it is chasing
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlimeAI {
    pub turns: u8,
}

/// AI behavior state for feral dogs
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DogAI {
    pub behavior: crate::pathfinding::DogBehavior,
    pub behavior_timer: u64, // Ticks remaining in current behavior
    pub circle_center: Option<Position>, // Center for circular movement
    pub steps_taken: u32,    // Steps taken in current behavior (reset on behavior change)
}

/// Component for entities that can engage in combat
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct Combat {
    pub triggered: bool,
}

/// Component for entities that have escaped combat and cannot re-engage for a while
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BattleDelay {
    pub remaining_ticks: u64,
}

impl BattleDelay {
    pub fn new(ticks: u64) -> Self {
        Self {
            remaining_ticks: ticks,
        }
    }
}

/// Component for entity health
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Health {
    pub current: u32,
    pub max: u32,
}

impl Health {
    pub fn new(max: u32) -> Self {
        Self { current: max, max }
    }

    pub fn damage(&mut self, amount: u32) {
        self.current = self.current.saturating_sub(amount);
    }

    pub fn heal(&mut self, amount: u32) {
        self.current = (self.current + amount).min(self.max);
    }

    pub fn is_alive(&self) -> bool {
        self.current > 0
    }

    pub fn percentage(&self) -> f32 {
        if self.max == 0 {
            0.0
        } else {
            self.current as f32 / self.max as f32
        }
    }
}

/// Component for entity energy (replaces mana)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Energy {
    pub current: u32,
    pub max: u32,
}

impl Energy {
    pub fn new(max: u32) -> Self {
        Self { current: max, max }
    }

    pub fn consume(&mut self, amount: u32) -> bool {
        if self.current >= amount {
            self.current -= amount;
            true
        } else {
            false
        }
    }

    pub fn restore(&mut self, amount: u32) {
        self.current = (self.current + amount).min(self.max);
    }

    pub fn percentage(&self) -> f32 {
        if self.max == 0 {
            0.0
        } else {
            self.current as f32 / self.max as f32
        }
    }
}

/// Component for entities that are stunned
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Stunned {
    pub remaining_ticks: u64,
}

impl Stunned {
    pub fn new(ticks: u64) -> Self {
        Self {
            remaining_ticks: ticks,
        }
    }
}

/// Marker for entities that block movement
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BlocksMovement;

// Body lives in `crate::model::body` now.

// -----------------------------
// Items & Inventory Components
// -----------------------------

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash, PartialOrd, Ord, EnumIter,
)]
pub enum ItemKind {
    Log,
    WoodenPlank,
    Acorn,
    Stick,
    Nail,
    Stone,
    String,
    Torch,
    WoodenShavings,
    ScrapElectronics,
    Diamond,
    Leather,
    Meat,
    PlankBlock,
    Chest,
    IronScrap,
    WoodenConveyor,
    BurnerGenerator,
    PowerPole,
    AutoPlanter,
    AutoHarvester,
    Biochar,
    CrudeFurnace,
    Extractor,
    FilterExtractor,
    BeltRouter,
    VerticalConveyor,
    Stairs,
    NuclearWaste,
    PlasteelScrap,
    ElectricFurnace,
    SalvagedArm,
    CrudeHead,
    CrudeTorso,
    CrudeFusionCore,
    CrudeArm,
    CrudeLeg,
    StandardHead,
    StandardTorso,
    StandardFusionCore,
    StandardArm,
    StandardLeg,
    AdvancedHead,
    AdvancedTorso,
    AdvancedFusionCore,
    AdvancedArm,
    AdvancedLeg,
    CrudeVat,
    StandardVat,
    AdvancedVat,
    ScrapLantern,
    Fence,
    Gate,
    Spawner,
    MeatSlurry,
    FeralDogEmbryoPoor,
    FeralDogEmbryoFlawed,
    FeralDogEmbryoFlawless,
    NuclearSlimeEmbryoPoor,
    NuclearSlimeEmbryoFlawed,
    NuclearSlimeEmbryoFlawless,
    StoneBrick,
    Teleporter,
    Battery,
    CrudePowerCell,
    StandardPowerCell,
    AdvancedPowerCell,
    MobileTeleAccessDevice,
    Rtg,
    ItemCharger,
    Riverglass,
    WheatSeeds,
    Wheat,
    Potato,
    Carrot,
    Straw,
    Mash,
    Ethanol,
    Fermenter,
    AutoFeeder,
    AnimalCounter,
    LivestockProcessor,
    BioVat,
    BiomechEmbryoDeformed,
    BiomechEmbryoWrinkly,
    BiomechEmbryoPerfect,
    BrassCartridge,
}

pub fn itemkind_name(kind: ItemKind) -> &'static str {
    // Corresponds to a human-readable name for the item kind.
    match kind {
        ItemKind::Log => "Log",
        ItemKind::WoodenPlank => "Wooden Plank",
        ItemKind::Acorn => "Acorn",
        ItemKind::Stick => "Stick",
        ItemKind::Nail => "Nail",
        ItemKind::Stone => "Stone",
        ItemKind::String => "String",
        ItemKind::Torch => "Torch",
        ItemKind::WoodenShavings => "Wooden Shavings",
        ItemKind::ScrapElectronics => "Scrap Electronics",
        ItemKind::Diamond => "Diamond",
        ItemKind::Leather => "Leather",
        ItemKind::Meat => "Meat",
        ItemKind::PlankBlock => "Plank Block",
        ItemKind::Chest => "Chest",
        ItemKind::IronScrap => "Iron Scrap",
        ItemKind::WoodenConveyor => "Wooden Conveyor",
        ItemKind::BurnerGenerator => "Burner Generator",
        ItemKind::PowerPole => "Power Pole",
        ItemKind::AutoPlanter => "Auto-Planter",
        ItemKind::AutoHarvester => "Auto-Harvester",
        ItemKind::Biochar => "Biochar",
        ItemKind::CrudeFurnace => "Crude Furnace",
        ItemKind::Extractor => "Extractor",
        ItemKind::FilterExtractor => "Filter Extractor",
        ItemKind::BeltRouter => "Belt Router",
        ItemKind::VerticalConveyor => "Vertical Conveyor",
        ItemKind::Stairs => "Stairs",
        ItemKind::NuclearWaste => "Nuclear Waste",
        ItemKind::PlasteelScrap => "Plasteel Scrap",
        ItemKind::ElectricFurnace => "Electric Furnace",
        ItemKind::SalvagedArm => "Salvaged SapienCorp Arm",
        ItemKind::CrudeHead => "Crude Head",
        ItemKind::CrudeTorso => "Crude Torso",
        ItemKind::CrudeFusionCore => "Crude Fusion Core",
        ItemKind::CrudeArm => "Crude Arm",
        ItemKind::CrudeLeg => "Crude Leg",
        ItemKind::StandardHead => "Standard Head",
        ItemKind::StandardTorso => "Standard Torso",
        ItemKind::StandardFusionCore => "Standard Fusion Core",
        ItemKind::StandardArm => "Standard Arm",
        ItemKind::StandardLeg => "Standard Leg",
        ItemKind::AdvancedHead => "Advanced Head",
        ItemKind::AdvancedTorso => "Advanced Torso",
        ItemKind::AdvancedFusionCore => "Advanced Fusion Core",
        ItemKind::AdvancedArm => "Advanced Arm",
        ItemKind::AdvancedLeg => "Advanced Leg",
        ItemKind::CrudeVat => "Crude Resurrection Vat",
        ItemKind::StandardVat => "Standard Resurrection Vat",
        ItemKind::AdvancedVat => "Advanced Resurrection Vat",
        ItemKind::ScrapLantern => "Scrap Lantern",
        ItemKind::Fence => "Fence",
        ItemKind::Gate => "Gate",
        ItemKind::Spawner => "Spawner",
        ItemKind::MeatSlurry => "Meat Slurry",
        ItemKind::FeralDogEmbryoPoor => "Feral Dog Embryo (Poor)",
        ItemKind::FeralDogEmbryoFlawed => "Feral Dog Embryo (Flawed)",
        ItemKind::FeralDogEmbryoFlawless => "Feral Dog Embryo (Flawless)",
        ItemKind::NuclearSlimeEmbryoPoor => "Nuclear Slime Embryo (Poor)",
        ItemKind::NuclearSlimeEmbryoFlawed => "Nuclear Slime Embryo (Flawed)",
        ItemKind::NuclearSlimeEmbryoFlawless => "Nuclear Slime Embryo (Flawless)",
        ItemKind::StoneBrick => "Stone Brick",
        ItemKind::Teleporter => "Teleporter",
        ItemKind::Battery => "Battery",
        ItemKind::CrudePowerCell => "Crude Power Cell",
        ItemKind::StandardPowerCell => "Standard Power Cell",
        ItemKind::AdvancedPowerCell => "Advanced Power Cell",
        ItemKind::MobileTeleAccessDevice => "Mobile Tele-Access Device",
        ItemKind::Rtg => "RTG",
        ItemKind::ItemCharger => "Item Charger",
        ItemKind::Riverglass => "Riverglass",
        ItemKind::WheatSeeds => "Wheat Seeds",
        ItemKind::Wheat => "Wheat",
        ItemKind::Potato => "Potato",
        ItemKind::Carrot => "Carrot",
        ItemKind::Straw => "Straw",
        ItemKind::Mash => "Mash",
        ItemKind::Ethanol => "Bottled Ethanol",
        ItemKind::Fermenter => "Fermenter",
        ItemKind::AutoFeeder => "Auto-Feeder",
        ItemKind::AnimalCounter => "Animal Counter",
        ItemKind::LivestockProcessor => "Livestock Processor",
        ItemKind::BioVat => "Bio-Vat",
        ItemKind::BiomechEmbryoDeformed => "Deformed Biomech Embryo",
        ItemKind::BiomechEmbryoWrinkly => "Wrinkly Biomech Embryo",
        ItemKind::BiomechEmbryoPerfect => "Perfect Biomech Embryo",
        ItemKind::BrassCartridge => "Brass Cartridge",
    }
}

pub fn itemkind_sprite_name(kind: ItemKind) -> &'static str {
    // Corresponds to the folder within assets/sprites/items
    match kind {
        ItemKind::Log => "log",
        ItemKind::WoodenPlank => "wooden_plank",
        ItemKind::ScrapElectronics => "scrap_electronics",
        ItemKind::Diamond => "diamond",
        ItemKind::Acorn => "acorn",
        ItemKind::Stick => "stick",
        ItemKind::Nail => "nail",
        ItemKind::Stone => "stone",
        ItemKind::String => "string",
        ItemKind::Torch => "torch",
        ItemKind::WoodenShavings => "wooden_shavings",
        ItemKind::Leather => "leather",
        ItemKind::Meat => "meat",
        ItemKind::PlankBlock => "plank_block",
        ItemKind::Chest => "chest",
        ItemKind::IronScrap => "iron_scrap",
        ItemKind::WoodenConveyor => "wooden_conveyor",
        ItemKind::BurnerGenerator => "burner_generator",
        ItemKind::PowerPole => "power_pole",
        ItemKind::AutoPlanter => "auto_planter",
        ItemKind::AutoHarvester => "auto_harvester",
        ItemKind::Biochar => "biochar",
        ItemKind::CrudeFurnace => "crude_furnace",
        ItemKind::Extractor => "extractor",
        ItemKind::FilterExtractor => "filter_extractor",
        ItemKind::BeltRouter => "belt_router",
        ItemKind::VerticalConveyor => "vertical_conveyor",
        ItemKind::Stairs => "stairs",
        ItemKind::NuclearWaste => "nuclear_waste",
        ItemKind::PlasteelScrap => "plasteel_scrap",
        ItemKind::ElectricFurnace => "electric_furnace",
        ItemKind::SalvagedArm => "salvaged_arm",
        ItemKind::CrudeHead => "crude_head",
        ItemKind::CrudeTorso => "crude_torso",
        ItemKind::CrudeFusionCore => "crude_fusion_core",
        ItemKind::CrudeArm => "crude_arm",
        ItemKind::CrudeLeg => "crude_leg",
        ItemKind::StandardHead => "standard_head",
        ItemKind::StandardTorso => "standard_torso",
        ItemKind::StandardFusionCore => "standard_fusion_core",
        ItemKind::StandardArm => "standard_arm",
        ItemKind::StandardLeg => "standard_leg",
        ItemKind::AdvancedHead => "advanced_head",
        ItemKind::AdvancedTorso => "advanced_torso",
        ItemKind::AdvancedFusionCore => "advanced_fusion_core",
        ItemKind::AdvancedArm => "advanced_arm",
        ItemKind::AdvancedLeg => "advanced_leg",
        ItemKind::CrudeVat => "crude_vat",
        ItemKind::StandardVat => "standard_vat",
        ItemKind::AdvancedVat => "advanced_vat",
        ItemKind::ScrapLantern => "scrap_lantern",
        ItemKind::Fence => "fence",
        ItemKind::Gate => "gate",
        ItemKind::Spawner => "spawner",
        ItemKind::MeatSlurry => "meat_slurry",
        ItemKind::FeralDogEmbryoPoor => "feral_dog_embryo_poor",
        ItemKind::FeralDogEmbryoFlawed => "feral_dog_embryo_flawed",
        ItemKind::FeralDogEmbryoFlawless => "feral_dog_embryo_flawless",
        ItemKind::NuclearSlimeEmbryoPoor => "nuclear_slime_embryo_poor",
        ItemKind::NuclearSlimeEmbryoFlawed => "nuclear_slime_embryo_flawed",
        ItemKind::NuclearSlimeEmbryoFlawless => "nuclear_slime_embryo_flawless",
        ItemKind::StoneBrick => "stone_brick",
        ItemKind::Teleporter => "teleporter",
        ItemKind::Battery => "battery",
        ItemKind::CrudePowerCell => "crude_power_cell",
        ItemKind::StandardPowerCell => "standard_power_cell",
        ItemKind::AdvancedPowerCell => "advanced_power_cell",
        ItemKind::MobileTeleAccessDevice => "mobile_tele_access_device",
        ItemKind::Rtg => "rtg",
        ItemKind::ItemCharger => "item_charger",
        ItemKind::Riverglass => "riverglass",
        ItemKind::WheatSeeds => "wheat_seeds",
        ItemKind::Wheat => "wheat",
        ItemKind::Potato => "potato",
        ItemKind::Carrot => "carrot",
        ItemKind::Straw => "straw",
        ItemKind::Mash => "mash",
        ItemKind::Ethanol => "ethanol",
        ItemKind::Fermenter => "fermenter",
        ItemKind::AutoFeeder => "auto_feeder",
        ItemKind::AnimalCounter => "animal_counter",
        ItemKind::LivestockProcessor => "livestock_processor",
        ItemKind::BioVat => "bio_vat",
        ItemKind::BiomechEmbryoDeformed => "biomech_embryo_deformed",
        ItemKind::BiomechEmbryoWrinkly => "biomech_embryo_wrinkly",
        ItemKind::BiomechEmbryoPerfect => "biomech_embryo_perfect",
        ItemKind::BrassCartridge => "brass_cartridge",
    }
}

pub fn parse_itemkind_from_sprite_name(sprite_name: &str) -> Option<ItemKind> {
    // Parse ItemKind from sprite name (case-insensitive)
    match sprite_name.to_lowercase().as_str() {
        "log" => Some(ItemKind::Log),
        "wooden_plank" => Some(ItemKind::WoodenPlank),
        "scrap_electronics" => Some(ItemKind::ScrapElectronics),
        "diamond" => Some(ItemKind::Diamond),
        "acorn" => Some(ItemKind::Acorn),
        "stick" => Some(ItemKind::Stick),
        "nail" => Some(ItemKind::Nail),
        "stone" => Some(ItemKind::Stone),
        "string" => Some(ItemKind::String),
        "torch" => Some(ItemKind::Torch),
        "wooden_shavings" => Some(ItemKind::WoodenShavings),
        "leather" => Some(ItemKind::Leather),
        "meat" => Some(ItemKind::Meat),
        "plank_block" => Some(ItemKind::PlankBlock),
        "chest" => Some(ItemKind::Chest),
        "iron_scrap" => Some(ItemKind::IronScrap),
        "wooden_conveyor" => Some(ItemKind::WoodenConveyor),
        "burner_generator" => Some(ItemKind::BurnerGenerator),
        "power_pole" => Some(ItemKind::PowerPole),
        "auto_planter" => Some(ItemKind::AutoPlanter),
        "auto_harvester" => Some(ItemKind::AutoHarvester),
        "biochar" => Some(ItemKind::Biochar),
        "crude_furnace" => Some(ItemKind::CrudeFurnace),
        "extractor" => Some(ItemKind::Extractor),
        "filter_extractor" => Some(ItemKind::FilterExtractor),
        "belt_router" => Some(ItemKind::BeltRouter),
        "vertical_conveyor" => Some(ItemKind::VerticalConveyor),
        "stairs" => Some(ItemKind::Stairs),
        "nuclear_waste" => Some(ItemKind::NuclearWaste),
        "plasteel_scrap" => Some(ItemKind::PlasteelScrap),
        "electric_furnace" => Some(ItemKind::ElectricFurnace),
        "salvaged_arm" => Some(ItemKind::SalvagedArm),
        "crude_head" => Some(ItemKind::CrudeHead),
        "crude_torso" => Some(ItemKind::CrudeTorso),
        "crude_fusion_core" => Some(ItemKind::CrudeFusionCore),
        "crude_arm" => Some(ItemKind::CrudeArm),
        "crude_leg" => Some(ItemKind::CrudeLeg),
        "standard_head" => Some(ItemKind::StandardHead),
        "standard_torso" => Some(ItemKind::StandardTorso),
        "standard_fusion_core" => Some(ItemKind::StandardFusionCore),
        "standard_arm" => Some(ItemKind::StandardArm),
        "standard_leg" => Some(ItemKind::StandardLeg),
        "advanced_head" => Some(ItemKind::AdvancedHead),
        "advanced_torso" => Some(ItemKind::AdvancedTorso),
        "advanced_fusion_core" => Some(ItemKind::AdvancedFusionCore),
        "advanced_arm" => Some(ItemKind::AdvancedArm),
        "advanced_leg" => Some(ItemKind::AdvancedLeg),
        "crude_vat" => Some(ItemKind::CrudeVat),
        "standard_vat" => Some(ItemKind::StandardVat),
        "advanced_vat" => Some(ItemKind::AdvancedVat),
        "scrap_lantern" => Some(ItemKind::ScrapLantern),
        "fence" => Some(ItemKind::Fence),
        "gate" => Some(ItemKind::Gate),
        "spawner" => Some(ItemKind::Spawner),
        "meat_slurry" => Some(ItemKind::MeatSlurry),
        "feral_dog_embryo_poor" => Some(ItemKind::FeralDogEmbryoPoor),
        "feral_dog_embryo_flawed" => Some(ItemKind::FeralDogEmbryoFlawed),
        "feral_dog_embryo_flawless" => Some(ItemKind::FeralDogEmbryoFlawless),
        "nuclear_slime_embryo_poor" => Some(ItemKind::NuclearSlimeEmbryoPoor),
        "nuclear_slime_embryo_flawed" => Some(ItemKind::NuclearSlimeEmbryoFlawed),
        "nuclear_slime_embryo_flawless" => Some(ItemKind::NuclearSlimeEmbryoFlawless),
        "stone_brick" => Some(ItemKind::StoneBrick),
        "teleporter" => Some(ItemKind::Teleporter),
        "battery" => Some(ItemKind::Battery),
        "crude_power_cell" => Some(ItemKind::CrudePowerCell),
        "standard_power_cell" => Some(ItemKind::StandardPowerCell),
        "advanced_power_cell" => Some(ItemKind::AdvancedPowerCell),
        "mobile_tele_access_device" => Some(ItemKind::MobileTeleAccessDevice),
        "rtg" => Some(ItemKind::Rtg),
        "item_charger" => Some(ItemKind::ItemCharger),
        "riverglass" => Some(ItemKind::Riverglass),
        "wheat_seeds" => Some(ItemKind::WheatSeeds),
        "wheat" => Some(ItemKind::Wheat),
        "potato" => Some(ItemKind::Potato),
        "carrot" => Some(ItemKind::Carrot),
        "straw" => Some(ItemKind::Straw),
        "mash" => Some(ItemKind::Mash),
        "ethanol" => Some(ItemKind::Ethanol),
        "fermenter" => Some(ItemKind::Fermenter),
        "auto_feeder" => Some(ItemKind::AutoFeeder),
        "animal_counter" => Some(ItemKind::AnimalCounter),
        "livestock_processor" => Some(ItemKind::LivestockProcessor),
        "bio_vat" => Some(ItemKind::BioVat),
        "biomech_embryo_deformed" => Some(ItemKind::BiomechEmbryoDeformed),
        "biomech_embryo_wrinkly" => Some(ItemKind::BiomechEmbryoWrinkly),
        "biomech_embryo_perfect" => Some(ItemKind::BiomechEmbryoPerfect),
        "brass_cartridge" => Some(ItemKind::BrassCartridge),
        _ => None,
    }
}

pub fn get_all_item_sprite_names() -> Vec<&'static str> {
    // Get all available item sprite names for help text
    use strum::IntoEnumIterator;
    ItemKind::iter().map(itemkind_sprite_name).collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemStack {
    pub kind: ItemKind,
    pub qty: u32,
    /// A power cell's charge in joules; `None` for everything else (and an empty cell)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub charge_j: Option<u64>,
}

impl ItemStack {
    pub fn new(kind: ItemKind, qty: u32) -> Self {
        Self {
            kind,
            qty,
            charge_j: None,
        }
    }
}

/// A stack's name for lists: the item's name, with a power cell's charge after it.
pub fn stack_name(stack: &ItemStack) -> String {
    match crate::power_cells::charge_label(stack) {
        Some(charge) => format!("{} ({charge})", itemkind_name(stack.kind)),
        None => itemkind_name(stack.kind).to_string(),
    }
}

/// Whether items of this kind share a stack. Power cells don't: each keeps its own charge.
pub fn stacks(kind: ItemKind) -> bool {
    !crate::power_cells::is_cell(kind)
}

// -----------------------------
// Entity taxonomy
// -----------------------------

/// High-level kind for an entity. Prefer ECS composition for behavior; `EntityKind`
/// is a convenient category for rendering defaults and simple logic tables.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash, EnumIter)]
pub enum EntityKind {
    Player,
    Sheep,
    FeralDog,
    Corpse,
    QuestTutorialBrokenAndroid,
    NuclearSlime,
    MiningBot,
    MilitaryAndroid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    pub slots: Vec<ItemStack>,
    pub max_slots: usize,
    pub auto_pickup: bool,
}

impl Default for Inventory {
    fn default() -> Self {
        Self {
            slots: Vec::new(),
            max_slots: 20,
            auto_pickup: false,
        }
    }
}

impl Inventory {
    /// Add `qty` items. Kinds that stack join their stack; others (power cells) each take
    /// a slot of their own, as far as there are slots.
    pub fn add(&mut self, kind: ItemKind, qty: u32) {
        if qty == 0 {
            return;
        }
        if !stacks(kind) {
            for _ in 0..qty {
                self.add_stack(ItemStack::new(kind, 1));
            }
            return;
        }
        if let Some(s) = self.slots.iter_mut().find(|s| s.kind == kind) {
            s.qty = s.qty.saturating_add(qty);
            return;
        }
        if self.slots.len() < self.max_slots {
            self.slots.push(ItemStack::new(kind, qty));
        }
    }

    /// Whether one more item of `kind` fits.
    pub fn has_room_for(&self, kind: ItemKind) -> bool {
        (stacks(kind) && self.slots.iter().any(|s| s.kind == kind))
            || self.slots.len() < self.max_slots
    }

    /// Add a whole stack, keeping a power cell's charge. Returns whether it fit.
    pub fn add_stack(&mut self, stack: ItemStack) -> bool {
        if stacks(stack.kind) {
            if !self.has_room_for(stack.kind) {
                return false;
            }
            self.add(stack.kind, stack.qty);
            return true;
        }
        if self.slots.len() >= self.max_slots {
            return false;
        }
        self.slots.push(stack);
        true
    }

    /// How many items of `kind` are carried.
    pub fn count(&self, kind: ItemKind) -> u32 {
        self.slots
            .iter()
            .filter(|s| s.kind == kind)
            .map(|s| s.qty)
            .sum()
    }

    /// Take up to `qty` items of `kind`, emptying stacks in order; returns how many.
    pub fn remove(&mut self, kind: ItemKind, qty: u32) -> u32 {
        let mut left = qty;
        for stack in self.slots.iter_mut().filter(|s| s.kind == kind) {
            let n = stack.qty.min(left);
            stack.qty -= n;
            left -= n;
        }
        self.slots.retain(|s| s.qty > 0);
        qty - left
    }

    /// Take one item from stack `slot`, with its charge if it is a cell.
    pub fn take_one(&mut self, slot: usize) -> Option<ItemStack> {
        let stack = self.slots.get_mut(slot).filter(|s| s.qty > 0)?;
        stack.qty -= 1;
        let taken = ItemStack { qty: 1, ..*stack };
        if stack.qty == 0 {
            self.slots.remove(slot);
        }
        Some(taken)
    }
}

/// World entity representing an item lying on the ground
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DroppedItem {
    pub kind: ItemKind,
    pub qty: u32,
}

/// Data-driven sprite reference for renderers to pick correct art
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpriteRef {
    pub category: String,
    pub name: String,
}

/// Marker component for entities that are dead
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Dead;

/// Quest-giving NPC marker
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct QuestTesty;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct QuestTutorialBrokenAndroid;

/// NPC mood states for dialogue system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NPCMood {
    Happy,
    Sad,
    Neutral,
    Weird,
}

/// Component for NPCs that can engage in dialogue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dialogue {
    pub current_mood: NPCMood,
    pub met_before: bool,
    pub current_dialogue_id: Option<DialogueNodeID>,
    pub name: String,
}

impl SpriteRef {
    pub fn new(category: &str, name: &str) -> Self {
        Self {
            category: category.to_string(),
            name: name.to_string(),
        }
    }
}

/// Fog of war component tracking which tiles the player has visited and what's currently illuminated
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FogOfWar {
    /// Set of (x, y, z) positions that have been visited by the player
    pub visited_tiles: std::collections::BTreeSet<(i64, i64, i64)>,
}

impl FogOfWar {
    pub fn mark_visited(&mut self, x: i64, y: i64, z: i64) {
        self.visited_tiles.insert((x, y, z));
    }

    pub fn is_visited(&self, x: i64, y: i64, z: i64) -> bool {
        self.visited_tiles.contains(&(x, y, z))
    }
}

/// Light source component for entities that emit light
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LightSource {
    pub radius: u32,
    pub torch_equipped: bool,
}

impl Default for LightSource {
    fn default() -> Self {
        Self {
            radius: 2, // Default player light radius
            torch_equipped: false,
        }
    }
}

impl LightSource {
    pub fn new(radius: u32) -> Self {
        Self {
            radius,
            torch_equipped: false,
        }
    }

    pub fn set_torch_equipped(&mut self, equipped: bool) {
        self.torch_equipped = equipped;
        self.radius = if equipped { 8 } else { 2 };
    }
}

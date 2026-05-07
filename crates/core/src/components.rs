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

#[derive(Debug, Clone, Copy)]
pub struct Player;

/// Marker for all in-world game entities (parents Player, Sheep, etc.)
#[derive(Debug, Clone, Copy)]
pub struct GameEntity;

/// Renderable glyph for ASCII views
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Glyph(pub char);

/// Marker for a simple wandering sheep
#[derive(Debug, Clone, Copy)]
pub struct Sheep;

/// Marker for a simple wandering feral dog
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FeralDog;

/// AI behavior state for feral dogs
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DogAI {
    pub behavior: crate::pathfinding::DogBehavior,
    pub behavior_timer: u64, // Ticks remaining in current behavior
    pub circle_center: Option<Position>, // Center for circular movement
    pub steps_taken: u32,    // Steps taken in current behavior (reset on behavior change)
}

/// Component for entities that can engage in combat
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Combat {
    pub triggered: bool,
}

impl Default for Combat {
    fn default() -> Self {
        Self { triggered: false }
    }
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
#[derive(Debug, Clone, Copy)]
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
#[derive(Debug, Clone, Copy)]
pub struct BlocksMovement;

// Body lives in `crate::model::body` now.

// -----------------------------
// Items & Inventory Components
// -----------------------------

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash, EnumIter)]
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
        _ => None,
    }
}

pub fn get_all_item_sprite_names() -> Vec<&'static str> {
    // Get all available item sprite names for help text
    use strum::IntoEnumIterator;
    ItemKind::iter().map(itemkind_sprite_name).collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemStack {
    pub kind: ItemKind,
    pub qty: u32,
}

impl ItemStack {
    pub fn new(kind: ItemKind, qty: u32) -> Self {
        Self { kind, qty }
    }
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
    pub fn add(&mut self, kind: ItemKind, qty: u32) {
        if qty == 0 {
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
#[derive(Debug, Clone, Copy)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FogOfWar {
    /// Set of (x, y, z) positions that have been visited by the player
    pub visited_tiles: std::collections::HashSet<(i64, i64, i64)>,
}

impl Default for FogOfWar {
    fn default() -> Self {
        Self {
            visited_tiles: std::collections::HashSet::new(),
        }
    }
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

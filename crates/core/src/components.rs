use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
    pub z: i32,
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
#[derive(Debug, Clone, Copy)]
pub struct FeralDog;

/// Marker for entities that block movement
#[derive(Debug, Clone, Copy)]
pub struct BlocksMovement;

// Body lives in `crate::model::body` now.

// -----------------------------
// Items & Inventory Components
// -----------------------------

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum ItemKind {
    Log,
    WoodenPlank,
    Acorn,
    Stick,
    Nail,
    Stone,
    IronOre,
    String,
    StoneAxe,
    StonePickaxe,
    Torch,
    Rope,
    IronBar,
    WoodenShavings,
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
        ItemKind::IronOre => "Iron Ore",
        ItemKind::String => "String",
        ItemKind::StoneAxe => "Stone Axe",
        ItemKind::StonePickaxe => "Stone Pickaxe",
        ItemKind::Torch => "Torch",
        ItemKind::Rope => "Rope",
        ItemKind::IronBar => "Iron Bar",
        ItemKind::WoodenShavings => "Wooden Shavings",
    }
}

pub fn itemkind_sprite_name(kind: ItemKind) -> &'static str {
    // Corresponds to the folder within assets/sprites/items
    match kind {
        ItemKind::Log => "log",
        ItemKind::WoodenPlank => "wooden_plank",
        ItemKind::Acorn => "acorn",
        ItemKind::Stick => "stick",
        ItemKind::Nail => "nail",
        ItemKind::Stone => "stone",
        ItemKind::IronOre => "ironore",
        ItemKind::String => "string",
        ItemKind::StoneAxe => "stoneaxe",
        ItemKind::StonePickaxe => "stonepickaxe",
        ItemKind::Torch => "torch",
        ItemKind::Rope => "rope",
        ItemKind::IronBar => "ironbar",
        ItemKind::WoodenShavings => "wooden_shavings",
    }
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
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum EntityKind {
    Player,
    Sheep,
    FeralDog,
    Bot,
    Turret,
    Chest,
    Rock,
    Corpse,
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

impl SpriteRef {
    pub fn new(category: &str, name: &str) -> Self {
        Self {
            category: category.to_string(),
            name: name.to_string(),
        }
    }
}

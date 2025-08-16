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

/// Marker for entities that block movement
#[derive(Debug, Clone, Copy)]
pub struct BlocksMovement;

// Body lives in `crate::model::body` now.

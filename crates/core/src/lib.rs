pub mod blueprints;
pub mod combat_state_manager;
pub mod component_access;
pub mod components;
pub mod config;
pub mod crafting;
pub mod death;
pub mod debug_worlds;
pub mod default_config;
pub mod dialogue;
pub mod dialogue_database;
pub mod factory;
pub mod furnace;
pub mod game;
pub mod game_config;
pub mod game_events;
pub mod game_time;
pub mod intent;
pub mod keycode_mapping;
pub mod logistics;
pub mod machines;
pub mod message_log;
pub mod model;
pub mod moves;
pub mod palettekey;
pub mod pathfinding;
pub mod plants;
pub mod player_state;
pub mod power;
pub mod recipe_handler;
pub mod resources;
pub mod save_load;
mod serde_pairs;
pub mod showcase;
pub mod spawn_utils;
pub mod stairs;
pub mod structure;
pub mod structure_lint;
pub mod structure_plan;
pub mod system_scheduler;
pub mod systems;
#[cfg(test)]
mod systems_tests;
pub mod target_tracker;
#[cfg(any(test, feature = "test-support"))]
pub mod test_support;
pub mod tile_registry;
pub mod tiles;
pub mod view;
pub mod world;
pub mod world_state;
#[cfg(test)]
mod worldgen_tests;

// Re-export commonly used types
pub use game::Game;
pub use resources::Resources;
pub use tiles::TileKind;

pub mod combat_state_manager;
pub mod component_access;
pub mod components;
pub mod config;
pub mod default_config;
pub mod dialogue;
pub mod dialogue_database;
pub mod game;
pub mod game_config;
pub mod game_events;
pub mod game_time;
pub mod intent;
pub mod keycode_mapping;
pub mod message_log;
pub mod model;
pub mod moves;
pub mod palettekey;
pub mod pathfinding;
pub mod player_state;
pub mod recipe_handler;
pub mod resources;
pub mod save_load;
pub mod spawn_utils;
pub mod structure;
pub mod system_scheduler;
pub mod systems;
#[cfg(test)]
mod systems_tests;
pub mod target_tracker;
pub mod tile_registry;
pub mod tiles;
pub mod view;
pub mod world;
pub mod world_state;

// Re-export commonly used types
pub use game::Game;
pub use resources::Resources;
pub use tiles::TileKind;

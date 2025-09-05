pub mod components;
pub mod config;
pub mod default_config;
pub mod game;
pub mod game_config;
pub mod game_time;
pub mod intent;
pub mod keycode_mapping;
pub mod message_log;
pub mod model;
pub mod moves;
pub mod palettekey;
pub mod player_state;
pub mod recipe_handler;
pub mod resources;
pub mod save_load;
pub mod structure;
pub mod system_scheduler;
pub mod systems;
pub mod tiles;
pub mod view;
pub mod world;
pub mod world_state;

// Re-export commonly used types
pub use game::Game;
pub use resources::Resources;
pub use tiles::TileKind;

// Re-export hecs Entity for client access
pub use hecs::Entity;

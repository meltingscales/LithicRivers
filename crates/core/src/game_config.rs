use crate::config::ConfigManager;

/// Config service that caches commonly-used settings and provides clean API
#[derive(Debug)]
pub struct GameConfig {
    config: ConfigManager,
    // Cached settings for performance
    pub developer_mode: bool,
    pub player_name: String,
    pub default_player_position: [i32; 3],
    pub auto_pickup_enabled: bool,
}

impl GameConfig {
    pub fn new() -> Self {
        let cfg = ConfigManager::new();

        // Cache commonly-used settings

        let developer_mode = cfg
            .get_setting("game", "DEVELOPER_MODE")
            .and_then(|v| v.as_bool())
            .unwrap_or_else(|| {
                panic!(
                    "game.DEVELOPER_MODE must be set. Raw value: {:?}",
                    cfg.get_setting("game", "DEVELOPER_MODE")
                )
            });

        let player_name = cfg
            .get_setting("game", "DEFAULT_PLAYER_NAME")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| {
                panic!(
                    "game.DEFAULT_PLAYER_NAME must be set. Raw value: {:?}",
                    cfg.get_setting("game", "DEFAULT_PLAYER_NAME")
                )
            })
            .to_string();

        let default_player_position =
            cfg.get_vector_setting("world", "DEFAULT_PLAYER_POSITION", "production");

        let auto_pickup_enabled = cfg
            .get_setting("inventory", "TOGGLE_ITEM_AUTO_PICKUP_DEFAULT_ENABLED")
            .and_then(|v| v.as_bool())
            .unwrap_or_else(|| {
                panic!(
                "inventory.TOGGLE_ITEM_AUTO_PICKUP_DEFAULT_ENABLED must be set. Raw value: {:?}", 
                cfg.get_setting("inventory", "TOGGLE_ITEM_AUTO_PICKUP_DEFAULT_ENABLED")
            )
            });

        Self {
            config: cfg,
            developer_mode,
            player_name,
            default_player_position,
            auto_pickup_enabled,
        }
    }

    /// Clean API for config access without double-nesting
    pub fn get_setting(&self, category: &str, key: &str) -> Option<&serde_json::Value> {
        self.config.get_setting(category, key)
    }

    /// Get vector setting with clean API
    pub fn get_vector_setting(&self, category: &str, key: &str, environment: &str) -> [i32; 3] {
        self.config.get_vector_setting(category, key, environment)
    }

    /// Get config source label for debugging
    pub fn source_label(&self) -> String {
        self.config.source_label()
    }
}

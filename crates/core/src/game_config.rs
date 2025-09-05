use crate::config::ConfigManager;

#[derive(Debug)]
pub struct GameConfig {
    pub config: ConfigManager,
    pub developer_mode: bool,
    pub player_name: String,
}

impl GameConfig {
    pub fn new() -> Self {
        let cfg = ConfigManager::new();
        let developer_mode = cfg
            .get_setting("game", "DEVELOPER_MODE")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let player_name = cfg
            .get_setting("game", "DEFAULT_PLAYER_NAME")
            .and_then(|v| v.as_str())
            .unwrap_or("Player")
            .to_string();
        Self {
            config: cfg,
            developer_mode,
            player_name,
        }
    }
}

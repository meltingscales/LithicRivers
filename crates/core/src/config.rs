use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigRoot {
    pub keybinds: serde_json::Value,
    pub settings: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigSourceKind {
    ExternalFile,
    EmbeddedDefaults,
}

#[derive(Debug, Clone)]
pub struct ConfigManager {
    pub data: ConfigRoot,
    pub file_path: PathBuf,
    pub source: ConfigSourceKind,
}

impl ConfigManager {
    pub fn new() -> Self {
        let (file_path, _dir) = Self::resolve_config_path();
        let (data, source) = match Self::load_from_file(&file_path) {
            Ok(d) => (d, ConfigSourceKind::ExternalFile),
            Err(_) => {
                // Create with defaults
                let defaults = Self::default_config();
                if let Some(parent) = file_path.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let _ = fs::write(&file_path, serde_json::to_string_pretty(&defaults).unwrap());
                (defaults, ConfigSourceKind::EmbeddedDefaults)
            }
        };
        Self {
            data,
            file_path,
            source,
        }
    }

    fn resolve_config_path() -> (PathBuf, PathBuf) {
        // Prefer directory adjacent to the executable: <exe_dir>/config/lithicrivers-config.json
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        let config_dir = exe_dir.join("config");
        let file_path = config_dir.join("lithicrivers-config.json");
        (file_path, config_dir)
    }

    fn load_from_file(path: &Path) -> anyhow::Result<ConfigRoot> {
        let s = fs::read_to_string(path)?;
        let data: serde_json::Value = serde_json::from_str(&s)?;
        if !data.is_object() {
            anyhow::bail!("config root is not an object");
        }
        let keybinds = data
            .get("keybinds")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));
        let settings = data
            .get("settings")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));
        Ok(ConfigRoot { keybinds, settings })
    }

    pub fn save(&self) -> anyhow::Result<()> {
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let s = serde_json::to_string_pretty(&self.data)?;
        fs::write(&self.file_path, s)?;
        Ok(())
    }

    pub fn source_label(&self) -> String {
        match self.source {
            ConfigSourceKind::ExternalFile => {
                format!("External file: {}", self.file_path.to_string_lossy())
            }
            ConfigSourceKind::EmbeddedDefaults => {
                "Embedded defaults (created file if missing)".to_string()
            }
        }
    }

    pub fn get_setting<'a>(&'a self, category: &str, key: &str) -> Option<&'a serde_json::Value> {
        self.data.settings.get(category).and_then(|c| c.get(key))
    }

    pub fn get_vector_setting(&self, category: &str, key: &str, environment: &str) -> [i32; 3] {
        let v = self.data.settings.get(category).and_then(|c| c.get(key));
        if let Some(obj) = v.and_then(|v| v.as_object()) {
            if let Some(arr) = obj.get(environment).and_then(|x| x.as_array()) {
                return Self::vec3_from_json(arr);
            }
        } else if let Some(arr) = v.and_then(|v| v.as_array()) {
            return Self::vec3_from_json(arr);
        }
        [0, 0, 0]
    }

    fn vec3_from_json(arr: &[serde_json::Value]) -> [i32; 3] {
        let mut out = [0i32; 3];
        for (i, v) in arr.iter().take(3).enumerate() {
            if let Some(n) = v.as_i64() {
                out[i] = n as i32;
            }
        }
        out
    }

    fn default_config() -> ConfigRoot {
        // Match python-old defaults
        let keybinds = serde_json::json!({
            "movement": {
                "MOVE_NORTHWEST": ["NUMPAD_7"],
                "MOVE_NORTH": ["NUMPAD_8"],
                "MOVE_NORTHEAST": ["NUMPAD_9"],
                "MOVE_WEST": ["NUMPAD_4"],
                "WAIT": ["NUMPAD_5"],
                "MOVE_EAST": ["NUMPAD_6"],
                "MOVE_SOUTHWEST": ["NUMPAD_1"],
                "MOVE_SOUTH": ["NUMPAD_2"],
                "MOVE_SOUTHEAST": ["NUMPAD_3"],
                "MOVE_UP": ["q"],
                "MOVE_DOWN": ["e"]
            },
            "viewport": {
                "RESET_VIEWPORT": ["r"],
                "SLIDE_VIEWPORT_WEST": ["["],
                "SLIDE_VIEWPORT_EAST": ["]"],
                "TOGGLE_VIEWPORT": ["v"],
                "VIEW_Z_UP": ["PAGEUP"],
                "VIEW_Z_DOWN": ["PAGEDOWN"]
            },
            "scale": {"SCALE_UP": ["=", "+"], "SCALE_DOWN": ["-"], "SCALE_RESET": ["0"]},
            "action": {"MINE": ["u"], "INTERACT": ["i"], "PICKUP_ITEMS": ["g"]},
            "ui": {
                "CLOSE_HELP_MENU": ["ESCAPE"],
                "OPEN_COMMAND_MENU": ["/"],
                "MENU_ACTIVATE": ["ENTER", "SPACE"],
                "MENU_PREV": ["LEFT"],
                "MENU_NEXT": ["RIGHT"],
                "CREDITS_SCROLL_UP": ["UP"],
                "CREDITS_SCROLL_DOWN": ["DOWN"],
                "SAVE_JSON": ["S"],
                "LOAD_JSON": ["L"],
                "QUIT": ["q"]
            },
            "inventory": {"DROP_ITEM": ["d"], "DESTROY_ITEM": ["x"], "CHEAT_DUPLICATE_ITEM": ["."]}
        });
        let settings = serde_json::json!({
            "game": {
                "GAME_NAME": "LithicRivers",
                "LOGFILENAME": "LithicRivers.log",
                "LOGGINGLEVEL": "INFO",
                "SAVES_FOLDER": "lithicrivers-saves/saves",
                "SNAPSHOTS_FOLDER": "lithicrivers-saves/snapshots",
                "DEVELOPER_MODE": true,
                "DEFAULT_SEED": 4669201609u64,
                "DEFAULT_PLAYER_NAME": "melty"
            },
            "world": {
                "DEFAULT_SIZE_RADIUS": {"production": [50, 50, 3], "testing": [3, 3, 1]},
                "DEFAULT_PLAYER_POSITION": {"production": [25, 25, 0], "testing": [0, 0, 0]}
            },
            "viewport": {"VIEWPORT_RADIUS": [8, 8, 0], "VIEWPORT_WIGGLE": 2},
            "performance": {"MAX_CPU_THREADS": 64},
            "worldgen": {"CHUNK_SIZE": 16}
        });
        ConfigRoot { keybinds, settings }
    }
}

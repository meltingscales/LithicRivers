use crate::default_config::default_config;
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
                let defaults = default_config();
                if let Some(parent) = file_path.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let _ = fs::write(&file_path, serde_json::to_string_pretty(&defaults).unwrap());
                (defaults, ConfigSourceKind::EmbeddedDefaults)
            }
        };

        tracing::info!(target: "game", "loaded config: {:?}", data);
        tracing::info!(target: "game", "config source: {:?}", source);
        tracing::info!(target: "game", "config file path: {:?}", file_path);

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

    pub fn get_keybind(&self, category: &str, key: &str) -> Option<&serde_json::Value> {
        self.data.keybinds.get(category).and_then(|c| c.get(key))
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
}

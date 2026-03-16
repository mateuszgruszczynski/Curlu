use serde::{Deserialize, Serialize};
use std::path::PathBuf;

fn config_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("rttp");
    std::fs::create_dir_all(&path).ok();
    path.push("settings.json");
    path
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Settings {
    pub default_directory: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            default_directory: None,
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        std::fs::read_to_string(config_path())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(config_path(), json);
        }
    }
}

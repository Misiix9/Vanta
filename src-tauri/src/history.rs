use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct History {
    pub usage: HashMap<String, u32>,
    #[serde(skip)]
    file_path: Option<PathBuf>,
}

impl History {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load_or_create(config_dir: &Path) -> Self {
        let file_path = config_dir.join("vanta_history.json");
        
        let mut history = if file_path.exists() {
            fs::read_to_string(&file_path)
                .ok()
                .and_then(|content| serde_json::from_str(&content).ok())
                .unwrap_or_default()
        } else {
            Self::default()
        };
        
        history.file_path = Some(file_path);
        history
    }

    pub fn increment(&mut self, exec: &str) {
        *self.usage.entry(exec.to_string()).or_insert(0) += 1;
        self.save();
    }

    pub fn get_usage(&self, exec: &str) -> u32 {
        *self.usage.get(exec).unwrap_or(&0)
    }

    fn save(&self) {
        if let Some(path) = &self.file_path {
            if let Ok(content) = serde_json::to_string_pretty(self) {
                let _ = fs::write(path, content);
            }
        }
    }
}

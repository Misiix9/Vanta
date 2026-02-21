use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct History {
    pub usage: HashMap<String, u32>,
    #[serde(skip)]
    file_path: Option<PathBuf>,
    #[serde(skip)]
    dirty_count: u32,
    #[serde(skip)]
    last_save_at: Option<Instant>,
}

impl History {
    pub fn new() -> Self {
        Self {
            usage: HashMap::new(),
            file_path: None,
            dirty_count: 0,
            last_save_at: Some(Instant::now()),
        }
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
        history.dirty_count = 0;
        history.last_save_at = Some(Instant::now());
        history
    }

    pub fn increment(&mut self, exec: &str) {
        *self.usage.entry(exec.to_string()).or_insert(0) += 1;
        self.dirty_count = self.dirty_count.saturating_add(1);

        let should_flush_by_count = self.dirty_count >= 20;
        let should_flush_by_time = self
            .last_save_at
            .map(|t| t.elapsed() >= Duration::from_secs(2))
            .unwrap_or(true);

        if should_flush_by_count || should_flush_by_time {
            self.save();
            self.dirty_count = 0;
            self.last_save_at = Some(Instant::now());
        }
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

impl Drop for History {
    fn drop(&mut self) {
        if self.dirty_count > 0 {
            self.save();
            self.dirty_count = 0;
        }
    }
}

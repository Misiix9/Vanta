use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Half-life in seconds for frecency decay (12 hours).
const HALF_LIFE_SECS: f64 = 43200.0;

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// A single access record used for frecency computation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrecencyEntry {
    /// Total number of accesses (kept for backward compat / debugging).
    pub count: u32,
    /// Unix timestamps (seconds) of recent accesses, capped at 10.
    pub timestamps: Vec<u64>,
}

impl FrecencyEntry {
    fn new(ts: u64) -> Self {
        Self {
            count: 1,
            timestamps: vec![ts],
        }
    }

    fn record_access(&mut self, ts: u64) {
        self.count = self.count.saturating_add(1);
        self.timestamps.push(ts);
        // Keep only the 10 most recent timestamps.
        if self.timestamps.len() > 10 {
            self.timestamps.sort_unstable();
            let excess = self.timestamps.len() - 10;
            self.timestamps.drain(..excess);
        }
    }

    /// Compute frecency score using exponential decay over access timestamps.
    pub fn frecency(&self, now: u64) -> f64 {
        self.timestamps
            .iter()
            .map(|&ts| {
                let age = now.saturating_sub(ts) as f64;
                (-age * (2.0_f64.ln()) / HALF_LIFE_SECS).exp()
            })
            .sum()
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct History {
    /// Frecency entries keyed by exec string.
    pub entries: HashMap<String, FrecencyEntry>,
    /// Legacy field — only present when migrating old data.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub usage: HashMap<String, u32>,
    /// Recent search queries for recall (most-recent first, capped at 50).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recent_queries: Vec<String>,
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
            entries: HashMap::new(),
            usage: HashMap::new(),
            recent_queries: Vec::new(),
            file_path: None,
            dirty_count: 0,
            last_save_at: Some(Instant::now()),
        }
    }

    pub fn load_or_create(config_dir: &Path) -> Self {
        let file_path = config_dir.join("vanta_history.json");

        let mut history: History = if file_path.exists() {
            fs::read_to_string(&file_path)
                .ok()
                .and_then(|content| serde_json::from_str(&content).ok())
                .unwrap_or_default()
        } else {
            Self::default()
        };

        // Migrate legacy `usage` counts into frecency entries.
        if !history.usage.is_empty() && history.entries.is_empty() {
            let now = now_secs();
            for (exec, count) in &history.usage {
                let entry = FrecencyEntry {
                    count: *count,
                    // Seed with a single "now" timestamp so migrated items
                    // don't start at zero frecency.
                    timestamps: vec![now],
                };
                history.entries.insert(exec.clone(), entry);
            }
            history.usage.clear();
        }

        history.file_path = Some(file_path);
        history.dirty_count = 0;
        history.last_save_at = Some(Instant::now());
        history
    }

    pub fn increment(&mut self, exec: &str) {
        let ts = now_secs();
        self.entries
            .entry(exec.to_string())
            .and_modify(|e| e.record_access(ts))
            .or_insert_with(|| FrecencyEntry::new(ts));

        self.dirty_count = self.dirty_count.saturating_add(1);
        self.flush_if_needed();
    }

    /// Return a frecency score for the given exec, suitable for ranking.
    /// Returned as u32 (frecency × 1000, clamped) for parity with old API.
    pub fn get_usage(&self, exec: &str) -> u32 {
        match self.entries.get(exec) {
            Some(entry) => {
                let f = entry.frecency(now_secs()) * 1000.0;
                (f as u64).min(u32::MAX as u64) as u32
            }
            None => 0,
        }
    }

    /// Build a snapshot map of exec → frecency-score for bulk lookups.
    pub fn usage_map(&self) -> HashMap<String, u32> {
        let now = now_secs();
        self.entries
            .iter()
            .map(|(k, v)| {
                let f = v.frecency(now) * 1000.0;
                (k.clone(), (f as u64).min(u32::MAX as u64) as u32)
            })
            .collect()
    }

    /// Record a search query for later recall.  Deduplicates and caps at 50.
    pub fn push_query(&mut self, query: &str) {
        let q = query.trim().to_string();
        if q.is_empty() {
            return;
        }
        // Remove duplicate if already present.
        self.recent_queries.retain(|existing| existing != &q);
        self.recent_queries.insert(0, q);
        if self.recent_queries.len() > 50 {
            self.recent_queries.truncate(50);
        }
        self.dirty_count = self.dirty_count.saturating_add(1);
        self.flush_if_needed();
    }

    pub fn get_recent_queries(&self) -> &[String] {
        &self.recent_queries
    }

    fn flush_if_needed(&mut self) {
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

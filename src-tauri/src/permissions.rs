use crate::config;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::Emitter;

const MAX_BLOCK_EVENTS: usize = 200;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Capability {
    Network,
    Shell,
    Filesystem,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Decision {
    Allow,
    Deny,
    Ask,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRecord {
    pub script_id: String,
    pub decision: Decision,
    pub timestamp: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(default)]
    pub requested_caps: Vec<Capability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockEvent {
    pub script_id: String,
    pub capability: Capability,
    pub timestamp: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionsStore {
    #[serde(default)]
    pub decisions: HashMap<String, DecisionRecord>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub block_events: Vec<BlockEvent>,
}

impl Default for PermissionsStore {
    fn default() -> Self {
        Self {
            decisions: HashMap::new(),
            block_events: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionDecisionResponse {
    pub decision: Decision,
    pub missing_caps: Vec<Capability>,
}

pub fn permissions_path() -> PathBuf {
    config::config_dir().join("permissions.json")
}

fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

pub fn load_permissions() -> PermissionsStore {
    load_permissions_raw().unwrap_or_else(|err| {
        log::warn!("Falling back to empty permissions store: {}", err);
        PermissionsStore::default()
    })
}

fn load_permissions_raw() -> Result<PermissionsStore, String> {
    let path = permissions_path();
    if !path.exists() {
        return Ok(PermissionsStore::default());
    }

    let contents = fs::read_to_string(&path)
        .map_err(|e| format!("Could not read permissions.json: {}", e))?;

    if contents.trim().is_empty() {
        log::warn!("permissions.json is empty; resetting to defaults");
        return Ok(PermissionsStore::default());
    }

    let store = serde_json::from_str::<PermissionsStore>(&contents)
        .map_err(|e| format!("Invalid permissions.json: {}", e))?;

    Ok(store)
}

pub fn save_permissions(store: &PermissionsStore) -> Result<(), String> {
    let path = permissions_path();
    let dir = path
        .parent()
        .map(PathBuf::from)
        .ok_or_else(|| "Invalid permissions path".to_string())?;

    fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create permissions dir {}: {}", dir.display(), e))?;

    let tmp_path = path.with_extension("json.tmp");
    let json = serde_json::to_string_pretty(store)
        .map_err(|e| format!("Failed to serialize permissions: {}", e))?;

    fs::write(&tmp_path, json)
        .map_err(|e| format!("Failed to write temp permissions file: {}", e))?;

    fs::rename(&tmp_path, &path)
        .map_err(|e| format!("Failed to replace permissions.json: {}", e))?;

    Ok(())
}

/// Ensure new scripts have a seed decision so capability prompts work immediately.
/// Only inserts when no decision exists; it never overwrites user choices.
pub fn seed_missing_decisions(seeds: &[(String, Vec<Capability>)]) -> Result<(), String> {
    if seeds.is_empty() {
        return Ok(());
    }

    let mut store = load_permissions_raw()?;
    let mut updated = false;

    for (script_id, caps) in seeds {
        if caps.is_empty() {
            continue;
        }

        if store.decisions.contains_key(script_id) {
            continue;
        }

        let record = DecisionRecord {
            script_id: script_id.clone(),
            decision: Decision::Ask,
            timestamp: now_millis(),
            note: None,
            requested_caps: caps.clone(),
        };

        store.decisions.insert(script_id.clone(), record);
        updated = true;
    }

    if updated {
        save_permissions(&store)?;
    }

    Ok(())
}

pub fn set_decision(
    script_id: &str,
    decision: Decision,
    note: Option<String>,
    requested_caps: Vec<Capability>,
) -> Result<DecisionRecord, String> {
    let mut store = load_permissions_raw()?;
    let record = DecisionRecord {
        script_id: script_id.to_string(),
        decision: decision.clone(),
        timestamp: now_millis(),
        note,
        requested_caps,
    };
    store.decisions.insert(script_id.to_string(), record.clone());
    save_permissions(&store)?;
    Ok(record)
}

pub fn record_block_event(
    script_id: &str,
    capability: Capability,
    note: Option<String>,
) -> Result<(), String> {
    let mut store = load_permissions_raw()?;
    let event = BlockEvent {
        script_id: script_id.to_string(),
        capability,
        timestamp: now_millis(),
        note,
    };
    store.block_events.push(event);
    if store.block_events.len() > MAX_BLOCK_EVENTS {
        let excess = store.block_events.len().saturating_sub(MAX_BLOCK_EVENTS);
        store.block_events.drain(0..excess);
    }
    save_permissions(&store)
}

pub fn get_decision_for(script_id: &str, requested_caps: &[Capability]) -> PermissionDecisionResponse {
    let store = load_permissions();
    let record = store.decisions.get(script_id);
    let decision = record.map(|r| r.decision.clone()).unwrap_or(Decision::Ask);

    // If previously allowed, only skip prompt for capabilities already approved.
    let missing_caps = match decision {
        Decision::Allow => {
            if let Some(rec) = record {
                requested_caps
                    .iter()
                    .filter(|c| !rec.requested_caps.contains(c))
                    .cloned()
                    .collect()
            } else {
                requested_caps.to_vec()
            }
        }
        Decision::Deny => Vec::new(),
        Decision::Ask => requested_caps.to_vec(),
    };

    PermissionDecisionResponse {
        decision,
        missing_caps,
    }
}

#[tauri::command]
pub fn get_permission_decision(
    script_id: String,
    requested_caps: Vec<Capability>,
) -> Result<PermissionDecisionResponse, String> {
    Ok(get_decision_for(&script_id, &requested_caps))
}

#[tauri::command]
pub fn set_permission_decision(
    app_handle: tauri::AppHandle,
    script_id: String,
    requested_caps: Vec<Capability>,
    decision: Decision,
    note: Option<String>,
) -> Result<DecisionRecord, String> {
    let record = set_decision(&script_id, decision, note, requested_caps)?;
    let _ = app_handle.emit("permissions-updated", &record);
    Ok(record)
}

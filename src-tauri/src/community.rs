use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::{Emitter, State};

use crate::config::{self, AppearanceConfig, ProfileConfig, WindowConfig, WorkflowMacro};
use crate::errors::VantaError;
use crate::permissions::Capability;
use crate::AppState;

const COMMUNITY_SCHEMA_VERSION: u32 = 1;
const MAX_MESSAGE_LEN: usize = 2000;
const MAX_CONTACT_LEN: usize = 200;
const MAX_TOPIC_LEN: usize = 64;
const MAX_ENTRIES: usize = 1000;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommunityTrustMetadata {
    pub source: String,
    pub publisher: String,
    pub verified: bool,
    pub risk_level: String,
    #[serde(default)]
    pub signature_hint: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PopularWorkflowFeedEntry {
    pub id: String,
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub trust: CommunityTrustMetadata,
    pub workflow: WorkflowMacro,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThemeSnippetData {
    pub appearance: AppearanceConfig,
    pub window: WindowConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CommunitySnippetPayload {
    pub schema_version: u32,
    pub created_at: i64,
    pub kind: String,
    pub name: String,
    pub trust: CommunityTrustMetadata,
    pub payload: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommunityFeedbackSummary {
    pub schema_version: u32,
    pub updated_at: i64,
    pub total_feedback: usize,
    pub top_votes: Vec<CommunityVoteCount>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommunityVoteCount {
    pub topic: String,
    pub votes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommunityFeedbackReceipt {
    pub id: String,
    pub created_at: i64,
    pub total_feedback: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CommunityFeedbackStore {
    #[serde(default = "default_schema")]
    schema_version: u32,
    #[serde(default)]
    updated_at: i64,
    #[serde(default)]
    entries: Vec<CommunityFeedbackEntry>,
    #[serde(default)]
    votes: HashMap<String, u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CommunityFeedbackEntry {
    id: String,
    created_at: i64,
    message: String,
    #[serde(default)]
    contact: Option<String>,
    #[serde(default)]
    roadmap_topic: Option<String>,
}

fn default_schema() -> u32 {
    COMMUNITY_SCHEMA_VERSION
}

impl Default for CommunityFeedbackStore {
    fn default() -> Self {
        Self {
            schema_version: COMMUNITY_SCHEMA_VERSION,
            updated_at: now_millis(),
            entries: Vec::new(),
            votes: HashMap::new(),
        }
    }
}

fn now_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn storage_path() -> std::path::PathBuf {
    config::config_dir().join("community-feedback.json")
}

fn community_default_trust(verified: bool) -> CommunityTrustMetadata {
    CommunityTrustMetadata {
        source: if verified {
            "vanta-community-feed".to_string()
        } else {
            "local-export".to_string()
        },
        publisher: if verified {
            "Vanta Team".to_string()
        } else {
            "Local User".to_string()
        },
        verified,
        risk_level: if verified {
            "medium".to_string()
        } else {
            "unknown".to_string()
        },
        signature_hint: if verified {
            Some("verified-by-vanta-registry".to_string())
        } else {
            None
        },
    }
}

fn workflow_has_risky_caps(workflow: &WorkflowMacro) -> bool {
    workflow.steps.iter().any(|step| match step {
        crate::config::MacroStep::Extension { capabilities, .. }
        | crate::config::MacroStep::System { capabilities, .. } => capabilities.iter().any(|cap| {
            matches!(cap, Capability::Shell | Capability::Filesystem | Capability::Network)
        }),
    })
}

fn is_valid_identifier(id: &str) -> bool {
    !id.is_empty()
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn popular_workflow_feed() -> Vec<PopularWorkflowFeedEntry> {
    vec![
        PopularWorkflowFeedEntry {
            id: "workspace-sync-quickstart".to_string(),
            title: "Workspace Sync Quickstart".to_string(),
            description: "Open a project folder then trigger extension sync with branch context.".to_string(),
            tags: vec!["git".to_string(), "productivity".to_string(), "workspace".to_string()],
            trust: community_default_trust(true),
            workflow: WorkflowMacro {
                id: "workspace-sync-quickstart".to_string(),
                name: "Workspace Sync Quickstart".to_string(),
                description: Some("Open workspace and run sync extension".to_string()),
                enabled: true,
                args: vec![
                    crate::config::MacroArg {
                        name: "project_path".to_string(),
                        description: Some("Path to project folder".to_string()),
                        required: true,
                        default_value: None,
                    },
                    crate::config::MacroArg {
                        name: "branch".to_string(),
                        description: Some("Target branch".to_string()),
                        required: false,
                        default_value: Some("main".to_string()),
                    },
                ],
                steps: vec![
                    crate::config::MacroStep::System {
                        command: "xdg-open".to_string(),
                        args: vec!["{project_path}".to_string()],
                        capabilities: vec![Capability::Filesystem],
                    },
                    crate::config::MacroStep::Extension {
                        ext_id: "sync-project".to_string(),
                        command: "sync".to_string(),
                        args: vec!["--branch".to_string(), "{branch}".to_string()],
                        capabilities: vec![Capability::Shell],
                    },
                ],
            },
        },
        PopularWorkflowFeedEntry {
            id: "focus-mode-clean-start".to_string(),
            title: "Focus Mode Clean Start".to_string(),
            description: "Launch a focused workspace start sequence with low-risk system actions.".to_string(),
            tags: vec!["focus".to_string(), "workflow".to_string()],
            trust: CommunityTrustMetadata {
                source: "vanta-community-feed".to_string(),
                publisher: "Vanta Team".to_string(),
                verified: true,
                risk_level: "low".to_string(),
                signature_hint: Some("verified-by-vanta-registry".to_string()),
            },
            workflow: WorkflowMacro {
                id: "focus-mode-clean-start".to_string(),
                name: "Focus Mode Clean Start".to_string(),
                description: Some("Quickly open an app and clear distractions".to_string()),
                enabled: true,
                args: vec![crate::config::MacroArg {
                    name: "app_exec".to_string(),
                    description: Some("Executable to open".to_string()),
                    required: true,
                    default_value: None,
                }],
                steps: vec![crate::config::MacroStep::System {
                    command: "{app_exec}".to_string(),
                    args: Vec::new(),
                    capabilities: Vec::new(),
                }],
            },
        },
    ]
}

fn sanitize_topic(topic: &str) -> Result<String, VantaError> {
    let trimmed = topic.trim().to_ascii_lowercase();
    if trimmed.is_empty() {
        return Err("Roadmap topic cannot be empty".into());
    }
    if trimmed.len() > MAX_TOPIC_LEN {
        return Err(format!("Roadmap topic cannot exceed {} characters", MAX_TOPIC_LEN).into());
    }
    if !trimmed
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
    {
        return Err("Roadmap topic can only include letters, numbers, '-' and '_'".into());
    }
    Ok(trimmed)
}

fn load_store() -> CommunityFeedbackStore {
    let path = storage_path();
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return CommunityFeedbackStore::default();
    };

    let mut parsed = serde_json::from_str::<CommunityFeedbackStore>(&raw).unwrap_or_default();
    if parsed.schema_version < COMMUNITY_SCHEMA_VERSION {
        parsed.schema_version = COMMUNITY_SCHEMA_VERSION;
    }
    parsed
}

fn save_store(store: &CommunityFeedbackStore) -> Result<(), VantaError> {
    let path = storage_path();
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("Failed to create community feedback directory: {}", e))?;
    }

    let data = serde_json::to_string_pretty(store)
        .map_err(|e| format!("Failed to serialize community feedback store: {}", e))?;

    Ok(std::fs::write(path, data).map_err(|e| format!("Failed to persist community feedback: {}", e))?)
}

#[tauri::command]
pub async fn submit_community_feedback(
    message: String,
    contact: Option<String>,
    roadmap_topic: Option<String>,
) -> Result<CommunityFeedbackReceipt, VantaError> {
    let message = message.trim().to_string();
    if message.len() < 8 {
        return Err("Feedback message must contain at least 8 characters".into());
    }
    if message.len() > MAX_MESSAGE_LEN {
        return Err(format!("Feedback message cannot exceed {} characters", MAX_MESSAGE_LEN).into());
    }

    let contact = contact
        .map(|c| c.trim().to_string())
        .filter(|c| !c.is_empty());
    if let Some(c) = &contact {
        if c.len() > MAX_CONTACT_LEN {
            return Err(format!("Contact field cannot exceed {} characters", MAX_CONTACT_LEN).into());
        }
    }

    let roadmap_topic = roadmap_topic
        .map(|topic| sanitize_topic(&topic))
        .transpose()?;

    let mut store = load_store();
    let created_at = now_millis();
    let id = format!("fb-{}-{}", created_at, store.entries.len() + 1);

    store.entries.push(CommunityFeedbackEntry {
        id: id.clone(),
        created_at,
        message,
        contact,
        roadmap_topic: roadmap_topic.clone(),
    });

    if let Some(topic) = roadmap_topic {
        *store.votes.entry(topic).or_insert(0) += 1;
    }

    if store.entries.len() > MAX_ENTRIES {
        let overflow = store.entries.len() - MAX_ENTRIES;
        store.entries.drain(0..overflow);
    }

    store.updated_at = now_millis();
    save_store(&store)?;

    Ok(CommunityFeedbackReceipt {
        id,
        created_at,
        total_feedback: store.entries.len(),
    })
}

#[tauri::command]
pub async fn vote_roadmap_item(topic: String) -> Result<CommunityVoteCount, VantaError> {
    let topic = sanitize_topic(&topic)?;

    let mut store = load_store();
    let votes = store.votes.entry(topic.clone()).or_insert(0);
    *votes += 1;
    let total = *votes;

    store.updated_at = now_millis();
    save_store(&store)?;

    Ok(CommunityVoteCount { topic, votes: total })
}

#[tauri::command]
pub async fn get_community_feedback_summary() -> Result<CommunityFeedbackSummary, VantaError> {
    let store = load_store();
    let mut vote_rows: Vec<CommunityVoteCount> = store
        .votes
        .iter()
        .map(|(topic, votes)| CommunityVoteCount {
            topic: topic.clone(),
            votes: *votes,
        })
        .collect();

    vote_rows.sort_by(|a, b| b.votes.cmp(&a.votes).then_with(|| a.topic.cmp(&b.topic)));
    vote_rows.truncate(6);

    Ok(CommunityFeedbackSummary {
        schema_version: store.schema_version,
        updated_at: store.updated_at,
        total_feedback: store.entries.len(),
        top_votes: vote_rows,
    })
}

#[tauri::command]
pub async fn get_popular_workflows_feed(
    state: State<'_, AppState>,
) -> Result<Vec<PopularWorkflowFeedEntry>, VantaError> {
    let cfg = state
        .config
        .lock()
        .map_err(|_| "Failed to access config".to_string())?;

    if !cfg.general.community_feed_opt_in {
        return Ok(Vec::new());
    }

    Ok(popular_workflow_feed())
}

#[tauri::command]
pub async fn install_popular_workflow(
    workflow_id: String,
    state: State<'_, AppState>,
) -> Result<WorkflowMacro, VantaError> {
    let feed = popular_workflow_feed();
    let selected = feed
        .into_iter()
        .find(|item| item.id == workflow_id)
        .ok_or_else(|| format!("Workflow '{}' not found in feed", workflow_id))?;

    let mut cfg = state
        .config
        .lock()
        .map_err(|_| "Failed to access config".to_string())?;

    if !cfg.general.community_feed_opt_in {
        return Err("Popular workflows feed is disabled. Enable opt-in first.".into());
    }

    if let Some(pos) = cfg
        .workflows
        .macros
        .iter()
        .position(|m| m.id == selected.workflow.id)
    {
        cfg.workflows.macros[pos] = selected.workflow.clone();
    } else {
        cfg.workflows.macros.push(selected.workflow.clone());
    }
    cfg.save()?;

    Ok(selected.workflow)
}

#[tauri::command]
pub async fn export_community_snippet(
    kind: String,
    target_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<String, VantaError> {
    let cfg = state
        .config
        .lock()
        .map_err(|_| "Failed to access config".to_string())?;

    let kind = kind.trim().to_ascii_lowercase();
    let (name, payload) = match kind.as_str() {
        "workflow" => {
            let id = target_id.ok_or_else(|| "Workflow export requires target_id".to_string())?;
            let workflow = cfg
                .workflows
                .macros
                .iter()
                .find(|m| m.id == id)
                .cloned()
                .ok_or_else(|| format!("Workflow '{}' not found", id))?;
            let payload = serde_json::to_value(workflow)
                .map_err(|e| format!("Failed to serialize workflow: {}", e))?;
            (id, payload)
        }
        "profile" => {
            let id = target_id.unwrap_or_else(|| cfg.profiles.active_profile_id.clone());
            let profile = cfg
                .profiles
                .entries
                .iter()
                .find(|p| p.id == id)
                .cloned()
                .ok_or_else(|| format!("Profile '{}' not found", id))?;
            let payload = serde_json::to_value(profile)
                .map_err(|e| format!("Failed to serialize profile: {}", e))?;
            (id, payload)
        }
        "theme" => {
            let data = ThemeSnippetData {
                appearance: cfg.appearance.clone(),
                window: cfg.window.clone(),
            };
            let payload = serde_json::to_value(data)
                .map_err(|e| format!("Failed to serialize theme snippet: {}", e))?;
            (cfg.appearance.theme.clone(), payload)
        }
        _ => {
            return Err("Unsupported snippet kind. Use workflow, profile, or theme.".into());
        }
    };

    let snippet = CommunitySnippetPayload {
        schema_version: COMMUNITY_SCHEMA_VERSION,
        created_at: now_millis(),
        kind,
        name,
        trust: community_default_trust(false),
        payload,
    };

    Ok(serde_json::to_string_pretty(&snippet)
        .map_err(|e| format!("Failed to encode snippet: {}", e))?)
}

#[tauri::command]
pub async fn import_community_snippet(
    snippet_json: String,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, VantaError> {
    let snippet: CommunitySnippetPayload = serde_json::from_str(&snippet_json)
        .map_err(|e| format!("Invalid snippet payload: {}", e))?;

    let mut cfg = state
        .config
        .lock()
        .map_err(|_| "Failed to access config".to_string())?;

    let kind = snippet.kind.trim().to_ascii_lowercase();
    let imported_name = snippet.name.clone();

    match kind.as_str() {
        "workflow" => {
            let workflow: WorkflowMacro = serde_json::from_value(snippet.payload)
                .map_err(|e| format!("Invalid workflow snippet: {}", e))?;
            if !is_valid_identifier(&workflow.id) {
                return Err("Workflow id must be alphanumeric with '-' or '_'".into());
            }
            if !snippet.trust.verified && workflow_has_risky_caps(&workflow) {
                return Err("Unverified workflow snippet contains risky capabilities; import blocked.".into());
            }

            if let Some(pos) = cfg.workflows.macros.iter().position(|m| m.id == workflow.id) {
                cfg.workflows.macros[pos] = workflow;
            } else {
                cfg.workflows.macros.push(workflow);
            }
        }
        "profile" => {
            let profile: ProfileConfig = serde_json::from_value(snippet.payload)
                .map_err(|e| format!("Invalid profile snippet: {}", e))?;
            if !is_valid_identifier(&profile.id) {
                return Err("Profile id must be alphanumeric with '-' or '_'".into());
            }

            if let Some(pos) = cfg.profiles.entries.iter().position(|p| p.id == profile.id) {
                cfg.profiles.entries[pos] = profile;
            } else {
                cfg.profiles.entries.push(profile);
            }
        }
        "theme" => {
            let mut theme: ThemeSnippetData = serde_json::from_value(snippet.payload)
                .map_err(|e| format!("Invalid theme snippet: {}", e))?;
            let _ = config::clamp_window_size(&mut theme.window);
            let _ = config::clamp_accessibility(&mut cfg.accessibility);
            cfg.appearance = theme.appearance;
            cfg.window = theme.window;
        }
        _ => {
            return Err("Unsupported snippet kind. Use workflow, profile, or theme.".into());
        }
    }

    cfg.save()?;
    let _ = app_handle.emit("config-updated", &cfg.clone());

    Ok(format!("Imported {} snippet '{}'", kind, imported_name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_topic_accepts_slug_format() {
        assert_eq!(
            sanitize_topic(" Popular-Workflows_Feed ").unwrap(),
            "popular-workflows_feed"
        );
    }

    #[test]
    fn sanitize_topic_rejects_invalid_chars() {
        let err = sanitize_topic("phase 20").unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("can only include"));
    }

    #[test]
    fn risky_capabilities_are_detected() {
        let workflow = WorkflowMacro {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: None,
            enabled: true,
            args: Vec::new(),
            steps: vec![crate::config::MacroStep::System {
                command: "echo".to_string(),
                args: vec!["hello".to_string()],
                capabilities: vec![Capability::Shell],
            }],
        };

        assert!(workflow_has_risky_caps(&workflow));
    }
}

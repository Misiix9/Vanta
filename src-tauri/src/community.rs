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
const MAX_IMPORT_HISTORY: usize = 120;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommunityTrustMetadata {
    pub source: String,
    pub publisher: String,
    pub verified: bool,
    pub risk_level: String,
    #[serde(default)]
    pub signature_hint: Option<String>,
    #[serde(default)]
    pub publisher_url: Option<String>,
    #[serde(default)]
    pub compatibility_hint: Option<String>,
    #[serde(default)]
    pub rating: Option<f32>,
    #[serde(default)]
    pub vote_count: Option<u64>,
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
pub struct CommunitySnippetPreview {
    pub schema_version: u32,
    pub kind: String,
    pub name: String,
    pub trust: CommunityTrustMetadata,
    pub target_id: Option<String>,
    pub will_replace_existing: bool,
    pub compatible: bool,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    pub badges: Vec<String>,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommunityImportHistoryEntry {
    pub id: String,
    pub created_at: i64,
    pub snippet_kind: String,
    pub snippet_name: String,
    pub target_id: Option<String>,
    pub conflict_strategy: String,
    pub trust_verified: bool,
    pub rollback_available: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CommunityImportConflictStrategy {
    Replace,
    KeepLocal,
}

impl CommunityImportConflictStrategy {
    fn as_str(self) -> &'static str {
        match self {
            Self::Replace => "replace",
            Self::KeepLocal => "keep_local",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CommunityImportHistoryStore {
    #[serde(default = "default_schema")]
    schema_version: u32,
    #[serde(default)]
    updated_at: i64,
    #[serde(default)]
    entries: Vec<CommunityImportRecord>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CommunityImportRecord {
    id: String,
    created_at: i64,
    snippet_kind: String,
    snippet_name: String,
    #[serde(default)]
    target_id: Option<String>,
    conflict_strategy: String,
    trust_verified: bool,
    rollback: CommunityImportRollback,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum CommunityImportRollback {
    Workflow {
        id: String,
        #[serde(default)]
        previous: Option<WorkflowMacro>,
    },
    Profile {
        id: String,
        #[serde(default)]
        previous: Option<ProfileConfig>,
    },
    Theme {
        previous: ThemeSnippetData,
    },
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

impl Default for CommunityImportHistoryStore {
    fn default() -> Self {
        Self {
            schema_version: COMMUNITY_SCHEMA_VERSION,
            updated_at: now_millis(),
            entries: Vec::new(),
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

fn import_history_path() -> std::path::PathBuf {
    config::config_dir().join("community-import-history.json")
}

fn parse_conflict_strategy(
    on_conflict: Option<String>,
) -> Result<CommunityImportConflictStrategy, VantaError> {
    let Some(raw) = on_conflict.map(|v| v.trim().to_ascii_lowercase()) else {
        return Ok(CommunityImportConflictStrategy::Replace);
    };
    if raw.is_empty() {
        return Ok(CommunityImportConflictStrategy::Replace);
    }
    match raw.as_str() {
        "replace" => Ok(CommunityImportConflictStrategy::Replace),
        "keep_local" | "keep-local" => Ok(CommunityImportConflictStrategy::KeepLocal),
        _ => Err("Unsupported conflict strategy. Use replace or keep_local.".into()),
    }
}

fn history_entry_from_record(record: &CommunityImportRecord) -> CommunityImportHistoryEntry {
    CommunityImportHistoryEntry {
        id: record.id.clone(),
        created_at: record.created_at,
        snippet_kind: record.snippet_kind.clone(),
        snippet_name: record.snippet_name.clone(),
        target_id: record.target_id.clone(),
        conflict_strategy: record.conflict_strategy.clone(),
        trust_verified: record.trust_verified,
        rollback_available: true,
    }
}

fn load_import_history_store() -> CommunityImportHistoryStore {
    let path = import_history_path();
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return CommunityImportHistoryStore::default();
    };

    let mut parsed = serde_json::from_str::<CommunityImportHistoryStore>(&raw).unwrap_or_default();
    if parsed.schema_version < COMMUNITY_SCHEMA_VERSION {
        parsed.schema_version = COMMUNITY_SCHEMA_VERSION;
    }
    parsed
}

fn save_import_history_store(store: &CommunityImportHistoryStore) -> Result<(), VantaError> {
    let path = import_history_path();
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("Failed to create import history directory: {}", e))?;
    }

    let data = serde_json::to_string_pretty(store)
        .map_err(|e| format!("Failed to serialize import history: {}", e))?;

    Ok(std::fs::write(path, data).map_err(|e| format!("Failed to save import history: {}", e))?)
}

fn append_import_history_record(
    snippet_kind: &str,
    snippet_name: &str,
    target_id: Option<String>,
    conflict_strategy: CommunityImportConflictStrategy,
    trust_verified: bool,
    rollback: CommunityImportRollback,
) -> Result<String, VantaError> {
    let mut store = load_import_history_store();
    let created_at = now_millis();
    let id = format!("imp-{}-{}", created_at, store.entries.len() + 1);

    store.entries.push(CommunityImportRecord {
        id: id.clone(),
        created_at,
        snippet_kind: snippet_kind.to_string(),
        snippet_name: snippet_name.to_string(),
        target_id,
        conflict_strategy: conflict_strategy.as_str().to_string(),
        trust_verified,
        rollback,
    });

    if store.entries.len() > MAX_IMPORT_HISTORY {
        let overflow = store.entries.len() - MAX_IMPORT_HISTORY;
        store.entries.drain(0..overflow);
    }

    store.updated_at = now_millis();
    save_import_history_store(&store)?;
    Ok(id)
}

fn apply_import_rollback(
    cfg: &mut config::VantaConfig,
    rollback: &CommunityImportRollback,
) -> Result<(), VantaError> {
    match rollback {
        CommunityImportRollback::Workflow { id, previous } => {
            if let Some(prev_workflow) = previous {
                if let Some(pos) = cfg.workflows.macros.iter().position(|m| &m.id == id) {
                    cfg.workflows.macros[pos] = prev_workflow.clone();
                } else {
                    cfg.workflows.macros.push(prev_workflow.clone());
                }
            } else {
                cfg.workflows.macros.retain(|m| &m.id != id);
            }
        }
        CommunityImportRollback::Profile { id, previous } => {
            if let Some(prev_profile) = previous {
                if let Some(pos) = cfg.profiles.entries.iter().position(|p| &p.id == id) {
                    cfg.profiles.entries[pos] = prev_profile.clone();
                } else {
                    cfg.profiles.entries.push(prev_profile.clone());
                }
            } else {
                cfg.profiles.entries.retain(|p| &p.id != id);
                if cfg.profiles.entries.is_empty() {
                    return Err("Rollback would remove all profiles; operation blocked.".into());
                }
                if !cfg
                    .profiles
                    .entries
                    .iter()
                    .any(|p| p.id == cfg.profiles.active_profile_id)
                {
                    cfg.profiles.active_profile_id = cfg.profiles.entries[0].id.clone();
                }
            }
        }
        CommunityImportRollback::Theme { previous } => {
            let mut restored = previous.clone();
            let _ = config::clamp_window_size(&mut restored.window);
            cfg.appearance = restored.appearance;
            cfg.window = restored.window;
        }
    }

    Ok(())
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
        publisher_url: if verified {
            Some("https://github.com/Misiix9/Vanta".to_string())
        } else {
            None
        },
        compatibility_hint: if verified {
            Some("validated-with-stable-community-schema".to_string())
        } else {
            None
        },
        rating: None,
        vote_count: None,
    }
}

fn workflow_has_risky_caps(workflow: &WorkflowMacro) -> bool {
    fn steps_have_risky_caps(steps: &[crate::config::MacroStep]) -> bool {
        steps.iter().any(|step| match step {
            crate::config::MacroStep::Extension { capabilities, .. }
            | crate::config::MacroStep::System { capabilities, .. } => capabilities.iter().any(
                |cap| matches!(cap, Capability::Shell | Capability::Filesystem | Capability::Network),
            ),
            crate::config::MacroStep::If {
                condition,
                then_steps,
                else_steps,
                ..
            } => {
                let cond_risky = match condition {
                    crate::config::WorkflowCondition::SystemCommandExitCode {
                        capabilities,
                        ..
                    } => capabilities.iter().any(|cap| {
                        matches!(cap, Capability::Shell | Capability::Filesystem | Capability::Network)
                    }),
                    _ => false,
                };
                cond_risky || steps_have_risky_caps(then_steps) || steps_have_risky_caps(else_steps)
            }
            crate::config::MacroStep::Workflow { .. } => true,
        })
    }

    steps_have_risky_caps(&workflow.steps)
}

fn is_valid_identifier(id: &str) -> bool {
    !id.is_empty()
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn evaluate_snippet_preview(
    snippet: &CommunitySnippetPayload,
    cfg: &config::VantaConfig,
) -> Result<CommunitySnippetPreview, VantaError> {
    let kind = snippet.kind.trim().to_ascii_lowercase();
    let mut blockers = Vec::new();
    let mut warnings = Vec::new();
    let mut badges = vec![
        format!("schema:v{}", snippet.schema_version),
        if snippet.trust.verified {
            "verified".to_string()
        } else {
            "unverified".to_string()
        },
        format!("risk:{}", snippet.trust.risk_level),
    ];

    if snippet.schema_version > COMMUNITY_SCHEMA_VERSION {
        blockers.push(format!(
            "Snippet schema v{} is newer than supported schema v{}",
            snippet.schema_version, COMMUNITY_SCHEMA_VERSION
        ));
    } else if snippet.schema_version < COMMUNITY_SCHEMA_VERSION {
        warnings.push(format!(
            "Snippet schema v{} is older than current schema v{}",
            snippet.schema_version, COMMUNITY_SCHEMA_VERSION
        ));
    }

    let mut target_id = None;

    let will_replace_existing = match kind.as_str() {
        "workflow" => {
            let workflow: WorkflowMacro = serde_json::from_value(snippet.payload.clone())
                .map_err(|e| format!("Invalid workflow snippet: {}", e))?;

            target_id = Some(workflow.id.clone());

            if !is_valid_identifier(&workflow.id) {
                blockers.push("Workflow id must be alphanumeric with '-' or '_'".to_string());
            }

            let risky = workflow_has_risky_caps(&workflow);
            if risky {
                badges.push("risky-capabilities".to_string());
            }
            if !snippet.trust.verified && risky {
                blockers.push(
                    "Unverified workflow snippet contains risky capabilities; import blocked."
                        .to_string(),
                );
            }

            let will_replace_existing = cfg.workflows.macros.iter().any(|m| m.id == workflow.id);
            if will_replace_existing {
                warnings.push(format!(
                    "Workflow '{}' already exists and will be replaced",
                    workflow.id
                ));
            }
            will_replace_existing
        }
        "profile" => {
            let profile: ProfileConfig = serde_json::from_value(snippet.payload.clone())
                .map_err(|e| format!("Invalid profile snippet: {}", e))?;

            target_id = Some(profile.id.clone());

            if !is_valid_identifier(&profile.id) {
                blockers.push("Profile id must be alphanumeric with '-' or '_'".to_string());
            }

            let will_replace_existing = cfg.profiles.entries.iter().any(|p| p.id == profile.id);
            if will_replace_existing {
                warnings.push(format!(
                    "Profile '{}' already exists and will be replaced",
                    profile.id
                ));
            }
            will_replace_existing
        }
        "theme" => {
            let _theme: ThemeSnippetData = serde_json::from_value(snippet.payload.clone())
                .map_err(|e| format!("Invalid theme snippet: {}", e))?;

            warnings.push(
                "Theme import will overwrite current appearance and window settings".to_string(),
            );
            true
        }
        _ => {
            return Err("Unsupported snippet kind. Use workflow, profile, or theme.".into());
        }
    };

    Ok(CommunitySnippetPreview {
        schema_version: snippet.schema_version,
        kind,
        name: snippet.name.clone(),
        trust: snippet.trust.clone(),
        target_id,
        will_replace_existing,
        compatible: blockers.is_empty(),
        blockers,
        warnings,
        badges,
    })
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
                timeout_ms: None,
                timeout_behavior: crate::config::TimeoutBehavior::Abort,
                schedule: None,
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
                        on_error: crate::config::StepErrorHandling::default(),
                        timeout_ms: None,
                        timeout_behavior: crate::config::TimeoutBehavior::Abort,
                    },
                    crate::config::MacroStep::Extension {
                        ext_id: "sync-project".to_string(),
                        command: "sync".to_string(),
                        args: vec!["--branch".to_string(), "{branch}".to_string()],
                        capabilities: vec![Capability::Shell],
                        on_error: crate::config::StepErrorHandling::default(),
                        timeout_ms: None,
                        timeout_behavior: crate::config::TimeoutBehavior::Abort,
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
                publisher_url: Some("https://github.com/Misiix9/Vanta".to_string()),
                compatibility_hint: Some("validated-with-stable-community-schema".to_string()),
                rating: Some(4.9),
                vote_count: Some(37),
            },
            workflow: WorkflowMacro {
                id: "focus-mode-clean-start".to_string(),
                name: "Focus Mode Clean Start".to_string(),
                description: Some("Quickly open an app and clear distractions".to_string()),
                enabled: true,
                timeout_ms: None,
                timeout_behavior: crate::config::TimeoutBehavior::Abort,
                schedule: None,
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
                    on_error: crate::config::StepErrorHandling::default(),
                    timeout_ms: None,
                    timeout_behavior: crate::config::TimeoutBehavior::Abort,
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
        .read()
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
        .write()
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
    cfg.save_with_source("community")?;

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
        .read()
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
pub async fn preview_community_snippet(
    snippet_json: String,
    state: State<'_, AppState>,
) -> Result<CommunitySnippetPreview, VantaError> {
    let snippet: CommunitySnippetPayload = serde_json::from_str(&snippet_json)
        .map_err(|e| format!("Invalid snippet payload: {}", e))?;

    let cfg = state
        .config
        .read()
        .map_err(|_| "Failed to access config".to_string())?;

    evaluate_snippet_preview(&snippet, &cfg)
}

#[tauri::command]
pub async fn get_community_import_history() -> Result<Vec<CommunityImportHistoryEntry>, VantaError> {
    let store = load_import_history_store();
    let mut entries: Vec<CommunityImportHistoryEntry> = store
        .entries
        .iter()
        .rev()
        .map(history_entry_from_record)
        .collect();
    entries.truncate(25);
    Ok(entries)
}

#[tauri::command]
pub async fn import_community_snippet(
    snippet_json: String,
    on_conflict: Option<String>,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, VantaError> {
    let snippet: CommunitySnippetPayload = serde_json::from_str(&snippet_json)
        .map_err(|e| format!("Invalid snippet payload: {}", e))?;

    let preview = {
        let cfg = state
            .config
            .read()
            .map_err(|_| "Failed to access config".to_string())?;
        evaluate_snippet_preview(&snippet, &cfg)?
    };

    if !preview.compatible {
        return Err(format!("Import blocked: {}", preview.blockers.join(" ")).into());
    }

    let conflict_strategy = parse_conflict_strategy(on_conflict)?;

    let mut cfg = state
        .config
        .write()
        .map_err(|_| "Failed to access config".to_string())?;

    let kind = snippet.kind.trim().to_ascii_lowercase();
    let imported_name = snippet.name.clone();
    let rollback: Option<CommunityImportRollback> = match kind.as_str() {
        "workflow" => {
            let workflow: WorkflowMacro = serde_json::from_value(snippet.payload.clone())
                .map_err(|e| format!("Invalid workflow snippet: {}", e))?;
            if !is_valid_identifier(&workflow.id) {
                return Err("Workflow id must be alphanumeric with '-' or '_'".into());
            }
            if !snippet.trust.verified && workflow_has_risky_caps(&workflow) {
                return Err("Unverified workflow snippet contains risky capabilities; import blocked.".into());
            }

            if let Some(pos) = cfg.workflows.macros.iter().position(|m| m.id == workflow.id) {
                if conflict_strategy == CommunityImportConflictStrategy::KeepLocal {
                    return Ok(format!(
                        "Skipped workflow import '{}' because local item already exists.",
                        workflow.id
                    ));
                }
                let previous = cfg.workflows.macros[pos].clone();
                cfg.workflows.macros[pos] = workflow.clone();
                Some(CommunityImportRollback::Workflow {
                    id: workflow.id,
                    previous: Some(previous),
                })
            } else {
                cfg.workflows.macros.push(workflow.clone());
                Some(CommunityImportRollback::Workflow {
                    id: workflow.id,
                    previous: None,
                })
            }
        }
        "profile" => {
            let profile: ProfileConfig = serde_json::from_value(snippet.payload.clone())
                .map_err(|e| format!("Invalid profile snippet: {}", e))?;
            if !is_valid_identifier(&profile.id) {
                return Err("Profile id must be alphanumeric with '-' or '_'".into());
            }

            if let Some(pos) = cfg.profiles.entries.iter().position(|p| p.id == profile.id) {
                if conflict_strategy == CommunityImportConflictStrategy::KeepLocal {
                    return Ok(format!(
                        "Skipped profile import '{}' because local item already exists.",
                        profile.id
                    ));
                }
                let previous = cfg.profiles.entries[pos].clone();
                cfg.profiles.entries[pos] = profile.clone();
                Some(CommunityImportRollback::Profile {
                    id: profile.id,
                    previous: Some(previous),
                })
            } else {
                cfg.profiles.entries.push(profile.clone());
                Some(CommunityImportRollback::Profile {
                    id: profile.id,
                    previous: None,
                })
            }
        }
        "theme" => {
            if conflict_strategy == CommunityImportConflictStrategy::KeepLocal {
                return Ok("Skipped theme import because conflict strategy is keep_local.".to_string());
            }
            let previous = ThemeSnippetData {
                appearance: cfg.appearance.clone(),
                window: cfg.window.clone(),
            };
            let mut theme: ThemeSnippetData = serde_json::from_value(snippet.payload.clone())
                .map_err(|e| format!("Invalid theme snippet: {}", e))?;
            let _ = config::clamp_window_size(&mut theme.window);
            let _ = config::clamp_accessibility(&mut cfg.accessibility);
            cfg.appearance = theme.appearance;
            cfg.window = theme.window;
            Some(CommunityImportRollback::Theme { previous })
        }
        _ => {
            return Err("Unsupported snippet kind. Use workflow, profile, or theme.".into());
        }
    };

    cfg.save_with_source("community")?;
    let _ = app_handle.emit("config-updated", &cfg.clone());

    let rollback_msg = if let Some(snapshot) = rollback {
        match append_import_history_record(
            &kind,
            &imported_name,
            preview.target_id.clone(),
            conflict_strategy,
            snippet.trust.verified,
            snapshot,
        ) {
            Ok(id) => format!(" Rollback id: {}.", id),
            Err(e) => format!(" Warning: rollback history not saved ({}).", e),
        }
    } else {
        String::new()
    };

    Ok(format!(
        "Imported {} snippet '{}' with {} strategy.{}",
        kind,
        imported_name,
        conflict_strategy.as_str(),
        rollback_msg
    ))
}

#[tauri::command]
pub async fn rollback_community_import(
    import_id: Option<String>,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, VantaError> {
    let mut store = load_import_history_store();
    if store.entries.is_empty() {
        return Err("No import history is available for rollback.".into());
    }

    let index = if let Some(id) = import_id.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()) {
        store
            .entries
            .iter()
            .position(|entry| entry.id == id)
            .ok_or_else(|| format!("Import record '{}' not found.", id))?
    } else {
        store.entries.len() - 1
    };

    let record = store.entries[index].clone();

    let mut cfg = state
        .config
        .write()
        .map_err(|_| "Failed to access config".to_string())?;
    apply_import_rollback(&mut cfg, &record.rollback)?;
    cfg.save_with_source("community-rollback")?;
    let _ = app_handle.emit("config-updated", &cfg.clone());

    store.entries.remove(index);
    store.updated_at = now_millis();
    let save_warning = match save_import_history_store(&store) {
        Ok(_) => String::new(),
        Err(e) => format!(" Warning: rollback history cleanup failed ({}).", e),
    };

    Ok(format!(
        "Rolled back {} import '{}' (record {}).{}",
        record.snippet_kind, record.snippet_name, record.id, save_warning
    ))
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
            timeout_ms: None,
            timeout_behavior: crate::config::TimeoutBehavior::Abort,
            schedule: None,
            args: Vec::new(),
            steps: vec![crate::config::MacroStep::System {
                command: "echo".to_string(),
                args: vec!["hello".to_string()],
                capabilities: vec![Capability::Shell],
                on_error: crate::config::StepErrorHandling::default(),
                timeout_ms: None,
                timeout_behavior: crate::config::TimeoutBehavior::Abort,
            }],
        };

        assert!(workflow_has_risky_caps(&workflow));
    }

    #[test]
    fn preview_blocks_unverified_risky_workflow() {
        let workflow = WorkflowMacro {
            id: "test-risky".to_string(),
            name: "Risky".to_string(),
            description: None,
            enabled: true,
            timeout_ms: None,
            timeout_behavior: crate::config::TimeoutBehavior::Abort,
            schedule: None,
            args: Vec::new(),
            steps: vec![crate::config::MacroStep::System {
                command: "echo".to_string(),
                args: vec!["hello".to_string()],
                capabilities: vec![Capability::Shell],
                on_error: crate::config::StepErrorHandling::default(),
                timeout_ms: None,
                timeout_behavior: crate::config::TimeoutBehavior::Abort,
            }],
        };

        let snippet = CommunitySnippetPayload {
            schema_version: COMMUNITY_SCHEMA_VERSION,
            created_at: now_millis(),
            kind: "workflow".to_string(),
            name: workflow.name.clone(),
            trust: community_default_trust(false),
            payload: serde_json::to_value(workflow).unwrap(),
        };

        let cfg = config::VantaConfig::default();
        let preview = evaluate_snippet_preview(&snippet, &cfg).unwrap();

        assert!(!preview.compatible);
        assert!(preview
            .blockers
            .iter()
            .any(|msg| msg.contains("risky capabilities")));
    }

    #[test]
    fn parse_conflict_strategy_defaults_to_replace() {
        assert_eq!(
            parse_conflict_strategy(None).unwrap(),
            CommunityImportConflictStrategy::Replace
        );
        assert_eq!(
            parse_conflict_strategy(Some("".to_string())).unwrap(),
            CommunityImportConflictStrategy::Replace
        );
    }

    #[test]
    fn rollback_removes_created_workflow() {
        let mut cfg = config::VantaConfig::default();
        cfg.workflows.macros.push(WorkflowMacro {
            id: "temp-flow".to_string(),
            name: "Temp Flow".to_string(),
            description: None,
            enabled: true,
            timeout_ms: None,
            timeout_behavior: crate::config::TimeoutBehavior::Abort,
            schedule: None,
            args: Vec::new(),
            steps: vec![crate::config::MacroStep::System {
                command: "echo".to_string(),
                args: vec!["hello".to_string()],
                capabilities: Vec::new(),
                on_error: crate::config::StepErrorHandling::default(),
                timeout_ms: None,
                timeout_behavior: crate::config::TimeoutBehavior::Abort,
            }],
        });

        let rollback = CommunityImportRollback::Workflow {
            id: "temp-flow".to_string(),
            previous: None,
        };

        apply_import_rollback(&mut cfg, &rollback).unwrap();
        assert!(!cfg.workflows.macros.iter().any(|m| m.id == "temp-flow"));
    }

    #[test]
    fn rollback_restores_previous_profile() {
        let mut cfg = config::VantaConfig::default();
        cfg.profiles.entries[0].name = "Current".to_string();

        let previous = config::ProfileConfig {
            id: cfg.profiles.entries[0].id.clone(),
            name: "Previous".to_string(),
            hotkey: cfg.profiles.entries[0].hotkey.clone(),
            theme: cfg.profiles.entries[0].theme.clone(),
            search: cfg.profiles.entries[0].search.clone(),
        };

        let rollback = CommunityImportRollback::Profile {
            id: previous.id.clone(),
            previous: Some(previous.clone()),
        };

        apply_import_rollback(&mut cfg, &rollback).unwrap();
        let restored = cfg
            .profiles
            .entries
            .iter()
            .find(|p| p.id == previous.id)
            .unwrap();
        assert_eq!(restored.name, "Previous");
    }
}

use jsonschema::JSONSchema;
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tauri::Emitter;

use crate::permissions::Capability;
use crate::errors::VantaError;

// Embedded default config for fallback writes.
const DEFAULT_CONFIG_JSON: &str = include_str!("../resources/config.json");
pub const CONFIG_SCHEMA_VERSION: u32 = 5;
pub const WORKFLOWS_SCHEMA_VERSION: u32 = 2;
pub const PROFILES_SCHEMA_VERSION: u32 = 1;
pub const PROFILE_EXPORT_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Serialize)]
pub struct ConfigMigrationReport {
    pub config_updated: bool,
    pub schema_from: u32,
    pub schema_to: u32,
    pub workflows_schema_from: u32,
    pub workflows_schema_to: u32,
}

//
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct VantaConfig {
    #[serde(default = "default_config_schema_version")]
    pub schema_version: u32,
    pub general: GeneralConfig,
    pub appearance: AppearanceConfig,
    pub window: WindowConfig,
    #[serde(default)]
    pub accessibility: AccessibilityConfig,
    #[serde(default)]
    pub extensions: ExtensionsConfig,
    #[serde(default)]
    pub files: FilesConfig,
    #[serde(default)]
    pub search: SearchConfig,
    #[serde(default)]
    pub workflows: WorkflowsConfig,
    #[serde(default)]
    pub profiles: ProfilesConfig,
    #[serde(default)]
    pub policy: PolicyConfig,
    #[serde(default)]
    pub notes: NotesConfig,
    #[serde(default)]
    pub bookmarks: BookmarksConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct FileBookmark {
    pub path: String,
    pub created_at_ms: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct BookmarksConfig {
    #[serde(default)]
    pub entries: Vec<FileBookmark>,
    #[serde(default = "default_bookmarks_max_entries")]
    pub max_entries: usize,
}

fn default_bookmarks_max_entries() -> usize {
    200
}

impl Default for BookmarksConfig {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            max_entries: default_bookmarks_max_entries(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct QuickNote {
    pub id: String,
    pub text: String,
    pub created_at_ms: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct NotesConfig {
    #[serde(default)]
    pub entries: Vec<QuickNote>,
    #[serde(default = "default_notes_max_entries")]
    pub max_entries: usize,
}

fn default_notes_max_entries() -> usize {
    200
}

impl Default for NotesConfig {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            max_entries: default_notes_max_entries(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct PolicyConfig {
    #[serde(default = "default_policy_restricted_mode")]
    pub restricted_mode: bool,
    #[serde(default)]
    pub allowed_extensions: Vec<String>,
    #[serde(default)]
    pub blocked_capabilities: Vec<Capability>,
    #[serde(default)]
    pub require_verified_extensions: bool,
}

fn default_policy_restricted_mode() -> bool {
    false
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            restricted_mode: default_policy_restricted_mode(),
            allowed_extensions: Vec::new(),
            blocked_capabilities: Vec::new(),
            require_verified_extensions: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ProfileConfig {
    pub id: String,
    pub name: String,
    pub hotkey: String,
    pub theme: String,
    pub search: SearchConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ProfilesConfig {
    #[serde(default = "default_profiles_schema_version")]
    pub schema_version: u32,
    #[serde(default = "default_active_profile_id")]
    pub active_profile_id: String,
    #[serde(default = "default_profiles")]
    pub entries: Vec<ProfileConfig>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ProfileExportPayload {
    #[serde(default = "default_profile_export_schema_version")]
    schema_version: u32,
    app_schema_version: u32,
    profile: ProfileConfig,
}

fn default_profile_export_schema_version() -> u32 {
    PROFILE_EXPORT_SCHEMA_VERSION
}

fn default_profiles_schema_version() -> u32 {
    PROFILES_SCHEMA_VERSION
}

fn default_active_profile_id() -> String {
    "default".to_string()
}

fn default_profiles() -> Vec<ProfileConfig> {
    vec![ProfileConfig {
        id: default_active_profile_id(),
        name: "Default".to_string(),
        hotkey: "Alt+Space".to_string(),
        theme: "default".to_string(),
        search: SearchConfig::default(),
    }]
}

impl Default for ProfilesConfig {
    fn default() -> Self {
        Self {
            schema_version: default_profiles_schema_version(),
            active_profile_id: default_active_profile_id(),
            entries: default_profiles(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct SearchConfig {
    #[serde(default)]
    pub applications: SourcePreference,
    #[serde(default)]
    pub windows: SourcePreference,
    #[serde(default)]
    pub calculator: SourcePreference,
    #[serde(default)]
    pub files: SourcePreference,
    #[serde(default = "default_windows_cap")]
    pub windows_max_results: usize,
    #[serde(default = "default_show_explain_panel")]
    pub show_explain_panel: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct SourcePreference {
    #[serde(default = "default_source_enabled")]
    pub enabled: bool,
    #[serde(default = "default_source_weight")]
    pub weight: u32,
}

fn default_source_enabled() -> bool {
    true
}

fn default_source_weight() -> u32 {
    100
}

impl Default for SourcePreference {
    fn default() -> Self {
        Self {
            enabled: true,
            weight: 100,
        }
    }
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            applications: SourcePreference::default(),
            windows: SourcePreference::default(),
            calculator: SourcePreference::default(),
            files: SourcePreference::default(),
            windows_max_results: default_windows_cap(),
            show_explain_panel: default_show_explain_panel(),
        }
    }
}

fn default_windows_cap() -> usize {
    0
}

fn default_show_explain_panel() -> bool {
    true
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct FilesConfig {
    #[serde(default)]
    pub include_hidden: bool,
    #[serde(default = "default_max_depth")]
    pub max_depth: usize,
    #[serde(default = "default_opener")]
    pub file_manager: String,
    #[serde(default = "default_opener")]
    pub file_editor: String,
    #[serde(default)]
    pub open_docs_in_manager: bool,
    #[serde(default)]
    pub include_globs: Vec<String>,
    #[serde(default)]
    pub exclude_globs: Vec<String>,
    #[serde(default)]
    pub allowed_extensions: Vec<String>,
    #[serde(default = "default_type_filter")]
    pub type_filter: String, // "any" | "file" | "dir"
    #[serde(default)]
    pub indexed_at: Option<u64>, // epoch millis, informational freshness marker
}

fn default_max_depth() -> usize {
    3
}

fn default_opener() -> String {
    "default".to_string()
}

fn default_type_filter() -> String {
    "any".to_string()
}

fn default_theme() -> String {
    "default".to_string()
}

impl Default for FilesConfig {
    fn default() -> Self {
        Self {
            include_hidden: false,
            max_depth: 3,
            file_manager: "default".to_string(),
            file_editor: "default".to_string(),
            open_docs_in_manager: false,
            include_globs: Vec::new(),
            exclude_globs: Vec::new(),
            allowed_extensions: Vec::new(),
            type_filter: default_type_filter(),
            indexed_at: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct GeneralConfig {
    pub hotkey: String,
    pub max_results: usize,
    pub launch_on_login: bool,
    #[serde(default)]
    pub community_feed_opt_in: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct WindowConfig {
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct AccessibilityConfig {
    #[serde(default)]
    pub reduced_motion: bool,
    #[serde(default = "default_text_scale")]
    pub text_scale: f64,
    #[serde(default = "default_spacing_preset")]
    pub spacing_preset: String,
}

fn default_text_scale() -> f64 {
    1.0
}

fn default_spacing_preset() -> String {
    "comfortable".to_string()
}

impl Default for AccessibilityConfig {
    fn default() -> Self {
        Self {
            reduced_motion: false,
            text_scale: default_text_scale(),
            spacing_preset: default_spacing_preset(),
        }
    }
}

pub fn clamp_accessibility(cfg: &mut AccessibilityConfig) -> bool {
    let mut changed = false;

    let clamped = cfg.text_scale.clamp(0.85, 1.4);
    if (clamped - cfg.text_scale).abs() > f64::EPSILON {
        cfg.text_scale = clamped;
        changed = true;
    }

    if !matches!(cfg.spacing_preset.as_str(), "compact" | "comfortable" | "relaxed") {
        cfg.spacing_preset = "comfortable".to_string();
        changed = true;
    }

    changed
}

/// Clamp window dimensions to reasonable bounds to avoid oversized popups.
/// Returns true if values were adjusted.
pub fn clamp_window_size(cfg: &mut WindowConfig) -> bool {
    let mut changed = false;
    let min_w = 400.0;
    let max_w = 1920.0;
    let min_h = 300.0;
    let max_h = 1080.0;

    if cfg.width < min_w {
        cfg.width = min_w;
        changed = true;
    } else if cfg.width > max_w {
        cfg.width = max_w;
        changed = true;
    }

    if cfg.height < min_h {
        cfg.height = min_h;
        changed = true;
    } else if cfg.height > max_h {
        cfg.height = max_h;
        changed = true;
    }

    changed
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct AppearanceConfig {
    pub blur_radius: u32,
    pub opacity: f64,
    pub border_radius: u32,
    #[serde(default = "default_theme")]
    pub theme: String,
    pub colors: ColorsConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ColorsConfig {
    pub background: String,
    pub surface: String,
    pub accent: String,
    pub accent_glow: String,
    pub text_primary: String,
    pub text_secondary: String,
    pub border: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ExtensionsConfig {
    #[serde(default = "default_extensions_dir")]
    pub directory: String,
    #[serde(default)]
    pub dev_mode: bool,
}

fn default_extensions_dir() -> String {
    "~/.config/vanta/extensions".to_string()
}

impl Default for ExtensionsConfig {
    fn default() -> Self {
        Self {
            directory: default_extensions_dir(),
            dev_mode: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct WorkflowsConfig {
    #[serde(default = "default_workflows_schema_version")]
    pub schema_version: u32,
    #[serde(default)]
    pub macros: Vec<WorkflowMacro>,
    #[serde(default)]
    pub command_templates: Vec<CommandTemplate>,
}

fn default_config_schema_version() -> u32 {
    CONFIG_SCHEMA_VERSION
}

fn default_workflows_schema_version() -> u32 {
    WORKFLOWS_SCHEMA_VERSION
}

impl Default for WorkflowsConfig {
    fn default() -> Self {
        Self {
            schema_version: default_workflows_schema_version(),
            macros: default_workflow_macros(),
            command_templates: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct CommandTemplate {
    pub id: String,
    pub name: String,
    pub macro_id: String,
    #[serde(default)]
    pub args: HashMap<String, String>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

fn default_workflow_macros() -> Vec<WorkflowMacro> {
    vec![WorkflowMacro {
        id: "open-and-sync".to_string(),
        name: "Open and Sync".to_string(),
        description: Some("Open a project folder then sync it".to_string()),
        enabled: false,
        args: vec![
            MacroArg {
                name: "project_path".to_string(),
                description: Some("Path to the project root".to_string()),
                required: true,
                default_value: None,
            },
            MacroArg {
                name: "branch".to_string(),
                description: Some("Branch name".to_string()),
                required: false,
                default_value: Some("main".to_string()),
            },
        ],
        steps: vec![
            MacroStep::System {
                command: "xdg-open".to_string(),
                args: vec!["{project_path}".to_string()],
                capabilities: vec![Capability::Filesystem],
                on_error: StepErrorHandling::default(),
                timeout_ms: None,
                timeout_behavior: TimeoutBehavior::Abort,
            },
            MacroStep::Extension {
                ext_id: "sync-project".to_string(),
                command: "sync".to_string(),
                args: vec!["--branch".to_string(), "{branch}".to_string()],
                capabilities: vec![Capability::Shell],
                on_error: StepErrorHandling::default(),
                timeout_ms: None,
                timeout_behavior: TimeoutBehavior::Abort,
            },
        ],
        timeout_ms: None,
        timeout_behavior: TimeoutBehavior::Abort,
        schedule: None,
    }]
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct WorkflowMacro {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub args: Vec<MacroArg>,
    #[serde(default)]
    pub steps: Vec<MacroStep>,
    #[serde(default = "default_macro_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    #[serde(default = "default_timeout_behavior")]
    pub timeout_behavior: TimeoutBehavior,
    #[serde(default)]
    pub schedule: Option<WorkflowSchedule>,
}

fn default_macro_enabled() -> bool {
    true
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TimeoutBehavior {
    Abort,
    Skip,
}

fn default_timeout_behavior() -> TimeoutBehavior {
    TimeoutBehavior::Abort
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowScheduleEvent {
    Startup,
    NetworkConnected,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct WorkflowSchedule {
    #[serde(default)]
    pub enabled: bool,
    pub interval_minutes: u64,
    #[serde(default)]
    pub run_on_startup: bool,
    #[serde(default)]
    pub on_events: Vec<WorkflowScheduleEvent>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct MacroArg {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub default_value: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WorkflowCondition {
    StepOutputContains {
        step: usize,
        value: String,
    },
    StepOutputEquals {
        step: usize,
        value: String,
    },
    SystemCommandExitCode {
        command: String,
        #[serde(default)]
        args: Vec<String>,
        equals: i32,
        #[serde(default)]
        capabilities: Vec<Capability>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct StepErrorHandling {
    #[serde(default)]
    pub retry_count: u8,
    #[serde(default)]
    pub skip_on_failure: bool,
    #[serde(default)]
    pub finally_steps: Vec<MacroStep>,
}

impl Default for StepErrorHandling {
    fn default() -> Self {
        Self {
            retry_count: 0,
            skip_on_failure: false,
            finally_steps: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MacroStep {
    Extension {
        ext_id: String,
        command: String,
        #[serde(default)]
        args: Vec<String>,
        #[serde(default)]
        capabilities: Vec<Capability>,
        #[serde(default)]
        on_error: StepErrorHandling,
        #[serde(default)]
        timeout_ms: Option<u64>,
        #[serde(default = "default_timeout_behavior")]
        timeout_behavior: TimeoutBehavior,
    },
    System {
        command: String,
        #[serde(default)]
        args: Vec<String>,
        #[serde(default)]
        capabilities: Vec<Capability>,
        #[serde(default)]
        on_error: StepErrorHandling,
        #[serde(default)]
        timeout_ms: Option<u64>,
        #[serde(default = "default_timeout_behavior")]
        timeout_behavior: TimeoutBehavior,
    },
    If {
        condition: WorkflowCondition,
        #[serde(default)]
        then_steps: Vec<MacroStep>,
        #[serde(default)]
        else_steps: Vec<MacroStep>,
        #[serde(default)]
        on_error: StepErrorHandling,
    },
    Workflow {
        macro_id: String,
        #[serde(default)]
        args: HashMap<String, String>,
        #[serde(default)]
        on_error: StepErrorHandling,
    },
}

impl Default for VantaConfig {
    fn default() -> Self {
        Self {
            schema_version: default_config_schema_version(),
            general: GeneralConfig {
                hotkey: "Alt+Space".to_string(),
                max_results: 8,
                launch_on_login: false,
                community_feed_opt_in: false,
            },
            window: WindowConfig {
                width: 680.0,
                height: 420.0,
            },
            accessibility: AccessibilityConfig::default(),
            appearance: AppearanceConfig {
                blur_radius: 40,
                opacity: 0.85,
                border_radius: 24,
                theme: "default".to_string(),
                colors: ColorsConfig {
                    background: "#000000".to_string(), // Pure OLED Black
                    surface: "#0A0A0A".to_string(),    // Item BG
                    accent: "#FFFFFF".to_string(),     // Monochrome Accent
                    accent_glow: "rgba(255, 255, 255, 0.25)".to_string(),
                    text_primary: "#F5F5F5".to_string(), // Off-White
                    text_secondary: "#888888".to_string(), // Grey
                    border: "rgba(255, 255, 255, 0.08)".to_string(),
                },
            },
            extensions: ExtensionsConfig::default(),
            files: FilesConfig {
                include_hidden: false,
                max_depth: 3,
                file_manager: "default".to_string(),
                file_editor: "default".to_string(),
                open_docs_in_manager: false,
                include_globs: Vec::new(),
                exclude_globs: Vec::new(),
                allowed_extensions: Vec::new(),
                type_filter: default_type_filter(),
                indexed_at: None,
            },
            search: SearchConfig::default(),
            workflows: WorkflowsConfig::default(),
            profiles: ProfilesConfig::default(),
            policy: PolicyConfig::default(),
            notes: NotesConfig::default(),
            bookmarks: BookmarksConfig::default(),
        }
    }
}

fn profile_from_runtime(cfg: &VantaConfig, id: &str, name: &str) -> ProfileConfig {
    ProfileConfig {
        id: id.to_string(),
        name: name.to_string(),
        hotkey: cfg.general.hotkey.clone(),
        theme: cfg.appearance.theme.clone(),
        search: cfg.search.clone(),
    }
}

fn validate_profile_id(id: &str) -> bool {
    !id.is_empty()
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn ensure_profiles(cfg: &mut VantaConfig) -> bool {
    let mut changed = false;

    if cfg.profiles.schema_version < PROFILES_SCHEMA_VERSION {
        cfg.profiles.schema_version = PROFILES_SCHEMA_VERSION;
        changed = true;
    }

    cfg.profiles.entries.retain(|p| validate_profile_id(&p.id));
    if cfg.profiles.entries.is_empty() {
        cfg.profiles
            .entries
            .push(profile_from_runtime(cfg, "default", "Default"));
        changed = true;
    }

    if cfg.profiles.active_profile_id.is_empty()
        || !cfg
            .profiles
            .entries
            .iter()
            .any(|p| p.id == cfg.profiles.active_profile_id)
    {
        cfg.profiles.active_profile_id = cfg.profiles.entries[0].id.clone();
        changed = true;
    }

    if sync_active_profile_snapshot(cfg) {
        changed = true;
    }

    changed
}

fn sync_active_profile_snapshot(cfg: &mut VantaConfig) -> bool {
    let Some(active) = cfg
        .profiles
        .entries
        .iter_mut()
        .find(|p| p.id == cfg.profiles.active_profile_id)
    else {
        return false;
    };

    let mut changed = false;
    if active.hotkey != cfg.general.hotkey {
        active.hotkey = cfg.general.hotkey.clone();
        changed = true;
    }
    if active.theme != cfg.appearance.theme {
        active.theme = cfg.appearance.theme.clone();
        changed = true;
    }
    if active.search != cfg.search {
        active.search = cfg.search.clone();
        changed = true;
    }

    changed
}

pub fn list_profiles(cfg: &VantaConfig) -> ProfilesConfig {
    cfg.profiles.clone()
}

pub fn switch_profile_in_config(cfg: &mut VantaConfig, profile_id: &str) -> Result<(), VantaError> {
    let profile = cfg
        .profiles
        .entries
        .iter()
        .find(|p| p.id == profile_id)
        .cloned()
        .ok_or_else(|| format!("Profile '{}' not found", profile_id))?;

    cfg.general.hotkey = profile.hotkey;
    cfg.appearance.theme = profile.theme;
    cfg.search = profile.search;
    cfg.profiles.active_profile_id = profile.id;
    Ok(())
}

pub fn export_profile_to_path(
    cfg: &VantaConfig,
    profile_id: &str,
    output_path: &str,
) -> Result<(), VantaError> {
    let profile = cfg
        .profiles
        .entries
        .iter()
        .find(|p| p.id == profile_id)
        .cloned()
        .ok_or_else(|| format!("Profile '{}' not found", profile_id))?;

    let payload = ProfileExportPayload {
        schema_version: PROFILE_EXPORT_SCHEMA_VERSION,
        app_schema_version: CONFIG_SCHEMA_VERSION,
        profile,
    };

    let json = serde_json::to_string_pretty(&payload)
        .map_err(|e| format!("Failed to serialize profile export: {}", e))?;
    Ok(fs::write(output_path, json)
        .map_err(|e| format!("Failed to write profile export '{}': {}", output_path, e))?)
}

pub fn import_profile_from_path(
    cfg: &mut VantaConfig,
    input_path: &str,
    replace_existing: bool,
) -> Result<ProfileConfig, VantaError> {
    let raw = fs::read_to_string(input_path)
        .map_err(|e| format!("Failed to read profile import '{}': {}", input_path, e))?;
    let payload: ProfileExportPayload = serde_json::from_str(&raw)
        .map_err(|e| format!("Failed to parse profile import '{}': {}", input_path, e))?;

    if payload.schema_version > PROFILE_EXPORT_SCHEMA_VERSION {
        return Err(format!(
            "Profile export schema {} is newer than supported {}",
            payload.schema_version, PROFILE_EXPORT_SCHEMA_VERSION
        ).into());
    }
    if payload.app_schema_version > CONFIG_SCHEMA_VERSION {
        return Err(format!(
            "Profile export app schema {} is newer than supported {}",
            payload.app_schema_version, CONFIG_SCHEMA_VERSION
        ).into());
    }
    if !validate_profile_id(&payload.profile.id) {
        return Err("Profile id is invalid. Use only [A-Za-z0-9_-].".into());
    }

    if let Some(idx) = cfg
        .profiles
        .entries
        .iter()
        .position(|p| p.id == payload.profile.id)
    {
        if !replace_existing {
            return Err(format!(
                "Profile '{}' already exists. Enable replace to overwrite.",
                payload.profile.id
            ).into());
        }
        cfg.profiles.entries[idx] = payload.profile.clone();
    } else {
        cfg.profiles.entries.push(payload.profile.clone());
    }

    Ok(payload.profile)
}

/// Returns the path to the Vanta config directory (~/.config/vanta/).
pub fn config_dir() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| {
        dirs::home_dir()
            .map(|home| home.join(".config"))
            .unwrap_or_else(|| PathBuf::from("/tmp"))
    });
    base.join("vanta")
}

/// Returns the path to the config file (~/.config/vanta/config.json).
pub fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

pub fn config_audit_path() -> PathBuf {
    config_dir().join("config-audit.jsonl")
}

pub fn config_schema_path() -> PathBuf {
    config_dir().join("config.schema.json")
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigSchemaValidationReport {
    pub valid: bool,
    pub errors: Vec<String>,
}

pub fn generated_config_schema() -> Value {
    serde_json::to_value(schema_for!(VantaConfig)).unwrap_or_else(|_| Value::Null)
}

pub fn write_config_schema_file() -> Result<(), VantaError> {
    let schema = generated_config_schema();
    let path = config_schema_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create schema directory {}: {}", parent.display(), e))?;
    }
    let raw = serde_json::to_string_pretty(&schema)
        .map_err(|e| format!("Failed to serialize config schema: {}", e))?;
    fs::write(&path, raw)
        .map_err(|e| format!("Failed to write config schema to {}: {}", path.display(), e))?;
    Ok(())
}

fn validate_config_json_against_schema(json: &Value) -> ConfigSchemaValidationReport {
    let schema = generated_config_schema();
    match JSONSchema::compile(&schema) {
        Ok(compiled) => {
            let mut errors = Vec::new();
            if let Err(iter) = compiled.validate(json) {
                for err in iter {
                    errors.push(err.to_string());
                }
            }
            ConfigSchemaValidationReport {
                valid: errors.is_empty(),
                errors,
            }
        }
        Err(err) => ConfigSchemaValidationReport {
            valid: false,
            errors: vec![format!("Failed to compile generated schema: {}", err)],
        },
    }
}

pub fn validate_config_file_against_schema() -> ConfigSchemaValidationReport {
    let path = config_path();
    let raw = match fs::read_to_string(&path) {
        Ok(raw) => raw,
        Err(err) => {
            return ConfigSchemaValidationReport {
                valid: false,
                errors: vec![format!("Failed to read {}: {}", path.display(), err)],
            };
        }
    };

    let parsed: Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(err) => {
            return ConfigSchemaValidationReport {
                valid: false,
                errors: vec![format!("Invalid JSON in {}: {}", path.display(), err)],
            };
        }
    };

    validate_config_json_against_schema(&parsed)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigDiffEntry {
    pub path: String,
    pub before: Option<Value>,
    pub after: Option<Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigAuditEntry {
    pub timestamp_ms: i64,
    pub source: String,
    pub diff: Vec<ConfigDiffEntry>,
}

fn now_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn join_diff_path(base: &str, next: &str) -> String {
    if base.is_empty() {
        next.to_string()
    } else {
        format!("{}.{}", base, next)
    }
}

fn collect_config_diff(path: &str, before: Option<&Value>, after: Option<&Value>, out: &mut Vec<ConfigDiffEntry>) {
    match (before, after) {
        (Some(Value::Object(a)), Some(Value::Object(b))) => {
            let mut keys: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
            keys.extend(a.keys().cloned());
            keys.extend(b.keys().cloned());
            for key in keys {
                collect_config_diff(
                    &join_diff_path(path, &key),
                    a.get(&key),
                    b.get(&key),
                    out,
                );
            }
        }
        (Some(Value::Array(a)), Some(Value::Array(b))) => {
            if a != b {
                out.push(ConfigDiffEntry {
                    path: path.to_string(),
                    before: Some(Value::Array(a.clone())),
                    after: Some(Value::Array(b.clone())),
                });
            }
        }
        (a, b) => {
            if a != b {
                out.push(ConfigDiffEntry {
                    path: path.to_string(),
                    before: a.cloned(),
                    after: b.cloned(),
                });
            }
        }
    }
}

fn diff_configs(before: Option<&VantaConfig>, after: &VantaConfig) -> Vec<ConfigDiffEntry> {
    let before_json = before.and_then(|v| serde_json::to_value(v).ok());
    let after_json = serde_json::to_value(after).unwrap_or(Value::Null);
    let mut diff = Vec::new();
    collect_config_diff("", before_json.as_ref(), Some(&after_json), &mut diff);
    diff
}

fn append_config_audit_entry(entry: &ConfigAuditEntry) {
    let path = config_audit_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(line) = serde_json::to_string(entry) {
        use std::io::Write;
        if let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(path) {
            let _ = writeln!(file, "{}", line);
        }
    }
}

pub fn read_config_audit(limit: usize) -> Vec<ConfigAuditEntry> {
    let path = config_audit_path();
    let Ok(raw) = fs::read_to_string(path) else {
        return Vec::new();
    };

    let mut entries: Vec<ConfigAuditEntry> = raw
        .lines()
        .filter_map(|line| serde_json::from_str::<ConfigAuditEntry>(line).ok())
        .collect();

    if limit > 0 && entries.len() > limit {
        entries = entries.split_off(entries.len() - limit);
    }
    entries
}

/// Load config from disk or create the default config file.
pub fn load_or_create_default() -> VantaConfig {
    let path = config_path();
    let _ = write_config_schema_file();

    if path.exists() {
        match fs::read_to_string(&path) {
            Ok(contents) => {
                if let Ok(raw_json) = serde_json::from_str::<Value>(&contents) {
                    let report = validate_config_json_against_schema(&raw_json);
                    if !report.valid {
                        log::warn!(
                            "Config schema validation found {} issue(s) for {}",
                            report.errors.len(),
                            path.display()
                        );
                    }
                }

                match serde_json::from_str::<VantaConfig>(&contents) {
                Ok(mut config) => {
                    log::info!("Loaded config from {}", path.display());
                    // Clamp window bounds eagerly and persist if adjusted.
                    let mut changed = false;
                    if config.schema_version < CONFIG_SCHEMA_VERSION {
                        config.schema_version = CONFIG_SCHEMA_VERSION;
                        changed = true;
                    }
                    if config.workflows.schema_version < WORKFLOWS_SCHEMA_VERSION {
                        config.workflows.schema_version = WORKFLOWS_SCHEMA_VERSION;
                        changed = true;
                    }
                    if ensure_profiles(&mut config) {
                        changed = true;
                    }
                    if clamp_window_size(&mut config.window) {
                        changed = true;
                    }
                    if clamp_accessibility(&mut config.accessibility) {
                        changed = true;
                    }
                    if changed {
                        let _ = write_config_with_source(&config, "migration");
                    }
                    println!(
                        "[vanta][config] loaded {:?} window={}x{}",
                        path,
                        config.window.width,
                        config.window.height
                    );
                    return config;
                }
                Err(e) => {
                    log::warn!("Invalid config.json, rewriting defaults: {}", e);
                    return rewrite_default_config();
                }
                }
            }
            Err(e) => {
                log::warn!("Could not read config.json, rewriting defaults: {}", e);
                return rewrite_default_config();
            }
        }
    }

    let dir = config_dir();
    if let Err(e) = fs::create_dir_all(&dir) {
        log::error!("Could not create config directory {}: {}", dir.display(), e);
        return VantaConfig::default();
    }

    let extensions_dir = dir.join("extensions");
    if let Err(e) = fs::create_dir_all(&extensions_dir) {
        log::warn!("Could not create extensions directory: {}", e);
    }

    rewrite_default_config()
}

pub fn migrate_config_on_disk() -> Result<ConfigMigrationReport, VantaError> {
    let _ = write_config_schema_file();
    let path = config_path();
    if !path.exists() {
        return Ok(ConfigMigrationReport {
            config_updated: false,
            schema_from: CONFIG_SCHEMA_VERSION,
            schema_to: CONFIG_SCHEMA_VERSION,
            workflows_schema_from: WORKFLOWS_SCHEMA_VERSION,
            workflows_schema_to: WORKFLOWS_SCHEMA_VERSION,
        });
    }

    let raw = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read config for migration: {}", e))?;
    let mut cfg: VantaConfig = serde_json::from_str(&raw)
        .map_err(|e| format!("Failed to parse config for migration: {}", e))?;

    let from = cfg.schema_version;
    let wf_from = cfg.workflows.schema_version;
    let mut changed = false;

    if cfg.schema_version < CONFIG_SCHEMA_VERSION {
        cfg.schema_version = CONFIG_SCHEMA_VERSION;
        changed = true;
    }
    if cfg.workflows.schema_version < WORKFLOWS_SCHEMA_VERSION {
        cfg.workflows.schema_version = WORKFLOWS_SCHEMA_VERSION;
        changed = true;
    }
    if ensure_profiles(&mut cfg) {
        changed = true;
    }
    if clamp_window_size(&mut cfg.window) {
        changed = true;
    }
    if clamp_accessibility(&mut cfg.accessibility) {
        changed = true;
    }

    if changed {
        write_config_with_source(&cfg, "migration")?;
    }

    Ok(ConfigMigrationReport {
        config_updated: changed,
        schema_from: from,
        schema_to: cfg.schema_version,
        workflows_schema_from: wf_from,
        workflows_schema_to: cfg.workflows.schema_version,
    })
}

pub fn factory_reset_on_disk() -> Result<VantaConfig, VantaError> {
    let mut cfg = VantaConfig::default();
    let _ = clamp_window_size(&mut cfg.window);
    let _ = clamp_accessibility(&mut cfg.accessibility);
    write_config_with_source(&cfg, "factory-reset")?;
    Ok(cfg)
}

impl VantaConfig {
    /// Save the current configuration to disk.
    pub fn save(&self) -> Result<(), VantaError> {
        self.save_with_source("system")
    }

    pub fn save_with_source(&self, source: &str) -> Result<(), VantaError> {
        write_config_with_source(self, source)
    }
}

fn write_config_with_source(cfg: &VantaConfig, source: &str) -> Result<(), VantaError> {
    let mut cfg = cfg.clone();
    let _ = ensure_profiles(&mut cfg);

    let old_cfg: Option<VantaConfig> = fs::read_to_string(config_path())
        .ok()
        .and_then(|raw| serde_json::from_str::<VantaConfig>(&raw).ok());

    let path = config_path();
    let json = serde_json::to_string_pretty(&cfg)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&path, json)
        .map_err(|e| format!("Failed to write config to {}: {}", path.display(), e))?;

    let diff = diff_configs(old_cfg.as_ref(), &cfg);
    if !diff.is_empty() {
        append_config_audit_entry(&ConfigAuditEntry {
            timestamp_ms: now_millis(),
            source: source.to_string(),
            diff,
        });
    }

    Ok(())
}

/// Attempt to rewrite the default config using serialized defaults, falling back
/// to an embedded copy if serialization or write fails.
fn rewrite_default_config() -> VantaConfig {
    let mut default_config = VantaConfig::default();

    // Clamp before writing defaults.
    let _ = clamp_window_size(&mut default_config.window);
    let _ = clamp_accessibility(&mut default_config.accessibility);

    // First try normal serialization+write.
    if let Err(write_err) = write_config_with_source(&default_config, "factory-default") {
        log::warn!("Primary default write failed: {}. Trying embedded copy...", write_err);
        // Fallback: write embedded JSON blob.
        let path = config_path();
        if let Err(e) = fs::write(&path, DEFAULT_CONFIG_JSON) {
            log::error!("Could not write embedded default config to {}: {}", path.display(), e);
        } else {
            log::info!("Wrote embedded default config to {}", path.display());
        }
    }

    default_config
}
pub fn watch_config(app_handle: tauri::AppHandle) {
    use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
    use std::sync::mpsc;
    use std::time::Duration;

    let path = config_path();
    let watch_dir = config_dir();

    if !watch_dir.exists() {
        log::warn!("Config directory does not exist, skipping watcher");
        return;
    }

    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();

    let mut watcher: RecommendedWatcher = match Watcher::new(
        tx,
        notify::Config::default().with_poll_interval(Duration::from_secs(1)),
    ) {
        Ok(w) => w,
        Err(e) => {
            log::error!("Failed to create config watcher: {}", e);
            return;
        }
    };

    if let Err(e) = watcher.watch(&watch_dir, RecursiveMode::NonRecursive) {
        log::error!("Failed to watch config dir: {}", e);
        return;
    }

    log::info!("Watching config at {}", path.display());

    let mut last_emit = std::time::Instant::now() - Duration::from_millis(250);

    for event in rx {
        match event {
            Ok(ev) => {
                let dominated_by_config = ev.paths.iter().any(|p| p.ends_with("config.json"));
                let is_modify = matches!(ev.kind, EventKind::Modify(_) | EventKind::Create(_));

                if dominated_by_config
                    && is_modify
                    && last_emit.elapsed() >= Duration::from_millis(250)
                {
                    last_emit = std::time::Instant::now();

                    match fs::read_to_string(&path) {
                        Ok(contents) => match serde_json::from_str::<VantaConfig>(&contents) {
                            Ok(new_config) => {
                                log::info!("Config updated, emitting event");
                                let _ = app_handle.emit("config-updated", &new_config);
                            }
                            Err(e) => {
                                log::warn!("Config parse error after change: {}", e);
                            }
                        },
                        Err(e) => {
                            log::warn!("Could not read config after change: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                log::error!("Config watcher error: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

        #[test]
        fn accessibility_defaults_and_legacy_backcompat() {
                let cfg = VantaConfig::default();
                assert_eq!(cfg.schema_version, CONFIG_SCHEMA_VERSION);
                assert!(!cfg.accessibility.reduced_motion);
                assert_eq!(cfg.accessibility.text_scale, 1.0);
                assert_eq!(cfg.accessibility.spacing_preset, "comfortable");
                assert_eq!(cfg.workflows.schema_version, WORKFLOWS_SCHEMA_VERSION);
                assert_eq!(cfg.profiles.schema_version, PROFILES_SCHEMA_VERSION);
                assert_eq!(cfg.profiles.active_profile_id, "default");
                assert_eq!(cfg.profiles.entries.len(), 1);
                assert_eq!(cfg.notes.max_entries, 200);
                assert!(cfg.notes.entries.is_empty());
                assert_eq!(cfg.bookmarks.max_entries, 200);
                assert!(cfg.bookmarks.entries.is_empty());

                // Legacy payload without accessibility should still deserialize.
                let legacy = r##"{
                        "general": {"hotkey":"Alt+Space","max_results":8,"launch_on_login":false},
                        "appearance": {
                            "blur_radius":40,
                            "opacity":0.85,
                            "border_radius":24,
                            "theme":"default",
                            "colors": {
                                "background":"#000000","surface":"#0A0A0A","accent":"#FFFFFF",
                                "accent_glow":"rgba(255, 255, 255, 0.25)","text_primary":"#F5F5F5",
                                "text_secondary":"#888888","border":"rgba(255, 255, 255, 0.08)"
                            }
                        },
                        "window": {"width":680.0,"height":420.0},
                        "extensions": {"directory":"~/.config/vanta/extensions","dev_mode":false},
                        "files": {
                            "include_hidden":false,
                            "max_depth":3,
                            "file_manager":"default",
                            "file_editor":"default",
                            "open_docs_in_manager":false,
                            "include_globs":[],
                            "exclude_globs":[],
                            "allowed_extensions":[],
                            "type_filter":"any",
                            "indexed_at":null
                        },
                        "search": {
                            "applications": {"enabled":true,"weight":100},
                            "windows": {"enabled":true,"weight":100},
                            "calculator": {"enabled":true,"weight":100},
                            "files": {"enabled":true,"weight":100},
                            "windows_max_results":0
                        },
                        "workflows": {"macros":[]}
                }"##;

                let parsed: VantaConfig = serde_json::from_str(legacy).expect("legacy config parse");
                assert_eq!(parsed.schema_version, CONFIG_SCHEMA_VERSION);
                assert!(!parsed.accessibility.reduced_motion);
                assert_eq!(parsed.accessibility.text_scale, 1.0);
                assert_eq!(parsed.accessibility.spacing_preset, "comfortable");
                assert_eq!(parsed.workflows.schema_version, WORKFLOWS_SCHEMA_VERSION);
                assert_eq!(parsed.profiles.active_profile_id, "default");
                assert_eq!(parsed.profiles.entries.len(), 1);
                assert_eq!(parsed.notes.max_entries, 200);
                assert!(parsed.notes.entries.is_empty());
                assert_eq!(parsed.bookmarks.max_entries, 200);
                assert!(parsed.bookmarks.entries.is_empty());
        }

        #[test]
        fn accessibility_clamp_enforces_safe_ranges() {
                let mut cfg = AccessibilityConfig {
                        reduced_motion: false,
                        text_scale: 2.4,
                        spacing_preset: "unknown".to_string(),
                };

                assert!(clamp_accessibility(&mut cfg));
                assert_eq!(cfg.text_scale, 1.4);
                assert_eq!(cfg.spacing_preset, "comfortable");
        }

    #[test]
    fn files_config_defaults_and_serde_round_trip() {
        let cfg = FilesConfig::default();

        assert!(!cfg.include_hidden);
        assert_eq!(cfg.max_depth, 3);
        assert_eq!(cfg.file_manager, "default");
        assert_eq!(cfg.file_editor, "default");
        assert!(!cfg.open_docs_in_manager);
        assert!(cfg.include_globs.is_empty());
        assert!(cfg.exclude_globs.is_empty());
        assert!(cfg.allowed_extensions.is_empty());
        assert_eq!(cfg.type_filter, "any");
        assert!(cfg.indexed_at.is_none());

        let serialized = serde_json::to_string(&cfg).expect("serialize FilesConfig");
        let deserialized: FilesConfig = serde_json::from_str(&serialized).expect("deserialize FilesConfig");

        assert_eq!(deserialized.include_hidden, cfg.include_hidden);
        assert_eq!(deserialized.max_depth, cfg.max_depth);
        assert_eq!(deserialized.file_manager, cfg.file_manager);
        assert_eq!(deserialized.file_editor, cfg.file_editor);
        assert_eq!(deserialized.open_docs_in_manager, cfg.open_docs_in_manager);
        assert_eq!(deserialized.include_globs, cfg.include_globs);
        assert_eq!(deserialized.exclude_globs, cfg.exclude_globs);
        assert_eq!(deserialized.allowed_extensions, cfg.allowed_extensions);
        assert_eq!(deserialized.type_filter, cfg.type_filter);
        assert_eq!(deserialized.indexed_at, cfg.indexed_at);
    }

    #[test]
    fn workflows_macro_round_trip() {
        let cfg = WorkflowsConfig {
            schema_version: WORKFLOWS_SCHEMA_VERSION,
            macros: vec![WorkflowMacro {
                id: "open-and-sync".to_string(),
                name: "Open and Sync".to_string(),
                description: Some("Open a project folder then sync it".to_string()),
                enabled: true,
                args: vec![
                    MacroArg {
                        name: "project_path".to_string(),
                        description: Some("Path to the project root".to_string()),
                        required: true,
                        default_value: None,
                    },
                    MacroArg {
                        name: "branch".to_string(),
                        description: Some("Branch name".to_string()),
                        required: false,
                        default_value: Some("main".to_string()),
                    },
                ],
                steps: vec![
                    MacroStep::System {
                        command: "xdg-open".to_string(),
                        args: vec!["{project_path}".to_string()],
                        capabilities: vec![Capability::Filesystem],
                        on_error: StepErrorHandling::default(),
                        timeout_ms: None,
                        timeout_behavior: TimeoutBehavior::Abort,
                    },
                    MacroStep::Extension {
                        ext_id: "sync-project".to_string(),
                        command: "sync".to_string(),
                        args: vec!["--branch".to_string(), "{branch}".to_string()],
                        capabilities: vec![Capability::Shell],
                        on_error: StepErrorHandling::default(),
                        timeout_ms: None,
                        timeout_behavior: TimeoutBehavior::Abort,
                    },
                ],
                timeout_ms: None,
                timeout_behavior: TimeoutBehavior::Abort,
                schedule: None,
            }],
            command_templates: vec![CommandTemplate {
                id: "template-1".to_string(),
                name: "Open Sync Main".to_string(),
                macro_id: "open-and-sync".to_string(),
                args: HashMap::from([
                    ("project_path".to_string(), "/tmp/project".to_string()),
                    ("branch".to_string(), "main".to_string()),
                ]),
                created_at_ms: 1,
                updated_at_ms: 2,
            }],
        };

        let serialized = serde_json::to_string(&cfg).expect("serialize workflows config");
        let parsed: WorkflowsConfig = serde_json::from_str(&serialized).expect("deserialize workflows config");

        assert_eq!(parsed.schema_version, WORKFLOWS_SCHEMA_VERSION);
        assert_eq!(parsed.macros.len(), 1);
        assert_eq!(parsed.command_templates.len(), 1);
        assert_eq!(parsed.command_templates[0].name, "Open Sync Main");
        assert_eq!(parsed.command_templates[0].macro_id, "open-and-sync");
        assert_eq!(
            parsed.command_templates[0].args.get("branch"),
            Some(&"main".to_string())
        );
        let macro_def = &parsed.macros[0];
        assert!(macro_def.enabled);
        assert_eq!(macro_def.id, "open-and-sync");
        assert_eq!(macro_def.args.len(), 2);
        assert_eq!(macro_def.steps.len(), 2);

        match &macro_def.steps[0] {
            MacroStep::System {
                command,
                args,
                capabilities,
                ..
            } => {
                assert_eq!(command, "xdg-open");
                assert_eq!(args, &vec!["{project_path}".to_string()]);
                assert_eq!(capabilities, &vec![Capability::Filesystem]);
            }
            _ => panic!("expected system step"),
        }

        match &macro_def.steps[1] {
            MacroStep::Extension {
                ext_id,
                command,
                args,
                capabilities,
                ..
            } => {
                assert_eq!(ext_id, "sync-project");
                assert_eq!(command, "sync");
                assert_eq!(args, &vec!["--branch".to_string(), "{branch}".to_string()]);
                assert_eq!(capabilities, &vec![Capability::Shell]);
            }
            _ => panic!("expected extension step"),
        }
    }

    #[test]
    fn switch_profile_applies_runtime_fields() {
        let mut cfg = VantaConfig::default();
        cfg.profiles.entries.push(ProfileConfig {
            id: "work".to_string(),
            name: "Work".to_string(),
            hotkey: "Ctrl+Space".to_string(),
            theme: "high-contrast".to_string(),
            search: SearchConfig {
                applications: SourcePreference { enabled: true, weight: 130 },
                windows: SourcePreference { enabled: true, weight: 110 },
                calculator: SourcePreference { enabled: true, weight: 80 },
                files: SourcePreference { enabled: true, weight: 140 },
                windows_max_results: 6,
                show_explain_panel: true,
            },
        });

        switch_profile_in_config(&mut cfg, "work").expect("switch profile");

        assert_eq!(cfg.profiles.active_profile_id, "work");
        assert_eq!(cfg.general.hotkey, "Ctrl+Space");
        assert_eq!(cfg.appearance.theme, "high-contrast");
        assert_eq!(cfg.search.files.weight, 140);
        assert_eq!(cfg.search.windows_max_results, 6);
    }

    #[test]
    fn import_profile_rejects_future_export_schema() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("profile.json");

        let payload = serde_json::json!({
            "schema_version": PROFILE_EXPORT_SCHEMA_VERSION + 1,
            "app_schema_version": CONFIG_SCHEMA_VERSION,
            "profile": {
                "id": "future",
                "name": "Future",
                "hotkey": "Alt+P",
                "theme": "default",
                "search": SearchConfig::default(),
            }
        });

        std::fs::write(&path, serde_json::to_string_pretty(&payload).unwrap()).expect("write payload");

        let mut cfg = VantaConfig::default();
        let err = import_profile_from_path(&mut cfg, path.to_str().unwrap(), false)
            .expect_err("should reject future schema");
        assert!(err.to_string().contains("newer than supported"));
    }

    #[test]
    fn config_diff_tracks_nested_changes() {
        let mut before = VantaConfig::default();
        let mut after = before.clone();
        after.window.width = 777.0;
        after.search.windows_max_results = 9;
        after.profiles.active_profile_id = "work".to_string();

        let diff = diff_configs(Some(&before), &after);
        assert!(diff.iter().any(|d| d.path == "window.width"));
        assert!(diff.iter().any(|d| d.path == "search.windows_max_results"));
        assert!(diff.iter().any(|d| d.path == "profiles.active_profile_id"));

        before.window.width = 777.0;
        before.search.windows_max_results = 9;
        before.profiles.active_profile_id = "work".to_string();
        let no_diff = diff_configs(Some(&before), &after);
        assert!(no_diff.is_empty());
    }

    #[test]
    fn notes_config_defaults_and_round_trip() {
        let cfg = NotesConfig::default();
        assert!(cfg.entries.is_empty());
        assert_eq!(cfg.max_entries, 200);

        let raw = serde_json::to_string(&cfg).expect("serialize notes config");
        let parsed: NotesConfig = serde_json::from_str(&raw).expect("deserialize notes config");

        assert_eq!(parsed, cfg);
    }

    #[test]
    fn bookmarks_config_defaults_and_round_trip() {
        let cfg = BookmarksConfig::default();
        assert!(cfg.entries.is_empty());
        assert_eq!(cfg.max_entries, 200);

        let raw = serde_json::to_string(&cfg).expect("serialize bookmarks config");
        let parsed: BookmarksConfig = serde_json::from_str(&raw).expect("deserialize bookmarks config");

        assert_eq!(parsed, cfg);
    }
}

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::Emitter;

use crate::permissions::Capability;

// Embedded default config for fallback writes.
const DEFAULT_CONFIG_JSON: &str = include_str!("../resources/config.json");

//
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VantaConfig {
    pub general: GeneralConfig,
    pub appearance: AppearanceConfig,
    pub window: WindowConfig,
    #[serde(default)]
    pub extensions: ExtensionsConfig,
    #[serde(default)]
    pub files: FilesConfig,
    #[serde(default)]
    pub search: SearchConfig,
    #[serde(default)]
    pub workflows: WorkflowsConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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
        }
    }
}

fn default_windows_cap() -> usize {
    0
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub hotkey: String,
    pub max_results: usize,
    pub launch_on_login: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WindowConfig {
    pub width: f64,
    pub height: f64,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppearanceConfig {
    pub blur_radius: u32,
    pub opacity: f64,
    pub border_radius: u32,
    #[serde(default = "default_theme")]
    pub theme: String,
    pub colors: ColorsConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ColorsConfig {
    pub background: String,
    pub surface: String,
    pub accent: String,
    pub accent_glow: String,
    pub text_primary: String,
    pub text_secondary: String,
    pub border: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WorkflowsConfig {
    #[serde(default)]
    pub macros: Vec<WorkflowMacro>,
}

impl Default for WorkflowsConfig {
    fn default() -> Self {
        Self {
            macros: default_workflow_macros(),
        }
    }
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
            },
            MacroStep::Extension {
                ext_id: "sync-project".to_string(),
                command: "sync".to_string(),
                args: vec!["--branch".to_string(), "{branch}".to_string()],
                capabilities: vec![Capability::Shell],
            },
        ],
    }]
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
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
}

fn default_macro_enabled() -> bool {
    true
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MacroArg {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub default_value: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MacroStep {
    Extension {
        ext_id: String,
        command: String,
        #[serde(default)]
        args: Vec<String>,
        #[serde(default)]
        capabilities: Vec<Capability>,
    },
    System {
        command: String,
        #[serde(default)]
        args: Vec<String>,
        #[serde(default)]
        capabilities: Vec<Capability>,
    },
}

impl Default for VantaConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig {
                hotkey: "Alt+Space".to_string(),
                max_results: 8,
                launch_on_login: false,
            },
            window: WindowConfig {
                width: 680.0,
                height: 420.0,
            },
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
        }
    }
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

/// Load config from disk or create the default config file.
pub fn load_or_create_default() -> VantaConfig {
    let path = config_path();

    if path.exists() {
        match fs::read_to_string(&path) {
            Ok(contents) => match serde_json::from_str::<VantaConfig>(&contents) {
                Ok(mut config) => {
                    log::info!("Loaded config from {}", path.display());
                    // Clamp window bounds eagerly and persist if adjusted.
                    if clamp_window_size(&mut config.window) {
                        let _ = write_config(&config);
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
            },
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

impl VantaConfig {
    /// Save the current configuration to disk.
    pub fn save(&self) -> Result<(), String> {
        write_config(self)
    }
}

fn write_config(cfg: &VantaConfig) -> Result<(), String> {
    let path = config_path();
    let json = serde_json::to_string_pretty(cfg)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&path, json)
        .map_err(|e| format!("Failed to write config to {}: {}", path.display(), e))?;

    Ok(())
}

/// Attempt to rewrite the default config using serialized defaults, falling back
/// to an embedded copy if serialization or write fails.
fn rewrite_default_config() -> VantaConfig {
    let mut default_config = VantaConfig::default();

    // Clamp before writing defaults.
    let _ = clamp_window_size(&mut default_config.window);

    // First try normal serialization+write.
    if let Err(write_err) = write_config(&default_config) {
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
                    },
                    MacroStep::Extension {
                        ext_id: "sync-project".to_string(),
                        command: "sync".to_string(),
                        args: vec!["--branch".to_string(), "{branch}".to_string()],
                        capabilities: vec![Capability::Shell],
                    },
                ],
            }],
        };

        let serialized = serde_json::to_string(&cfg).expect("serialize workflows config");
        let parsed: WorkflowsConfig = serde_json::from_str(&serialized).expect("deserialize workflows config");

        assert_eq!(parsed.macros.len(), 1);
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
            } => {
                assert_eq!(ext_id, "sync-project");
                assert_eq!(command, "sync");
                assert_eq!(args, &vec!["--branch".to_string(), "{branch}".to_string()]);
                assert_eq!(capabilities, &vec![Capability::Shell]);
            }
            _ => panic!("expected extension step"),
        }
    }
}

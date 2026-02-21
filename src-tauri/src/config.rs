use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::Emitter;

//
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VantaConfig {
    pub general: GeneralConfig,
    pub appearance: AppearanceConfig,
    pub window: WindowConfig,
    pub scripts: ScriptsConfig,
    #[serde(default)]
    pub files: FilesConfig,
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
}

fn default_max_depth() -> usize {
    3
}

fn default_opener() -> String {
    "default".to_string()
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
pub struct ScriptsConfig {
    pub directory: String,
    pub timeout_ms: u64,
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
                width: 800.0,
                height: 600.0,
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
            scripts: ScriptsConfig {
                directory: "~/.config/vanta/scripts".to_string(),
                timeout_ms: 5000,
            },
            files: FilesConfig {
                include_hidden: false,
                max_depth: 3,
                file_manager: "default".to_string(),
                file_editor: "default".to_string(),
                open_docs_in_manager: false,
            },
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
                Ok(config) => {
                    log::info!("Loaded config from {}", path.display());
                    return config;
                }
                Err(e) => {
                    log::warn!("Invalid config.json, using defaults: {}", e);
                }
            },
            Err(e) => {
                log::warn!("Could not read config.json, using defaults: {}", e);
            }
        }
    }

    // Create default config
    let default_config = VantaConfig::default();
    let dir = config_dir();

    if let Err(e) = fs::create_dir_all(&dir) {
        log::error!("Could not create config directory {}: {}", dir.display(), e);
        return default_config;
    }

    // Also create the scripts directory
    let scripts_dir = dir.join("scripts");
    if let Err(e) = fs::create_dir_all(&scripts_dir) {
        log::warn!("Could not create scripts directory: {}", e);
    }

    match serde_json::to_string_pretty(&default_config) {
        Ok(json) => {
            if let Err(e) = fs::write(&path, &json) {
                log::error!("Could not write default config: {}", e);
            } else {
                log::info!("Created default config at {}", path.display());
            }
        }
        Err(e) => {
            log::error!("Could not serialize default config: {}", e);
        }
    }

    default_config
}

impl VantaConfig {
    /// Save the current configuration to disk.
    pub fn save(&self) -> Result<(), String> {
        let path = config_path();
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write(&path, json)
            .map_err(|e| format!("Failed to write config to {}: {}", path.display(), e))?;

        Ok(())
    }
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

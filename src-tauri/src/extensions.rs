use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::config;
use crate::permissions::{self, Capability, Decision};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum CommandMode {
    View,
    NoView,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExtensionCommand {
    pub name: String,
    pub title: String,
    #[serde(default)]
    pub subtitle: Option<String>,
    pub mode: CommandMode,
    #[serde(default)]
    pub icon: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExtensionManifest {
    pub name: String,
    pub title: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub permissions: Vec<Capability>,
    pub commands: Vec<ExtensionCommand>,
}

fn default_version() -> String {
    "0.1.0".to_string()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExtensionEntry {
    pub manifest: ExtensionManifest,
    pub path: String,
    pub has_bundle: bool,
    pub has_styles: bool,
}

pub fn extensions_dir() -> PathBuf {
    let dir = config::config_dir().join("extensions");
    if !dir.exists() {
        if let Err(e) = fs::create_dir_all(&dir) {
            log::warn!("Could not create extensions directory: {}", e);
        }
    }
    dir
}

fn validate_manifest(manifest: &ExtensionManifest, ext_path: &Path) -> Result<(), String> {
    if manifest.name.is_empty() {
        return Err("Extension name is empty".to_string());
    }

    if manifest.name.contains("..") || manifest.name.contains('/') || manifest.name.contains('\\') {
        return Err(format!(
            "Extension name '{}' contains path traversal characters",
            manifest.name
        ));
    }

    if manifest.commands.is_empty() {
        return Err(format!(
            "Extension '{}' has no commands defined",
            manifest.name
        ));
    }

    for cmd in &manifest.commands {
        if cmd.name.is_empty() || cmd.title.is_empty() {
            return Err(format!(
                "Extension '{}' has a command with empty name or title",
                manifest.name
            ));
        }
    }

    let canonical = ext_path
        .canonicalize()
        .map_err(|e| format!("Cannot resolve extension path: {}", e))?;
    let ext_dir_canonical = extensions_dir()
        .canonicalize()
        .unwrap_or_else(|_| extensions_dir());
    if !canonical.starts_with(&ext_dir_canonical) {
        return Err(format!(
            "Extension path escapes the extensions directory: {}",
            canonical.display()
        ));
    }

    Ok(())
}

pub fn scan_extensions() -> Vec<ExtensionEntry> {
    let dir = extensions_dir();
    if !dir.exists() {
        return Vec::new();
    }

    let read_dir = match fs::read_dir(&dir) {
        Ok(rd) => rd,
        Err(e) => {
            log::warn!("Could not read extensions dir: {}", e);
            return Vec::new();
        }
    };

    let mut entries = Vec::new();

    for entry in read_dir.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let manifest_path = path.join("manifest.json");
        if !manifest_path.exists() {
            continue;
        }

        let manifest_str = match fs::read_to_string(&manifest_path) {
            Ok(s) => s,
            Err(e) => {
                log::warn!(
                    "Could not read manifest at {}: {}",
                    manifest_path.display(),
                    e
                );
                continue;
            }
        };

        let manifest: ExtensionManifest = match serde_json::from_str(&manifest_str) {
            Ok(m) => m,
            Err(e) => {
                log::warn!(
                    "Invalid manifest at {}: {}",
                    manifest_path.display(),
                    e
                );
                continue;
            }
        };

        if let Err(e) = validate_manifest(&manifest, &path) {
            log::warn!("Manifest validation failed for {}: {}", path.display(), e);
            continue;
        }

        let has_bundle = path.join("dist").join("index.js").exists();
        let has_styles = path.join("dist").join("style.css").exists();

        entries.push(ExtensionEntry {
            manifest,
            path: path.to_string_lossy().to_string(),
            has_bundle,
            has_styles,
        });
    }

    entries.sort_by(|a, b| a.manifest.name.cmp(&b.manifest.name));
    log::info!("Discovered {} extensions", entries.len());
    entries
}

pub fn load_extension_bundle(ext_id: &str) -> Result<String, String> {
    let dir = extensions_dir();
    let bundle_path = dir.join(ext_id).join("dist").join("index.js");

    if !bundle_path.exists() {
        return Err(format!(
            "No compiled bundle found for extension '{}'. Expected at {}",
            ext_id,
            bundle_path.display()
        ));
    }

    let canonical = bundle_path
        .canonicalize()
        .map_err(|e| format!("Cannot resolve bundle path: {}", e))?;
    let ext_dir_canonical = dir.canonicalize().unwrap_or_else(|_| dir.clone());
    if !canonical.starts_with(&ext_dir_canonical) {
        return Err("Bundle path escapes extensions directory".to_string());
    }

    fs::read_to_string(&bundle_path).map_err(|e| {
        format!(
            "Failed to read bundle for '{}': {}",
            ext_id, e
        )
    })
}

pub fn load_extension_styles(ext_id: &str) -> Result<Option<String>, String> {
    let dir = extensions_dir();
    let style_path = dir.join(ext_id).join("dist").join("style.css");

    if !style_path.exists() {
        return Ok(None);
    }

    let css = fs::read_to_string(&style_path).map_err(|e| {
        format!(
            "Failed to read styles for '{}': {}",
            ext_id, e
        )
    })?;

    Ok(Some(css))
}

#[derive(Debug)]
pub enum PermissionError {
    Deny,
    NeedsPrompt { missing_caps: Vec<Capability> },
}

pub fn check_extension_permissions(
    ext_id: &str,
    requested_caps: &[Capability],
) -> Result<(), PermissionError> {
    if requested_caps.is_empty() {
        return Ok(());
    }

    let response = permissions::get_decision_for(ext_id, requested_caps);

    match response.decision {
        Decision::Allow if response.missing_caps.is_empty() => Ok(()),
        Decision::Allow | Decision::Ask => Err(PermissionError::NeedsPrompt {
            missing_caps: response.missing_caps,
        }),
        Decision::Deny => {
            for capability in requested_caps {
                if let Err(err) =
                    permissions::record_block_event(ext_id, capability.clone(), None)
                {
                    log::warn!(
                        "Failed to record block event for {}: {}",
                        ext_id,
                        err
                    );
                }
            }
            Err(PermissionError::Deny)
        }
    }
}

pub fn watch_extensions(app_handle: tauri::AppHandle) {
    use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
    use std::sync::mpsc;
    use tauri::Emitter;

    let dir = extensions_dir();
    if !dir.exists() {
        log::warn!("Extensions directory does not exist, skipping watcher");
        return;
    }

    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();

    let mut watcher: RecommendedWatcher = match Watcher::new(
        tx,
        notify::Config::default().with_poll_interval(Duration::from_secs(2)),
    ) {
        Ok(w) => w,
        Err(e) => {
            log::error!("Failed to create extensions watcher: {}", e);
            return;
        }
    };

    if let Err(e) = watcher.watch(&dir, RecursiveMode::Recursive) {
        log::error!("Failed to watch extensions dir: {}", e);
        return;
    }

    log::info!("Watching extensions directory: {}", dir.display());

    let mut last_scan = std::time::Instant::now() - Duration::from_millis(800);

    for event in rx {
        match event {
            Ok(ev) => {
                let is_relevant = matches!(
                    ev.kind,
                    EventKind::Create(_) | EventKind::Remove(_) | EventKind::Modify(_)
                );

                if is_relevant && last_scan.elapsed() > Duration::from_millis(800) {
                    last_scan = std::time::Instant::now();

                    let extensions = scan_extensions();
                    let seeds: Vec<(String, Vec<Capability>)> = extensions
                        .iter()
                        .map(|e| (e.manifest.name.clone(), e.manifest.permissions.clone()))
                        .collect();
                    if let Err(err) = permissions::seed_missing_decisions(&seeds) {
                        log::warn!(
                            "Failed to seed permissions for extensions: {}",
                            err
                        );
                    }
                    log::info!("Extensions re-scanned: {} extensions", extensions.len());
                    let _ = app_handle.emit("extensions-changed", &extensions);
                }
            }
            Err(e) => {
                log::error!("Extensions watcher error: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_manifest_full() {
        let json = r#"{
            "name": "test-ext",
            "title": "Test Extension",
            "version": "1.0.0",
            "description": "A test extension",
            "author": "tester",
            "icon": "fa-solid fa-star",
            "permissions": ["Network"],
            "commands": [
                {
                    "name": "hello",
                    "title": "Say Hello",
                    "subtitle": "Greets the user",
                    "mode": "no-view",
                    "icon": "fa-solid fa-hand-wave"
                },
                {
                    "name": "list-items",
                    "title": "List Items",
                    "mode": "view"
                }
            ]
        }"#;

        let manifest: ExtensionManifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.name, "test-ext");
        assert_eq!(manifest.title, "Test Extension");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.commands.len(), 2);
        assert_eq!(manifest.commands[0].mode, CommandMode::NoView);
        assert_eq!(manifest.commands[1].mode, CommandMode::View);
        assert_eq!(manifest.permissions.len(), 1);
    }

    #[test]
    fn parse_manifest_minimal() {
        let json = r#"{
            "name": "minimal",
            "title": "Minimal",
            "commands": [{"name": "run", "title": "Run", "mode": "no-view"}]
        }"#;

        let manifest: ExtensionManifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.name, "minimal");
        assert_eq!(manifest.version, "0.1.0");
        assert!(manifest.description.is_none());
        assert!(manifest.permissions.is_empty());
    }

    #[test]
    fn validate_rejects_empty_name() {
        let manifest = ExtensionManifest {
            name: "".to_string(),
            title: "Test".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            author: None,
            icon: None,
            permissions: vec![],
            commands: vec![ExtensionCommand {
                name: "run".to_string(),
                title: "Run".to_string(),
                subtitle: None,
                mode: CommandMode::NoView,
                icon: None,
            }],
        };
        assert!(validate_manifest(&manifest, Path::new("/tmp/test")).is_err());
    }

    #[test]
    fn validate_rejects_path_traversal() {
        let manifest = ExtensionManifest {
            name: "../escape".to_string(),
            title: "Bad".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            author: None,
            icon: None,
            permissions: vec![],
            commands: vec![ExtensionCommand {
                name: "run".to_string(),
                title: "Run".to_string(),
                subtitle: None,
                mode: CommandMode::NoView,
                icon: None,
            }],
        };
        assert!(validate_manifest(&manifest, Path::new("/tmp/test")).is_err());
    }

    #[test]
    fn validate_rejects_no_commands() {
        let manifest = ExtensionManifest {
            name: "empty".to_string(),
            title: "Empty".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            author: None,
            icon: None,
            permissions: vec![],
            commands: vec![],
        };
        assert!(validate_manifest(&manifest, Path::new("/tmp/test")).is_err());
    }

    #[test]
    fn extension_entry_serializes() {
        let entry = ExtensionEntry {
            manifest: ExtensionManifest {
                name: "test".to_string(),
                title: "Test".to_string(),
                version: "1.0.0".to_string(),
                description: None,
                author: None,
                icon: None,
                permissions: vec![],
                commands: vec![ExtensionCommand {
                    name: "run".to_string(),
                    title: "Run".to_string(),
                    subtitle: None,
                    mode: CommandMode::View,
                    icon: None,
                }],
            },
            path: "/home/user/.config/vanta/extensions/test".to_string(),
            has_bundle: true,
            has_styles: false,
        };

        let json = serde_json::to_string(&entry).unwrap();
        let parsed: ExtensionEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.manifest.name, "test");
        assert!(parsed.has_bundle);
        assert!(!parsed.has_styles);
    }
}

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use tauri::Manager;

use crate::config;
use crate::errors::VantaError;
use crate::permissions::{self, Capability, Decision};

pub const EXTENSION_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Serialize)]
pub struct ExtensionMigrationReport {
    pub scanned: usize,
    pub updated: usize,
    pub failed: Vec<String>,
}

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
pub struct ExtensionDependency {
    pub ext_id: String,
    pub version_range: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ExtensionPermissionScopes {
    #[serde(default)]
    pub filesystem_paths: Vec<String>,
    #[serde(default)]
    pub network_domains: Vec<String>,
    #[serde(default)]
    pub shell_commands: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExtensionManifest {
    pub name: String,
    pub title: String,
    #[serde(default = "default_extension_schema_version")]
    pub schema_version: u32,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub publisher: Option<String>,
    #[serde(default)]
    pub safe: Option<bool>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub permissions: Vec<Capability>,
    #[serde(default)]
    pub requires: Vec<ExtensionDependency>,
    #[serde(default)]
    pub permission_scopes: ExtensionPermissionScopes,
    pub commands: Vec<ExtensionCommand>,
}

fn default_version() -> String {
    "0.1.0".to_string()
}

fn default_extension_schema_version() -> u32 {
    EXTENSION_SCHEMA_VERSION
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

fn is_valid_extension_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

#[tauri::command]
pub async fn create_extension_template(name: String) -> Result<String, VantaError> {
    let normalized = name.trim().to_ascii_lowercase();
    if !is_valid_extension_name(&normalized) {
        return Err("Extension name must use letters, numbers, '-' or '_'".into());
    }

    let ext_root = extensions_dir().join(&normalized);
    if ext_root.exists() {
        return Err(format!("Extension '{}' already exists", normalized).into());
    }

    fs::create_dir_all(ext_root.join("dist"))
        .map_err(|e| format!("Failed to create template directory: {}", e))?;

    let manifest = serde_json::json!({
        "name": normalized,
        "title": format!("{} Extension", name.trim()),
        "schema_version": EXTENSION_SCHEMA_VERSION,
        "version": "0.1.0",
        "description": "Starter extension template",
        "author": "You",
        "publisher": "Community",
        "safe": false,
        "permissions": [],
        "requires": [],
        "permission_scopes": {
            "filesystem_paths": [],
            "network_domains": [],
            "shell_commands": []
        },
        "commands": [
            {
                "name": "hello",
                "title": "Hello",
                "subtitle": "Starter command",
                "mode": "no-view",
                "icon": "fa-solid fa-wand-magic-sparkles"
            }
        ]
    });

    let manifest_str = serde_json::to_string_pretty(&manifest)
        .map_err(|e| format!("Failed to serialize template manifest: {}", e))?;

    let starter_js = r#"(function(vanta) {
  vanta.registerExtension('EXTENSION_NAME', {
    commands: {
      hello: {
        handler: async (api) => {
          api.toast({
            title: 'Template Ready',
            message: 'Edit dist/index.js and manifest.json to build your extension.',
            type: 'success'
          });
        }
      }
    }
  });
})(window.__vanta_host);
"#
    .replace("EXTENSION_NAME", &normalized);

    let starter_css = "/* Extension-scoped styles */\n";
    let starter_readme = format!(
        "# {}\n\nYour extension template is ready.\n\n1. Edit `manifest.json`\n2. Update `dist/index.js`\n3. Reload Vanta and search your command title.\n",
        normalized
    );

    fs::write(ext_root.join("manifest.json"), manifest_str)
        .map_err(|e| format!("Failed to write manifest.json: {}", e))?;
    fs::write(ext_root.join("dist").join("index.js"), starter_js)
        .map_err(|e| format!("Failed to write dist/index.js: {}", e))?;
    fs::write(ext_root.join("dist").join("style.css"), starter_css)
        .map_err(|e| format!("Failed to write dist/style.css: {}", e))?;
    fs::write(ext_root.join("README.md"), starter_readme)
        .map_err(|e| format!("Failed to write README.md: {}", e))?;

    Ok(ext_root.to_string_lossy().to_string())
}

fn validate_manifest(manifest: &ExtensionManifest, ext_path: &Path) -> Result<(), VantaError> {
    if manifest.name.is_empty() {
        return Err("Extension name is empty".into());
    }

    if manifest.name.contains("..") || manifest.name.contains('/') || manifest.name.contains('\\') {
        return Err(format!(
            "Extension name '{}' contains path traversal characters",
            manifest.name
        ).into());
    }

    if manifest.commands.is_empty() {
        return Err(format!(
            "Extension '{}' has no commands defined",
            manifest.name
        ).into());
    }

    for cmd in &manifest.commands {
        if cmd.name.is_empty() || cmd.title.is_empty() {
            return Err(format!(
                "Extension '{}' has a command with empty name or title",
                manifest.name
            ).into());
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
        ).into());
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

        if let Err(e) = migrate_manifest_file(&manifest_path) {
            log::warn!("Manifest migration failed for {}: {}", manifest_path.display(), e);
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

fn migrate_manifest_file(path: &Path) -> Result<bool, VantaError> {
    let raw = fs::read_to_string(path)
        .map_err(|e| format!("failed to read manifest: {}", e))?;
    let mut value: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|e| format!("failed to parse manifest json: {}", e))?;

    let Some(obj) = value.as_object_mut() else {
        return Err("manifest root is not an object".into());
    };

    let mut changed = false;
    match obj.get("schema_version").and_then(|v| v.as_u64()) {
        Some(v) if v >= EXTENSION_SCHEMA_VERSION as u64 => {}
        _ => {
            obj.insert(
                "schema_version".to_string(),
                serde_json::json!(EXTENSION_SCHEMA_VERSION),
            );
            changed = true;
        }
    }

    if changed {
        let serialized = serde_json::to_string_pretty(&value)
            .map_err(|e| format!("failed to serialize migrated manifest: {}", e))?;
        fs::write(path, serialized)
            .map_err(|e| format!("failed to write migrated manifest: {}", e))?;
    }

    Ok(changed)
}

pub fn migrate_extension_manifests() -> ExtensionMigrationReport {
    let dir = extensions_dir();
    let mut report = ExtensionMigrationReport {
        scanned: 0,
        updated: 0,
        failed: Vec::new(),
    };

    let Ok(read_dir) = fs::read_dir(&dir) else {
        return report;
    };

    for entry in read_dir.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let manifest_path = path.join("manifest.json");
        if !manifest_path.exists() {
            continue;
        }

        report.scanned += 1;
        match migrate_manifest_file(&manifest_path) {
            Ok(true) => report.updated += 1,
            Ok(false) => {}
            Err(e) => report
                .failed
                .push(format!("{}: {}", manifest_path.display(), e)),
        }
    }

    report
}

pub fn load_extension_bundle(ext_id: &str) -> Result<String, VantaError> {
    let dir = extensions_dir();
    let bundle_path = dir.join(ext_id).join("dist").join("index.js");

    if !bundle_path.exists() {
        return Err(format!(
            "No compiled bundle found for extension '{}'. Expected at {}",
            ext_id,
            bundle_path.display()
        ).into());
    }

    let canonical = bundle_path
        .canonicalize()
        .map_err(|e| format!("Cannot resolve bundle path: {}", e))?;
    let ext_dir_canonical = dir.canonicalize().unwrap_or_else(|_| dir.clone());
    if !canonical.starts_with(&ext_dir_canonical) {
        return Err("Bundle path escapes extensions directory".into());
    }

    Ok(fs::read_to_string(&bundle_path).map_err(|e| {
        format!(
            "Failed to read bundle for '{}': {}",
            ext_id, e
        )
    })?)
}

pub fn load_extension_styles(ext_id: &str) -> Result<Option<String>, VantaError> {
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

    let cfg = config::load_or_create_default();
    if cfg.policy.restricted_mode
        && !cfg.policy.allowed_extensions.iter().any(|id| id == ext_id)
    {
        for capability in requested_caps {
            let _ = permissions::record_block_event(
                ext_id,
                capability.clone(),
                Some("blocked-by-policy-restricted-mode".to_string()),
            );
        }
        let _ = permissions::record_audit_event(
            "policy_extension_block",
            "system",
            ext_id,
            "denied",
            Some("restricted mode allowlist".to_string()),
        );
        return Err(PermissionError::Deny);
    }

    if let Some(blocked) = requested_caps
        .iter()
        .find(|cap| cfg.policy.blocked_capabilities.contains(cap))
    {
        let _ = permissions::record_block_event(
            ext_id,
            blocked.clone(),
            Some("blocked-by-policy-capability".to_string()),
        );
        let _ = permissions::record_audit_event(
            "policy_capability_block",
            "system",
            ext_id,
            "denied",
            Some(format!("blocked capability: {:?}", blocked)),
        );
        return Err(PermissionError::Deny);
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
                    if let Some(state) = app_handle.try_state::<crate::AppState>() {
                        if let Ok(mut cached) = state.extensions_cache.lock() {
                            *cached = extensions.clone();
                        }
                    }
                    let _ = app_handle.emit("extensions-changed", &extensions);
                }
            }
            Err(e) => {
                log::error!("Extensions watcher error: {}", e);
            }
        }
    }
}

fn load_extension_manifest(ext_id: &str) -> Result<ExtensionManifest, VantaError> {
    let manifest_path = extensions_dir().join(ext_id).join("manifest.json");
    if !manifest_path.exists() {
        return Err(format!("Extension '{}' is not installed", ext_id).into());
    }

    let manifest_str = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Could not read manifest for '{}': {}", ext_id, e))?;
    serde_json::from_str::<ExtensionManifest>(&manifest_str)
        .map_err(|e| format!("Invalid manifest for '{}': {}", ext_id, e).into())
}

fn ensure_extension_capability(
    ext_id: &str,
    manifest: &ExtensionManifest,
    capability: Capability,
) -> Result<(), VantaError> {
    if manifest.permissions.contains(&capability) {
        return Ok(());
    }
    let _ = permissions::record_block_event(
        ext_id,
        capability.clone(),
        Some("capability-not-declared".to_string()),
    );
    Err(format!("Extension '{}' is missing required capability '{:?}'", ext_id, capability).into())
}

fn command_basename(command: &str) -> &str {
    command.rsplit('/').next().unwrap_or(command)
}

fn is_shell_command_allowed(command: &str, allowed: &[String]) -> bool {
    if allowed.is_empty() {
        // Backward-compatibility: no scope list means legacy unrestricted shell capability.
        return true;
    }

    let cmd = command.trim();
    let base = command_basename(cmd);
    allowed
        .iter()
        .map(|s| s.trim())
        .any(|allowed_cmd| allowed_cmd.eq_ignore_ascii_case(cmd) || allowed_cmd.eq_ignore_ascii_case(base))
}

fn is_network_domain_allowed(url: &str, allowed: &[String]) -> bool {
    if allowed.is_empty() {
        // Backward-compatibility: no scope list means legacy unrestricted network capability.
        return true;
    }

    let parsed = match reqwest::Url::parse(url) {
        Ok(url) => url,
        Err(_) => return false,
    };

    let host = match parsed.host_str() {
        Some(host) => host.to_ascii_lowercase(),
        None => return false,
    };

    allowed.iter().any(|raw| {
        let item = raw.trim().to_ascii_lowercase();
        if item.is_empty() {
            return false;
        }

        if let Some(suffix) = item.strip_prefix("*.") {
            return host == suffix || host.ends_with(&format!(".{}", suffix));
        }

        host == item || host.ends_with(&format!(".{}", item))
    })
}

fn expand_home(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    }
    PathBuf::from(path)
}

fn is_absolute_or_home_path_token(token: &str) -> bool {
    token.starts_with('/') || token.starts_with("~/")
}

fn is_filesystem_path_allowed(path: &str, allowed_paths: &[String]) -> bool {
    if allowed_paths.is_empty() {
        // Backward-compatibility: no path scopes means legacy unrestricted filesystem capability.
        return true;
    }

    let candidate = expand_home(path);
    let candidate_normalized = candidate
        .canonicalize()
        .unwrap_or_else(|_| candidate.clone());

    allowed_paths.iter().any(|allowed| {
        let allowed_buf = expand_home(allowed.trim());
        let allowed_normalized = allowed_buf
            .canonicalize()
            .unwrap_or_else(|_| allowed_buf.clone());
        candidate_normalized.starts_with(&allowed_normalized)
    })
}

fn extension_data_dir() -> PathBuf {
    let dir = config::config_dir().join("extension-data");
    if !dir.exists() {
        if let Err(e) = fs::create_dir_all(&dir) {
            log::warn!("Could not create extension-data directory: {}", e);
        }
    }
    dir
}

fn load_storage_map(ext_id: &str) -> Result<HashMap<String, String>, VantaError> {
    let path = extension_data_dir().join(format!("{}.json", ext_id));
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let data = fs::read_to_string(&path).map_err(|e| format!("Failed to read storage: {}", e))?;
    Ok(serde_json::from_str(&data).map_err(|e| format!("Failed to parse storage: {}", e))?)
}

fn save_storage_map(ext_id: &str, map: &HashMap<String, String>) -> Result<(), VantaError> {
    let path = extension_data_dir().join(format!("{}.json", ext_id));
    let data = serde_json::to_string_pretty(map)
        .map_err(|e| format!("Failed to serialize storage: {}", e))?;
    Ok(fs::write(&path, data).map_err(|e| format!("Failed to write storage: {}", e))?)
}

#[tauri::command]
pub async fn extension_fetch(
    ext_id: String,
    url: String,
    method: Option<String>,
) -> Result<String, VantaError> {
    let manifest = load_extension_manifest(&ext_id)?;
    ensure_extension_capability(&ext_id, &manifest, Capability::Network)?;
    if !is_network_domain_allowed(&url, &manifest.permission_scopes.network_domains) {
        let _ = permissions::record_block_event(
            &ext_id,
            Capability::Network,
            Some(format!("domain-not-allowed: {}", url)),
        );
        return Err(format!("Network request blocked for extension '{}': domain is not allowed", ext_id).into());
    }

    let client = reqwest::Client::new();
    let req = match method.as_deref().unwrap_or("GET") {
        "POST" => client.post(&url),
        "PUT" => client.put(&url),
        "DELETE" => client.delete(&url),
        _ => client.get(&url),
    };
    let resp = req
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;
    Ok(resp.text()
        .await
        .map_err(|e| format!("Failed to read response body: {}", e))?)
}

#[tauri::command]
pub async fn extension_shell_execute(
    ext_id: String,
    command: String,
    args: Option<Vec<String>>,
) -> Result<String, VantaError> {
    let manifest = load_extension_manifest(&ext_id)?;
    ensure_extension_capability(&ext_id, &manifest, Capability::Shell)?;
    if !is_shell_command_allowed(&command, &manifest.permission_scopes.shell_commands) {
        let _ = permissions::record_block_event(
            &ext_id,
            Capability::Shell,
            Some(format!("shell-command-not-allowed: {}", command)),
        );
        return Err(format!("Shell command blocked for extension '{}': '{}' is not allowed", ext_id, command).into());
    }

    let args = args.unwrap_or_default();

    if manifest.permissions.contains(&Capability::Filesystem)
        && !manifest.permission_scopes.filesystem_paths.is_empty()
    {
        let mut path_tokens: Vec<String> = Vec::new();

        if is_absolute_or_home_path_token(&command) {
            path_tokens.push(command.clone());
        }

        for arg in &args {
            if arg.starts_with('-') {
                continue;
            }
            if is_absolute_or_home_path_token(arg) {
                path_tokens.push(arg.clone());
            }
        }

        if let Some(blocked) = path_tokens
            .iter()
            .find(|token| !is_filesystem_path_allowed(token, &manifest.permission_scopes.filesystem_paths))
        {
            let _ = permissions::record_block_event(
                &ext_id,
                Capability::Filesystem,
                Some(format!("filesystem-path-not-allowed: {}", blocked)),
            );
            return Err(format!(
                "Filesystem path blocked for extension '{}': '{}' is outside allowed paths",
                ext_id, blocked
            )
            .into());
        }
    }

    let output = Command::new(&command)
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to execute command '{}': {}", command, e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Command failed ({}): {}", output.status, stderr).into())
    }
}

#[tauri::command]
pub async fn extension_storage_get(
    ext_id: String,
    key: String,
) -> Result<Option<String>, VantaError> {
    let map = load_storage_map(&ext_id)?;
    Ok(map.get(&key).cloned())
}

#[tauri::command]
pub async fn extension_storage_set(
    ext_id: String,
    key: String,
    value: String,
) -> Result<(), VantaError> {
    let mut map = load_storage_map(&ext_id)?;
    map.insert(key, value);
    save_storage_map(&ext_id, &map)
}

pub fn resolve_ext_icon(icon: Option<&str>, ext_path: &str) -> Option<String> {
    let icon = icon?;
    if icon.starts_with("fa-") || icon.starts_with("<svg") || icon.starts_with("http") {
        return Some(icon.to_string());
    }
    let resolved = PathBuf::from(ext_path).join(icon);
    if resolved.exists() {
        Some(resolved.to_string_lossy().to_string())
    } else {
        Some(icon.to_string())
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
        assert_eq!(manifest.schema_version, 1);
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
        assert_eq!(manifest.schema_version, 1);
        assert_eq!(manifest.version, "0.1.0");
        assert!(manifest.description.is_none());
        assert!(manifest.permissions.is_empty());
    }

    #[test]
    fn validate_rejects_empty_name() {
        let manifest = ExtensionManifest {
            name: "".to_string(),
            title: "Test".to_string(),
            schema_version: 1,
            version: "1.0.0".to_string(),
            description: None,
            author: None,
            publisher: None,
            safe: None,
            icon: None,
            permissions: vec![],
            requires: vec![],
            permission_scopes: ExtensionPermissionScopes::default(),
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
            schema_version: 1,
            version: "1.0.0".to_string(),
            description: None,
            author: None,
            publisher: None,
            safe: None,
            icon: None,
            permissions: vec![],
            requires: vec![],
            permission_scopes: ExtensionPermissionScopes::default(),
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
            schema_version: 1,
            version: "1.0.0".to_string(),
            description: None,
            author: None,
            publisher: None,
            safe: None,
            icon: None,
            permissions: vec![],
            requires: vec![],
            permission_scopes: ExtensionPermissionScopes::default(),
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
                schema_version: 1,
                version: "1.0.0".to_string(),
                description: None,
                author: None,
                publisher: None,
                safe: None,
                icon: None,
                permissions: vec![],
                requires: vec![],
                permission_scopes: ExtensionPermissionScopes::default(),
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

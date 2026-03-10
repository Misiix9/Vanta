use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::extensions::{self, ExtensionManifest};
use crate::config;
use crate::permissions;

const REGISTRY_URL: &str =
    "https://raw.githubusercontent.com/Misiix9/vanta-extensions/main/registry.json";
const RATINGS_URL: &str =
    "https://raw.githubusercontent.com/Misiix9/vanta-extensions/main/ratings.json";
const BASE_URL: &str =
    "https://raw.githubusercontent.com/Misiix9/vanta-extensions/main/extensions";
const RATINGS_FEEDBACK_URL: &str =
    "https://github.com/Misiix9/vanta-extensions/issues/new?labels=rating";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoreExtensionInfo {
    pub name: String,
    pub title: String,
    pub version: String,
    pub description: String,
    pub author: String,
    #[serde(default = "default_publisher")]
    pub publisher: String,
    #[serde(default = "default_safe")]
    pub safe: bool,
    pub icon: Option<String>,
    pub permissions: Vec<String>,
    #[serde(default = "default_category")]
    pub category: String,
    #[serde(default)]
    pub rating: Option<f32>,
    #[serde(default)]
    pub rating_count: u64,
    #[serde(default)]
    pub install_count: u64,
    #[serde(default = "default_trust_badge")]
    pub trust_badge: String,
    #[serde(default)]
    pub changelog: Vec<String>,
    #[serde(default = "default_permission_risk")]
    pub permission_risk: String,
    #[serde(default)]
    pub commands_count: usize,
    #[serde(skip_deserializing)]
    pub installed: bool,
}

fn default_category() -> String {
    "Uncategorized".to_string()
}

fn default_publisher() -> String {
    "Community".to_string()
}

fn default_safe() -> bool {
    false
}

fn default_trust_badge() -> String {
    "community".to_string()
}

fn default_permission_risk() -> String {
    "Low".to_string()
}

/// Wrapper for the legacy `{ "version": ..., "extensions": [...] }` format.
#[derive(Debug, Deserialize)]
struct RegistryFileWrapped {
    #[allow(dead_code)]
    version: u32,
    extensions: Vec<StoreExtensionInfo>,
}

#[derive(Debug, Deserialize)]
struct RatingsFile {
    ratings: Vec<RatingEntry>,
}

#[derive(Debug, Deserialize)]
struct RatingEntry {
    name: String,
    average: f32,
    count: u64,
}

fn installed_extension_names() -> Vec<String> {
    extensions::scan_extensions()
        .into_iter()
        .map(|e| e.manifest.name)
        .collect()
}

fn normalize_store_entry(entry: &mut StoreExtensionInfo) {
    if entry.publisher.trim().is_empty() || entry.publisher == "Community" {
        if entry.author.trim().eq_ignore_ascii_case("Vanta Team") {
            entry.publisher = "Vanta Team".to_string();
        }
    }

    if !entry.safe && entry.publisher.trim().eq_ignore_ascii_case("Vanta Team") {
        entry.safe = true;
    }

    if entry.category.trim().is_empty() || entry.category == "Uncategorized" {
        entry.category = infer_category(&entry.name, &entry.permissions);
    }
    entry.permission_risk = classify_permission_risk(&entry.permissions);

    if entry.trust_badge.trim().is_empty() || entry.trust_badge == "community" {
        if entry.author.trim().eq_ignore_ascii_case("Vanta Team") {
            entry.trust_badge = "verified".to_string();
        }
    }

    if let Some(rating) = entry.rating {
        entry.rating = Some(rating.clamp(0.0, 5.0));
    }
}

fn apply_ratings_snapshot(exts: &mut [StoreExtensionInfo], ratings: &[RatingEntry]) {
    let mut by_name = std::collections::HashMap::new();
    for item in ratings {
        by_name.insert(item.name.as_str(), item);
    }

    for ext in exts {
        if let Some(item) = by_name.get(ext.name.as_str()) {
            ext.rating = Some(item.average.clamp(0.0, 5.0));
            ext.rating_count = item.count;
        }
    }
}

fn infer_category(name: &str, permissions: &[String]) -> String {
    let lower = name.to_ascii_lowercase();
    if lower.contains("weather") || lower.contains("network") {
        return "Networking".to_string();
    }
    if lower.contains("spotify") || lower.contains("music") {
        return "Media".to_string();
    }
    if lower.contains("sys") || lower.contains("process") {
        return "System".to_string();
    }
    if permissions.iter().any(|p| p.eq_ignore_ascii_case("Filesystem")) {
        return "File Tools".to_string();
    }
    "Productivity".to_string()
}

fn classify_permission_risk(permissions: &[String]) -> String {
    if permissions.iter().any(|p| p.eq_ignore_ascii_case("Shell")) {
        return "High".to_string();
    }
    if permissions.iter().any(|p| {
        p.eq_ignore_ascii_case("Network") || p.eq_ignore_ascii_case("Filesystem")
    }) {
        return "Medium".to_string();
    }
    "Low".to_string()
}

fn validate_requested_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Extension name cannot be empty".to_string());
    }
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return Err("Extension name contains invalid path characters".to_string());
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err("Extension name can only include letters, numbers, '-' and '_'".to_string());
    }
    Ok(())
}

fn validate_downloaded_manifest(
    requested_name: &str,
    manifest_bytes: &[u8],
) -> Result<ExtensionManifest, String> {
    let manifest: ExtensionManifest = serde_json::from_slice(manifest_bytes)
        .map_err(|e| format!("manifest.json is invalid JSON: {}", e))?;

    if manifest.name != requested_name {
        return Err(format!(
            "manifest name '{}' does not match requested extension '{}'.",
            manifest.name, requested_name
        ));
    }

    if manifest.title.trim().is_empty() {
        return Err("manifest title cannot be empty".to_string());
    }

    if manifest.commands.is_empty() {
        return Err("manifest must define at least one command".to_string());
    }

    for cmd in &manifest.commands {
        if cmd.name.trim().is_empty() || cmd.title.trim().is_empty() {
            return Err(format!(
                "manifest contains a command with empty name/title for extension '{}'.",
                requested_name
            ));
        }
    }

    Ok(manifest)
}

#[tauri::command]
pub async fn fetch_store_registry() -> Result<Vec<StoreExtensionInfo>, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(REGISTRY_URL)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch store registry: {}", e))?;

    let text = resp
        .text()
        .await
        .map_err(|e| format!("Failed to read registry response: {}", e))?;

    let mut exts: Vec<StoreExtensionInfo> =
        serde_json::from_str::<Vec<StoreExtensionInfo>>(&text)
            .or_else(|_| {
                serde_json::from_str::<RegistryFileWrapped>(&text)
                    .map(|r| r.extensions)
            })
            .map_err(|e| format!("Failed to parse registry: {}", e))?;

    if let Ok(resp) = client.get(RATINGS_URL).send().await {
        if let Ok(text) = resp.text().await {
            if let Ok(ratings) = serde_json::from_str::<RatingsFile>(&text) {
                apply_ratings_snapshot(&mut exts, &ratings.ratings);
            }
        }
    }

    let installed = installed_extension_names();
    for ext in &mut exts {
        normalize_store_entry(ext);
        ext.installed = installed.contains(&ext.name);
    }

    Ok(exts)
}

#[tauri::command]
pub async fn submit_extension_rating(
    name: String,
    rating: u8,
    comment: Option<String>,
) -> Result<String, String> {
    validate_requested_name(&name)?;
    if !(1..=5).contains(&rating) {
        return Err("Rating must be between 1 and 5".to_string());
    }

    let mut url = format!(
        "{}&title={}&body={}",
        RATINGS_FEEDBACK_URL,
        urlencoding::encode(&format!("Rating: {} ({} stars)", name, rating)),
        urlencoding::encode(&format!(
            "Extension: {}\nRating: {}\n\nComment:\n{}",
            name,
            rating,
            comment.unwrap_or_default()
        ))
    );

    if url.len() > 2000 {
        url = format!(
            "{}&title={}&body={}",
            RATINGS_FEEDBACK_URL,
            urlencoding::encode(&format!("Rating: {} ({} stars)", name, rating)),
            urlencoding::encode(&format!("Extension: {}\nRating: {}", name, rating))
        );
    }

    open::that(&url).map_err(|e| format!("Failed to open rating form: {}", e))?;
    Ok(url)
}

async fn download_file(client: &reqwest::Client, url: &str) -> Result<Vec<u8>, String> {
    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Failed to download {}: {}", url, e))?;

    if !resp.status().is_success() {
        return Err(format!("HTTP {} for {}", resp.status(), url));
    }

    resp.bytes()
        .await
        .map(|b| b.to_vec())
        .map_err(|e| format!("Failed to read bytes from {}: {}", url, e))
}

#[tauri::command]
pub async fn install_store_extension(name: String) -> Result<(), String> {
    validate_requested_name(&name)?;

    let cfg = config::load_or_create_default();
    if cfg.policy.restricted_mode
        && !cfg.policy.allowed_extensions.iter().any(|id| id == &name)
    {
        let _ = permissions::record_audit_event(
            "extension_install",
            "store",
            &name,
            "denied",
            Some("blocked by restricted mode allowlist".to_string()),
        );
        return Err("Install blocked by policy: extension not in allowlist".to_string());
    }

    let ext_dir = extensions::extensions_dir().join(&name);
    let dist_dir = ext_dir.join("dist");
    fs::create_dir_all(&dist_dir)
        .map_err(|e| format!("Failed to create extension directory: {}", e))?;

    let client = reqwest::Client::new();
    let base = format!("{}/{}", BASE_URL, name);

    let manifest_bytes = download_file(&client, &format!("{}/manifest.json", base)).await?;
    let manifest = validate_downloaded_manifest(&name, &manifest_bytes)
        .map_err(|e| format!("Install blocked by manifest validation: {}", e))?;

    if cfg.policy.require_verified_extensions {
        let is_verified = manifest
            .publisher
            .as_deref()
            .map(|a| a.eq_ignore_ascii_case("Vanta Team"))
            .unwrap_or_else(|| {
                manifest
                    .author
                    .as_deref()
                    .map(|a| a.eq_ignore_ascii_case("Vanta Team"))
                    .unwrap_or(false)
            })
            && manifest.safe.unwrap_or(false);
        if !is_verified {
            let _ = permissions::record_audit_event(
                "extension_install",
                "store",
                &name,
                "denied",
                Some("require_verified_extensions=true".to_string()),
            );
            return Err("Install blocked by policy: only verified extensions are allowed".to_string());
        }
    }

    let normalized_manifest = serde_json::to_string_pretty(&manifest)
        .map_err(|e| format!("Failed to serialize normalized manifest: {}", e))?;

    if let Err(e) = fs::write(ext_dir.join("manifest.json"), normalized_manifest) {
        let _ = fs::remove_dir_all(&ext_dir);
        return Err(format!(
            "Failed to write manifest.json: {}. Installation rolled back.",
            e
        ));
    }

    let bundle_bytes = download_file(&client, &format!("{}/dist/index.js", base))
        .await
        .map_err(|e| {
            let _ = fs::remove_dir_all(&ext_dir);
            format!("Failed to download extension bundle: {}", e)
        })?;
    if let Err(e) = fs::write(dist_dir.join("index.js"), &bundle_bytes) {
        let _ = fs::remove_dir_all(&ext_dir);
        return Err(format!(
            "Failed to write dist/index.js: {}. Installation rolled back.",
            e
        ));
    }

    if let Ok(style_bytes) = download_file(&client, &format!("{}/dist/style.css", base)).await {
        let _ = fs::write(dist_dir.join("style.css"), &style_bytes);
    }

    if let Ok(icon_bytes) = download_file(&client, &format!("{}/icon.png", base)).await {
        let _ = fs::write(ext_dir.join("icon.png"), &icon_bytes);
    }

    log::info!(
        "Installed extension '{}' ({} commands) to {}",
        name,
        manifest.commands.len(),
        ext_dir.display()
    );
    let _ = permissions::record_audit_event(
        "extension_install",
        "store",
        &name,
        "allowed",
        Some(format!("commands={}", manifest.commands.len())),
    );
    Ok(())
}

#[tauri::command]
pub async fn uninstall_extension(name: String) -> Result<(), String> {
    let ext_dir = extensions::extensions_dir().join(&name);
    if !ext_dir.exists() {
        return Err(format!("Extension '{}' is not installed", name));
    }

    let canonical = ext_dir
        .canonicalize()
        .map_err(|e| format!("Cannot resolve path: {}", e))?;
    let base_canonical = extensions::extensions_dir()
        .canonicalize()
        .unwrap_or_else(|_| extensions::extensions_dir());
    if !canonical.starts_with(&base_canonical) {
        return Err("Path traversal detected".to_string());
    }

    fs::remove_dir_all(&ext_dir)
        .map_err(|e| format!("Failed to remove extension '{}': {}", name, e))?;

    let data_file = PathBuf::from(
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("vanta")
            .join("extension-data")
            .join(format!("{}.json", name)),
    );
    if data_file.exists() {
        let _ = fs::remove_file(&data_file);
    }

    log::info!("Uninstalled extension '{}'", name);
    let _ = permissions::record_audit_event(
        "extension_uninstall",
        "store",
        &name,
        "allowed",
        None,
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_store_registry_accepts_discoverability_fields() {
        let raw = r#"[
            {
                "name": "weather",
                "title": "Weather",
                "version": "1.0.0",
                "description": "Weather insights",
                "author": "Vanta Team",
                "icon": null,
                "permissions": ["Network"],
                "category": "Productivity",
                "rating": 4.8,
                "install_count": 2042,
                "trust_badge": "verified",
                "changelog": ["Added severe weather alerts"]
            }
        ]"#;

        let parsed: Vec<StoreExtensionInfo> = serde_json::from_str(raw).unwrap();
        assert_eq!(parsed.len(), 1);
        let ext = &parsed[0];
        assert_eq!(ext.category, "Productivity");
        assert_eq!(ext.rating, Some(4.8));
        assert_eq!(ext.install_count, 2042);
        assert_eq!(ext.trust_badge, "verified");
        assert_eq!(ext.changelog, vec!["Added severe weather alerts".to_string()]);
    }

    #[test]
    fn classify_permission_risk_uses_max_capability_weight() {
        assert_eq!(classify_permission_risk(&[]), "Low");
        assert_eq!(classify_permission_risk(&["Network".to_string()]), "Medium");
        assert_eq!(
            classify_permission_risk(&["Network".to_string(), "Shell".to_string()]),
            "High"
        );
    }

    #[test]
    fn validate_downloaded_manifest_rejects_mismatched_name() {
        let manifest = br#"{
            "name": "another-ext",
            "title": "Another",
            "commands": [{"name": "run", "title": "Run", "mode": "no-view"}],
            "permissions": []
        }"#;

        let err = validate_downloaded_manifest("weather", manifest).unwrap_err();
        assert!(err.contains("does not match requested extension"));
    }

    #[test]
    fn validate_downloaded_manifest_rejects_empty_commands() {
        let manifest = br#"{
            "name": "weather",
            "title": "Weather",
            "commands": [],
            "permissions": []
        }"#;

        let err = validate_downloaded_manifest("weather", manifest).unwrap_err();
        assert!(err.contains("at least one command"));
    }
}

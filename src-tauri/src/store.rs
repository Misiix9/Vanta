use serde::{Deserialize, Serialize};
use serde_json::json;
use semver::Version;
use std::fs;
use std::path::{Path, PathBuf};

use crate::extensions::{self, ExtensionDependency, ExtensionManifest};
use crate::errors::VantaError;
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
const SUPABASE_URL_ENV: &str = "VANTA_SUPABASE_URL";
const SUPABASE_KEY_ENV: &str = "VANTA_SUPABASE_ANON_KEY";

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExtensionUpdateInfo {
    pub name: String,
    pub current_version: String,
    pub latest_version: String,
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

#[derive(Debug, Deserialize)]
struct SupabaseMetricRow {
    name: String,
    #[serde(default)]
    rating: Option<f32>,
    #[serde(default)]
    rating_count: u64,
    #[serde(default)]
    install_count: u64,
}

#[derive(Debug, Serialize)]
struct SupabaseStoreExtensionRow {
    name: String,
    title: String,
    version: String,
    description: String,
    author: String,
    publisher: String,
    safe: bool,
    icon: Option<String>,
    category: String,
    trust_badge: String,
    permission_risk: String,
    commands_count: i32,
    installed: bool,
    synced_at: String,
    updated_at: String,
}

#[derive(Debug, Serialize)]
struct SupabaseStorePermissionRow {
    extension_name: String,
    permission: String,
    position: i32,
}

#[derive(Debug, Serialize)]
struct SupabaseStoreChangelogRow {
    extension_name: String,
    entry: String,
    position: i32,
}

fn supabase_config_from_env() -> Option<(String, String)> {
    let url = std::env::var(SUPABASE_URL_ENV).ok()?;
    let key = std::env::var(SUPABASE_KEY_ENV).ok()?;
    let normalized = url.trim_end_matches('/').to_string();
    if normalized.is_empty() || key.trim().is_empty() {
        return None;
    }
    Some((normalized, key))
}

fn apply_supabase_metrics(exts: &mut [StoreExtensionInfo], rows: Vec<SupabaseMetricRow>) {
    let mut by_name = std::collections::HashMap::new();
    for row in rows {
        let key = row.name.clone();
        by_name.insert(key, row);
    }

    for ext in exts {
        if let Some(row) = by_name.get(&ext.name) {
            ext.rating = row.rating.map(|v| v.clamp(0.0, 5.0));
            ext.rating_count = row.rating_count;
            ext.install_count = row.install_count;
        }
    }
}

async fn fetch_supabase_metrics(
    client: &reqwest::Client,
    supabase_url: &str,
    supabase_key: &str,
) -> Result<Vec<SupabaseMetricRow>, VantaError> {
    let url = format!(
        "{}/rest/v1/v_store_extensions_with_metrics?select=name,rating,rating_count,install_count",
        supabase_url
    );

    let resp = client
        .get(&url)
        .header("apikey", supabase_key)
        .header("Authorization", format!("Bearer {}", supabase_key))
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch Supabase metrics: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Supabase metrics HTTP {}", resp.status()).into());
    }

    let text = resp
        .text()
        .await
        .map_err(|e| format!("Failed to read Supabase metrics response: {}", e))?;

    Ok(serde_json::from_str::<Vec<SupabaseMetricRow>>(&text)
        .map_err(|e| format!("Failed to decode Supabase metrics response: {}", e))?)
}

async fn sync_store_metadata_to_supabase(
    client: &reqwest::Client,
    supabase_url: &str,
    supabase_key: &str,
    exts: &[StoreExtensionInfo],
) -> Result<(), VantaError> {
    let now = chrono::Utc::now().to_rfc3339();

    let base_rows: Vec<SupabaseStoreExtensionRow> = exts
        .iter()
        .map(|ext| SupabaseStoreExtensionRow {
            name: ext.name.clone(),
            title: ext.title.clone(),
            version: ext.version.clone(),
            description: ext.description.clone(),
            author: ext.author.clone(),
            publisher: ext.publisher.clone(),
            safe: ext.safe,
            icon: ext.icon.clone(),
            category: ext.category.clone(),
            trust_badge: ext.trust_badge.clone(),
            permission_risk: ext.permission_risk.clone(),
            commands_count: ext.commands_count as i32,
            installed: false,
            synced_at: now.clone(),
            updated_at: now.clone(),
        })
        .collect();

    let body = serde_json::to_string(&base_rows)
        .map_err(|e| format!("Failed to encode store extension rows: {}", e))?;

    let upsert_url = format!(
        "{}/rest/v1/store_extensions?on_conflict=name",
        supabase_url
    );
    let upsert_resp = client
        .post(&upsert_url)
        .header("apikey", supabase_key)
        .header("Authorization", format!("Bearer {}", supabase_key))
        .header("Content-Type", "application/json")
        .header("Prefer", "resolution=merge-duplicates,return=minimal")
        .body(body)
        .send()
        .await
        .map_err(|e| format!("Failed to upsert store extensions: {}", e))?;

    if !upsert_resp.status().is_success() {
        return Err(format!(
            "Store extension upsert HTTP {}",
            upsert_resp.status()
        ).into());
    }

    let delete_permissions_url = format!(
        "{}/rest/v1/store_extension_permissions?id=gt.0",
        supabase_url
    );
    let del_perm_resp = client
        .delete(&delete_permissions_url)
        .header("apikey", supabase_key)
        .header("Authorization", format!("Bearer {}", supabase_key))
        .header("Prefer", "return=minimal")
        .send()
        .await
        .map_err(|e| format!("Failed to clear store permissions rows: {}", e))?;
    if !del_perm_resp.status().is_success() {
        return Err(format!(
            "Store permissions clear HTTP {}",
            del_perm_resp.status()
        ).into());
    }

    let mut permission_rows: Vec<SupabaseStorePermissionRow> = Vec::new();
    for ext in exts {
        for (idx, perm) in ext.permissions.iter().enumerate() {
            permission_rows.push(SupabaseStorePermissionRow {
                extension_name: ext.name.clone(),
                permission: perm.clone(),
                position: idx as i32,
            });
        }
    }

    if !permission_rows.is_empty() {
        let body = serde_json::to_string(&permission_rows)
            .map_err(|e| format!("Failed to encode permission rows: {}", e))?;
        let perm_url = format!("{}/rest/v1/store_extension_permissions", supabase_url);
        let perm_resp = client
            .post(&perm_url)
            .header("apikey", supabase_key)
            .header("Authorization", format!("Bearer {}", supabase_key))
            .header("Content-Type", "application/json")
            .header("Prefer", "return=minimal")
            .body(body)
            .send()
            .await
            .map_err(|e| format!("Failed to insert store permissions rows: {}", e))?;
        if !perm_resp.status().is_success() {
            return Err(format!(
                "Store permissions insert HTTP {}",
                perm_resp.status()
            ).into());
        }
    }

    let delete_changelog_url = format!(
        "{}/rest/v1/store_extension_changelog?id=gt.0",
        supabase_url
    );
    let del_changelog_resp = client
        .delete(&delete_changelog_url)
        .header("apikey", supabase_key)
        .header("Authorization", format!("Bearer {}", supabase_key))
        .header("Prefer", "return=minimal")
        .send()
        .await
        .map_err(|e| format!("Failed to clear store changelog rows: {}", e))?;
    if !del_changelog_resp.status().is_success() {
        return Err(format!(
            "Store changelog clear HTTP {}",
            del_changelog_resp.status()
        ).into());
    }

    let mut changelog_rows: Vec<SupabaseStoreChangelogRow> = Vec::new();
    for ext in exts {
        for (idx, entry) in ext.changelog.iter().enumerate() {
            changelog_rows.push(SupabaseStoreChangelogRow {
                extension_name: ext.name.clone(),
                entry: entry.clone(),
                position: idx as i32,
            });
        }
    }

    if !changelog_rows.is_empty() {
        let body = serde_json::to_string(&changelog_rows)
            .map_err(|e| format!("Failed to encode changelog rows: {}", e))?;
        let changelog_url = format!("{}/rest/v1/store_extension_changelog", supabase_url);
        let changelog_resp = client
            .post(&changelog_url)
            .header("apikey", supabase_key)
            .header("Authorization", format!("Bearer {}", supabase_key))
            .header("Content-Type", "application/json")
            .header("Prefer", "return=minimal")
            .body(body)
            .send()
            .await
            .map_err(|e| format!("Failed to insert store changelog rows: {}", e))?;
        if !changelog_resp.status().is_success() {
            return Err(format!(
                "Store changelog insert HTTP {}",
                changelog_resp.status()
            ).into());
        }
    }

    Ok(())
}

async fn submit_supabase_rating_rpc(
    client: &reqwest::Client,
    supabase_url: &str,
    supabase_key: &str,
    name: &str,
    rating: u8,
    comment: Option<String>,
) -> Result<(), VantaError> {
    let url = format!("{}/rest/v1/rpc/submit_extension_rating", supabase_url);
    let payload = json!({
        "p_extension_name": name,
        "p_rating": rating,
        "p_comment": comment,
        "p_source": "vanta_app"
    })
    .to_string();

    let resp = client
        .post(&url)
        .header("apikey", supabase_key)
        .header("Authorization", format!("Bearer {}", supabase_key))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .body(payload)
        .send()
        .await
        .map_err(|e| format!("Failed to submit Supabase rating RPC: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Supabase rating RPC HTTP {}", resp.status()).into());
    }

    Ok(())
}

async fn increment_supabase_download_rpc(
    client: &reqwest::Client,
    supabase_url: &str,
    supabase_key: &str,
    name: &str,
) -> Result<(), VantaError> {
    let url = format!("{}/rest/v1/rpc/increment_extension_download", supabase_url);
    let payload = json!({
        "p_extension_name": name,
        "p_source": "vanta_app_install"
    })
    .to_string();

    let resp = client
        .post(&url)
        .header("apikey", supabase_key)
        .header("Authorization", format!("Bearer {}", supabase_key))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .body(payload)
        .send()
        .await
        .map_err(|e| format!("Failed to submit Supabase download RPC: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Supabase download RPC HTTP {}", resp.status()).into());
    }

    Ok(())
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

fn is_registry_version_newer(registry_version: &str, installed_version: &str) -> bool {
    let registry_trimmed = registry_version.trim().trim_start_matches('v');
    let installed_trimmed = installed_version.trim().trim_start_matches('v');

    match (Version::parse(registry_trimmed), Version::parse(installed_trimmed)) {
        (Ok(registry), Ok(installed)) => registry > installed,
        _ => registry_trimmed != installed_trimmed,
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

fn validate_requested_name(name: &str) -> Result<(), VantaError> {
    if name.is_empty() {
        return Err("Extension name cannot be empty".into());
    }
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return Err("Extension name contains invalid path characters".into());
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err("Extension name can only include letters, numbers, '-' and '_'".into());
    }
    Ok(())
}

fn validate_downloaded_manifest(
    requested_name: &str,
    manifest_bytes: &[u8],
) -> Result<ExtensionManifest, VantaError> {
    let manifest: ExtensionManifest = serde_json::from_slice(manifest_bytes)
        .map_err(|e| format!("manifest.json is invalid JSON: {}", e))?;

    if manifest.name != requested_name {
        return Err(format!(
            "manifest name '{}' does not match requested extension '{}'.",
            manifest.name, requested_name
        ).into());
    }

    if manifest.title.trim().is_empty() {
        return Err("manifest title cannot be empty".into());
    }

    if manifest.commands.is_empty() {
        return Err("manifest must define at least one command".into());
    }

    for cmd in &manifest.commands {
        if cmd.name.trim().is_empty() || cmd.title.trim().is_empty() {
            return Err(format!(
                "manifest contains a command with empty name/title for extension '{}'.",
                requested_name
            ).into());
        }
    }

    Ok(manifest)
}

fn collect_installed_extension_versions() -> std::collections::HashMap<String, String> {
    let mut versions = std::collections::HashMap::new();
    for entry in extensions::scan_extensions() {
        versions.insert(entry.manifest.name, entry.manifest.version);
    }
    versions
}

fn validate_dependency(
    dependency: &ExtensionDependency,
    installed_versions: &std::collections::HashMap<String, String>,
) -> Result<(), String> {
    let ext_id = dependency.ext_id.trim();
    let version_range = dependency.version_range.trim();

    if ext_id.is_empty() {
        return Err("dependency ext_id cannot be empty".to_string());
    }
    if version_range.is_empty() {
        return Err(format!("dependency '{}' has empty version_range", ext_id));
    }

    let installed = installed_versions
        .get(ext_id)
        .ok_or_else(|| format!("missing dependency '{}'", ext_id))?;

    let req = semver::VersionReq::parse(version_range)
        .map_err(|e| format!("invalid version range '{}' for '{}': {}", version_range, ext_id, e))?;

    let installed_version = Version::parse(installed.trim().trim_start_matches('v'))
        .map_err(|e| format!("installed dependency '{}' has invalid version '{}': {}", ext_id, installed, e))?;

    if req.matches(&installed_version) {
        return Ok(());
    }

    Err(format!(
        "dependency '{}' requires '{}' but installed version is '{}'",
        ext_id, version_range, installed
    ))
}

fn validate_manifest_dependencies(
    manifest: &ExtensionManifest,
    installed_versions: &std::collections::HashMap<String, String>,
) -> Result<(), VantaError> {
    let mut failures = Vec::new();

    for dependency in &manifest.requires {
        if dependency.ext_id == manifest.name {
            failures.push(format!(
                "dependency '{}' cannot reference itself",
                dependency.ext_id
            ));
            continue;
        }

        if let Err(e) = validate_dependency(dependency, installed_versions) {
            failures.push(e);
        }
    }

    if failures.is_empty() {
        return Ok(());
    }

    Err(format!(
        "Install blocked by dependency validation for '{}': {}",
        manifest.name,
        failures.join("; ")
    )
    .into())
}

fn extension_backup_root() -> PathBuf {
    config::config_dir().join("extension-versions")
}

fn extension_backup_dir(name: &str) -> PathBuf {
    extension_backup_root().join(name)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), VantaError> {
    if !src.exists() {
        return Err(format!("Source path does not exist: {}", src.display()).into());
    }

    fs::create_dir_all(dst)
        .map_err(|e| format!("Failed to create backup dir '{}': {}", dst.display(), e))?;

    for entry in fs::read_dir(src)
        .map_err(|e| format!("Failed to read dir '{}': {}", src.display(), e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read dir entry: {}", e))?;
        let path = entry.path();
        let target = dst.join(entry.file_name());

        if path.is_dir() {
            copy_dir_recursive(&path, &target)?;
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create parent dir '{}': {}", parent.display(), e))?;
            }
            fs::copy(&path, &target).map_err(|e| {
                format!(
                    "Failed to copy '{}' to '{}': {}",
                    path.display(),
                    target.display(),
                    e
                )
            })?;
        }
    }

    Ok(())
}

fn create_extension_backup(name: &str, ext_dir: &Path) -> Result<PathBuf, VantaError> {
    let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();
    let backup_dir = extension_backup_dir(name).join(timestamp);
    copy_dir_recursive(ext_dir, &backup_dir)?;
    Ok(backup_dir)
}

fn restore_extension_from_backup(ext_dir: &Path, backup_dir: &Path) -> Result<(), VantaError> {
    if ext_dir.exists() {
        fs::remove_dir_all(ext_dir)
            .map_err(|e| format!("Failed to remove extension dir '{}': {}", ext_dir.display(), e))?;
    }
    copy_dir_recursive(backup_dir, ext_dir)?;
    Ok(())
}

fn latest_backup_for_extension(name: &str) -> Result<PathBuf, VantaError> {
    let backups_root = extension_backup_dir(name);
    if !backups_root.exists() {
        return Err(format!("No rollback backup available for extension '{}'", name).into());
    }

    let mut backup_dirs: Vec<PathBuf> = fs::read_dir(&backups_root)
        .map_err(|e| format!("Failed to read backup dir '{}': {}", backups_root.display(), e))?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.is_dir())
        .collect();

    backup_dirs.sort();
    backup_dirs
        .pop()
        .ok_or_else(|| format!("No rollback backup available for extension '{}'", name).into())
}

#[tauri::command]
pub async fn fetch_store_registry() -> Result<Vec<StoreExtensionInfo>, VantaError> {
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

    for ext in &mut exts {
        normalize_store_entry(ext);
    }

    if let Some((supabase_url, supabase_key)) = supabase_config_from_env() {
        match sync_store_metadata_to_supabase(&client, &supabase_url, &supabase_key, &exts).await {
            Ok(_) => {}
            Err(err) => log::warn!("Store metadata sync to Supabase failed: {}", err),
        }
        match fetch_supabase_metrics(&client, &supabase_url, &supabase_key).await {
            Ok(rows) => apply_supabase_metrics(&mut exts, rows),
            Err(err) => log::warn!("Store metrics fallback to JSON snapshot: {}", err),
        }
    }

    let installed = installed_extension_names();
    for ext in &mut exts {
        ext.installed = installed.contains(&ext.name);
    }

    Ok(exts)
}

#[tauri::command]
pub async fn check_store_extension_updates() -> Result<Vec<ExtensionUpdateInfo>, VantaError> {
    let registry = fetch_store_registry().await?;
    let installed_entries = extensions::scan_extensions();

    let mut installed_by_name: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    for entry in installed_entries {
        installed_by_name.insert(entry.manifest.name, entry.manifest.version);
    }

    let mut updates = Vec::new();
    for ext in registry {
        if let Some(current_version) = installed_by_name.get(&ext.name) {
            if is_registry_version_newer(&ext.version, current_version) {
                updates.push(ExtensionUpdateInfo {
                    name: ext.name,
                    current_version: current_version.clone(),
                    latest_version: ext.version,
                });
            }
        }
    }

    updates.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(updates)
}

#[tauri::command]
pub async fn submit_extension_rating(
    name: String,
    rating: u8,
    comment: Option<String>,
) -> Result<String, VantaError> {
    validate_requested_name(&name)?;
    if !(1..=5).contains(&rating) {
        return Err("Rating must be between 1 and 5".into());
    }

    let client = reqwest::Client::new();
    if let Some((supabase_url, supabase_key)) = supabase_config_from_env() {
        match submit_supabase_rating_rpc(&client, &supabase_url, &supabase_key, &name, rating, comment.clone()).await {
            Ok(_) => return Ok("submitted_to_supabase".to_string()),
            Err(err) => log::warn!("Supabase rating submit failed, falling back to issue form: {}", err),
        }
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

async fn download_file(client: &reqwest::Client, url: &str) -> Result<Vec<u8>, VantaError> {
    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Failed to download {}: {}", url, e))?;

    if !resp.status().is_success() {
        return Err(format!("HTTP {} for {}", resp.status(), url).into());
    }

    Ok(resp.bytes()
        .await
        .map(|b| b.to_vec())
        .map_err(|e| format!("Failed to read bytes from {}: {}", url, e))?)
}

#[tauri::command]
pub async fn install_store_extension(name: String) -> Result<(), VantaError> {
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
        return Err("Install blocked by policy: extension not in allowlist".into());
    }

    let ext_dir = extensions::extensions_dir().join(&name);
    let dist_dir = ext_dir.join("dist");
    let ext_existed = ext_dir.exists();
    let backup_path = if ext_existed {
        Some(create_extension_backup(&name, &ext_dir).map_err(|e| {
            format!("Failed to create backup before update: {}", e)
        })?)
    } else {
        None
    };

    fs::create_dir_all(&dist_dir)
        .map_err(|e| format!("Failed to create extension directory: {}", e))?;

    let client = reqwest::Client::new();
    let base = format!("{}/{}", BASE_URL, name);

    let manifest_bytes = download_file(&client, &format!("{}/manifest.json", base)).await?;
    let manifest = validate_downloaded_manifest(&name, &manifest_bytes)
        .map_err(|e| format!("Install blocked by manifest validation: {}", e))?;

    let installed_versions = collect_installed_extension_versions();
    validate_manifest_dependencies(&manifest, &installed_versions)?;

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
            return Err("Install blocked by policy: only verified extensions are allowed".into());
        }
    }

    let normalized_manifest = serde_json::to_string_pretty(&manifest)
        .map_err(|e| format!("Failed to serialize normalized manifest: {}", e))?;

    if let Err(e) = fs::write(ext_dir.join("manifest.json"), normalized_manifest) {
        if let Some(backup) = &backup_path {
            let _ = restore_extension_from_backup(&ext_dir, backup);
        } else {
            let _ = fs::remove_dir_all(&ext_dir);
        }
        return Err(format!(
            "Failed to write manifest.json: {}. Installation rolled back.",
            e
        ).into());
    }

    let bundle_bytes = download_file(&client, &format!("{}/dist/index.js", base))
        .await
        .map_err(|e| {
            if let Some(backup) = &backup_path {
                let _ = restore_extension_from_backup(&ext_dir, backup);
            } else {
                let _ = fs::remove_dir_all(&ext_dir);
            }
            format!("Failed to download extension bundle: {}", e)
        })?;
    if let Err(e) = fs::write(dist_dir.join("index.js"), &bundle_bytes) {
        if let Some(backup) = &backup_path {
            let _ = restore_extension_from_backup(&ext_dir, backup);
        } else {
            let _ = fs::remove_dir_all(&ext_dir);
        }
        return Err(format!(
            "Failed to write dist/index.js: {}. Installation rolled back.",
            e
        ).into());
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

    if let Some((supabase_url, supabase_key)) = supabase_config_from_env() {
        if let Err(err) = increment_supabase_download_rpc(&client, &supabase_url, &supabase_key, &name).await {
            log::warn!("Failed to increment Supabase download metric for '{}': {}", name, err);
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn rollback_store_extension(name: String) -> Result<String, VantaError> {
    validate_requested_name(&name)?;

    let ext_dir = extensions::extensions_dir().join(&name);
    let backup_dir = latest_backup_for_extension(&name)?;

    if ext_dir.exists() {
        let _ = create_extension_backup(&name, &ext_dir);
    }

    restore_extension_from_backup(&ext_dir, &backup_dir)?;

    let manifest_path = ext_dir.join("manifest.json");
    let restored_version = fs::read_to_string(&manifest_path)
        .ok()
        .and_then(|text| serde_json::from_str::<ExtensionManifest>(&text).ok())
        .map(|m| m.version)
        .unwrap_or_else(|| "unknown".to_string());

    let _ = permissions::record_audit_event(
        "extension_rollback",
        "store",
        &name,
        "allowed",
        Some(format!("restored_version={}", restored_version)),
    );

    Ok(restored_version)
}

#[tauri::command]
pub async fn uninstall_extension(name: String) -> Result<(), VantaError> {
    let ext_dir = extensions::extensions_dir().join(&name);
    if !ext_dir.exists() {
        return Err(format!("Extension '{}' is not installed", name).into());
    }

    let canonical = ext_dir
        .canonicalize()
        .map_err(|e| format!("Cannot resolve path: {}", e))?;
    let base_canonical = extensions::extensions_dir()
        .canonicalize()
        .unwrap_or_else(|_| extensions::extensions_dir());
    if !canonical.starts_with(&base_canonical) {
        return Err("Path traversal detected".into());
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
        assert!(err.to_string().contains("does not match requested extension"));
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
        assert!(err.to_string().contains("at least one command"));
    }
}

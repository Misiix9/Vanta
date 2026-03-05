use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::extensions;

const REGISTRY_URL: &str =
    "https://raw.githubusercontent.com/Misiix9/vanta-extensions/main/registry.json";
const BASE_URL: &str =
    "https://raw.githubusercontent.com/Misiix9/vanta-extensions/main/extensions";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoreExtensionInfo {
    pub name: String,
    pub title: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub icon: Option<String>,
    pub permissions: Vec<String>,
    #[serde(default)]
    pub commands_count: usize,
    #[serde(skip_deserializing)]
    pub installed: bool,
}

#[derive(Debug, Deserialize)]
struct RegistryFile {
    #[allow(dead_code)]
    version: u32,
    extensions: Vec<StoreExtensionInfo>,
}

fn installed_extension_names() -> Vec<String> {
    extensions::scan_extensions()
        .into_iter()
        .map(|e| e.manifest.name)
        .collect()
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

    let registry: RegistryFile =
        serde_json::from_str(&text).map_err(|e| format!("Failed to parse registry: {}", e))?;

    let installed = installed_extension_names();
    let mut exts = registry.extensions;
    for ext in &mut exts {
        ext.installed = installed.contains(&ext.name);
    }

    Ok(exts)
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
    let ext_dir = extensions::extensions_dir().join(&name);
    let dist_dir = ext_dir.join("dist");
    fs::create_dir_all(&dist_dir)
        .map_err(|e| format!("Failed to create extension directory: {}", e))?;

    let client = reqwest::Client::new();
    let base = format!("{}/{}", BASE_URL, name);

    let manifest_bytes = download_file(&client, &format!("{}/manifest.json", base)).await?;
    fs::write(ext_dir.join("manifest.json"), &manifest_bytes)
        .map_err(|e| format!("Failed to write manifest.json: {}", e))?;

    let bundle_bytes = download_file(&client, &format!("{}/dist/index.js", base)).await?;
    fs::write(dist_dir.join("index.js"), &bundle_bytes)
        .map_err(|e| format!("Failed to write dist/index.js: {}", e))?;

    if let Ok(style_bytes) = download_file(&client, &format!("{}/dist/style.css", base)).await {
        let _ = fs::write(dist_dir.join("style.css"), &style_bytes);
    }

    if let Ok(icon_bytes) = download_file(&client, &format!("{}/icon.png", base)).await {
        let _ = fs::write(ext_dir.join("icon.png"), &icon_bytes);
    }

    log::info!("Installed extension '{}' to {}", name, ext_dir.display());
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
    Ok(())
}

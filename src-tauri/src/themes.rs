use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThemeMeta {
    pub id: String, // Filename without .css
    pub name: String,
    pub width: f64,
    pub height: f64,
    pub css_content: String,
}

pub fn get_themes_dir() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"));
    path.push("vanta");
    path.push("themes");
    if !path.exists() {
        let _ = fs::create_dir_all(&path);
    }
    path
}

pub fn seed_default_theme(app: &tauri::AppHandle) {
    let themes_dir = get_themes_dir();
    let dest = themes_dir.join("default.css");
    if !dest.exists() {
        if let Ok(resource_path) = app.path().resolve(
            "resources/themes/default.css",
            tauri::path::BaseDirectory::Resource,
        ) {
            if let Err(e) = fs::copy(&resource_path, &dest) {
                log::warn!("Could not seed default.css: {}", e);
            } else {
                log::info!("Seeded default theme to {:?}", dest);
            }
        } else {
            log::warn!("Bundled default.css resource not found — skipping seed");
        }
    }
}

#[tauri::command]
pub fn get_installed_themes() -> Result<Vec<ThemeMeta>, String> {
    let dir = get_themes_dir();
    let mut themes = Vec::new();

    let name_regex = Regex::new(r"/\* Theme Name: (.*?) \*/").unwrap();
    let width_regex = Regex::new(r"--vanta-width:\s*([0-9]+)px").unwrap();
    let height_regex = Regex::new(r"--vanta-height:\s*([0-9]+)px").unwrap();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("css") {
                if let Ok(content) = fs::read_to_string(&path) {
                    let id = path.file_stem().unwrap().to_string_lossy().to_string();

                    // Fallbacks
                    let mut name = id.clone();
                    let mut width = 680.0;
                    let mut height = 420.0;

                    if let Some(caps) = name_regex.captures(&content) {
                        name = caps.get(1).unwrap().as_str().trim().to_string();
                    }
                    if let Some(caps) = width_regex.captures(&content) {
                        if let Ok(w) = caps.get(1).unwrap().as_str().parse::<f64>() {
                            width = w;
                        }
                    }
                    if let Some(caps) = height_regex.captures(&content) {
                        if let Ok(h) = caps.get(1).unwrap().as_str().parse::<f64>() {
                            height = h;
                        }
                    }

                    themes.push(ThemeMeta {
                        id,
                        name,
                        width,
                        height,
                        css_content: content.clone(),
                    });
                }
            }
        }
    }

    // Sort so "default" is always first if it exists
    themes.sort_by(|a, b| {
        if a.id == "default" {
            std::cmp::Ordering::Less
        } else if b.id == "default" {
            std::cmp::Ordering::Greater
        } else {
            a.name.cmp(&b.name)
        }
    });

    Ok(themes)
}

#[tauri::command]
pub fn resize_window_for_theme(
    width: f64,
    height: f64,
    window: tauri::Window,
) -> Result<(), String> {
    let size = tauri::LogicalSize::new(width, height);
    if let Err(e) = window.set_size(size) {
        return Err(format!("Failed to resize window: {}", e));
    }
    // Also center it since dimensions changed
    let _ = window.center();
    Ok(())
}

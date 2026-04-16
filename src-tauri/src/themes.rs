use crate::errors::VantaError;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

const BASE_THEME_CSS: &str = include_str!("../resources/themes/base.css");
const DEFAULT_THEME_CSS: &str = include_str!("../resources/themes/default.css");
const UNIVERSAL_THEME_CSS: &str = include_str!("../resources/themes/universal.css");

// ── Phase 30 · Theme Token Contract ──────────────────────────

const REQUIRED_TOKENS: &[&str] = &[
    "--vanta-width",
    "--vanta-height",
    "--bg",
    "--surface",
    "--text-primary",
    "--text-secondary",
    "--purple",
    "--ds-accent",
    "--ds-surface-0",
    "--ds-surface-1",
    "--ds-border",
    "--ds-text-primary",
    "--ds-text-secondary",
    "--font-ui",
    "--radius",
];

const RECOMMENDED_TOKENS: &[&str] = &[
    "--ds-surface-2",
    "--ds-surface-3",
    "--ds-accent-glow",
    "--ds-text-muted",
    "--ds-success",
    "--ds-warning",
    "--ds-danger",
    "--ds-info",
    "--ds-neutral",
    "--font-mono",
    "--type-display",
    "--type-heading",
    "--type-body",
    "--type-caption",
    "--space-1",
    "--space-2",
    "--space-3",
    "--radius-item",
    "--motion-enter-duration",
    "--motion-ease-enter",
    "--icon-sm",
    "--icon-md",
    "--icon-lg",
    "--icon-xl",
    "--border-window",
];

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThemeDiagnostic {
    pub level: String,   // "error" | "warning" | "info"
    pub token: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThemeMeta {
    pub id: String, // Filename without .css
    pub name: String,
    pub width: f64,
    pub height: f64,
    pub css_content: String,
    pub diagnostics: Vec<ThemeDiagnostic>,
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
    let base_dest = themes_dir.join("base.css");
    let dest = themes_dir.join("default.css");
    let base_needs_update = if base_dest.exists() {
        match fs::read_to_string(&base_dest) {
            Ok(existing) => existing != BASE_THEME_CSS,
            Err(_) => true,
        }
    } else {
        true
    };

    if base_needs_update {
        if let Ok(resource_path) = app.path().resolve(
            "resources/themes/base.css",
            tauri::path::BaseDirectory::Resource,
        ) {
            if let Err(e) = fs::copy(&resource_path, &base_dest) {
                log::warn!("Could not seed base.css from resource: {}", e);
            } else {
                log::info!("Updated base theme at {:?}", base_dest);
            }
        }

        if let Err(e) = fs::write(&base_dest, BASE_THEME_CSS) {
            log::warn!("Could not write embedded base theme: {}", e);
        } else {
            log::info!("Updated base theme from embedded copy at {:?}", base_dest);
        }
    }

    let needs_update = if dest.exists() {
        match fs::read_to_string(&dest) {
            Ok(existing) => existing != DEFAULT_THEME_CSS,
            Err(_) => true,
        }
    } else {
        true
    };

    if needs_update {
        if let Ok(resource_path) = app.path().resolve(
            "resources/themes/default.css",
            tauri::path::BaseDirectory::Resource,
        ) {
            if let Err(e) = fs::copy(&resource_path, &dest) {
                log::warn!("Could not seed default.css from resource: {}", e);
            } else {
                log::info!("Updated default theme at {:?}", dest);
            }
        }

        if let Err(e) = fs::write(&dest, DEFAULT_THEME_CSS) {
            log::warn!("Could not write embedded default theme: {}", e);
        } else {
            log::info!("Updated default theme from embedded copy at {:?}", dest);
        }
    }

    // Seed a universal redesign-compatible theme for users who want the new
    // layout language without losing backwards-compatible class coverage.
    let universal_dest = themes_dir.join("universal.css");
    if universal_dest.exists() {
        return;
    }

    if let Ok(resource_path) = app.path().resolve(
        "resources/themes/universal.css",
        tauri::path::BaseDirectory::Resource,
    ) {
        if let Err(e) = fs::copy(&resource_path, &universal_dest) {
            log::warn!("Could not seed universal.css from resource: {}", e);
        } else {
            log::info!("Seeded universal theme at {:?}", universal_dest);
            return;
        }
    }

    if let Err(e) = fs::write(&universal_dest, UNIVERSAL_THEME_CSS) {
        log::warn!("Could not write embedded universal theme: {}", e);
    } else {
        log::info!("Seeded universal theme from embedded copy at {:?}", universal_dest);
    }
}

fn validate_theme_tokens(
    css: &str,
    width: f64,
    height: f64,
    token_regex: &Regex,
) -> Vec<ThemeDiagnostic> {
    let mut diagnostics = Vec::new();

    // Collect all defined tokens
    let defined: std::collections::HashSet<String> = token_regex
        .captures_iter(css)
        .filter_map(|c| c.get(1).map(|m| format!("--{}", m.as_str())))
        .collect();

    for &token in REQUIRED_TOKENS {
        if !defined.contains(token) {
            diagnostics.push(ThemeDiagnostic {
                level: "error".to_string(),
                token: token.to_string(),
                message: format!(
                    "Required token {} is missing. The UI may not render correctly.",
                    token
                ),
            });
        }
    }

    for &token in RECOMMENDED_TOKENS {
        if !defined.contains(token) {
            diagnostics.push(ThemeDiagnostic {
                level: "warning".to_string(),
                token: token.to_string(),
                message: format!(
                    "Recommended token {} is missing. Some visual details may fall back to defaults.",
                    token
                ),
            });
        }
    }

    if width < 400.0 || width > 1200.0 {
        diagnostics.push(ThemeDiagnostic {
            level: "warning".to_string(),
            token: "--vanta-width".to_string(),
            message: format!(
                "--vanta-width is {}px (expected 400–1200px). Window may be unusable.",
                width
            ),
        });
    }
    if height < 400.0 || height > 1200.0 {
        diagnostics.push(ThemeDiagnostic {
            level: "warning".to_string(),
            token: "--vanta-height".to_string(),
            message: format!(
                "--vanta-height is {}px (expected 400–1200px). Window may be unusable.",
                height
            ),
        });
    }

    diagnostics
}

fn compose_theme_css(theme_css: &str) -> String {
    format!("{}\n\n{}", BASE_THEME_CSS.trim_end(), theme_css.trim_start())
}

fn is_selectable_theme_id(id: &str) -> bool {
    id != "base"
}

#[tauri::command]
pub fn get_installed_themes() -> Result<Vec<ThemeMeta>, VantaError> {
    let dir = get_themes_dir();
    let mut themes = Vec::new();

    let name_regex = Regex::new(r"/\* Theme Name: (.*?) \*/")
        .map_err(|e| format!("Invalid theme name regex: {}", e))?;
    let width_regex = Regex::new(r"--vanta-width:\s*([0-9]+)px")
        .map_err(|e| format!("Invalid theme width regex: {}", e))?;
    let height_regex = Regex::new(r"--vanta-height:\s*([0-9]+)px")
        .map_err(|e| format!("Invalid theme height regex: {}", e))?;
    let token_regex = Regex::new(r"--([\w-]+)\s*:")
        .map_err(|e| format!("Invalid token regex: {}", e))?;

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("css") {
                if let Ok(content) = fs::read_to_string(&path) {
                    let id = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("theme")
                        .to_string();

                    if !is_selectable_theme_id(&id) {
                        continue;
                    }

                    // Fallbacks
                    let mut name = id.clone();
                    let mut width = 680.0;
                    let mut height = 420.0;

                    if let Some(caps) = name_regex.captures(&content) {
                        if let Some(name_cap) = caps.get(1) {
                            name = name_cap.as_str().trim().to_string();
                        }
                    }
                    if let Some(caps) = width_regex.captures(&content) {
                        if let Some(width_cap) = caps.get(1) {
                            if let Ok(w) = width_cap.as_str().parse::<f64>() {
                                width = w;
                            }
                        }
                    }
                    if let Some(caps) = height_regex.captures(&content) {
                        if let Some(height_cap) = caps.get(1) {
                            if let Ok(h) = height_cap.as_str().parse::<f64>() {
                                height = h;
                            }
                        }
                    }

                    let diagnostics = validate_theme_tokens(&content, width, height, &token_regex);

                    themes.push(ThemeMeta {
                        id,
                        name,
                        width,
                        height,
                        css_content: compose_theme_css(&content),
                        diagnostics,
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
) -> Result<(), VantaError> {
    let size = tauri::LogicalSize::new(width, height);
    if let Err(e) = window.set_size(size) {
        return Err(format!("Failed to resize window: {}", e).into());
    }
    // Also center it since dimensions changed
    let _ = window.center();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compose_theme_css_prepends_base_layer() {
        let composed = compose_theme_css(":root { --bg: #000; }");

        assert!(composed.contains(":root { --bg: #000; }"));
        assert!(composed.starts_with(BASE_THEME_CSS));
    }

    #[test]
    fn compose_theme_css_separates_base_and_theme_content() {
        let composed = compose_theme_css(".foo { color: red; }");

        assert!(composed.contains(".foo { color: red; }"));
        assert!(composed.contains("\n\n.foo { color: red; }"));
    }

    #[test]
    fn base_theme_id_is_not_selectable() {
        assert!(!is_selectable_theme_id("base"));
    }

    #[test]
    fn regular_theme_ids_remain_selectable() {
        assert!(is_selectable_theme_id("default"));
        assert!(is_selectable_theme_id("universal"));
    }
}

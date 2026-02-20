use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::Manager;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppEntry {
    pub name: String,
    pub generic_name: Option<String>,
    pub comment: Option<String>,
    pub exec: String,
    pub icon: Option<String>,
    pub categories: Vec<String>,
    pub terminal: bool,
    pub startup_wm_class: Option<String>,
    pub desktop_file_path: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct IconCache {
    icons: HashMap<String, String>,
}

impl IconCache {
    fn get_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        path.push("vanta");
        let _ = fs::create_dir_all(&path);
        path.push("icon-cache.json");
        path
    }

    fn load() -> Self {
        let path = Self::get_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(cache) = serde_json::from_str(&content) {
                    return cache;
                }
            }
        }
        Self::default()
    }

    fn save(&self) {
        let path = Self::get_path();
        if let Ok(content) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, content);
        }
    }
}

fn desktop_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![
        PathBuf::from("/usr/share/applications"),
        PathBuf::from("/usr/local/share/applications"),
        PathBuf::from("/var/lib/flatpak/exports/share/applications"),
        PathBuf::from("/snap/gui"),
        PathBuf::from("/var/lib/snapd/desktop/applications"),
    ];

    if let Some(home) = dirs::home_dir() {
        dirs.push(home.join(".local/share/applications"));
        dirs.push(home.join(".gnome/apps"));
    }

    if let Ok(data_dirs) = std::env::var("XDG_DATA_DIRS") {
        for dir in data_dirs.split(':') {
            let app_dir = PathBuf::from(dir).join("applications");
            if !dirs.contains(&app_dir) {
                dirs.push(app_dir);
            }
        }
    }

    dirs
}

// Tries to find an icon path via absolute path, cache, or XDG lookup.
fn resolve_icon_path(icon_name: &str, cache: &mut IconCache) -> Option<String> {
    // 1. Absolute path check (User Case 1)
    let path = Path::new(icon_name);
    if path.is_absolute() && path.exists() {
        if let Ok(canon) = fs::canonicalize(path) {
            return Some(canon.to_string_lossy().to_string());
        }
        return Some(icon_name.to_string());
    }

    // 2. Check persistent disk cache
    if let Some(cached) = cache.icons.get(icon_name) {
        if Path::new(cached).exists() {
            return Some(cached.clone());
        }
        cache.icons.remove(icon_name);
    }

    // 3. XDG freedesktop lookup
    let theme = freedesktop_icons::default_theme_gtk().unwrap_or_else(|| "hicolor".to_string());

    if let Some(icon) = freedesktop_icons::lookup(icon_name)
        .with_scale(1)
        .with_size(48)
        .with_theme(&theme)
        .with_cache()
        .find()
    {
        let path = icon;
        if let Ok(canon) = fs::canonicalize(&path) {
            let s = canon.to_string_lossy().to_string();
            cache.icons.insert(icon_name.to_string(), s.clone());
            return Some(s);
        }
        let s = path.to_string_lossy().to_string();
        cache.icons.insert(icon_name.to_string(), s.clone());
        return Some(s);
    }

    // 4. Fallback: try hicolor explicit fallback
    if theme != "hicolor" {
        if let Some(icon) = freedesktop_icons::lookup(icon_name)
            .with_scale(1)
            .with_size(48)
            .with_theme("hicolor")
            .with_cache()
            .find()
        {
            let path = icon;
            if let Ok(canon) = fs::canonicalize(&path) {
                let s = canon.to_string_lossy().to_string();
                cache.icons.insert(icon_name.to_string(), s.clone());
                return Some(s);
            }
            let s = path.to_string_lossy().to_string();
            cache.icons.insert(icon_name.to_string(), s.clone());
            return Some(s);
        }
    }

    log::warn!("Could not find icon for '{}'", icon_name);
    None
}

fn parse_desktop_file(path: &Path, cache: &mut IconCache) -> Option<AppEntry> {
    let contents = fs::read_to_string(path).ok()?;

    let mut name: Option<String> = None;
    let mut generic_name: Option<String> = None;
    let mut comment: Option<String> = None;
    let mut exec: Option<String> = None;
    let mut icon_name: Option<String> = None;
    let mut startup_wm_class: Option<String> = None;
    let mut categories: Vec<String> = Vec::new();
    let mut terminal = false;
    let mut no_display = false;
    let mut hidden = false;
    let mut in_desktop_entry = false;

    for line in contents.lines() {
        let line = line.trim();

        if line == "[Desktop Entry]" {
            in_desktop_entry = true;
            continue;
        }

        // Stop parsing if we hit another section
        if line.starts_with('[') && line.ends_with(']') && in_desktop_entry {
            break;
        }

        if !in_desktop_entry {
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();

            match key {
                "Name" => name = Some(value.to_string()),
                "GenericName" => generic_name = Some(value.to_string()),
                "Comment" => comment = Some(value.to_string()),
                "Exec" => exec = Some(value.to_string()),
                "Icon" => icon_name = Some(value.to_string()),
                "StartupWMClass" => startup_wm_class = Some(value.to_string()),
                "Terminal" => terminal = value.eq_ignore_ascii_case("true"),
                "NoDisplay" => no_display = value.eq_ignore_ascii_case("true"),
                "Hidden" => hidden = value.eq_ignore_ascii_case("true"),
                "Categories" => {
                    categories = value
                        .split(';')
                        .filter(|c| !c.is_empty())
                        .map(|c| c.to_string())
                        .collect();
                }
                _ => {}
            }
        }
    }

    // Filter out entries that shouldn't be displayed
    if no_display || hidden {
        return None;
    }

    // Name and Exec are required
    let name = name?;
    let exec = exec?;

    // Resolve icon path
    let icon = icon_name.and_then(|n| resolve_icon_path(&n, cache));

    Some(AppEntry {
        name,
        generic_name,
        comment,
        exec,
        icon,
        categories,
        terminal,
        startup_wm_class,
        desktop_file_path: path.to_string_lossy().to_string(),
    })
}

// Scans standard desktop dirs. Last writer wins for duplicate filenames.
pub fn scan_desktop_entries() -> Vec<AppEntry> {
    let start = std::time::Instant::now();

    // Load existing cache
    let mut cache = IconCache::load();
    let initial_cache_size = cache.icons.len();

    let mut entries: Vec<AppEntry> = Vec::new();
    let mut seen_names: HashSet<String> = HashSet::new();

    // Add Vanta Store generic entry
    entries.push(AppEntry {
        name: "Install Script (Vanta Store)".to_string(),
        generic_name: Some("Type 'install <github-url>' or a local file path to fetch".to_string()),
        comment: Some("Downloads and installs scripts directly into Vanta".to_string()),
        exec: "install:".to_string(),
        icon: Some("system-software-install".to_string()),
        categories: vec![],
        terminal: false,
        startup_wm_class: None,
        desktop_file_path: "vanta://store".to_string(),
    });
    seen_names.insert("Install Script (Vanta Store)".to_string());

    for dir in desktop_dirs() {
        if !dir.exists() {
            continue;
        }

        let read_dir = match fs::read_dir(&dir) {
            Ok(rd) => rd,
            Err(e) => {
                log::warn!("Could not read {}: {}", dir.display(), e);
                continue;
            }
        };

        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("desktop") {
                continue;
            }

            if let Some(app) = parse_desktop_file(&path, &mut cache) {
                // Deduplicate by name (later dirs override earlier)
                if !seen_names.contains(&app.name) {
                    seen_names.insert(app.name.clone());
                    entries.push(app);
                }
            }
        }
    }

    // Save cache if updated
    if cache.icons.len() != initial_cache_size {
        cache.save();
        log::info!(
            "Updated icon cache with {} new entries",
            cache.icons.len() - initial_cache_size
        );
    }

    let elapsed = start.elapsed();
    log::info!(
        "Desktop scan complete: {} entries in {:?}",
        entries.len(),
        elapsed
    );

    // Sort alphabetically by name
    entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    entries
}

pub fn watch_desktop_entries(app_handle: tauri::AppHandle) {
    use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
    use std::sync::mpsc;
    use std::time::Duration;

    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();

    let mut watcher: RecommendedWatcher = match Watcher::new(
        tx,
        notify::Config::default().with_poll_interval(Duration::from_secs(5)),
    ) {
        Ok(w) => w,
        Err(e) => {
            log::error!("Failed to create desktop watcher: {}", e);
            return;
        }
    };

    for dir in desktop_dirs() {
        if dir.exists() {
            if let Err(e) = watcher.watch(&dir, RecursiveMode::NonRecursive) {
                log::warn!("Could not watch {}: {}", dir.display(), e);
            }
        }
    }

    log::info!("Watching desktop entry directories for changes");

    let mut last_scan = std::time::Instant::now();

    for event in rx {
        match event {
            Ok(ev) => {
                let is_desktop_change = ev.paths.iter().any(|p| {
                    p.extension()
                        .and_then(|e| e.to_str())
                        .map(|e| e == "desktop")
                        .unwrap_or(false)
                });

                let is_modify = matches!(
                    ev.kind,
                    EventKind::Create(_) | EventKind::Remove(_) | EventKind::Modify(_)
                );

                // Debounce: don't rescan more than once per second
                if is_desktop_change && is_modify && last_scan.elapsed() > Duration::from_secs(1) {
                    last_scan = std::time::Instant::now();
                    // Brief delay to let the fs settle
                    std::thread::sleep(Duration::from_millis(200));

                    let new_apps = scan_desktop_entries();
                    log::info!("Desktop entries re-scanned: {} apps", new_apps.len());

                    // Update the app state
                    if let Some(state) = app_handle.try_state::<crate::AppState>() {
                        if let Ok(mut apps) = state.apps.lock() {
                            *apps = new_apps;
                        }
                    }
                }
            }
            Err(e) => {
                log::error!("Desktop watcher error: {}", e);
            }
        }
    }
}

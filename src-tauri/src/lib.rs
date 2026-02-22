pub mod clipboard; // New clipboard module
pub mod config;
pub mod errors;
pub mod history;
pub mod launcher;
pub mod matcher;
pub mod math; // New math module
pub mod scanner;
pub mod scripts;
pub mod store; // Script download module
pub mod files; // New files module
pub mod window;
pub mod windows; // New windows enumeration module
pub mod themes;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{Manager, Emitter}; // Added Emitter for .emit()
use serde::Serialize;

use config::VantaConfig;
use history::History;
use matcher::{ResultSource, SearchResult};
use scanner::AppEntry;
use scripts::{ScriptEntry, ScriptOutput};

use files::FileIndex;

// Everything is Mutex'd because Tauri commands are async.
pub struct AppState {
    pub apps: Mutex<Vec<AppEntry>>,
    pub config: Mutex<VantaConfig>,
    pub scripts_cache: Mutex<Vec<ScriptEntry>>,
    pub history: Mutex<History>,
    pub file_index: FileIndex,
}

static SEARCH_CALLS: AtomicU64 = AtomicU64::new(0);
static SEARCH_TOTAL_MS: AtomicU64 = AtomicU64::new(0);
static SEARCH_MAX_MS: AtomicU64 = AtomicU64::new(0);

static SUGGEST_CALLS: AtomicU64 = AtomicU64::new(0);
static SUGGEST_TOTAL_MS: AtomicU64 = AtomicU64::new(0);
static SUGGEST_MAX_MS: AtomicU64 = AtomicU64::new(0);

static LAUNCH_CALLS: AtomicU64 = AtomicU64::new(0);
static LAUNCH_TOTAL_MS: AtomicU64 = AtomicU64::new(0);
static LAUNCH_MAX_MS: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, Serialize)]
struct PerfStats {
    calls: u64,
    total_ms: u64,
    avg_ms: f64,
    max_ms: u64,
}

#[derive(Clone, Debug, Serialize)]
struct SearchDiagnostics {
    search: PerfStats,
    suggestions: PerfStats,
    launch: PerfStats,
}

fn weighted_score(base: u32, weight: u32) -> u32 {
    let clamped = weight.clamp(10, 300);
    let scaled = (base as u128 * clamped as u128) / 100;
    scaled.min(u32::MAX as u128) as u32
}

fn snapshot_perf(calls: &AtomicU64, total_ms: &AtomicU64, max_ms: &AtomicU64) -> PerfStats {
    let calls_val = calls.load(Ordering::Relaxed);
    let total_val = total_ms.load(Ordering::Relaxed);
    let max_val = max_ms.load(Ordering::Relaxed);
    let avg = if calls_val > 0 {
        total_val as f64 / calls_val as f64
    } else {
        0.0
    };

    PerfStats {
        calls: calls_val,
        total_ms: total_val,
        avg_ms: avg,
        max_ms: max_val,
    }
}

fn record_latency(
    label: &str,
    elapsed: Duration,
    calls: &AtomicU64,
    total_ms: &AtomicU64,
    max_ms: &AtomicU64,
) {
    let elapsed_ms = elapsed.as_millis().min(u64::MAX as u128) as u64;
    let current_calls = calls.fetch_add(1, Ordering::Relaxed) + 1;
    total_ms.fetch_add(elapsed_ms, Ordering::Relaxed);

    let mut observed = max_ms.load(Ordering::Relaxed);
    while elapsed_ms > observed {
        match max_ms.compare_exchange_weak(
            observed,
            elapsed_ms,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => break,
            Err(next) => observed = next,
        }
    }

    if current_calls % 50 == 0 {
        let total = total_ms.load(Ordering::Relaxed);
        let max = max_ms.load(Ordering::Relaxed);
        let avg = if current_calls > 0 {
            total as f64 / current_calls as f64
        } else {
            0.0
        };
        log::info!(
            "perf:{} calls={} avg_ms={:.2} max_ms={}",
            label,
            current_calls,
            avg,
            max
        );
    }
}

#[tauri::command]
async fn get_config(
    state: tauri::State<'_, AppState>,
) -> Result<VantaConfig, String> {
    let config = state
        .config
        .lock()
        .map_err(|_| "Failed to access config state".to_string())?;
    Ok(config.clone())
}

#[tauri::command]
async fn save_config(
    new_config: VantaConfig,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let new_files_config = new_config.files.clone();

    {
        let mut config = state
            .config
            .lock()
            .map_err(|_| "Failed to access config state".to_string())?;
        *config = new_config;
        config.save()?;
    }

    // Re-index in background whenever file settings may have changed
    let index_clone = state.file_index.clone();
    std::thread::spawn(move || {
        let entries = files::build_index(&new_files_config);
        if let Ok(mut guard) = index_clone.lock() {
            *guard = entries;
        }
    });

    Ok(())
}

#[tauri::command]
async fn search(
    query: String,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<SearchResult>, String> {
    let search_start = Instant::now();
    let apps = state
        .apps
        .lock()
        .map_err(|_| "Failed to access application cache".to_string())?;
    let (max_results, search_config) = {
        let config = state
            .config
            .lock()
            .map_err(|_| "Failed to access config".to_string())?;
        (config.general.max_results, config.search.clone())
    };

    // Get usage history for boosting
    let usage_map = {
        let history = state
            .history
            .lock()
            .map_err(|_| "Failed to access history".to_string())?;
        history.usage.clone()
    };

    let mut results = if search_config.applications.enabled {
        matcher::fuzzy_search(
            &query,
            &apps,
            max_results,
            &usage_map,
            search_config.applications.weight,
        )
    } else {
        Vec::new()
    };

    // Search Open Windows
    let open_windows = windows::list_windows();
    
    // Perform simple substring match for now (or fuzzy if I want to duplicate logic, but simple is faster for now)
    let query_lower = query.to_lowercase();
    if search_config.windows.enabled {
        for win in open_windows {
            if win.title.to_lowercase().contains(&query_lower)
                || win.class.to_lowercase().contains(&query_lower)
            {
             
             // Try to find matching app for icon
             let matched_app = apps.iter().find(|app| {
                 // 1. Match StartupWMClass (most accurate)
                 if let Some(ref wm_class) = app.startup_wm_class {
                     if wm_class.eq_ignore_ascii_case(&win.class) {
                         return true;
                     }
                 }
                 // 2. Match Exec (first part)
                 // e.g. Exec="gnome-terminal --wait" -> "gnome-terminal"
                 if let Some(cmd) = app.exec.split_whitespace().next() {
                     // Check against class
                     if cmd.eq_ignore_ascii_case(&win.class) {
                         return true;
                     }
                     // Some windows have class="Alacritty", exec="alacritty"
                 }
                 // 3. Match Name
                 if app.name.eq_ignore_ascii_case(&win.class) {
                     return true;
                 }
                 
                 false
             });

             let icon = matched_app.and_then(|a| a.icon.clone());

                let win_result = SearchResult {
                    title: win.title,
                    subtitle: Some(format!("Switch to Window (Workspace {})", win.workspace)),
                    icon: icon,
                    exec: format!("focus:{}", win.address),
                    score: weighted_score(950_000, search_config.windows.weight),
                    match_indices: vec![],
                    source: matcher::ResultSource::Window,
                    actions: None,
                };
                results.push(win_result);
            }
        }
    }

    // Check for math
    if search_config.calculator.enabled {
        if let Some(val) = math::evaluate(&query) {
        let val_str = format!("{}", val);
        let calc_result = SearchResult {
            title: format!("= {}", val_str),
            subtitle: Some("Click to Copy".to_string()),
            icon: Some("calculator".to_string()), 
            exec: format!("copy:{}", val_str), 
            score: weighted_score(900_000, search_config.calculator.weight),
            match_indices: vec![],
            source: matcher::ResultSource::Calculator,
            actions: None,
        };
        results.push(calc_result);
        }
    }

    // Check for file search - instant in-memory lookup!
    if query.starts_with('/') || query.starts_with("~/") {
        if !search_config.files.enabled {
            record_latency(
                "search",
                search_start.elapsed(),
                &SEARCH_CALLS,
                &SEARCH_TOTAL_MS,
                &SEARCH_MAX_MS,
            );
            return Ok(Vec::new());
        }

        let index_guard = state
            .file_index
            .lock()
            .map_err(|_| "Failed to access file index".to_string())?;
        let mut file_results = files::search_index(&index_guard, &query, 20);
        for file_result in &mut file_results {
            file_result.score = weighted_score(file_result.score, search_config.files.weight);
        }
        record_latency(
            "search",
            search_start.elapsed(),
            &SEARCH_CALLS,
            &SEARCH_TOTAL_MS,
            &SEARCH_MAX_MS,
        );
        return Ok(file_results);
    }

    if query.starts_with("install ") {
        let url = query.trim_start_matches("install ").trim().to_string();
        
        if url.starts_with('/') || url.starts_with("~/") {
            let expanded_url = if url.starts_with("~/") {
                 let home = dirs::home_dir().unwrap_or_default();
                 url.replacen("~/", &format!("{}/", home.display()), 1)
            } else if url.starts_with('/') {
                 let home_str = dirs::home_dir().unwrap_or_default().to_string_lossy().into_owned();
                 if url.starts_with(&home_str) {
                     url.clone()
                 } else {
                     if url == "/" {
                         format!("{}/", home_str)
                     } else {
                         url.replacen("/", &format!("{}/", home_str), 1)
                     }
                 }
            } else {
                 url.clone()
            };
            
            let path = std::path::Path::new(&expanded_url);
            
            let (dir_to_read, file_prefix) = if path.is_dir() && expanded_url.ends_with('/') {
                (path.to_path_buf(), "".to_string())
            } else {
                (path.parent().unwrap_or(std::path::Path::new("/")).to_path_buf(), path.file_name().unwrap_or_default().to_string_lossy().to_string())
            };
            
            if let Ok(entries) = std::fs::read_dir(&dir_to_read) {
                for entry in entries.flatten() {
                    let file_name = entry.file_name().to_string_lossy().into_owned();
                    // Don't suggest hidden files unless prefix starts with .
                    if file_name.starts_with('.') && !file_prefix.starts_with('.') {
                        continue;
                    }
                    if file_name.to_lowercase().starts_with(&file_prefix.to_lowercase()) {
                        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                        
                        let mut suggestion_path = dir_to_read.join(&file_name).to_string_lossy().into_owned();
                        if is_dir {
                            suggestion_path.push('/');
                        }
                        
                        let display_path = if url.starts_with("~/") {
                            let home_str = dirs::home_dir().unwrap_or_default().to_string_lossy().into_owned();
                            suggestion_path.replacen(&home_str, "~", 1)
                        } else if url.starts_with('/') {
                            let home_str = dirs::home_dir().unwrap_or_default().to_string_lossy().into_owned();
                            if url.starts_with(&home_str) {
                                suggestion_path.clone()
                            } else {
                                suggestion_path.replacen(&home_str, "", 1)
                            }
                        } else {
                            suggestion_path.clone()
                        };
                        
                        let dir_str = dir_to_read.to_string_lossy().into_owned();
                        let subtitle = if is_dir {
                             format!("{} (Dir)", dir_str)
                        } else {
                             dir_str
                        };
                        
                        let exec_cmd = if is_dir {
                             format!("fill:install {}", display_path)
                        } else {
                             format!("install:{}", suggestion_path)
                        };
                        
                        let icon = if is_dir { "dir".to_string() } else { format!("file:{}", suggestion_path) };
                        
                        results.push(SearchResult {
                            title: file_name,
                            subtitle: Some(subtitle),
                            icon: Some(icon),
                            exec: exec_cmd,
                            score: if is_dir { 900000 } else { 800000 },
                            match_indices: vec![],
                            source: matcher::ResultSource::File,
                            actions: None,
                        });
                    }
                }
            }
        } else if !url.is_empty() && !url.contains("github.com") && !url.starts_with("http") {
             let index_guard = state
            .file_index
            .lock()
            .map_err(|_| "Failed to access file index".to_string())?;
             let file_results = files::search_index(&index_guard, &url, 15);
             for mut res in file_results {
                 let path = std::path::Path::new(&res.exec);
                 let is_dir = path.is_dir();
                 
                 let dir_found = path.parent().unwrap_or(std::path::Path::new("")).to_string_lossy().into_owned();
                 
                 res.subtitle = Some(if is_dir { format!("{} (Dir)", dir_found) } else { dir_found });
                 
                 if is_dir {
                      res.exec = format!("fill:install {}/", res.exec);
                      res.score = 900000;
                 } else {
                      res.exec = format!("install:{}", res.exec);
                      res.score = 800000;
                 }
                 
                 results.push(res);
             }
        }

        if !url.is_empty() {
            let is_local = std::path::Path::new(&url).exists();
            
            let title = if is_local {
                format!("Install Local File: {}", std::path::Path::new(&url).file_name().unwrap_or_default().to_string_lossy())
            } else {
                format!("Install Web Script: {}", url)
            };
            
            let subtitle = if is_local {
                "Press Enter to extract/copy this file directly into the Vanta Store".to_string()
            } else {
                "Press Enter to download and install this script from GitHub".to_string()
            };

            let inst_result = SearchResult {
                title,
                subtitle: Some(subtitle),
                icon: Some("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.5\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><path d=\"M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4\"/><polyline points=\"7 10 12 15 17 10\"/><line x1=\"12\" x2=\"12\" y1=\"15\" y2=\"3\"/></svg>".to_string()),
                exec: format!("install:{}", url),
                score: 1000000, // Top priority
                match_indices: vec![],
                source: matcher::ResultSource::Application,
                actions: None,
            };
            // Put the exact match at the top
            results.insert(0, inst_result);
        }
    }

    results.sort_by(|a, b| b.score.cmp(&a.score));

    record_latency(
        "search",
        search_start.elapsed(),
        &SEARCH_CALLS,
        &SEARCH_TOTAL_MS,
        &SEARCH_MAX_MS,
    );
    Ok(results)
}

#[tauri::command]
async fn launch_app(
    exec: String,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let launch_start = Instant::now();
    // Track usage
    if let Ok(mut history) = state.history.lock() {
        history.increment(&exec);
    }
    let result = launcher::launch(&exec, Some(&app_handle))
        .map_err(|e| format!("Failed to launch: {}", e));
    record_latency(
        "launch",
        launch_start.elapsed(),
        &LAUNCH_CALLS,
        &LAUNCH_TOTAL_MS,
        &LAUNCH_MAX_MS,
    );
    result
}

#[tauri::command]
async fn get_suggestions(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<SearchResult>, String> {
    let suggestions_start = Instant::now();
    let apps = state
        .apps
        .lock()
        .map_err(|_| "Failed to access app cache".to_string())?;
    
    let history = state
        .history
        .lock()
        .map_err(|_| "Failed to access history".to_string())?;

    let config = state
        .config
        .lock()
        .map_err(|_| "Failed to access config".to_string())?;

    let max_results = config.general.max_results;

    if !config.search.applications.enabled {
        record_latency(
            "suggestions",
            suggestions_start.elapsed(),
            &SUGGEST_CALLS,
            &SUGGEST_TOTAL_MS,
            &SUGGEST_MAX_MS,
        );
        return Ok(Vec::new());
    }

    // Create a list of apps with their usage count
    let mut scored_apps: Vec<(&AppEntry, u32)> = apps
        .iter()
        .map(|app| (app, history.get_usage(&app.exec)))
        .collect();

    // Sort by usage count descending
    scored_apps.sort_by(|a, b| b.1.cmp(&a.1));

    // Convert to SearchResult without truncating
    let results: Vec<SearchResult> = scored_apps
        .into_iter()
        .take(max_results)
        .map(|(app, _count)| SearchResult {
            title: app.name.clone(),
            subtitle: app.generic_name.clone().or_else(|| app.comment.clone()),
            icon: app.icon.clone(),
            exec: app.exec.clone(),
            score: weighted_score(100, config.search.applications.weight),
            match_indices: vec![],
            source: ResultSource::Application,
            actions: None,
        })
        .collect();

    record_latency(
        "suggestions",
        suggestions_start.elapsed(),
        &SUGGEST_CALLS,
        &SUGGEST_TOTAL_MS,
        &SUGGEST_MAX_MS,
    );
    Ok(results)
}

#[tauri::command]
async fn get_search_diagnostics() -> Result<SearchDiagnostics, String> {
    Ok(SearchDiagnostics {
        search: snapshot_perf(&SEARCH_CALLS, &SEARCH_TOTAL_MS, &SEARCH_MAX_MS),
        suggestions: snapshot_perf(&SUGGEST_CALLS, &SUGGEST_TOTAL_MS, &SUGGEST_MAX_MS),
        launch: snapshot_perf(&LAUNCH_CALLS, &LAUNCH_TOTAL_MS, &LAUNCH_MAX_MS),
    })
}

#[tauri::command]
async fn rescan_apps(
    state: tauri::State<'_, AppState>,
) -> Result<usize, String> {
    let apps = scanner::scan_desktop_entries();
    let count = apps.len();
    let mut cached = state
        .apps
        .lock()
        .map_err(|_| "Failed to access app cache".to_string())?;
    *cached = apps;
    Ok(count)
}

#[tauri::command]
async fn get_apps(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<scanner::AppEntry>, String> {
    let apps = state
        .apps
        .lock()
        .map_err(|_| "Failed to access app cache".to_string())?;
    Ok(apps.clone())
}

#[tauri::command]
async fn hide_window(window: tauri::WebviewWindow) -> Result<(), String> {
    window::hide_window(&window)
}

#[tauri::command]
async fn show_window(window: tauri::WebviewWindow) -> Result<(), String> {
    window::show_window(&window)
}

#[tauri::command]
async fn get_scripts(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<ScriptEntry>, String> {
    let cached = state
        .scripts_cache
        .lock()
        .map_err(|_| "Failed to access scripts cache".to_string())?;
    Ok(cached.clone())
}

#[tauri::command]
async fn execute_script(
    keyword: String,
    args: String,
    state: tauri::State<'_, AppState>,
) -> Result<ScriptOutput, String> {
    let timeout_ms = {
        let config = state
            .config
            .lock()
            .map_err(|_| "Failed to access config".to_string())?;
        config.scripts.timeout_ms
    };
    // Run script off the main thread via tokio
    tokio::task::spawn_blocking(move || {
        scripts::execute_script(&keyword, &args, timeout_ms)
    })
    .await
    .map_err(|e| format!("Script task failed: {}", e))?
}

#[tauri::command]
async fn get_clipboard_history() -> Result<Vec<clipboard::ClipboardItem>, String> {
    clipboard::get_history().map_err(|e| format!("Failed to get history: {}", e))
}

#[tauri::command]
async fn open_path(
    path: String,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let path_obj = std::path::Path::new(&path);
    if !path_obj.exists() {
        return Err("Path does not exist".to_string());
    }

    let is_dir = path_obj.is_dir();
    
    let config = {
        state.config.lock()
            .map_err(|_| "Failed to access config".to_string())?
            .files.clone()
    };
    
    // Choose which configured app to use
    let app_exec_id = if is_dir || config.open_docs_in_manager {
        config.file_manager
    } else {
        config.file_editor
    };

    if app_exec_id == "default" {
        open::that(&path).map_err(|e| format!("Failed to open path: {}", e))?;
        return Ok(());
    }

    // Attempt to find the full exec string from apps cache
    let matched_app = {
        let apps = state.apps.lock()
            .map_err(|_| "Failed to access apps cache".to_string())?;
        apps.iter().find(|a| a.exec == app_exec_id).cloned()
    };

    if let Some(app) = matched_app {
        // Construct the execution string and send it to the launcher
        // The exec string usually looks like `nautilus %U` or `code %F`
        // We replace any % placeholders or just append the path.
        let mut final_exec = app.exec.clone();
        
        if final_exec.contains("%u") || final_exec.contains("%U") || final_exec.contains("%f") || final_exec.contains("%F") {
            final_exec = final_exec.replace("%u", &format!("\"{}\"", path));
            final_exec = final_exec.replace("%U", &format!("\"{}\"", path));
            final_exec = final_exec.replace("%f", &format!("\"{}\"", path));
            final_exec = final_exec.replace("%F", &format!("\"{}\"", path));
        } else {
            final_exec = format!("{} \"{}\"", final_exec, path);
        }
        
        launcher::launch(&final_exec, Some(&app_handle)).map_err(|e| format!("Failed to launch custom opener: {}", e))?;
    } else {
        // Fallback to default if app string was not found (maybe it was uninstalled)
        log::warn!("Custom opener '{}' not found, falling back to default.", app_exec_id);
        open::that(&path).map_err(|e| format!("Failed to open path: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
async fn reveal_in_file_manager(
    path: String,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let target = std::path::Path::new(&path);
    if !target.exists() {
        return Err("Path does not exist".to_string());
    }

    let dir = if target.is_dir() {
        target.to_path_buf()
    } else {
        target
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| std::path::PathBuf::from("/"))
    };

    let config = {
        state
            .config
            .lock()
            .map_err(|_| "Failed to access config".to_string())?
            .files
            .clone()
    };

    if config.file_manager == "default" {
        open::that(&dir).map_err(|e| format!("Failed to open directory: {}", e))?;
        return Ok(());
    }

    let matched_app = {
        let apps = state
            .apps
            .lock()
            .map_err(|_| "Failed to access apps cache".to_string())?;
        apps
            .iter()
            .find(|a| a.exec == config.file_manager)
            .cloned()
    };

    if let Some(app) = matched_app {
        let mut final_exec = app.exec.clone();
        if final_exec.contains("%u")
            || final_exec.contains("%U")
            || final_exec.contains("%f")
            || final_exec.contains("%F")
        {
            final_exec = final_exec.replace("%u", &format!("\"{}\"", dir.display()));
            final_exec = final_exec.replace("%U", &format!("\"{}\"", dir.display()));
            final_exec = final_exec.replace("%f", &format!("\"{}\"", dir.display()));
            final_exec = final_exec.replace("%F", &format!("\"{}\"", dir.display()));
        } else {
            final_exec = format!("{} \"{}\"", final_exec, dir.display());
        }

        launcher::launch(&final_exec, Some(&app_handle))
            .map_err(|e| format!("Failed to launch file manager: {}", e))?;
    } else {
        log::warn!("Custom file manager '{}' not found, falling back to default.", config.file_manager);
        open::that(&dir).map_err(|e| format!("Failed to open directory: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
async fn open_with_editor(
    path: String,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let path_obj = std::path::Path::new(&path);
    if !path_obj.exists() {
        return Err("Path does not exist".to_string());
    }
    if path_obj.is_dir() {
        return Err("Cannot open directory with editor".to_string());
    }

    let editor_id = {
        state
            .config
            .lock()
            .map_err(|_| "Failed to access config".to_string())?
            .files
            .file_editor
            .clone()
    };

    if editor_id == "default" {
        open::that(&path).map_err(|e| format!("Failed to open path: {}", e))?;
        return Ok(());
    }

    let matched_app = {
        let apps = state
            .apps
            .lock()
            .map_err(|_| "Failed to access apps cache".to_string())?;
        apps.iter().find(|a| a.exec == editor_id).cloned()
    };

    if let Some(app) = matched_app {
        let mut final_exec = app.exec.clone();
        if final_exec.contains("%u")
            || final_exec.contains("%U")
            || final_exec.contains("%f")
            || final_exec.contains("%F")
        {
            final_exec = final_exec.replace("%u", &format!("\"{}\"", path));
            final_exec = final_exec.replace("%U", &format!("\"{}\"", path));
            final_exec = final_exec.replace("%f", &format!("\"{}\"", path));
            final_exec = final_exec.replace("%F", &format!("\"{}\"", path));
        } else {
            final_exec = format!("{} \"{}\"", final_exec, path);
        }

        launcher::launch(&final_exec, Some(&app_handle))
            .map_err(|e| format!("Failed to launch editor: {}", e))?;
    } else {
        log::warn!("Custom editor '{}' not found, falling back to default.", editor_id);
        open::that(&path).map_err(|e| format!("Failed to open path: {}", e))?;
    }

    Ok(())
}



#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    // Init clipboard
    if let Err(e) = clipboard::init_db() {
        log::error!("Failed to init clipboard DB: {}", e);
    }
    clipboard::start_watcher();


    let vanta_config = config::load_or_create_default();
    let hotkey_str = vanta_config.general.hotkey.clone();


    // Keep startup fast: initial scans run in background during setup.
    let apps: Vec<AppEntry> = Vec::new();
    let discovered_scripts: Vec<ScriptEntry> = Vec::new();

    let history = if let Some(config_dir) = dirs::config_dir() {
        let vanta_dir = config_dir.join("vanta");
        if !vanta_dir.exists() {
            let _ = std::fs::create_dir_all(&vanta_dir);
        }
        History::load_or_create(&vanta_dir)
    } else {
        History::new()
    };

    // Create an empty file index. Background thread will populate it shortly.
    let file_index: files::FileIndex = std::sync::Arc::new(Mutex::new(Vec::new()));

    let app_state = AppState {
        apps: Mutex::new(apps),
        config: Mutex::new(vanta_config),
        scripts_cache: Mutex::new(discovered_scripts),
        history: Mutex::new(history),
        file_index: file_index.clone(),
    };

    let mut builder = tauri::Builder::default();


    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(
             |app, args, _cwd| {
                println!("Single instance triggered with args: {:?}", args);
                let lower_args: Vec<String> = args.iter().map(|s| s.to_lowercase()).collect();
                if lower_args.contains(&"--clipboard".to_string()) || lower_args.contains(&"-c".to_string()) {
                    println!("Opening clipboard mode");
                    if let Some(win) = app.get_webview_window("main") {
                        let _ = win.set_always_on_top(true);
                        let _ = win.center();
                        let _ = window::show_window(&win);
                        let _ = win.emit("open_clipboard", ());
                    }
                } else {
                    println!("Toggling window");
                    let _ = window::toggle_window(app);
                }
            },
        ));
    }

    builder
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            search,
            launch_app,
            rescan_apps,
            hide_window,
            show_window,
            get_scripts,
            execute_script,
            get_suggestions,
            get_search_diagnostics,
            get_clipboard_history,
            open_path,
            reveal_in_file_manager,
            open_with_editor,
            get_apps,
            themes::get_installed_themes,
            themes::resize_window_for_theme,
        ])
        .setup(move |app| {
            let app_handle = app.handle().clone();

            // Initialize window (apply blur/transparency/size)
            if let Some(win) = app.get_webview_window("main") {
                let _ = window::init_window(&win, &app_handle);
                
                // Apply window size from config
                let (width, height) = {
                    let state = app.state::<AppState>();
                    let dims = match state.config.lock() {
                        Ok(config) => (config.window.width, config.window.height),
                        Err(_) => (680.0, 420.0),
                    };
                    dims
                };
                let _ = win.set_size(tauri::LogicalSize::new(width, height));
            }

            // Seed the default theme on first run
            themes::seed_default_theme(&app_handle);

            // Register global hotkey (e.g. Alt+Space) AND Clipboard (Super+V)
            #[cfg(desktop)]
            {
                use tauri_plugin_global_shortcut::{
                    GlobalShortcutExt, ShortcutState, Shortcut,
                };
                use std::str::FromStr;

                let mut shortcuts = Vec::new();
                let mut config_shortcut: Option<Shortcut> = None;
                let mut clipboard_shortcut: Option<Shortcut> = None;

                // 1. Config Hotkey
                if let Ok(s) = Shortcut::from_str(&hotkey_str) {
                    config_shortcut = Some(s);
                    shortcuts.push(s);
                } else {
                    log::error!("Invalid config hotkey: {}", hotkey_str);
                }

                // 2. Clipboard Hotkey
                let clipboard_hotkey_str = "Super+V";
                if let Ok(s) = Shortcut::from_str(clipboard_hotkey_str) {
                    clipboard_shortcut = Some(s);
                    shortcuts.push(s);
                } else {
                    log::error!("Invalid clipboard hotkey: {}", clipboard_hotkey_str);
                }

                // Clone for move into closure
                let cfg_sc = config_shortcut.clone();
                let clip_sc = clipboard_shortcut.clone();

                let plugin = tauri_plugin_global_shortcut::Builder::new()
                    .with_handler(move |app, sc, event| {
                        if event.state() == ShortcutState::Pressed {
                            if let Some(ref cfg) = cfg_sc {
                                if sc == cfg {
                                    let _ = window::toggle_window(app);
                                    return;
                                }
                            }
                            if let Some(ref clip) = clip_sc {
                                if sc == clip {
                                    // Open window
                                    if let Some(win) = app.get_webview_window("main") {
                                        let _ = win.set_always_on_top(true);
                                        let _ = win.center();
                                        let _ = window::show_window(&win);
                                        // Emit event for clipboard mode
                                        let _ = win.emit("open_clipboard", ());
                                    }
                                }
                            }
                        }
                    })
                    .build();

                if let Err(e) = app.handle().plugin(plugin) {
                    log::error!("Failed to init global-shortcut plugin: {}", e);
                } else {
                    for s in shortcuts {
                        if let Err(e) = app.global_shortcut().register(s) {
                            log::error!("Failed to register hotkey: {}", e);
                        } else {
                            log::info!("Registered global hotkey: {}", s);
                        }
                    }
                }
            }

            // Background watchers for config/apps/scripts
            let handle_for_watcher = app_handle.clone();
            std::thread::spawn(move || {
                config::watch_config(handle_for_watcher);
            });


            let handle_for_scanner = app_handle.clone();
            std::thread::spawn(move || {
                scanner::watch_desktop_entries(handle_for_scanner);
            });


            let handle_for_scripts = app_handle.clone();
            std::thread::spawn(move || {
                scripts::watch_scripts(handle_for_scripts);
            });

            // Initial app/script scans in background (startup-critical path stays minimal)
            {
                let handle_for_initial_scan = app_handle.clone();
                std::thread::spawn(move || {
                    let apps = scanner::scan_desktop_entries();
                    log::info!("Initial desktop scan complete: {} apps", apps.len());
                    if let Some(state) = handle_for_initial_scan.try_state::<AppState>() {
                        if let Ok(mut cached_apps) = state.apps.lock() {
                            *cached_apps = apps;
                        }
                    }
                    let _ = handle_for_initial_scan.emit("apps-changed", ());

                    let scripts = scripts::scan_scripts();
                    log::info!("Initial script scan complete: {} scripts", scripts.len());
                    if let Some(state) = handle_for_initial_scan.try_state::<AppState>() {
                        if let Ok(mut cached_scripts) = state.scripts_cache.lock() {
                            *cached_scripts = scripts.clone();
                        }
                    }
                    let _ = handle_for_initial_scan.emit("scripts-changed", &scripts);
                });
            }

            // Build the file index in the background (so startup is instant)
            {
                let index_clone = app_handle.state::<AppState>().file_index.clone();
                let files_config = {
                    let state = app_handle.state::<AppState>();
                    let cfg = match state.config.lock() {
                        Ok(cfg) => cfg.files.clone(),
                        Err(_) => crate::config::FilesConfig::default(),
                    };
                    cfg
                };
                std::thread::spawn(move || {
                    log::info!("Building file index...");
                    let entries = files::build_index(&files_config);
                    log::info!("File index ready: {} entries", entries.len());
                    if let Ok(mut guard) = index_clone.lock() {
                        *guard = entries;
                    }
                });
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Vanta");
}

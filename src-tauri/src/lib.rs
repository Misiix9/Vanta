pub mod config;
pub mod errors;
pub mod history;
pub mod launcher;
pub mod matcher;
pub mod math; // New math module
pub mod scanner;
pub mod scripts;
pub mod window;
pub mod windows; // New windows enumeration module

use std::sync::Mutex;
use tauri::Manager;

use config::VantaConfig;
use history::History;
use matcher::{ResultSource, SearchResult};
use scanner::AppEntry;
use scripts::{ScriptEntry, ScriptOutput};

// Everything is Mutex'd because Tauri commands are async.
pub struct AppState {
    pub apps: Mutex<Vec<AppEntry>>,
    pub config: Mutex<VantaConfig>,
    pub scripts_cache: Mutex<Vec<ScriptEntry>>,
    pub history: Mutex<History>,
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
    let mut config = state
        .config
        .lock()
        .map_err(|_| "Failed to access config state".to_string())?;
    
    *config = new_config;
    config.save()?;
    
    Ok(())
}

#[tauri::command]
async fn search(
    query: String,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<SearchResult>, String> {
    let apps = state
        .apps
        .lock()
        .map_err(|_| "Failed to access application cache".to_string())?;
    let config = state
        .config
        .lock()
        .map_err(|_| "Failed to access config".to_string())?;

    let max_results = config.general.max_results;

    // Get usage history for boosting
    let usage_map = {
        let history = state
            .history
            .lock()
            .map_err(|_| "Failed to access history".to_string())?;
        history.usage.clone()
    };

    let mut results = matcher::fuzzy_search(&query, &apps, max_results, &usage_map);

    // Search Open Windows
    let open_windows = windows::list_windows();
    
    // Perform simple substring match for now (or fuzzy if I want to duplicate logic, but simple is faster for now)
    let query_lower = query.to_lowercase();
    for win in open_windows {
        if win.title.to_lowercase().contains(&query_lower) || win.class.to_lowercase().contains(&query_lower) {
             
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
                icon: icon, // Use matched app icon, or None (will fallback to emoji in frontend)
                exec: format!("focus:{}", win.address), 
                score: 1000000, // Very high priority
                match_indices: vec![],
                source: matcher::ResultSource::Window,
            };
            results.insert(0, win_result);
        }
    }

    // Check for math
    if let Some(val) = math::evaluate(&query) {
        let val_str = format!("{}", val);
        let calc_result = SearchResult {
            title: format!("= {}", val_str),
            subtitle: Some("Click to Copy".to_string()),
            icon: Some("calculator".to_string()), 
            exec: format!("copy:{}", val_str), 
            score: 999999, // High priority but below explicit window switch? Or above?
            match_indices: vec![],
            source: matcher::ResultSource::Calculator,
        };
        results.insert(0, calc_result);
    }

    Ok(results)
}

#[tauri::command]
async fn launch_app(
    exec: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    // Track usage
    if let Ok(mut history) = state.history.lock() {
        history.increment(&exec);
    }
    launcher::launch(&exec).map_err(|e| format!("Failed to launch: {}", e))
}

#[tauri::command]
async fn get_suggestions(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<SearchResult>, String> {
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

    let _max_results = config.general.max_results;

    // Create a list of apps with their usage count
    let mut scored_apps: Vec<(&AppEntry, u32)> = apps
        .iter()
        .map(|app| (app, history.get_usage(&app.exec)))
        .collect();

    // Sort by usage count descending
    scored_apps.sort_by(|a, b| b.1.cmp(&a.1));

    // Convert to SearchResult
    let results: Vec<SearchResult> = scored_apps
        .into_iter()
        .map(|(app, _count)| SearchResult {
            title: app.name.clone(),
            subtitle: app.generic_name.clone().or_else(|| app.comment.clone()),
            icon: app.icon.clone(),
            exec: app.exec.clone(),
            score: 0, // Not relevant for suggestions
            match_indices: vec![],
            source: ResultSource::Application,
        })
        .collect();

    Ok(results)
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();


    let vanta_config = config::load_or_create_default();
    let hotkey_str = vanta_config.general.hotkey.clone();


    let apps = scanner::scan_desktop_entries();
    log::info!("Scanned {} desktop applications", apps.len());


    let discovered_scripts = scripts::scan_scripts();
    log::info!("Discovered {} scripts", discovered_scripts.len());

    let history = if let Some(config_dir) = dirs::config_dir() {
        let vanta_dir = config_dir.join("vanta");
        if !vanta_dir.exists() {
            let _ = std::fs::create_dir_all(&vanta_dir);
        }
        History::load_or_create(&vanta_dir)
    } else {
        History::new()
    };

    let app_state = AppState {
        apps: Mutex::new(apps),
        config: Mutex::new(vanta_config),
        scripts_cache: Mutex::new(discovered_scripts),
        history: Mutex::new(history),
    };

    let mut builder = tauri::Builder::default();


    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(
             |app, _args, _cwd| {
                let _ = window::toggle_window(app);
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
        ])
        .setup(move |app| {
            let app_handle = app.handle().clone();

            // Initialize window (apply blur/transparency/size)
            if let Some(win) = app.get_webview_window("main") {
                let _ = window::init_window(&win, &app_handle);
                
                // Apply window size from config
                let (width, height) = {
                    let state = app.state::<AppState>();
                    let config = state.config.lock().unwrap();
                    (config.window.width, config.window.height)
                };
                let _ = win.set_size(tauri::LogicalSize::new(width, height));
            }

            // Register global hotkey (e.g. Alt+Space)
            #[cfg(desktop)]
            {
                use tauri_plugin_global_shortcut::{
                    GlobalShortcutExt, ShortcutState,
                };


                let shortcut_result: Result<
                    tauri_plugin_global_shortcut::Shortcut,
                    _,
                > = hotkey_str.parse();

                match shortcut_result {
                    Ok(shortcut) => {
                        let shortcut_clone = shortcut;
                        let plugin = tauri_plugin_global_shortcut::Builder::new()
                            .with_handler(move |app, sc, event| {
                                if sc == &shortcut_clone
                                    && event.state() == ShortcutState::Pressed
                                {
                                    let _ = window::toggle_window(app);
                                }
                            })
                            .build();

                        if let Err(e) = app.handle().plugin(plugin) {
                            log::error!("Failed to init global-shortcut plugin: {}", e);
                        } else if let Err(e) =
                            app.global_shortcut().register(shortcut)
                        {
                            log::error!(
                                "Failed to register hotkey '{}': {}",
                                hotkey_str,
                                e
                            );
                        } else {
                            log::info!("Global hotkey registered: {}", hotkey_str);
                        }
                    }
                    Err(e) => {
                        log::error!(
                            "Invalid hotkey '{}': {}. Using fallback — no global shortcut.",
                            hotkey_str,
                            e
                        );
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

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Vanta");
}

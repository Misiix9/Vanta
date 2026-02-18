pub mod config;
pub mod errors;
pub mod history;
pub mod launcher;
pub mod matcher;
pub mod scanner;
pub mod scripts;
pub mod window;

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

    let results = matcher::fuzzy_search(&query, &apps, max_results, &usage_map);
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
            search,
            launch_app,
            rescan_apps,
            hide_window,
            show_window,
            get_scripts,
            get_scripts,
            execute_script,
            get_suggestions,
        ])
        .setup(move |app| {
            let app_handle = app.handle().clone();

            // Initialize window (apply blur/transparency)
            if let Some(win) = app.get_webview_window("main") {
                let _ = window::init_window(&win, &app_handle);
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

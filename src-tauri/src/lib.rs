pub mod clipboard;
pub mod config;
pub mod errors;
pub mod extensions;
pub mod history;
pub mod launcher;
pub mod matcher;
pub mod math;
pub mod ranking_config;
pub mod scanner;
pub mod files;
pub mod store;
pub mod window;
pub mod windows;
pub mod themes;
pub mod permissions;
pub mod workflows;
pub mod community;
pub mod secrets;

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::sync::RwLock;
use std::time::{Duration, Instant};
use chrono::{Local, TimeZone, Timelike};
use tauri::{Manager, Emitter};
use serde::{Deserialize, Serialize};
use clap::Parser;

use config::{
    CommandTemplate, ProfileConfig, ProfilesConfig, VantaConfig, WorkflowMacro,
    WorkflowScheduleEvent,
};
use errors::VantaError;
use history::History;
use matcher::{ResultSource, SearchResult};
use scanner::AppEntry;
use extensions::ExtensionEntry;
use permissions::Capability;
use workflows::{MacroDryRunResult, MacroRunResult};
use windows::{list_windows_grouped, WindowGroup};
use config::{clamp_accessibility, clamp_window_size};

use files::FileIndex;
use tokio::time::sleep;
use tauri::WebviewWindow;
use tauri::{LogicalPosition, LogicalSize};

pub struct AppState {
    pub apps: Mutex<Vec<AppEntry>>,
    pub config: RwLock<VantaConfig>,
    pub extensions_cache: Mutex<Vec<ExtensionEntry>>,
    pub history: Mutex<History>,
    pub file_index: FileIndex,
    macro_jobs: Mutex<Vec<MacroJobRecord>>,
    canceled_jobs: Mutex<HashSet<String>>,
    startup_unclean: Mutex<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum MacroJobStatus {
    Running,
    Succeeded,
    Failed,
    Canceled,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct MacroJobRecord {
    id: String,
    macro_id: String,
    macro_name: String,
    args: HashMap<String, String>,
    status: MacroJobStatus,
    created_at: i64,
    updated_at: i64,
    completed_at: Option<i64>,
    steps: Vec<workflows::MacroRunStepResult>,
    error: Option<String>,
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
struct ScheduledWorkflowEvent {
    macro_id: String,
    status: String,
    error: Option<String>,
    ran_at: i64,
}

async fn run_scheduled_workflows_loop(app_handle: tauri::AppHandle) {
    use std::sync::Arc;

    let mut last_run: HashMap<String, Instant> = HashMap::new();
    let mut previous_network_online: Option<bool> = None;
    let running: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));

    loop {
        sleep(Duration::from_secs(30)).await;

        let macros = {
            let state = app_handle.state::<AppState>();
            let macros_result = match state.config.read() {
                Ok(cfg) => cfg.workflows.macros.clone(),
                Err(_) => continue,
            };
            macros_result
        };

        let now_instant = Instant::now();
        let network_online = has_network_connectivity().await;
        let network_connected_event = matches!(previous_network_online, Some(false)) && network_online;
        previous_network_online = Some(network_online);

        for macro_def in macros {
            if !macro_def.enabled {
                continue;
            }

            let Some(schedule) = macro_def.schedule.clone() else {
                continue;
            };

            if !schedule.enabled {
                continue;
            }

            let interval = Duration::from_secs(schedule.interval_minutes.max(1) * 60);
            let first_run = !last_run.contains_key(&macro_def.id);
            let startup_event = schedule
                .on_events
                .iter()
                .any(|e| matches!(e, WorkflowScheduleEvent::Startup));
            let network_event = schedule
                .on_events
                .iter()
                .any(|e| matches!(e, WorkflowScheduleEvent::NetworkConnected));

            let should_run_startup = first_run && (schedule.run_on_startup || startup_event);
            let should_run_interval = match last_run.get(&macro_def.id) {
                Some(prev) => now_instant.duration_since(*prev) >= interval,
                None => false,
            };
            let should_run_network_event = network_event && network_connected_event;

            let should_run = should_run_startup || should_run_interval || should_run_network_event;

            if !should_run {
                continue;
            }

            {
                let mut running_guard = match running.lock() {
                    Ok(guard) => guard,
                    Err(_) => continue,
                };
                if running_guard.contains(&macro_def.id) {
                    continue;
                }
                running_guard.insert(macro_def.id.clone());
            }

            last_run.insert(macro_def.id.clone(), now_instant);

            let app_for_task = app_handle.clone();
            let running_for_task = running.clone();
            let macro_id = macro_def.id.clone();
            tauri::async_runtime::spawn(async move {
                let macro_id_for_block = macro_id.clone();
                let app_for_block = app_for_task.clone();
                let run_result = tauri::async_runtime::spawn_blocking(move || {
                    let state = app_for_block.state::<AppState>();
                    workflows::run_macro_blocking(
                        &macro_id_for_block,
                        HashMap::new(),
                        &state,
                        &app_for_block,
                    )
                })
                .await;

                let event = match run_result {
                    Ok(Ok(_)) => ScheduledWorkflowEvent {
                        macro_id: macro_id.clone(),
                        status: "ok".to_string(),
                        error: None,
                        ran_at: now_millis(),
                    },
                    Ok(Err(err)) => ScheduledWorkflowEvent {
                        macro_id: macro_id.clone(),
                        status: "failed".to_string(),
                        error: Some(err.to_string()),
                        ran_at: now_millis(),
                    },
                    Err(join_err) => ScheduledWorkflowEvent {
                        macro_id: macro_id.clone(),
                        status: "failed".to_string(),
                        error: Some(format!("Scheduled workflow join error: {}", join_err)),
                        ran_at: now_millis(),
                    },
                };

                let _ = app_for_task.emit("scheduled-workflow-run", event);

                if let Ok(mut running_guard) = running_for_task.lock() {
                    running_guard.remove(&macro_id);
                }
            });
        }
    }
}

async fn has_network_connectivity() -> bool {
    let connect = tokio::net::TcpStream::connect(("1.1.1.1", 53));
    matches!(tokio::time::timeout(Duration::from_secs(2), connect).await, Ok(Ok(_)))
}

fn current_window_dims(app: &tauri::AppHandle) -> (f64, f64) {
    let state = app.state::<AppState>();
    let (w, h) = match state.config.read() {
        Ok(cfg) => (cfg.window.width, cfg.window.height),
        Err(_) => (680.0, 420.0),
    };

    let mut wc = config::WindowConfig { width: w, height: h };
    let _ = clamp_window_size(&mut wc);
    (wc.width, wc.height)
}

fn emit_clipboard_with_retries(win: &WebviewWindow) {
    let w1 = win.clone();
    let w2 = win.clone();
    tauri::async_runtime::spawn(async move {
        sleep(Duration::from_millis(120)).await;
        let _ = w1.emit("open_clipboard", ());
        sleep(Duration::from_millis(300)).await;
        let _ = w2.emit("open_clipboard", ());
    });
}

fn now_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn jobs_path() -> std::path::PathBuf {
    config::config_dir().join("macro-jobs.json")
}

fn startup_state_path() -> std::path::PathBuf {
    config::config_dir().join("startup-state.json")
}

fn ui_state_path() -> std::path::PathBuf {
    config::config_dir().join("ui-state.json")
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
struct UiState {
    #[serde(default)]
    window_width: Option<f64>,
    #[serde(default)]
    window_height: Option<f64>,
    #[serde(default)]
    window_x: Option<f64>,
    #[serde(default)]
    window_y: Option<f64>,
    #[serde(default)]
    last_view: Option<String>,
}

fn load_ui_state() -> UiState {
    let path = ui_state_path();
    std::fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str::<UiState>(&raw).ok())
        .unwrap_or_default()
}

fn save_ui_state(state: &UiState) {
    let path = ui_state_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(serialized) = serde_json::to_string_pretty(state) {
        let _ = std::fs::write(path, serialized);
    }
}

fn persist_window_state(win: &WebviewWindow) {
    let size = win.inner_size();
    let pos = win.outer_position();
    let (Ok(size), Ok(pos)) = (size, pos) else {
        return;
    };

    let scale = win.scale_factor().unwrap_or(1.0);
    let logical = size.to_logical::<f64>(scale);
    let logical_pos = pos.to_logical::<f64>(scale);

    let mut snapshot = load_ui_state();
    snapshot.window_width = Some(logical.width);
    snapshot.window_height = Some(logical.height);
    snapshot.window_x = Some(logical_pos.x);
    snapshot.window_y = Some(logical_pos.y);
    save_ui_state(&snapshot);
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct StartupState {
    in_progress: bool,
    updated_at: i64,
}

fn mark_startup_in_progress() -> bool {
    let path = startup_state_path();
    let previous_unclean = std::fs::read_to_string(&path)
        .ok()
        .and_then(|raw| serde_json::from_str::<StartupState>(&raw).ok())
        .map(|s| s.in_progress)
        .unwrap_or(false);

    let next = StartupState {
        in_progress: true,
        updated_at: now_millis(),
    };

    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(
        &path,
        serde_json::to_string_pretty(&next).unwrap_or_else(|_| "{}".to_string()),
    );

    previous_unclean
}

fn mark_startup_complete() {
    let path = startup_state_path();
    let done = StartupState {
        in_progress: false,
        updated_at: now_millis(),
    };
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(
        &path,
        serde_json::to_string_pretty(&done).unwrap_or_else(|_| "{}".to_string()),
    );
}

fn support_bundle_dir() -> std::path::PathBuf {
    config::config_dir().join("support-bundles")
}

fn load_jobs_from_disk() -> Vec<MacroJobRecord> {
    let path = jobs_path();
    let Ok(raw) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    serde_json::from_str::<Vec<MacroJobRecord>>(&raw).unwrap_or_default()
}

fn save_jobs_to_disk(jobs: &[MacroJobRecord]) {
    let path = jobs_path();
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Ok(serialized) = serde_json::to_string_pretty(jobs) {
        let _ = std::fs::write(path, serialized);
    }
}

fn emit_jobs_updated(app_handle: &tauri::AppHandle, jobs: &[MacroJobRecord]) {
    let _ = app_handle.emit("macro-jobs-updated", jobs);
}

#[derive(Parser, Debug)]
#[command(name = "vanta", about = "Vanta launcher and CLI toolkit")]
pub struct Cli {
    #[arg(long, default_value_t = false)]
    pub hidden: bool,

    #[arg(long, short = 'c', default_value_t = false)]
    pub clipboard: bool,
}

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

#[derive(Clone, Debug, Serialize)]
struct HealthCheck {
    name: String,
    status: String,
    detail: String,
}

#[derive(Clone, Debug, Serialize)]
struct HealthDashboard {
    generated_at: i64,
    config_schema: u32,
    active_profile_id: String,
    apps_cached: usize,
    extensions_cached: usize,
    file_index_entries: usize,
    macro_jobs_total: usize,
    checks: Vec<HealthCheck>,
}

#[derive(Clone, Debug, Serialize)]
struct UsageAppStat {
    exec: String,
    launches: u32,
    frecency: u32,
}

#[derive(Clone, Debug, Serialize)]
struct UsageHourStat {
    hour: u8,
    launches: u32,
}

#[derive(Clone, Debug, Serialize)]
struct UsageQueryStat {
    query: String,
    count: u32,
}

#[derive(Clone, Debug, Serialize)]
struct UsageAnalyticsReport {
    generated_at: i64,
    total_launch_events: u32,
    most_used_apps: Vec<UsageAppStat>,
    peak_usage_hours: Vec<UsageHourStat>,
    search_patterns: Vec<UsageQueryStat>,
}

fn is_internal_exec_for_analytics(exec: &str) -> bool {
    exec.starts_with("open-")
        || exec.starts_with("system-action:")
        || exec.starts_with("copy:")
        || exec.starts_with("copy-path:")
        || exec.starts_with("reveal:")
        || exec.starts_with("open-with:")
        || exec.starts_with("focus:")
        || exec.starts_with("close-window:")
        || exec.starts_with("minimize-window:")
        || exec.starts_with("move-window-current:")
        || exec.starts_with("fill:")
        || exec.starts_with("intent-workflow:")
        || exec.starts_with("profile-switch:")
        || exec.starts_with("macro:")
        || exec.starts_with("ext-view:")
        || exec.starts_with("ext-no-view:")
        || exec.starts_with("note-save:")
        || exec.starts_with("bookmark-")
}

#[derive(Clone, Debug, Serialize)]
struct SupportBundleReport {
    path: String,
    size_bytes: u64,
}

#[derive(Clone, Debug, Serialize)]
struct RecoveryHint {
    id: String,
    title: String,
    detail: String,
}

#[derive(Clone, Debug, Serialize)]
struct ContractMigrationReport {
    config: config::ConfigMigrationReport,
    extensions: extensions::ExtensionMigrationReport,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum CommandV1 {
    LaunchApp { exec: String },
    OpenFile { path: String },
    OpenSettings,
    OpenStore,
    OpenFeatureWindow { window: String },
    CopyText { value: String },
    CopyPath { value: String },
    RevealPath { path: String },
    OpenWithEditor { path: String },
    FocusWindow { id: String },
    CloseWindow { id: String },
    MinimizeWindow { id: String },
    MoveWindowCurrentWorkspace { id: String },
    SystemAction { action: String },
    MacroOpen { id: String },
    MacroTemplateOpen { template_id: String },
    ExtensionView { ext_id: String, command: String },
    ExtensionAction { ext_id: String, command: String },
    QueryFill { value: String },
    IntentWorkflow { steps: Vec<String> },
    ProfileSwitch { id: String },
    Unknown { exec: String },
}

#[derive(Clone, Debug, Serialize)]
struct ResultActionV3 {
    label: String,
    shortcut: Option<String>,
    command: CommandV1,
}

#[derive(Clone, Debug, Serialize)]
struct SearchResultV3 {
    title: String,
    subtitle: Option<String>,
    icon: Option<String>,
    score: u32,
    match_indices: Vec<u32>,
    source: matcher::ResultSource,
    id: Option<String>,
    group: Option<String>,
    section: Option<String>,
    version: u8,
    command: CommandV1,
    actions: Option<Vec<ResultActionV3>>,
    // Keep legacy compatibility during v3 migration.
    exec: String,
}

fn weighted_score(base: u32, weight: u32) -> u32 {
    let clamped = weight.clamp(ranking_config::WEIGHT_MIN, ranking_config::WEIGHT_MAX);
    let scaled = (base as u128 * clamped as u128) / 100;
    scaled.min(u32::MAX as u128) as u32
}

// ── Search Filters ───────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum FilterSource {
    App,
    File,
    Window,
    Clipboard,
    Extension,
}

#[derive(Debug, Clone)]
struct SearchFilter {
    source: Option<FilterSource>,
    extension_id: Option<String>,
    raw_query: String,
}

fn parse_search_filters(query: &str) -> SearchFilter {
    let mut source = None;
    let mut extension_id = None;
    let mut remaining = Vec::new();

    for token in query.split_whitespace() {
        if let Some(val) = token.strip_prefix("type:") {
            match val.to_lowercase().as_str() {
                "app" | "application" | "apps" => source = Some(FilterSource::App),
                "file" | "files" | "document" | "documents" => source = Some(FilterSource::File),
                "window" | "windows" => source = Some(FilterSource::Window),
                "clipboard" | "clip" => source = Some(FilterSource::Clipboard),
                "extension" | "ext" | "extensions" => source = Some(FilterSource::Extension),
                _ => remaining.push(token.to_string()),
            }
        } else if let Some(val) = token.strip_prefix("in:") {
            match val.to_lowercase().as_str() {
                "clipboard" | "clip" => source = Some(FilterSource::Clipboard),
                "apps" | "app" => source = Some(FilterSource::App),
                "files" | "file" => source = Some(FilterSource::File),
                "windows" | "window" => source = Some(FilterSource::Window),
                _ => remaining.push(token.to_string()),
            }
        } else if let Some(val) = token.strip_prefix("ext:") {
            extension_id = Some(val.to_string());
            source = Some(FilterSource::Extension);
        } else {
            remaining.push(token.to_string());
        }
    }

    SearchFilter {
        source,
        extension_id,
        raw_query: remaining.join(" "),
    }
}

fn filter_matches_source(filter: &SearchFilter, source: &ResultSource) -> bool {
    let Some(ref fs) = filter.source else {
        return true;
    };
    match fs {
        FilterSource::App => matches!(source, ResultSource::Application),
        FilterSource::File => matches!(source, ResultSource::File),
        FilterSource::Window => matches!(source, ResultSource::Window),
        FilterSource::Clipboard => matches!(source, ResultSource::Clipboard),
        FilterSource::Extension => {
            if let ResultSource::Extension { ref ext_id } = source {
                filter
                    .extension_id
                    .as_ref()
                    .map(|f| ext_id.to_lowercase().contains(&f.to_lowercase()))
                    .unwrap_or(true)
            } else {
                false
            }
        }
    }
}

// ── Typo tolerance (edit distance) ───────────────────────────────────

fn edit_distance(a: &str, b: &str) -> usize {
    let a_ch: Vec<char> = a.chars().collect();
    let b_ch: Vec<char> = b.chars().collect();
    let (m, n) = (a_ch.len(), b_ch.len());
    let mut prev = (0..=n).collect::<Vec<_>>();
    let mut curr = vec![0; n + 1];
    for i in 1..=m {
        curr[0] = i;
        for j in 1..=n {
            let cost = if a_ch[i - 1] == b_ch[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[n]
}

fn typo_suggestions(query: &str, apps: &[AppEntry], max: usize) -> Vec<String> {
    let q = query.trim().to_lowercase();
    if q.len() < 2 {
        return Vec::new();
    }
    let threshold = if q.len() <= 4 { 1 } else { 2 };
    let mut candidates: Vec<(usize, String)> = apps
        .iter()
        .filter_map(|app| {
            let name_lower = app.name.to_lowercase();
            let dist = edit_distance(&q, &name_lower);
            if dist > 0 && dist <= threshold {
                Some((dist, app.name.clone()))
            } else {
                None
            }
        })
        .collect();
    candidates.sort_by_key(|(dist, _)| *dist);
    candidates.dedup_by(|a, b| a.1.to_lowercase() == b.1.to_lowercase());
    candidates.into_iter().take(max).map(|(_, name)| name).collect()
}

fn split_intent_steps(query: &str) -> Vec<String> {
    let normalized = query
        .replace(" and then ", " then ")
        .replace(" && ", " then ")
        .replace(" -> ", " then ")
        .replace(';', " then ");

    normalized
        .split(" then ")
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| part.to_string())
        .collect()
}

fn resolve_app_exec_for_step(step: &str, apps: &[AppEntry]) -> Option<String> {
    let lower = step.to_lowercase();
    let normalized = lower
        .replace("open ", " ")
        .replace("launch ", " ")
        .replace("run ", " ")
        .replace("start ", " ");
    let tokens: Vec<&str> = normalized.split_whitespace().collect();
    if tokens.is_empty() {
        return None;
    }

    let mut best: Option<(&AppEntry, u32)> = None;
    for app in apps {
        let name = app.name.to_lowercase();
        let generic = app
            .generic_name
            .as_deref()
            .unwrap_or_default()
            .to_lowercase();
        let cmd = app
            .exec
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .to_lowercase();

        let mut score = 0u32;
        for t in &tokens {
            if name.contains(t) {
                score += 40;
            }
            if !generic.is_empty() && generic.contains(t) {
                score += 20;
            }
            if !cmd.is_empty() && cmd.contains(t) {
                score += 30;
            }
        }

        // Fuzzy match against app name improves short aliases like "zen" or "foot".
        if let Some((raw, _)) = matcher::fuzzy_score_text(&normalized, &app.name) {
            score = score.saturating_add(raw.saturating_mul(4));
        }

        if score == 0 {
            continue;
        }

        match best {
            Some((_, best_score)) if best_score >= score => {}
            _ => best = Some((app, score)),
        }
    }

    best.map(|(app, _)| app.exec.clone())
}

fn infer_intent_step_exec(step: &str, apps: &[AppEntry]) -> Option<String> {
    let s = step.to_lowercase();

    let app_launch_hint = s.contains("open ")
        || s.contains("launch ")
        || s.contains("run ")
        || s.contains("start ");
    if app_launch_hint {
        if let Some(app_exec) = resolve_app_exec_for_step(step, apps) {
            return Some(app_exec);
        }
    }

    if s.contains("setting") || s.contains("preference") || s.contains("config") {
        return Some("open-settings".to_string());
    }
    if s.contains("store") || s.contains("marketplace") {
        return Some("open-store".to_string());
    }
    if s.contains("feature hub") {
        return Some("open-window:featureHub".to_string());
    }
    if s.contains("community") {
        return Some("open-window:communityHub".to_string());
    }
    if s.contains("theme") || s.contains("profile studio") {
        return Some("open-window:themeHub".to_string());
    }
    if s.contains("extension") && (s.contains("wizard") || s.contains("template") || s.contains("creator")) {
        return Some("open-window:extensionsHub".to_string());
    }

    if s.contains("shutdown") || s.contains("power off") {
        return Some("system-action:shutdown".to_string());
    }
    if s.contains("restart") || s.contains("reboot") {
        if s.contains("bios") || s.contains("firmware") {
            return Some("system-action:bios".to_string());
        }
        return Some("system-action:restart".to_string());
    }
    if s.contains("sleep") || s.contains("suspend") {
        return Some("system-action:sleep".to_string());
    }
    if s.contains("logout") || s.contains("log out") {
        return Some("system-action:logout".to_string());
    }
    if s.contains("lock") {
        return Some("system-action:lock".to_string());
    }

    None
}

fn build_intent_results(query: &str, weight: u32, apps: &[AppEntry]) -> Vec<SearchResult> {
    let trimmed = query.trim();
    if trimmed.len() < ranking_config::INTENT_MIN_QUERY_LEN {
        return Vec::new();
    }

    let steps = split_intent_steps(trimmed);
    if steps.len() < ranking_config::INTENT_MIN_STEPS {
        return Vec::new();
    }

    let mut planned_exec_steps = Vec::new();
    for step in &steps {
        if let Some(exec) = infer_intent_step_exec(step, apps) {
            planned_exec_steps.push(exec);
        }
    }

    if planned_exec_steps.len() < 2 {
        return Vec::new();
    }

    let workflow_exec = format!("intent-workflow:{}", planned_exec_steps.join("|||"));
    vec![SearchResult {
        title: format!("Workflow: {}", steps.join(" -> ")),
        subtitle: Some(format!(
            "Adaptive Intent Engine: {} steps with explicit confirmation",
            planned_exec_steps.len()
        )),
        icon: Some("fa-solid fa-route".to_string()),
        exec: workflow_exec,
        score: weighted_score(ranking_config::INTENT_BASE_SCORE, weight),
        match_indices: vec![],
        source: ResultSource::Application,
        actions: Some(vec![matcher::ActionHint {
            label: "Preview Step 1".to_string(),
            exec: format!("fill:{}", steps.first().cloned().unwrap_or_default()),
            shortcut: Some("Tab".to_string()),
        }]),
        id: Some("adaptive-intent-workflow".to_string()),
        group: None,
        section: Some("Intent".to_string()),
    }]
}

fn build_command_palette_results(query: &str, weight: u32) -> Vec<SearchResult> {
    let needle = query.trim().to_lowercase();
    let commands: [(&str, &str, &str, &str); 6] = [
        ("Open Settings", "Configure Vanta preferences", "open-settings", "Commands"),
        ("Open Store", "Browse and install extensions", "open-store", "Commands"),
        ("Open Feature Hub", "Navigate to feature hub", "open-window:featureHub", "Commands"),
        ("Open Community Hub", "Open community workflows and snippets", "open-window:communityHub", "Commands"),
        ("Open Theme Studio", "Customize themes and profile visuals", "open-window:themeHub", "Commands"),
        ("Open Extensions Hub", "Manage extension templates and runtime", "open-window:extensionsHub", "Commands"),
    ];

    let mut out = Vec::new();
    for (title, subtitle, exec, section) in commands {
        let hay = format!("{} {} {}", title.to_lowercase(), subtitle.to_lowercase(), exec.to_lowercase());
        if !needle.is_empty() && !hay.contains(&needle) {
            continue;
        }
        out.push(SearchResult {
            title: title.to_string(),
            subtitle: Some(subtitle.to_string()),
            icon: Some("fa-solid fa-terminal".to_string()),
            exec: exec.to_string(),
            score: weighted_score(ranking_config::STORE_SEARCH_SCORE, weight),
            match_indices: vec![],
            source: ResultSource::Application,
            actions: None,
            id: Some(format!("cmd:{}", exec)),
            group: None,
            section: Some(section.to_string()),
        });
    }

    out
}

fn build_command_template_palette_results(
    query: &str,
    templates: &[CommandTemplate],
    macros: &[WorkflowMacro],
    weight: u32,
) -> Vec<SearchResult> {
    let needle = query.trim().to_lowercase();
    let macro_names = macros
        .iter()
        .map(|m| (m.id.clone(), m.name.clone()))
        .collect::<HashMap<_, _>>();

    let mut sorted = templates.to_vec();
    sorted.sort_by(|a, b| b.updated_at_ms.cmp(&a.updated_at_ms));

    let mut out = Vec::new();
    for (idx, template) in sorted.into_iter().enumerate() {
        let macro_name = macro_names
            .get(&template.macro_id)
            .cloned()
            .unwrap_or_else(|| template.macro_id.clone());

        let mut args_preview = template
            .args
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>();
        args_preview.sort();
        let args_summary = if args_preview.is_empty() {
            "No args".to_string()
        } else {
            args_preview.join(" ")
        };

        let hay = format!(
            "{} {} {} {}",
            template.name.to_lowercase(),
            macro_name.to_lowercase(),
            template.macro_id.to_lowercase(),
            args_summary.to_lowercase()
        );

        if !needle.is_empty() && !hay.contains(&needle) {
            continue;
        }

        let base = if needle.is_empty() {
            ranking_config::STORE_SEARCH_SCORE + 250
        } else if template.name.to_lowercase() == needle {
            ranking_config::STORE_SEARCH_SCORE + 500
        } else if template.name.to_lowercase().starts_with(&needle) {
            ranking_config::STORE_SEARCH_SCORE + 380
        } else {
            ranking_config::STORE_SEARCH_SCORE + 260
        };

        out.push(SearchResult {
            title: format!("Template: {}", template.name),
            subtitle: Some(format!("{} · {}", macro_name, args_summary)),
            icon: Some("fa-solid fa-layer-group".to_string()),
            exec: format!("macro-template:{}", template.id),
            score: weighted_score(base.saturating_sub(idx as u32), weight),
            match_indices: vec![],
            source: ResultSource::Application,
            actions: None,
            id: Some(format!("cmd-template:{}", template.id)),
            group: None,
            section: Some("Templates".to_string()),
        });
    }

    out
}

fn build_quick_note_results(
    note_query: &str,
    notes: &config::NotesConfig,
    weight: u32,
    max_results: usize,
) -> Vec<SearchResult> {
    let trimmed = note_query.trim();
    let needle = trimmed.to_lowercase();
    let mut out = Vec::new();

    if !trimmed.is_empty() {
        out.push(SearchResult {
            title: format!("Save Note: {}", trimmed),
            subtitle: Some("Store quick note in Vanta".to_string()),
            icon: Some("fa-solid fa-note-sticky".to_string()),
            exec: format!("note-save:{}", trimmed),
            score: weighted_score(ranking_config::SETTINGS_BASE_SCORE, weight),
            match_indices: vec![],
            source: ResultSource::Application,
            actions: None,
            id: Some("note:save".to_string()),
            group: None,
            section: Some("Notes".to_string()),
        });
    }

    for note in notes.entries.iter().rev() {
        if !needle.is_empty() && !note.text.to_lowercase().contains(&needle) {
            continue;
        }
        out.push(SearchResult {
            title: note.text.clone(),
            subtitle: Some("Quick note".to_string()),
            icon: Some("fa-solid fa-note-sticky".to_string()),
            exec: format!("copy:{}", note.text),
            score: weighted_score(ranking_config::CALCULATOR_BASE_SCORE, weight),
            match_indices: vec![],
            source: ResultSource::Application,
            actions: None,
            id: Some(format!("note:{}", note.id)),
            group: None,
            section: Some("Notes".to_string()),
        });
        if out.len() >= max_results {
            break;
        }
    }

    out
}

fn build_bookmark_results(
    bookmark_query: &str,
    bookmarks: &config::BookmarksConfig,
    weight: u32,
    max_results: usize,
) -> Vec<SearchResult> {
    let needle = bookmark_query.trim().to_lowercase();
    let mut out = Vec::new();

    for bookmark in bookmarks.entries.iter().rev() {
        if !needle.is_empty() && !bookmark.path.to_lowercase().contains(&needle) {
            continue;
        }

        let title = std::path::Path::new(&bookmark.path)
            .file_name()
            .and_then(|s| s.to_str())
            .filter(|s| !s.is_empty())
            .unwrap_or(&bookmark.path)
            .to_string();

        out.push(SearchResult {
            title,
            subtitle: Some(bookmark.path.clone()),
            icon: Some("fa-solid fa-star".to_string()),
            exec: bookmark.path.clone(),
            score: weighted_score(ranking_config::SETTINGS_BASE_SCORE, weight),
            match_indices: vec![],
            source: ResultSource::File,
            actions: Some(vec![matcher::ActionHint {
                label: "Remove Bookmark".to_string(),
                exec: format!("bookmark-remove:{}", bookmark.path),
                shortcut: None,
            }]),
            id: Some(format!("bookmark:{}", bookmark.path)),
            group: None,
            section: Some("Bookmarks".to_string()),
        });

        if out.len() >= max_results {
            break;
        }
    }

    out
}

fn query_relevance_bonus(query: &str, result: &SearchResult) -> u32 {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return 0;
    }

    let title = result.title.to_lowercase();
    let subtitle = result
        .subtitle
        .as_deref()
        .unwrap_or_default()
        .to_lowercase();
    let exec = result.exec.to_lowercase();

    let mut bonus = 0u32;
    if title == q {
        bonus += ranking_config::QR_TITLE_EXACT;
    } else if title.starts_with(&q) {
        bonus += ranking_config::QR_TITLE_PREFIX;
    } else if title.contains(&q) {
        bonus += ranking_config::QR_TITLE_CONTAINS;
    }

    if !subtitle.is_empty() && subtitle.contains(&q) {
        bonus += ranking_config::QR_SUBTITLE_CONTAINS;
    }

    if exec.starts_with(&q) || exec.contains(&q) {
        bonus += ranking_config::QR_EXEC_CONTAINS;
    }

    if q.split_whitespace().count() > 1 && q.split_whitespace().all(|t| title.contains(t)) {
        bonus += ranking_config::QR_MULTI_TOKEN_ALL;
    }

    bonus
}

fn source_intent_bonus(query: &str, result: &SearchResult) -> u32 {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return 0;
    }

    match result.source {
        ResultSource::Application => {
            if q.starts_with("open ") || q.starts_with("launch ") || q.starts_with("run ") {
                ranking_config::SI_APPLICATION
            } else {
                0
            }
        }
        ResultSource::File => {
            if q.starts_with("file ")
                || q.contains("document")
                || q.contains("folder")
                || q.contains("path")
                || q.contains("download")
            {
                ranking_config::SI_FILE
            } else {
                0
            }
        }
        ResultSource::Window => {
            if q.contains("window") || q.contains("switch") || q.contains("focus") {
                ranking_config::SI_WINDOW
            } else {
                0
            }
        }
        ResultSource::Calculator => {
            if q.chars().any(|c| c.is_ascii_digit()) {
                ranking_config::SI_CALCULATOR
            } else {
                0
            }
        }
        ResultSource::Extension { .. } => {
            if q.contains("extension") || q.contains("plugin") {
                ranking_config::SI_EXTENSION
            } else {
                0
            }
        }
        ResultSource::Clipboard => {
            if q.contains("clipboard") || q.contains("copy") || q.contains("snippet") {
                ranking_config::SI_CLIPBOARD
            } else {
                0
            }
        }
    }
}

fn app_entity_bonus(query: &str, result: &SearchResult) -> u32 {
    if !matches!(result.source, ResultSource::Application) {
        return 0;
    }

    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return 0;
    }

    let title = result.title.to_lowercase();
    if title == q {
        ranking_config::AE_EXACT
    } else if title.starts_with(&q) {
        ranking_config::AE_PREFIX
    } else if title.contains(&q) {
        ranking_config::AE_CONTAINS
    } else {
        0
    }
}

fn exec_to_command(exec: &str, source: &matcher::ResultSource) -> CommandV1 {
    if let Some(v) = exec.strip_prefix("intent-workflow:") {
        let steps = v
            .split("|||")
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        return CommandV1::IntentWorkflow { steps };
    }
    if let Some(v) = exec.strip_prefix("system-action:") {
        return CommandV1::SystemAction {
            action: v.to_string(),
        };
    }
    if exec == "open-settings" {
        return CommandV1::OpenSettings;
    }
    if exec == "open-store" {
        return CommandV1::OpenStore;
    }
    if let Some(v) = exec.strip_prefix("open-window:") {
        return CommandV1::OpenFeatureWindow {
            window: v.to_string(),
        };
    }
    if let Some(v) = exec.strip_prefix("copy:") {
        return CommandV1::CopyText {
            value: v.to_string(),
        };
    }
    if let Some(v) = exec.strip_prefix("copy-path:") {
        return CommandV1::CopyPath {
            value: v.to_string(),
        };
    }
    if let Some(v) = exec.strip_prefix("reveal:") {
        return CommandV1::RevealPath {
            path: v.to_string(),
        };
    }
    if let Some(v) = exec.strip_prefix("open-with:") {
        return CommandV1::OpenWithEditor {
            path: v.to_string(),
        };
    }
    if let Some(v) = exec.strip_prefix("focus:") {
        return CommandV1::FocusWindow { id: v.to_string() };
    }
    if let Some(v) = exec.strip_prefix("close-window:") {
        return CommandV1::CloseWindow { id: v.to_string() };
    }
    if let Some(v) = exec.strip_prefix("minimize-window:") {
        return CommandV1::MinimizeWindow { id: v.to_string() };
    }
    if let Some(v) = exec.strip_prefix("move-window-current:") {
        return CommandV1::MoveWindowCurrentWorkspace { id: v.to_string() };
    }
    if let Some(v) = exec.strip_prefix("fill:") {
        return CommandV1::QueryFill {
            value: v.to_string(),
        };
    }
    if let Some(v) = exec.strip_prefix("profile-switch:") {
        return CommandV1::ProfileSwitch { id: v.to_string() };
    }
    if let Some(v) = exec.strip_prefix("macro-template:") {
        return CommandV1::MacroTemplateOpen {
            template_id: v.to_string(),
        };
    }
    if let Some(v) = exec.strip_prefix("macro:") {
        return CommandV1::MacroOpen { id: v.to_string() };
    }
    if let Some(v) = exec.strip_prefix("ext-view:") {
        let mut parts = v.splitn(2, ':');
        let ext_id = parts.next().unwrap_or_default().to_string();
        let command = parts.next().unwrap_or_default().to_string();
        return CommandV1::ExtensionView { ext_id, command };
    }
    if let Some(v) = exec.strip_prefix("ext-no-view:") {
        let mut parts = v.splitn(2, ':');
        let ext_id = parts.next().unwrap_or_default().to_string();
        let command = parts.next().unwrap_or_default().to_string();
        return CommandV1::ExtensionAction { ext_id, command };
    }

    if matches!(source, matcher::ResultSource::File) {
        return CommandV1::OpenFile {
            path: exec.to_string(),
        };
    }

    if matches!(source, matcher::ResultSource::Application) {
        return CommandV1::LaunchApp {
            exec: exec.to_string(),
        };
    }

    CommandV1::Unknown {
        exec: exec.to_string(),
    }
}

fn to_v3_result(src: matcher::SearchResult) -> SearchResultV3 {
    let command = exec_to_command(&src.exec, &src.source);
    let actions = src.actions.as_ref().map(|items| {
        items
            .iter()
            .map(|a| ResultActionV3 {
                label: a.label.clone(),
                shortcut: a.shortcut.clone(),
                command: exec_to_command(&a.exec, &src.source),
            })
            .collect::<Vec<_>>()
    });

    SearchResultV3 {
        title: src.title,
        subtitle: src.subtitle,
        icon: src.icon,
        score: src.score,
        match_indices: src.match_indices,
        source: src.source,
        id: src.id,
        group: src.group,
        section: src.section,
        version: 1,
        command,
        actions,
        exec: src.exec,
    }
}

fn build_profile_results(
    query: &str,
    profiles: &ProfilesConfig,
    weight: u32,
) -> Vec<SearchResult> {
    let trimmed = query.trim();
    let lower = trimmed.to_lowercase();

    profiles
        .entries
        .iter()
        .enumerate()
        .filter_map(|(idx, profile)| {
            let title = format!("Profile: {}", profile.name);
            let subtitle = if profile.id == profiles.active_profile_id {
                Some(format!("{} (active)", profile.id))
            } else {
                Some(profile.id.clone())
            };

            let include = if trimmed.is_empty() {
                true
            } else {
                title.to_lowercase().contains(&lower)
                    || profile.id.to_lowercase().contains(&lower)
                    || "profile".contains(&lower)
            };

            if !include {
                return None;
            }

            let (base, indices) = if let Some((raw, idxs)) = matcher::fuzzy_score_text(query, &title) {
                (ranking_config::PROFILE_FUZZY_BASE.saturating_add(raw.saturating_mul(ranking_config::PROFILE_FUZZY_MULTIPLIER)), idxs)
            } else {
                (ranking_config::PROFILE_FALLBACK_BASE.saturating_sub(idx as u32), Vec::new())
            };

            Some(SearchResult {
                title,
                subtitle,
                icon: Some("fa-solid fa-user-gear".to_string()),
                exec: format!("profile-switch:{}", profile.id),
                score: weighted_score(base, weight),
                match_indices: indices,
                source: ResultSource::Application,
                actions: None,
                id: Some(format!("profile:{}", profile.id)),
                group: None,
                section: Some("Profiles".to_string()),
            })
        })
        .collect()
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

fn build_window_results<F>(
    query: &str,
    query_lower: &str,
    window_cap: usize,
    apps: &[AppEntry],
    weight: u32,
    provider: F,
) -> Vec<SearchResult>
where
    F: Fn(usize) -> Vec<WindowGroup>,
{
    fn backend_label(backend: &str) -> &str {
        match backend {
            "hyprland" => "Hyprland",
            "sway" => "Sway",
            "x11" => "X11",
            _ => "Unknown",
        }
    }

    fn supports_minimize(backend: &str) -> bool {
        backend == "sway" || backend == "x11"
    }

    fn supports_workspace_move(backend: &str) -> bool {
        backend == "sway"
    }

    let mut window_groups = provider(window_cap);

    if !query_lower.is_empty() {
        for g in &mut window_groups {
            g.entries.retain(|w| {
                let title_match = w.title.to_lowercase().contains(query_lower);
                let class_match = w.class.to_lowercase().contains(query_lower);
                if title_match || class_match {
                    return true;
                }

                if let Some(app) = apps.iter().find(|app| {
                    app.startup_wm_class
                        .as_deref()
                        .map(|c| c.eq_ignore_ascii_case(&w.class))
                        .unwrap_or(false)
                        || app
                            .exec
                            .split_whitespace()
                            .next()
                            .map(|cmd| cmd.eq_ignore_ascii_case(&w.class))
                            .unwrap_or(false)
                        || app.name.eq_ignore_ascii_case(&w.class)
                }) {
                    let name_lower = app.name.to_lowercase();
                    let generic_lower = app
                        .generic_name
                        .as_ref()
                        .map(|s| s.to_lowercase())
                        .unwrap_or_default();
                    return name_lower.contains(query_lower)
                        || (!generic_lower.is_empty() && generic_lower.contains(query_lower));
                }

                false
            });
        }
        window_groups.retain(|g| !g.entries.is_empty());
    }

    let mut results = Vec::new();
    let mut count = 0;
    for group in window_groups {
        if count >= window_cap {
            break;
        }
        let app_class = group.app_class.clone();
        let mut entries = group.entries;
        if count + entries.len() > window_cap {
            entries.truncate(window_cap - count);
        }
        count += entries.len();

        for (entry_idx, win) in entries.into_iter().enumerate() {
            let matched_app = apps.iter().find(|app| {
                if let Some(ref wm_class) = app.startup_wm_class {
                    if wm_class.eq_ignore_ascii_case(&win.class) {
                        return true;
                    }
                }
                if let Some(cmd) = app.exec.split_whitespace().next() {
                    if cmd.eq_ignore_ascii_case(&win.class) {
                        return true;
                    }
                }
                app.name.eq_ignore_ascii_case(&win.class)
            });

            let icon = matched_app.and_then(|a| a.icon.clone());

            let mut actions = vec![matcher::ActionHint {
                label: "Close".to_string(),
                exec: format!("close-window:{}", win.address),
                shortcut: Some("Alt+Enter".to_string()),
            }];

            if supports_minimize(&win.backend) {
                actions.push(matcher::ActionHint {
                    label: "Minimize".to_string(),
                    exec: format!("minimize-window:{}", win.address),
                    shortcut: Some("Ctrl+M".to_string()),
                });
            }

            if supports_workspace_move(&win.backend) {
                actions.push(matcher::ActionHint {
                    label: "Move To Current WS".to_string(),
                    exec: format!("move-window-current:{}", win.address),
                    shortcut: Some("Ctrl+Shift+M".to_string()),
                });
            }

            let mut subtitle = format!(
                "Workspace {} • {}",
                win.workspace,
                backend_label(&win.backend)
            );
            if !supports_minimize(&win.backend) && !supports_workspace_move(&win.backend) {
                subtitle.push_str(" • Limited actions");
            }

            let mut base_score = ranking_config::WINDOW_NO_QUERY_BASE;
            let mut match_indices = Vec::new();
            if !query.trim().is_empty() {
                if let Some((raw, idxs)) = matcher::fuzzy_score_text(query, &win.title) {
                    base_score = ranking_config::WINDOW_FUZZY_TITLE_BASE.saturating_add(raw.saturating_mul(ranking_config::WINDOW_FUZZY_TITLE_MULTIPLIER));
                    match_indices = idxs;
                } else if let Some((raw, _)) = matcher::fuzzy_score_text(query, &win.class) {
                    base_score = ranking_config::WINDOW_FUZZY_CLASS_BASE.saturating_add(raw.saturating_mul(ranking_config::WINDOW_FUZZY_CLASS_MULTIPLIER));
                } else {
                    base_score = ranking_config::WINDOW_NO_MATCH_BASE;
                }
            }
            // Earlier entries are usually more recent in grouped providers.
            let recency_bonus = (window_cap.saturating_sub(entry_idx) as u32).saturating_mul(ranking_config::WINDOW_RECENCY_MULTIPLIER).min(ranking_config::WINDOW_RECENCY_CAP);
            base_score = base_score.saturating_add(recency_bonus);

            results.push(SearchResult {
                title: win.title,
                subtitle: Some(subtitle),
                icon,
                exec: format!("focus:{}", win.address),
                score: weighted_score(base_score, weight),
                match_indices,
                source: matcher::ResultSource::Window,
                actions: Some(actions),
                id: Some(win.address.clone()),
                group: Some(app_class.clone()),
                section: Some("Running".to_string()),
            });
        }
    }

    results
}

fn build_extension_results(
    query: &str,
    extensions: &[ExtensionEntry],
    weight: u32,
) -> Vec<SearchResult> {
    if query.is_empty() {
        return Vec::new();
    }

    let query_lower = query.to_lowercase();
    let mut results = Vec::new();

    for ext in extensions {
        if !ext.has_bundle {
            continue;
        }
        for cmd in &ext.manifest.commands {
            let title_lower = cmd.title.to_lowercase();
            let ext_title_lower = ext.manifest.title.to_lowercase();

            let matches = title_lower.contains(&query_lower)
                || ext_title_lower.contains(&query_lower)
                || cmd.name.to_lowercase().contains(&query_lower);

            if !matches {
                if let Some((score, indices)) = matcher::fuzzy_score_text(query, &cmd.title) {
                    let exec_prefix = match cmd.mode {
                        extensions::CommandMode::View => "ext-view",
                        extensions::CommandMode::NoView => "ext-no-view",
                    };

                    results.push(SearchResult {
                        title: cmd.title.clone(),
                        subtitle: Some(ext.manifest.title.clone()),
                        icon: extensions::resolve_ext_icon(
                            cmd.icon.as_deref().or(ext.manifest.icon.as_deref()),
                            &ext.path,
                        ),
                        exec: format!("{}:{}:{}", exec_prefix, ext.manifest.name, cmd.name),
                        score: weighted_score(ranking_config::EXTENSION_FUZZY_BASE.saturating_add(score.saturating_mul(ranking_config::EXTENSION_FUZZY_MULTIPLIER)), weight),
                        match_indices: indices,
                        source: ResultSource::Extension {
                            ext_id: ext.manifest.name.clone(),
                        },
                        actions: None,
                        id: Some(format!("ext:{}:{}", ext.manifest.name, cmd.name)),
                        group: None,
                        section: Some("Extensions".to_string()),
                    });
                }
                continue;
            }

            let exec_prefix = match cmd.mode {
                extensions::CommandMode::View => "ext-view",
                extensions::CommandMode::NoView => "ext-no-view",
            };

            results.push(SearchResult {
                title: cmd.title.clone(),
                subtitle: Some(ext.manifest.title.clone()),
                    icon: extensions::resolve_ext_icon(
                        cmd.icon.as_deref().or(ext.manifest.icon.as_deref()),
                        &ext.path,
                    ),
                exec: format!("{}:{}:{}", exec_prefix, ext.manifest.name, cmd.name),
                score: weighted_score(ranking_config::EXTENSION_EXACT_SCORE, weight),
                match_indices: vec![],
                source: ResultSource::Extension {
                    ext_id: ext.manifest.name.clone(),
                },
                actions: None,
                id: Some(format!("ext:{}:{}", ext.manifest.name, cmd.name)),
                group: None,
                section: Some("Extensions".to_string()),
            });
        }
    }

    results
}

fn build_clipboard_results(query: &str, weight: u32, max_results: usize) -> Vec<SearchResult> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    let Ok(history) = clipboard::get_history() else {
        return Vec::new();
    };

    let lower = trimmed.to_lowercase();
    let mut results = Vec::new();

    for item in history.into_iter().take(ranking_config::CLIPBOARD_SCAN_LIMIT) {
        let content = item.content.replace('\n', " ");
        let title = content.chars().take(100).collect::<String>();
        let matched = title.to_lowercase().contains(&lower);
        let fuzzy = matcher::fuzzy_score_text(trimmed, &title);

        if !matched && fuzzy.is_none() {
            continue;
        }

        let mut score: u32 = if matched { ranking_config::CLIPBOARD_EXACT_BASE } else { ranking_config::CLIPBOARD_FUZZY_BASE };
        let mut indices = Vec::new();
        if let Some((raw, idxs)) = fuzzy {
            score = score.saturating_add(raw.saturating_mul(ranking_config::CLIPBOARD_FUZZY_MULTIPLIER));
            indices = idxs;
        }
        if item.pinned {
            score = score.saturating_add(ranking_config::CLIPBOARD_PINNED_BONUS);
        }

        results.push(SearchResult {
            title,
            subtitle: Some(item.timestamp.to_rfc3339()),
            icon: Some("clipboard".to_string()),
            exec: format!("copy:{}", item.content),
            score: weighted_score(score, weight),
            match_indices: indices,
            source: ResultSource::Clipboard,
            actions: None,
            id: Some(format!("clip:{}", item.id)),
            group: None,
            section: Some("Clipboard".to_string()),
        });

        if results.len() >= max_results {
            break;
        }
    }

    results
}

#[cfg(test)]
mod cli_tests {
    use super::*;

    #[test]
    fn parse_defaults_to_gui() {
        let cli = Cli::parse_from(["vanta"]);
        assert!(!cli.hidden);
        assert!(!cli.clipboard);
    }
}

#[cfg(test)]
mod window_search_tests {
    use super::*;
    use crate::windows::{WindowEntry, WindowGroup};

    fn make_app(name: &str, class: &str, icon: &str) -> AppEntry {
        AppEntry {
            name: name.to_string(),
            generic_name: None,
            comment: None,
            exec: class.to_lowercase(),
            icon: Some(icon.to_string()),
            categories: Vec::new(),
            terminal: false,
            startup_wm_class: Some(class.to_string()),
            desktop_file_path: format!("/usr/share/applications/{}.desktop", name),
        }
    }

    fn make_entry(title: &str, class: &str, addr: &str, workspace: &str, last_active: u64) -> WindowEntry {
        WindowEntry {
            title: title.to_string(),
            class: class.to_string(),
            address: addr.to_string(),
            workspace: workspace.to_string(),
            backend: "sway".to_string(),
            last_active,
        }
    }

    fn make_group(app_class: &str, entries: Vec<WindowEntry>) -> WindowGroup {
        let last_active = entries.iter().map(|w| w.last_active).max().unwrap_or(0);
        WindowGroup {
            app_class: app_class.to_string(),
            entries,
            last_active,
        }
    }

    #[test]
    fn window_results_cap_and_preserve_order() {
        let apps = vec![
            make_app("Alpha", "Alpha", "alpha.png"),
            make_app("Beta", "Beta", "beta.png"),
        ];

        let groups = vec![
            make_group(
                "Beta",
                vec![make_entry("Beta Main", "Beta", "b1", "1", 300)],
            ),
            make_group(
                "Alpha",
                vec![
                    make_entry("Alpha One", "Alpha", "a1", "2", 250),
                    make_entry("Alpha Two", "Alpha", "a2", "2", 200),
                ],
            ),
        ];

        let results = build_window_results("", "", 2, &apps, 100, |_| groups.clone());

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "Beta Main");
        assert_eq!(results[0].exec, "focus:b1");
        assert_eq!(results[0].group.as_deref(), Some("Beta"));
        assert!(results[0].score >= weighted_score(650, 100));
        assert_eq!(results[1].title, "Alpha One");
        assert_eq!(results[1].group.as_deref(), Some("Alpha"));
    }

    #[test]
    fn window_results_filter_by_query_and_keep_actions() {
        let apps = vec![make_app("Alpha", "Alpha", "alpha.png")];
        let groups = vec![
            make_group(
                "Alpha",
                vec![
                    make_entry("Terminal", "Alpha", "a1", "1", 30),
                    make_entry("Notes", "Alpha", "a2", "1", 20),
                ],
            ),
            make_group(
                "Other",
                vec![make_entry("Browser", "Other", "o1", "2", 10)],
            ),
        ];

        let results = build_window_results("term", "term", 5, &apps, 100, |_| groups.clone());

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Terminal");
        assert_eq!(results[0].subtitle.as_deref(), Some("Workspace 1 • Sway"));
        assert_eq!(results[0].group.as_deref(), Some("Alpha"));

        let actions = results[0]
            .actions
            .as_ref()
            .expect("window result should expose actions");
        assert_eq!(actions[0].label, "Close");
        assert_eq!(actions[0].exec, "close-window:a1");
        assert_eq!(actions[1].label, "Minimize");
        assert_eq!(actions[2].label, "Move To Current WS");
    }
}

#[cfg(test)]
mod intent_engine_tests {
    use super::*;

    fn app(name: &str, exec: &str) -> AppEntry {
        AppEntry {
            name: name.to_string(),
            generic_name: None,
            comment: None,
            exec: exec.to_string(),
            icon: None,
            categories: Vec::new(),
            terminal: false,
            startup_wm_class: None,
            desktop_file_path: String::new(),
        }
    }

    #[test]
    fn split_intent_steps_supports_then_and_arrow() {
        let steps = split_intent_steps("open settings then open store -> restart");
        assert_eq!(steps, vec!["open settings", "open store", "restart"]);
    }

    #[test]
    fn infer_intent_step_exec_maps_supported_actions() {
        let apps = vec![app("Zen Browser", "zen-browser")];
        assert_eq!(infer_intent_step_exec("open settings", &apps).as_deref(), Some("open-settings"));
        assert_eq!(infer_intent_step_exec("go to store", &apps).as_deref(), Some("open-store"));
        assert_eq!(
            infer_intent_step_exec("restart into bios", &apps).as_deref(),
            Some("system-action:bios")
        );
        assert_eq!(infer_intent_step_exec("open zen", &apps).as_deref(), Some("zen-browser"));
    }

    #[test]
    fn build_intent_results_requires_multiple_valid_steps() {
        let apps = vec![app("Zen Browser", "zen-browser"), app("Foot", "foot")];

        let none = build_intent_results("open settings", 100, &apps);
        assert!(none.is_empty());

        let results = build_intent_results("open zen then open foot", 100, &apps);
        assert_eq!(results.len(), 1);
        assert!(results[0].exec.starts_with("intent-workflow:"));
        assert!(results[0].exec.contains("zen-browser"));
        assert!(results[0].exec.contains("foot"));
        assert_eq!(results[0].section.as_deref(), Some("Intent"));
    }
}

#[cfg(test)]
mod command_palette_tests {
    use super::*;

    #[test]
    fn command_palette_filters_by_query() {
        let settings = build_command_palette_results("settings", 100);
        assert_eq!(settings.len(), 1);
        assert_eq!(settings[0].exec, "open-settings");

        let all = build_command_palette_results("", 100);
        assert_eq!(all.len(), 6);
        assert!(all
            .iter()
            .all(|r| r.id.as_deref().unwrap_or_default().starts_with("cmd:")));
    }

    #[test]
    fn command_template_palette_includes_saved_template() {
        let templates = vec![CommandTemplate {
            id: "tmpl-a".to_string(),
            name: "Deploy Main".to_string(),
            macro_id: "deploy".to_string(),
            args: HashMap::from([("branch".to_string(), "main".to_string())]),
            created_at_ms: 1,
            updated_at_ms: 2,
        }];
        let macros = vec![WorkflowMacro {
            id: "deploy".to_string(),
            name: "Deploy Workflow".to_string(),
            description: Some("Deploy application".to_string()),
            enabled: true,
            args: vec![],
            steps: vec![],
            timeout_ms: None,
            timeout_behavior: config::TimeoutBehavior::Abort,
            schedule: None,
        }];

        let results = build_command_template_palette_results("deploy", &templates, &macros, 100);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].exec, "macro-template:tmpl-a");
        assert_eq!(results[0].section.as_deref(), Some("Templates"));
    }

    #[test]
    fn exec_to_command_maps_open_window_contract() {
        let command = exec_to_command("open-window:featureHub", &ResultSource::Application);
        match command {
            CommandV1::OpenFeatureWindow { window } => assert_eq!(window, "featureHub"),
            other => panic!("unexpected command mapping: {:?}", other),
        }
    }

    #[test]
    fn exec_to_command_maps_macro_template_contract() {
        let command = exec_to_command("macro-template:tmpl-123", &ResultSource::Application);
        match command {
            CommandV1::MacroTemplateOpen { template_id } => assert_eq!(template_id, "tmpl-123"),
            other => panic!("unexpected command mapping: {:?}", other),
        }
    }
}

#[cfg(test)]
mod quick_note_tests {
    use super::*;

    #[test]
    fn quick_note_results_include_save_for_non_empty_query() {
        let notes = config::NotesConfig {
            entries: vec![config::QuickNote {
                id: "1".to_string(),
                text: "buy groceries".to_string(),
                created_at_ms: 1,
            }],
            max_entries: 200,
        };

        let results = build_quick_note_results("buy", &notes, 100, 8);
        assert_eq!(results[0].id.as_deref(), Some("note:save"));
        assert_eq!(results[0].exec, "note-save:buy");
        assert!(results.iter().any(|r| r.id.as_deref() == Some("note:1")));
    }

    #[test]
    fn quick_note_results_show_recent_notes_for_empty_query() {
        let notes = config::NotesConfig {
            entries: vec![
                config::QuickNote {
                    id: "a".to_string(),
                    text: "older".to_string(),
                    created_at_ms: 1,
                },
                config::QuickNote {
                    id: "b".to_string(),
                    text: "newer".to_string(),
                    created_at_ms: 2,
                },
            ],
            max_entries: 200,
        };

        let results = build_quick_note_results("", &notes, 100, 8);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "newer");
        assert_eq!(results[0].exec, "copy:newer");
    }
}

#[cfg(test)]
mod bookmark_tests {
    use super::*;

    #[test]
    fn bookmark_results_filter_and_add_remove_action() {
        let bookmarks = config::BookmarksConfig {
            entries: vec![
                config::FileBookmark {
                    path: "/tmp/report.md".to_string(),
                    created_at_ms: 1,
                },
                config::FileBookmark {
                    path: "/home/user/project".to_string(),
                    created_at_ms: 2,
                },
            ],
            max_entries: 200,
        };

        let results = build_bookmark_results("report", &bookmarks, 100, 8);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "report.md");
        let actions = results[0].actions.as_ref().expect("bookmark actions");
        assert_eq!(actions[0].exec, "bookmark-remove:/tmp/report.md");
    }

    #[test]
    fn bookmark_results_return_most_recent_first() {
        let bookmarks = config::BookmarksConfig {
            entries: vec![
                config::FileBookmark {
                    path: "/tmp/old.txt".to_string(),
                    created_at_ms: 1,
                },
                config::FileBookmark {
                    path: "/tmp/new.txt".to_string(),
                    created_at_ms: 2,
                },
            ],
            max_entries: 200,
        };

        let results = build_bookmark_results("", &bookmarks, 100, 8);
        assert_eq!(results[0].exec, "/tmp/new.txt");
        assert_eq!(results[1].exec, "/tmp/old.txt");
    }
}

#[cfg(test)]
mod relevance_ranking_tests {
    use super::*;

    fn result(title: &str, exec: &str, source: ResultSource, score: u32) -> SearchResult {
        SearchResult {
            title: title.to_string(),
            subtitle: None,
            icon: None,
            exec: exec.to_string(),
            score,
            match_indices: Vec::new(),
            source,
            actions: None,
            id: None,
            group: None,
            section: None,
        }
    }

    #[test]
    fn relevance_bonus_prefers_exact_app_name() {
        let q = "discord";
        let mut app = result("Discord", "discord", ResultSource::Application, 1_000);
        let mut file = result("discord-notes.txt", "/tmp/discord-notes.txt", ResultSource::File, 1_000);

        app.score = app
            .score
            .saturating_add(query_relevance_bonus(q, &app))
            .saturating_add(source_intent_bonus(q, &app));
        file.score = file
            .score
            .saturating_add(query_relevance_bonus(q, &file))
            .saturating_add(source_intent_bonus(q, &file));

        assert!(app.score > file.score);
    }

    #[test]
    fn open_query_prefers_matching_app_over_file_path() {
        let q = "open discord";
        let mut app = result("Discord", "discord", ResultSource::Application, 1_000);
        let mut file = result("discord-notes.txt", "/tmp/discord-notes.txt", ResultSource::File, 1_000);

        app.score = app
            .score
            .saturating_add(query_relevance_bonus(q, &app))
            .saturating_add(source_intent_bonus(q, &app))
            .saturating_add(app_entity_bonus(q, &app));
        file.score = file
            .score
            .saturating_add(query_relevance_bonus(q, &file))
            .saturating_add(source_intent_bonus(q, &file))
            .saturating_add(app_entity_bonus(q, &file));

        assert!(app.score > file.score);
    }

    #[test]
    fn file_can_outrank_app_when_file_is_more_relevant() {
        let q = "invoice pdf";
        let mut app = result("Discord", "discord", ResultSource::Application, 900);
        let mut file = result("invoice_2026.pdf", "/home/user/docs/invoice_2026.pdf", ResultSource::File, 1_100);

        app.score = app
            .score
            .saturating_add(query_relevance_bonus(q, &app))
            .saturating_add(source_intent_bonus(q, &app))
            .saturating_add(app_entity_bonus(q, &app));
        file.score = file
            .score
            .saturating_add(query_relevance_bonus(q, &file))
            .saturating_add(source_intent_bonus(q, &file))
            .saturating_add(app_entity_bonus(q, &file));

        assert!(file.score > app.score);
    }
}

#[tauri::command]
async fn get_config(
    state: tauri::State<'_, AppState>,
) -> Result<VantaConfig, VantaError> {
    let config = state
        .config
        .read()
        .map_err(|_| "Failed to access config state".to_string())?;
    Ok(config.clone())
}

#[tauri::command]
async fn save_config(
    mut new_config: VantaConfig,
    state: tauri::State<'_, AppState>,
) -> Result<(), VantaError> {
    let new_files_config = new_config.files.clone();

    let _ = clamp_window_size(&mut new_config.window);
    let _ = clamp_accessibility(&mut new_config.accessibility);

    {
        let mut config = state
            .config
            .write()
            .map_err(|_| "Failed to access config state".to_string())?;
        *config = new_config;
        config.save_with_source("user")?;
    }

    let index_clone = state.file_index.clone();
    std::thread::spawn(move || {
        let state = files::build_index(&new_files_config);
        if let Ok(mut guard) = index_clone.lock() {
            *guard = state;
        }
    });

    Ok(())
}

#[tauri::command]
async fn get_profiles(
    state: tauri::State<'_, AppState>,
) -> Result<ProfilesConfig, VantaError> {
    let config = state
        .config
        .read()
        .map_err(|_| "Failed to access config state".to_string())?;
    Ok(config::list_profiles(&config))
}

#[tauri::command]
async fn switch_profile(
    profile_id: String,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<VantaConfig, VantaError> {
    let updated = {
        let mut config = state
            .config
            .write()
            .map_err(|_| "Failed to access config state".to_string())?;
        config::switch_profile_in_config(&mut config, &profile_id)?;
        config.save_with_source("user")?;
        config.clone()
    };

    let _ = app_handle.emit("config-updated", &updated);
    Ok(updated)
}

#[tauri::command]
async fn export_profile(
    profile_id: String,
    output_path: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), VantaError> {
    let config = state
        .config
        .read()
        .map_err(|_| "Failed to access config state".to_string())?;
    config::export_profile_to_path(&config, &profile_id, &output_path)
}

#[tauri::command]
async fn import_profile(
    input_path: String,
    replace_existing: bool,
    activate: bool,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<ProfileConfig, VantaError> {
    let (imported, updated_cfg) = {
        let mut config = state
            .config
            .write()
            .map_err(|_| "Failed to access config state".to_string())?;
        let imported = config::import_profile_from_path(&mut config, &input_path, replace_existing)?;
        if activate {
            config::switch_profile_in_config(&mut config, &imported.id)?;
        }
        config.save_with_source("user")?;
        (imported, config.clone())
    };

    let _ = app_handle.emit("config-updated", &updated_cfg);
    Ok(imported)
}

#[tauri::command]
async fn rebuild_file_index(
    state: tauri::State<'_, AppState>,
) -> Result<Option<u64>, VantaError> {
    let files_config = {
        let cfg = state
            .config
            .read()
            .map_err(|_| "Failed to access config state".to_string())?;
        cfg.files.clone()
    };

    let index_state = files::build_index(&files_config);

    {
        let mut guard = state
            .file_index
            .lock()
            .map_err(|_| "Failed to update file index".to_string())?;
        *guard = index_state.clone();
    }

    {
        let mut cfg = state
            .config
            .write()
            .map_err(|_| "Failed to update config freshness".to_string())?;
        cfg.files.indexed_at = index_state.indexed_at;
        cfg.save_with_source("indexer")?;
    }

    Ok(index_state.indexed_at)
}

#[tauri::command]
async fn get_config_audit(limit: Option<usize>) -> Result<Vec<config::ConfigAuditEntry>, VantaError> {
    Ok(config::read_config_audit(limit.unwrap_or(200)))
}

#[tauri::command]
async fn validate_config_schema() -> Result<config::ConfigSchemaValidationReport, VantaError> {
    Ok(config::validate_config_file_against_schema())
}

#[tauri::command]
async fn factory_reset_config(
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<VantaConfig, VantaError> {
    let cfg = config::factory_reset_on_disk()?;

    {
        let mut current = state
            .config
            .write()
            .map_err(|_| "Failed to access config state".to_string())?;
        *current = cfg.clone();
    }

    let index_clone = state.file_index.clone();
    let files_cfg = cfg.files.clone();
    std::thread::spawn(move || {
        let index_state = files::build_index(&files_cfg);
        if let Ok(mut guard) = index_clone.lock() {
            *guard = index_state;
        }
    });

    let _ = app_handle.emit("config-updated", &cfg);
    Ok(cfg)
}

/// Monotonically increasing search generation counter for cancellation.
static SEARCH_GENERATION: AtomicU64 = AtomicU64::new(0);

#[tauri::command]
async fn search(
    query: String,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<Vec<SearchResult>, VantaError> {
    let search_start = Instant::now();
    let generation = SEARCH_GENERATION.fetch_add(1, Ordering::SeqCst) + 1;

    if let Some(command_query) = query.trim().strip_prefix('>') {
        let (templates, macros) = {
            let config = state
                .config
                .read()
                .map_err(|_| "Failed to access config".to_string())?;
            (
                config.workflows.command_templates.clone(),
                config.workflows.macros.clone(),
            )
        };
        let mut command_results = build_command_palette_results(command_query, 100);
        command_results.extend(build_command_template_palette_results(
            command_query,
            &templates,
            &macros,
            100,
        ));
        command_results.sort_by(|a, b| b.score.cmp(&a.score));
        record_latency(
            "search",
            search_start.elapsed(),
            &SEARCH_CALLS,
            &SEARCH_TOTAL_MS,
            &SEARCH_MAX_MS,
        );
        return Ok(command_results);
    }

    if let Some(note_query) = query.trim().strip_prefix("note:") {
        let config = state
            .config
            .read()
            .map_err(|_| "Failed to access config".to_string())?;
        let max_results = config.general.max_results;
        let notes = config.notes.clone();
        drop(config);

        let note_results = build_quick_note_results(note_query, &notes, 100, max_results);
        record_latency(
            "search",
            search_start.elapsed(),
            &SEARCH_CALLS,
            &SEARCH_TOTAL_MS,
            &SEARCH_MAX_MS,
        );
        return Ok(note_results);
    }

    let q_trimmed = query.trim();
    if let Some(bookmark_query) = q_trimmed
        .strip_prefix("bookmark:")
        .or_else(|| q_trimmed.strip_prefix("bm:"))
    {
        let config = state
            .config
            .read()
            .map_err(|_| "Failed to access config".to_string())?;
        let max_results = config.general.max_results;
        let bookmarks = config.bookmarks.clone();
        drop(config);

        let bookmark_results = build_bookmark_results(bookmark_query, &bookmarks, 100, max_results);
        record_latency(
            "search",
            search_start.elapsed(),
            &SEARCH_CALLS,
            &SEARCH_TOTAL_MS,
            &SEARCH_MAX_MS,
        );
        return Ok(bookmark_results);
    }

    // ── Snapshot state under locks, release immediately ──────────────
    let apps_snapshot;
    let max_results;
    let search_config;
    let profiles_config;
    let usage_map;
    let ext_snapshot;
    let file_index_snapshot;
    {
        apps_snapshot = state
            .apps
            .lock()
            .map_err(|_| "Failed to access application cache".to_string())?
            .clone();
        let config = state
            .config
            .read()
            .map_err(|_| "Failed to access config".to_string())?;
        max_results = config.general.max_results;
        search_config = config.search.clone();
        profiles_config = config.profiles.clone();
        drop(config);

        usage_map = state
            .history
            .lock()
            .map_err(|_| "Failed to access history".to_string())?
            .usage_map();
        ext_snapshot = state
            .extensions_cache
            .lock()
            .map_err(|_| "Failed to access extensions cache".to_string())?
            .clone();
        file_index_snapshot = state
            .file_index
            .lock()
            .map_err(|_| "Failed to access file index".to_string())?
            .clone();
    }

    // Check cancellation early.
    if SEARCH_GENERATION.load(Ordering::SeqCst) != generation {
        return Ok(Vec::new());
    }

    // ── Parse search filters ─────────────────────────────────────────
    let filter = parse_search_filters(&query);
    let effective_query = if filter.source.is_some() {
        filter.raw_query.clone()
    } else {
        query.clone()
    };
    let effective_lower = effective_query.to_lowercase();
    let has_filter = filter.source.is_some();

    let app_limit = if effective_query.trim().is_empty() {
        apps_snapshot.len()
    } else {
        max_results
    };

    let derived_cap = std::cmp::max(1, max_results / 2);
    let window_cap = if search_config.windows_max_results > 0 {
        search_config.windows_max_results
    } else {
        derived_cap
    };

    // ── Parallel source queries ──────────────────────────────────────
    // Clone references for move into async blocks.
    let q1 = effective_query.clone();
    let q2 = effective_query.clone();
    let q3 = effective_query.clone();
    let q4 = effective_query.clone();
    let q5 = effective_query.clone();
    let ql1 = effective_lower.clone();
    let apps1 = apps_snapshot.clone();
    let apps2 = apps_snapshot.clone();
    let apps3 = apps_snapshot.clone();
    let um1 = usage_map.clone();
    let sc = search_config.clone();
    let pc = profiles_config.clone();

    let app_handle_stream = app_handle.clone();

    let skip_apps = has_filter && !matches!(filter.source, Some(FilterSource::App));
    let skip_windows = has_filter && !matches!(filter.source, Some(FilterSource::Window));
    let skip_files = has_filter && !matches!(filter.source, Some(FilterSource::File));
    let skip_clipboard = has_filter && !matches!(filter.source, Some(FilterSource::Clipboard));
    let skip_misc = has_filter && !matches!(filter.source, Some(FilterSource::App | FilterSource::Extension));

    // Task 1: Fuzzy app search
    let app_task = tokio::task::spawn_blocking({
        let sc = sc.clone();
        move || -> Vec<SearchResult> {
            if skip_apps || !sc.applications.enabled {
                return Vec::new();
            }
            matcher::fuzzy_search(&q1, &apps1, app_limit, &um1, sc.applications.weight)
        }
    });

    // Task 2: Window search
    let window_task = tokio::task::spawn_blocking({
        let sc = sc.clone();
        move || -> Vec<SearchResult> {
            if skip_windows || !sc.windows.enabled {
                return Vec::new();
            }
            build_window_results(&q2, &ql1, window_cap, &apps2, sc.windows.weight, |cap| {
                list_windows_grouped(cap)
            })
        }
    });

    // Task 3: File search
    let file_task = tokio::task::spawn_blocking({
        let sc = sc.clone();
        move || -> Vec<SearchResult> {
            if skip_files || !sc.files.enabled {
                return Vec::new();
            }
            let mut file_results = files::search_index(&file_index_snapshot, &q3, max_results);
            for fr in &mut file_results {
                fr.score = weighted_score(fr.score, sc.files.weight);
            }
            file_results
        }
    });

    // Task 4: Clipboard search
    let clipboard_task = tokio::task::spawn_blocking({
        let sc = sc.clone();
        move || -> Vec<SearchResult> {
            if skip_clipboard || q4.trim().is_empty() {
                return Vec::new();
            }
            build_clipboard_results(&q4, sc.files.weight, max_results / 2)
        }
    });

    // Task 5: Extensions, profiles, intents (lightweight — group together)
    let misc_task = tokio::task::spawn_blocking({
        let sc = sc.clone();
        move || -> Vec<SearchResult> {
            if skip_misc {
                return Vec::new();
            }
            let mut results = Vec::new();
            if sc.applications.enabled {
                results.extend(build_profile_results(&q5, &pc, sc.applications.weight));
                results.extend(build_intent_results(&q5, sc.applications.weight, &apps3));
                results.extend(build_extension_results(&q5, &ext_snapshot, sc.applications.weight));
            }
            results
        }
    });

    // Await all concurrently.
    let (app_res, win_res, file_res, clip_res, misc_res) =
        tokio::join!(app_task, window_task, file_task, clipboard_task, misc_task);

    // Check cancellation after parallel work completes.
    if SEARCH_GENERATION.load(Ordering::SeqCst) != generation {
        return Ok(Vec::new());
    }

    // ── Merge results ────────────────────────────────────────────────
    let mut results: Vec<SearchResult> = Vec::new();

    let app_results = app_res.unwrap_or_default();
    if !app_results.is_empty() {
        let _ = app_handle_stream.emit("search-partial", &app_results.iter().map(|r| to_v3_result(r.clone())).collect::<Vec<_>>());
    }
    results.extend(app_results);

    results.extend(win_res.unwrap_or_default());
    results.extend(file_res.unwrap_or_default());
    results.extend(clip_res.unwrap_or_default());
    results.extend(misc_res.unwrap_or_default());

    // Calculator (fast, synchronous)
    if search_config.calculator.enabled && !has_filter {
        if let Some((display, copy_value)) = math::evaluate_timezone_display(&effective_query) {
            results.push(SearchResult {
                title: format!("= {}", display),
                subtitle: Some("Click to Copy".to_string()),
                icon: Some("calculator".to_string()),
                exec: format!("copy:{}", copy_value),
                score: weighted_score(ranking_config::CALCULATOR_BASE_SCORE, search_config.calculator.weight),
                match_indices: vec![],
                source: matcher::ResultSource::Calculator,
                actions: None,
                id: None,
                group: None,
                section: Some("Calculator".to_string()),
            });
        } else if let Some((display, copy_value)) = math::evaluate_display(&effective_query) {
            results.push(SearchResult {
                title: format!("= {}", display),
                subtitle: Some("Click to Copy".to_string()),
                icon: Some("calculator".to_string()),
                exec: format!("copy:{}", copy_value),
                score: weighted_score(ranking_config::CALCULATOR_BASE_SCORE, search_config.calculator.weight),
                match_indices: vec![],
                source: matcher::ResultSource::Calculator,
                actions: None,
                id: None,
                group: None,
                section: Some("Calculator".to_string()),
            });
        }
    }

    // Store / Settings injection
    if !has_filter {
        let wants_store = effective_lower.contains("store")
            || effective_lower.contains("install")
            || effective_lower.contains("extension")
            || effective_lower.contains("marketplace");

        if wants_store {
            results.push(SearchResult {
                title: "Vanta Store".to_string(),
                subtitle: Some("Browse and install extensions".to_string()),
                icon: Some("fa-solid fa-store".to_string()),
                exec: "open-store".to_string(),
                score: weighted_score(ranking_config::STORE_SEARCH_SCORE, 100),
                match_indices: vec![],
                source: ResultSource::Application,
                actions: None,
                id: Some("vanta-store".to_string()),
                group: None,
                section: Some("Vanta".to_string()),
            });
        }

        let wants_settings = effective_lower.contains("setting")
            || effective_lower.contains("preferences")
            || effective_lower.contains("config")
            || effective_lower.contains("option");

        if wants_settings && search_config.applications.enabled {
            if let Some((raw_score, indices)) = matcher::fuzzy_score_text(&effective_query, "Open Vanta Settings") {
                let base = ranking_config::SETTINGS_BASE_SCORE.saturating_add(raw_score.saturating_mul(ranking_config::SETTINGS_FUZZY_MULTIPLIER));
                results.push(SearchResult {
                    title: "Settings".to_string(),
                    subtitle: Some("Open Vanta settings".to_string()),
                    icon: Some("fa-solid fa-gear".to_string()),
                    exec: "open-settings".to_string(),
                    score: weighted_score(base, search_config.applications.weight),
                    match_indices: indices,
                    source: matcher::ResultSource::Application,
                    actions: None,
                    id: Some("settings".to_string()),
                    group: None,
                    section: Some("Settings".to_string()),
                });
            }
        }
    }

    // ── Apply source filter on merged results ────────────────────────
    if has_filter {
        results.retain(|r| filter_matches_source(&filter, &r.source));
    }

    // ── Bonus scoring & negative scoring ─────────────────────────────
    if !effective_query.trim().is_empty() {
        for result in &mut results {
            let bonus = query_relevance_bonus(&effective_query, result)
                .saturating_add(source_intent_bonus(&effective_query, result))
                .saturating_add(app_entity_bonus(&effective_query, result));
            result.score = result.score.saturating_add(bonus);

            // Negative scoring: penalise short fuzzy matches (likely noise).
            if result.match_indices.len() <= ranking_config::SHORT_MATCH_THRESHOLD
                && result.score > 0
                && !result.match_indices.is_empty()
            {
                result.score = result.score.saturating_sub(ranking_config::SHORT_MATCH_PENALTY);
            }
        }

        // Suppress below-threshold results.
        results.retain(|r| r.score >= ranking_config::NEGATIVE_SCORE_THRESHOLD);
    }

    // ── Typo suggestions when results are sparse ─────────────────────
    if !effective_query.trim().is_empty() && results.len() <= 2 {
        let suggestions = typo_suggestions(&effective_query, &apps_snapshot, 3);
        for suggestion in suggestions {
            results.push(SearchResult {
                title: format!("Did you mean: {}?", suggestion),
                subtitle: Some("Typo correction".to_string()),
                icon: Some("fa-solid fa-spell-check".to_string()),
                exec: format!("fill:{}", suggestion),
                score: ranking_config::TYPO_SUGGESTION_SCORE,
                match_indices: vec![],
                source: ResultSource::Application,
                actions: None,
                id: Some(format!("typo:{}", suggestion)),
                group: None,
                section: Some("Suggestions".to_string()),
            });
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
async fn search_v3(
    query: String,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<Vec<SearchResultV3>, VantaError> {
    let legacy = search(query, state, app_handle).await?;
    Ok(legacy.into_iter().map(to_v3_result).collect())
}

#[tauri::command]
async fn save_query_history(
    query: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), VantaError> {
    let mut history = state
        .history
        .lock()
        .map_err(|_| "Failed to access history".to_string())?;
    history.push_query(&query);
    Ok(())
}

#[tauri::command]
async fn get_query_history(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<String>, VantaError> {
    let history = state
        .history
        .lock()
        .map_err(|_| "Failed to access history".to_string())?;
    Ok(history.get_recent_queries().to_vec())
}

#[tauri::command]
async fn launch_app(
    exec: String,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), VantaError> {
    let launch_start = Instant::now();

    if let Some(note_text) = exec.strip_prefix("note-save:") {
        let trimmed = note_text.trim();
        if trimmed.is_empty() {
            return Err("Note cannot be empty".into());
        }

        let mut cfg = state
            .config
            .write()
            .map_err(|_| "Failed to access config".to_string())?;

        let note = config::QuickNote {
            id: format!("{}", now_millis()),
            text: trimmed.to_string(),
            created_at_ms: now_millis().max(0) as u64,
        };
        cfg.notes.entries.push(note);

        let max_entries = cfg.notes.max_entries.max(1);
        if cfg.notes.entries.len() > max_entries {
            let overflow = cfg.notes.entries.len() - max_entries;
            cfg.notes.entries.drain(0..overflow);
        }

        cfg.save_with_source("quick-note")?;
        drop(cfg);

        let updated = state
            .config
            .read()
            .map_err(|_| "Failed to access config".to_string())?
            .clone();
        let _ = app_handle.emit("config-updated", &updated);

        record_latency(
            "launch",
            launch_start.elapsed(),
            &LAUNCH_CALLS,
            &LAUNCH_TOTAL_MS,
            &LAUNCH_MAX_MS,
        );
        return Ok(());
    }

    if let Some(path) = exec.strip_prefix("bookmark-add:") {
        let trimmed = path.trim();
        if trimmed.is_empty() {
            return Err("Bookmark path cannot be empty".into());
        }

        let mut cfg = state
            .config
            .write()
            .map_err(|_| "Failed to access config".to_string())?;
        if !cfg.bookmarks.entries.iter().any(|b| b.path == trimmed) {
            cfg.bookmarks.entries.push(config::FileBookmark {
                path: trimmed.to_string(),
                created_at_ms: now_millis().max(0) as u64,
            });

            let max_entries = cfg.bookmarks.max_entries.max(1);
            if cfg.bookmarks.entries.len() > max_entries {
                let overflow = cfg.bookmarks.entries.len() - max_entries;
                cfg.bookmarks.entries.drain(0..overflow);
            }
            cfg.save_with_source("bookmark")?;
        }
        drop(cfg);

        let updated = state
            .config
            .read()
            .map_err(|_| "Failed to access config".to_string())?
            .clone();
        let _ = app_handle.emit("config-updated", &updated);
        return Ok(());
    }

    if let Some(path) = exec.strip_prefix("bookmark-remove:") {
        let trimmed = path.trim();
        if trimmed.is_empty() {
            return Err("Bookmark path cannot be empty".into());
        }

        let mut cfg = state
            .config
            .write()
            .map_err(|_| "Failed to access config".to_string())?;
        let before = cfg.bookmarks.entries.len();
        cfg.bookmarks.entries.retain(|b| b.path != trimmed);
        if cfg.bookmarks.entries.len() != before {
            cfg.save_with_source("bookmark")?;
        }
        drop(cfg);

        let updated = state
            .config
            .read()
            .map_err(|_| "Failed to access config".to_string())?
            .clone();
        let _ = app_handle.emit("config-updated", &updated);
        return Ok(());
    }

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
    Ok(result?)
}

#[tauri::command]
async fn system_action(action: String) -> Result<(), VantaError> {
    Ok(launcher::system_action(&action)?)
}

#[tauri::command]
async fn get_suggestions(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<SearchResult>, VantaError> {
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
        .read()
        .map_err(|_| "Failed to access config".to_string())?;

    let search_config = config.search.clone();
    let max_results = config.general.max_results;
    let profiles_config = config.profiles.clone();
    let bookmarks_config = config.bookmarks.clone();

    let file_index = state
        .file_index
        .lock()
        .map_err(|_| "Failed to access file index".to_string())?;

    let mut results: Vec<SearchResult> = Vec::new();

    let derived_cap = std::cmp::max(3, max_results / 2);
    let window_cap = if search_config.windows_max_results > 0 {
        search_config.windows_max_results
    } else {
        derived_cap
    };

    let window_results = build_window_results(
        "",
        "",
        window_cap,
        &apps,
        search_config.windows.weight,
        |cap| list_windows_grouped(cap),
    );
    results.extend(window_results);

    let bookmark_results = build_bookmark_results(
        "",
        &bookmarks_config,
        search_config.files.weight,
        std::cmp::max(3, max_results / 2),
    );
    results.extend(bookmark_results);

    let mut scored_apps: Vec<(&AppEntry, u32)> = apps
        .iter()
        .map(|app| (app, history.get_usage(&app.exec)))
        .collect();

    scored_apps.sort_by(|a, b| b.1.cmp(&a.1));

    let app_results: Vec<SearchResult> = scored_apps
        .into_iter()
        .map(|(app, _count)| SearchResult {
            title: app.name.clone(),
            subtitle: app.generic_name.clone().or_else(|| app.comment.clone()),
            icon: app.icon.clone(),
            exec: app.exec.clone(),
            score: weighted_score(ranking_config::APP_SUGGESTION_WEIGHT, search_config.applications.weight),
            match_indices: vec![],
            source: ResultSource::Application,
            actions: None,
            id: None,
            group: None,
            section: Some("Apps".to_string()),
        })
        .collect();

    results.extend(app_results);

    let mut doc_results = files::search_index(&file_index, "", ranking_config::SUGGESTION_DOC_LIMIT);
    doc_results.sort_by(|a, b| {
        let a_dir = a.icon.as_deref() == Some("dir");
        let b_dir = b.icon.as_deref() == Some("dir");
        b_dir.cmp(&a_dir)
    });
    for mut doc in doc_results {
        doc.section = Some("Documents".to_string());
        results.push(doc);
    }

    if let Ok(ext_cache) = state.extensions_cache.lock() {
        for ext in ext_cache.iter() {
            if !ext.has_bundle {
                continue;
            }
            for cmd in &ext.manifest.commands {
                let exec_prefix = match cmd.mode {
                    extensions::CommandMode::View => "ext-view",
                    extensions::CommandMode::NoView => "ext-no-view",
                };
                results.push(SearchResult {
                    title: cmd.title.clone(),
                    subtitle: Some(ext.manifest.title.clone()),
                    icon: extensions::resolve_ext_icon(
                        cmd.icon.as_deref().or(ext.manifest.icon.as_deref()),
                        &ext.path,
                    ),
                    exec: format!("{}:{}:{}", exec_prefix, ext.manifest.name, cmd.name),
                    score: weighted_score(ranking_config::EXTENSION_SUGGESTION_SCORE, search_config.applications.weight),
                    match_indices: vec![],
                    source: ResultSource::Extension {
                        ext_id: ext.manifest.name.clone(),
                    },
                    actions: None,
                    id: Some(format!("ext:{}:{}", ext.manifest.name, cmd.name)),
                    group: None,
                    section: Some("Extensions".to_string()),
                });
            }
        }
    }

    let profile_results = build_profile_results("", &profiles_config, search_config.applications.weight);
    results.extend(profile_results);

    results.push(SearchResult {
        title: "Vanta Store".to_string(),
        subtitle: Some("Browse and install extensions".to_string()),
        icon: Some("fa-solid fa-store".to_string()),
        exec: "open-store".to_string(),
        score: ranking_config::STORE_SUGGESTION_SCORE,
        match_indices: vec![],
        source: ResultSource::Application,
        actions: None,
        id: Some("vanta-store".to_string()),
        group: None,
        section: Some("Commands".to_string()),
    });

    results.push(SearchResult {
        title: "Settings".to_string(),
        subtitle: Some("Open Vanta settings".to_string()),
        icon: Some("fa-solid fa-gear".to_string()),
        exec: "open-settings".to_string(),
        score: ranking_config::SETTINGS_SUGGESTION_SCORE,
        match_indices: vec![],
        source: ResultSource::Application,
        actions: None,
        id: Some("settings".to_string()),
        group: None,
        section: Some("Settings".to_string()),
    });

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
async fn get_suggestions_v3(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<SearchResultV3>, VantaError> {
    let legacy = get_suggestions(state).await?;
    Ok(legacy.into_iter().map(to_v3_result).collect())
}

#[tauri::command]
async fn run_contract_migration(
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<ContractMigrationReport, VantaError> {
    let config_report = config::migrate_config_on_disk()?;
    let extension_report = extensions::migrate_extension_manifests();

    let refreshed = config::load_or_create_default();
    {
        let mut cfg = state
            .config
            .write()
            .map_err(|_| "Failed to access config state".to_string())?;
        *cfg = refreshed.clone();
    }

    let refreshed_exts = extensions::scan_extensions();
    {
        let mut exts = state
            .extensions_cache
            .lock()
            .map_err(|_| "Failed to access extensions cache".to_string())?;
        *exts = refreshed_exts.clone();
    }

    let _ = app_handle.emit("config-updated", &refreshed);
    let _ = app_handle.emit("extensions-changed", &refreshed_exts);

    Ok(ContractMigrationReport {
        config: config_report,
        extensions: extension_report,
    })
}

#[tauri::command]
async fn get_search_diagnostics() -> Result<SearchDiagnostics, VantaError> {
    Ok(SearchDiagnostics {
        search: snapshot_perf(&SEARCH_CALLS, &SEARCH_TOTAL_MS, &SEARCH_MAX_MS),
        suggestions: snapshot_perf(&SUGGEST_CALLS, &SUGGEST_TOTAL_MS, &SUGGEST_MAX_MS),
        launch: snapshot_perf(&LAUNCH_CALLS, &LAUNCH_TOTAL_MS, &LAUNCH_MAX_MS),
    })
}

#[tauri::command]
async fn get_health_dashboard(
    state: tauri::State<'_, AppState>,
) -> Result<HealthDashboard, VantaError> {
    let cfg = state
        .config
        .read()
        .map_err(|_| "Failed to access config".to_string())?
        .clone();
    let apps_cached = state
        .apps
        .lock()
        .map_err(|_| "Failed to access apps cache".to_string())?
        .len();
    let extensions_cached = state
        .extensions_cache
        .lock()
        .map_err(|_| "Failed to access extensions cache".to_string())?
        .len();
    let file_index_entries = state
        .file_index
        .lock()
        .map_err(|_| "Failed to access file index".to_string())?
        .entries
        .len();
    let macro_jobs_total = state
        .macro_jobs
        .lock()
        .map_err(|_| "Failed to access macro jobs".to_string())?
        .len();

    let mut checks = Vec::new();
    checks.push(HealthCheck {
        name: "config".to_string(),
        status: "ok".to_string(),
        detail: format!(
            "schema={} active_profile={} profiles={}",
            cfg.schema_version,
            cfg.profiles.active_profile_id,
            cfg.profiles.entries.len()
        ),
    });
    checks.push(HealthCheck {
        name: "apps_cache".to_string(),
        status: if apps_cached > 0 { "ok" } else { "warn" }.to_string(),
        detail: format!("{} apps cached", apps_cached),
    });
    checks.push(HealthCheck {
        name: "extensions_cache".to_string(),
        status: "ok".to_string(),
        detail: format!("{} extensions cached", extensions_cached),
    });
    checks.push(HealthCheck {
        name: "file_index".to_string(),
        status: if file_index_entries > 0 { "ok" } else { "warn" }.to_string(),
        detail: format!("{} indexed entries", file_index_entries),
    });
    checks.push(HealthCheck {
        name: "macro_jobs".to_string(),
        status: "ok".to_string(),
        detail: format!("{} jobs recorded", macro_jobs_total),
    });

    Ok(HealthDashboard {
        generated_at: now_millis(),
        config_schema: cfg.schema_version,
        active_profile_id: cfg.profiles.active_profile_id,
        apps_cached,
        extensions_cached,
        file_index_entries,
        macro_jobs_total,
        checks,
    })
}

#[tauri::command]
async fn get_usage_analytics(
    state: tauri::State<'_, AppState>,
) -> Result<UsageAnalyticsReport, VantaError> {
    let history = state
        .history
        .lock()
        .map_err(|_| "Failed to access history".to_string())?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let mut most_used_apps = history
        .entries
        .iter()
        .filter(|(exec, _)| !is_internal_exec_for_analytics(exec))
        .map(|(exec, entry)| UsageAppStat {
            exec: exec.clone(),
            launches: entry.count,
            frecency: ((entry.frecency(now) * 1000.0) as u64).min(u32::MAX as u64) as u32,
        })
        .collect::<Vec<_>>();

    most_used_apps.sort_by(|a, b| b.launches.cmp(&a.launches).then_with(|| b.frecency.cmp(&a.frecency)));
    most_used_apps.truncate(12);

    let mut bins = [0u32; 24];
    for entry in history.entries.values() {
        for &ts in &entry.timestamps {
            if let Some(dt) = Local.timestamp_opt(ts as i64, 0).single() {
                bins[dt.hour() as usize] = bins[dt.hour() as usize].saturating_add(1);
            }
        }
    }

    let mut peak_usage_hours = bins
        .iter()
        .enumerate()
        .filter(|(_, count)| **count > 0)
        .map(|(hour, count)| UsageHourStat {
            hour: hour as u8,
            launches: *count,
        })
        .collect::<Vec<_>>();
    peak_usage_hours.sort_by(|a, b| b.launches.cmp(&a.launches));
    peak_usage_hours.truncate(6);

    let mut search_patterns = history
        .query_counts
        .iter()
        .map(|(query, count)| UsageQueryStat {
            query: query.clone(),
            count: *count,
        })
        .collect::<Vec<_>>();
    search_patterns.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.query.cmp(&b.query)));
    search_patterns.truncate(12);

    let total_launch_events = history
        .entries
        .values()
        .fold(0u32, |acc, e| acc.saturating_add(e.count));

    Ok(UsageAnalyticsReport {
        generated_at: now_millis(),
        total_launch_events,
        most_used_apps,
        peak_usage_hours,
        search_patterns,
    })
}

#[tauri::command]
async fn create_support_bundle(
    output_path: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<SupportBundleReport, VantaError> {
    let dashboard = get_health_dashboard(state.clone()).await?;
    let perf = get_search_diagnostics().await?;
    let cfg = state
        .config
        .read()
        .map_err(|_| "Failed to access config".to_string())?
        .clone();

    let use_default_path = output_path.is_none();
    let mut target_path = output_path
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| support_bundle_dir().join(format!("support-{}.json", now_millis())));

    if target_path.is_dir() || use_default_path {
        target_path = target_path.join(format!("support-{}.json", now_millis()));
    }
    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create support bundle directory: {}", e))?;
    }

    let payload = serde_json::json!({
        "created_at": now_millis(),
        "health": dashboard,
        "search_diagnostics": perf,
        "config_summary": {
            "schema_version": cfg.schema_version,
            "active_profile_id": cfg.profiles.active_profile_id,
            "profiles": cfg.profiles.entries.iter().map(|p| serde_json::json!({
                "id": p.id,
                "name": p.name,
                "theme": p.theme,
                "hotkey": p.hotkey,
            })).collect::<Vec<_>>()
        }
    });

    let serialized = serde_json::to_string_pretty(&payload)
        .map_err(|e| format!("Failed to serialize support bundle: {}", e))?;
    std::fs::write(&target_path, serialized)
        .map_err(|e| format!("Failed to write support bundle: {}", e))?;
    let size = std::fs::metadata(&target_path)
        .map_err(|e| format!("Failed to stat support bundle: {}", e))?
        .len();

    Ok(SupportBundleReport {
        path: target_path.to_string_lossy().to_string(),
        size_bytes: size,
    })
}

#[tauri::command]
async fn get_recovery_hints(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<RecoveryHint>, VantaError> {
    let cfg = state
        .config
        .read()
        .map_err(|_| "Failed to access config".to_string())?
        .clone();
    let unclean = *state
        .startup_unclean
        .lock()
        .map_err(|_| "Failed to access startup state".to_string())?;
    let index_count = state
        .file_index
        .lock()
        .map_err(|_| "Failed to access file index".to_string())?
        .entries
        .len();

    let mut hints = Vec::new();
    if unclean {
        hints.push(RecoveryHint {
            id: "unclean-startup".to_string(),
            title: "Previous startup may have failed".to_string(),
            detail: "Try launching with default theme/profile and regenerate a support bundle for troubleshooting.".to_string(),
        });
    }
    if index_count == 0 {
        hints.push(RecoveryHint {
            id: "rebuild-file-index".to_string(),
            title: "File index is empty".to_string(),
            detail: "Open Settings -> File Search and run 'Rebuild Index' to restore document results.".to_string(),
        });
    }
    if cfg.profiles.entries.is_empty() {
        hints.push(RecoveryHint {
            id: "reseed-profiles".to_string(),
            title: "No profiles found".to_string(),
            detail: "Run contract migration or reset config to regenerate the default profile.".to_string(),
        });
    }

    if hints.is_empty() {
        hints.push(RecoveryHint {
            id: "all-good".to_string(),
            title: "No recovery action needed".to_string(),
            detail: "Core subsystems are healthy based on current diagnostics.".to_string(),
        });
    }

    Ok(hints)
}

#[tauri::command]
async fn rescan_apps(
    state: tauri::State<'_, AppState>,
) -> Result<usize, VantaError> {
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
) -> Result<Vec<scanner::AppEntry>, VantaError> {
    let apps = state
        .apps
        .lock()
        .map_err(|_| "Failed to access app cache".to_string())?;
    Ok(apps.clone())
}

#[tauri::command]
async fn hide_window(window: tauri::WebviewWindow) -> Result<(), VantaError> {
    window::hide_window(&window)
}

#[tauri::command]
async fn show_window(window: tauri::WebviewWindow) -> Result<(), VantaError> {
    window::show_window(&window)
}

#[tauri::command]
async fn get_ui_state() -> Result<serde_json::Value, VantaError> {
    let state = load_ui_state();
    Ok(serde_json::json!({
        "last_view": state.last_view,
    }))
}

#[tauri::command]
async fn set_ui_last_view(view: String) -> Result<(), VantaError> {
    let mut state = load_ui_state();
    state.last_view = Some(view);
    save_ui_state(&state);
    Ok(())
}

#[tauri::command]
async fn get_extensions(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<ExtensionEntry>, VantaError> {
    let cached = state
        .extensions_cache
        .lock()
        .map_err(|_| "Failed to access extensions cache".to_string())?;
    Ok(cached.clone())
}

#[tauri::command]
async fn get_extension_bundle(
    ext_id: String,
) -> Result<String, VantaError> {
    extensions::load_extension_bundle(&ext_id)
}

#[tauri::command]
async fn get_extension_styles(
    ext_id: String,
) -> Result<Option<String>, VantaError> {
    extensions::load_extension_styles(&ext_id)
}

#[tauri::command]
async fn get_workflows(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<WorkflowMacro>, VantaError> {
    workflows::list_macros(&state)
}

#[tauri::command]
async fn get_command_templates(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<CommandTemplate>, VantaError> {
    let config = state
        .config
        .read()
        .map_err(|_| "Failed to access config".to_string())?;
    Ok(config.workflows.command_templates.clone())
}

#[tauri::command]
async fn save_command_template(
    name: String,
    macro_id: String,
    args: HashMap<String, String>,
    template_id: Option<String>,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<CommandTemplate, VantaError> {
    let trimmed_name = name.trim();
    if trimmed_name.len() < 2 {
        return Err("Template name must be at least 2 characters".into());
    }

    let now = u64::try_from(now_millis()).unwrap_or(0);

    let (saved, updated_cfg, updated_templates) = {
        let mut config = state
            .config
            .write()
            .map_err(|_| "Failed to access config".to_string())?;

        let macro_def = config
            .workflows
            .macros
            .iter()
            .find(|m| m.id == macro_id)
            .cloned()
            .ok_or_else(|| format!("Macro '{}' not found", macro_id))?;

        let allowed_names = macro_def
            .args
            .iter()
            .map(|a| a.name.as_str())
            .collect::<HashSet<_>>();
        let filtered_args = args
            .into_iter()
            .filter(|(k, v)| allowed_names.contains(k.as_str()) && !v.trim().is_empty())
            .collect::<HashMap<_, _>>();

        let saved = if let Some(existing_id) = template_id {
            let existing = config
                .workflows
                .command_templates
                .iter_mut()
                .find(|t| t.id == existing_id)
                .ok_or_else(|| format!("Template '{}' not found", existing_id))?;
            existing.name = trimmed_name.to_string();
            existing.macro_id = macro_id.clone();
            existing.args = filtered_args;
            existing.updated_at_ms = now;
            existing.clone()
        } else {
            let mut slug = trimmed_name
                .chars()
                .map(|c| {
                    if c.is_ascii_alphanumeric() {
                        c.to_ascii_lowercase()
                    } else {
                        '-'
                    }
                })
                .collect::<String>()
                .trim_matches('-')
                .to_string();
            if slug.is_empty() {
                slug = "template".to_string();
            }

            let mut candidate = format!("tmpl-{}-{}", slug, now);
            let mut suffix = 1u32;
            while config
                .workflows
                .command_templates
                .iter()
                .any(|t| t.id == candidate)
            {
                candidate = format!("tmpl-{}-{}-{}", slug, now, suffix);
                suffix = suffix.saturating_add(1);
            }

            let template = CommandTemplate {
                id: candidate,
                name: trimmed_name.to_string(),
                macro_id: macro_id.clone(),
                args: filtered_args,
                created_at_ms: now,
                updated_at_ms: now,
            };
            config.workflows.command_templates.insert(0, template.clone());
            if config.workflows.command_templates.len() > 200 {
                config.workflows.command_templates.truncate(200);
            }
            template
        };

        config.workflows.schema_version = config::WORKFLOWS_SCHEMA_VERSION;
        config.save_with_source("user")?;

        (
            saved,
            config.clone(),
            config.workflows.command_templates.clone(),
        )
    };

    let _ = app_handle.emit("config-updated", &updated_cfg);
    let _ = app_handle.emit("command-templates-changed", &updated_templates);
    Ok(saved)
}

#[tauri::command]
async fn delete_command_template(
    template_id: String,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<bool, VantaError> {
    let (deleted, updated_cfg, updated_templates) = {
        let mut config = state
            .config
            .write()
            .map_err(|_| "Failed to access config".to_string())?;

        let before = config.workflows.command_templates.len();
        config
            .workflows
            .command_templates
            .retain(|t| t.id != template_id);
        let deleted = config.workflows.command_templates.len() != before;

        if !deleted {
            (false, None, Vec::new())
        } else {
            config.save_with_source("user")?;
            (
                true,
                Some(config.clone()),
                config.workflows.command_templates.clone(),
            )
        }
    };

    if let Some(cfg) = updated_cfg {
        let _ = app_handle.emit("config-updated", &cfg);
    }
    if deleted {
        let _ = app_handle.emit("command-templates-changed", &updated_templates);
    }

    Ok(deleted)
}

#[tauri::command]
async fn list_workflow_secret_names() -> Result<Vec<String>, VantaError> {
    secrets::list_secret_names()
}

#[tauri::command]
async fn set_workflow_secret(name: String, value: String) -> Result<(), VantaError> {
    secrets::set_secret(&name, &value)
}

#[tauri::command]
async fn delete_workflow_secret(name: String) -> Result<bool, VantaError> {
    secrets::delete_secret(&name)
}

#[tauri::command]
async fn dry_run_macro(
    macro_id: String,
    args: HashMap<String, String>,
    state: tauri::State<'_, AppState>,
) -> Result<MacroDryRunResult, VantaError> {
    workflows::dry_run_macro(&macro_id, args, &state)
}

#[tauri::command]
async fn run_macro(
    macro_id: String,
    args: HashMap<String, String>,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<MacroRunResult, VantaError> {
    workflows::run_macro(&macro_id, args, &state, &app_handle)
}

#[tauri::command]
async fn start_macro_job(
    macro_id: String,
    args: HashMap<String, String>,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<MacroJobRecord, VantaError> {
    let preflight = workflows::dry_run_macro(&macro_id, args.clone(), &state)?;
    if let Some(step) = preflight
        .steps
        .iter()
        .find(|s| !s.missing_caps.is_empty())
    {
        let payload = serde_json::json!({
            "script_id": format!("macro:{}:step:{}", preflight.macro_id, step.index),
            "missing_caps": step.missing_caps,
            "requested_caps": step.capabilities,
        });
        return Err(format!("PERMISSION_NEEDED:{}", payload).into());
    }
    if preflight
        .steps
        .iter()
        .any(|s| s.decision == permissions::Decision::Deny)
    {
        let payload = serde_json::json!({
            "script_id": format!("macro:{}", preflight.macro_id),
            "requested_caps": preflight.steps.into_iter().flat_map(|s| s.capabilities).collect::<Vec<_>>(),
        });
        return Err(format!("PERMISSION_DENIED:{}", payload).into());
    }

    let macros = workflows::list_macros(&state)?;
    let macro_name = macros
        .iter()
        .find(|m| m.id == macro_id)
        .map(|m| m.name.clone())
        .unwrap_or_else(|| macro_id.clone());

    let ts = now_millis();
    let job_id = format!("job-{}-{}", ts, SEARCH_CALLS.fetch_add(1, Ordering::Relaxed));
    let record = MacroJobRecord {
        id: job_id.clone(),
        macro_id: macro_id.clone(),
        macro_name,
        args: args.clone(),
        status: MacroJobStatus::Running,
        created_at: ts,
        updated_at: ts,
        completed_at: None,
        steps: Vec::new(),
        error: None,
    };

    {
        let mut jobs = state
            .macro_jobs
            .lock()
            .map_err(|_| "Failed to access macro jobs".to_string())?;
        jobs.insert(0, record.clone());
        if jobs.len() > 200 {
            jobs.truncate(200);
        }
        save_jobs_to_disk(&jobs);
        emit_jobs_updated(&app_handle, &jobs);
    }

    let app_for_task = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        let run_result = {
            let app_for_block = app_for_task.clone();
            let macro_id_for_block = macro_id.clone();
            let args_for_block = args.clone();
            tauri::async_runtime::spawn_blocking(move || {
                let app_state = app_for_block.state::<AppState>();
                workflows::run_macro_blocking(&macro_id_for_block, args_for_block, &app_state, &app_for_block)
            })
            .await
        };

        let now = now_millis();
        let canceled = {
            let app_state = app_for_task.state::<AppState>();
            app_state
                .canceled_jobs
                .lock()
                .ok()
                .map(|mut s| s.remove(&job_id))
                .unwrap_or(false)
        };

        {
            let app_state = app_for_task.state::<AppState>();
            let mut jobs = match app_state.macro_jobs.lock() {
                Ok(guard) => guard,
                Err(_) => return,
            };
            if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
                job.updated_at = now;
                job.completed_at = Some(now);

                if canceled {
                    job.status = MacroJobStatus::Canceled;
                    job.error = Some("Canceled by user".to_string());
                } else {
                    match run_result {
                        Ok(Ok(result)) => {
                            job.status = MacroJobStatus::Succeeded;
                            job.steps = result.steps;
                            job.error = None;
                        }
                        Ok(Err(err)) => {
                            job.status = MacroJobStatus::Failed;
                            job.error = Some(err.to_string());
                        }
                        Err(join_err) => {
                            job.status = MacroJobStatus::Failed;
                            job.error = Some(format!("Job execution join failure: {}", join_err));
                        }
                    }
                }
            }

            save_jobs_to_disk(&jobs);
            emit_jobs_updated(&app_for_task, &jobs);
        }
    });

    Ok(record)
}

#[tauri::command]
async fn list_macro_jobs(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<MacroJobRecord>, VantaError> {
    let jobs = state
        .macro_jobs
        .lock()
        .map_err(|_| "Failed to access macro jobs".to_string())?;
    Ok(jobs.clone())
}

#[tauri::command]
async fn cancel_macro_job(
    job_id: String,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), VantaError> {
    {
        let mut canceled = state
            .canceled_jobs
            .lock()
            .map_err(|_| "Failed to access canceled jobs".to_string())?;
        canceled.insert(job_id.clone());
    }

    let now = now_millis();
    let mut jobs = state
        .macro_jobs
        .lock()
        .map_err(|_| "Failed to access macro jobs".to_string())?;

    if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
        if matches!(job.status, MacroJobStatus::Running) {
            job.status = MacroJobStatus::Canceled;
            job.updated_at = now;
            job.completed_at = Some(now);
            job.error = Some("Canceled by user".to_string());
        }
    }

    save_jobs_to_disk(&jobs);
    emit_jobs_updated(&app_handle, &jobs);
    Ok(())
}

#[tauri::command]
async fn retry_macro_job(
    job_id: String,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<MacroJobRecord, VantaError> {
    let (macro_id, args) = {
        let jobs = state
            .macro_jobs
            .lock()
            .map_err(|_| "Failed to access macro jobs".to_string())?;
        let job = jobs
            .iter()
            .find(|j| j.id == job_id)
            .ok_or_else(|| format!("Job '{}' not found", job_id))?;
        (job.macro_id.clone(), job.args.clone())
    };

    start_macro_job(macro_id, args, state, app_handle).await
}

#[tauri::command]
async fn get_clipboard_history() -> Result<Vec<clipboard::ClipboardItem>, VantaError> {
    Ok(clipboard::get_history().map_err(|e| format!("Failed to get history: {}", e))?)
}

#[tauri::command]
async fn delete_clipboard_item(id: i64) -> Result<(), VantaError> {
    Ok(clipboard::delete_item(id).map_err(|e| format!("Failed to delete: {}", e))?)
}

#[tauri::command]
async fn toggle_clipboard_pin(id: i64) -> Result<bool, VantaError> {
    Ok(clipboard::toggle_pin(id).map_err(|e| format!("Failed to toggle pin: {}", e))?)
}

#[tauri::command]
async fn open_path(
    path: String,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), VantaError> {
    let path_obj = std::path::Path::new(&path);
    if !path_obj.exists() {
        return Err("Path does not exist".into());
    }

    let is_dir = path_obj.is_dir();

    let config = {
        state.config.read()
            .map_err(|_| "Failed to access config".to_string())?
            .files.clone()
    };

    let app_exec_id = if is_dir || config.open_docs_in_manager {
        config.file_manager
    } else {
        config.file_editor
    };

    if app_exec_id == "default" {
        open::that(&path).map_err(|e| format!("Failed to open path: {}", e))?;
        return Ok(());
    }

    let matched_app = {
        let apps = state.apps.lock()
            .map_err(|_| "Failed to access apps cache".to_string())?;
        apps.iter().find(|a| a.exec == app_exec_id).cloned()
    };

    if let Some(app) = matched_app {
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
        log::warn!("Custom opener '{}' not found, falling back to default.", app_exec_id);
        open::that(&path).map_err(|e| format!("Failed to open path: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
async fn open_external_link(url: String) -> Result<(), VantaError> {
    let trimmed = url.trim();
    if !(trimmed.starts_with("http://") || trimmed.starts_with("https://")) {
        return Err("Only http(s) URLs are allowed".into());
    }
    open::that(trimmed).map_err(|e| format!("Failed to open URL: {}", e))?;
    Ok(())
}

#[tauri::command]
async fn reveal_in_file_manager(
    path: String,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), VantaError> {
    let target = std::path::Path::new(&path);
    if !target.exists() {
        return Err("Path does not exist".into());
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
            .read()
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
) -> Result<(), VantaError> {
    let path_obj = std::path::Path::new(&path);
    if !path_obj.exists() {
        return Err("Path does not exist".into());
    }
    if path_obj.is_dir() {
        return Err("Cannot open directory with editor".into());
    }

    let editor_id = {
        state
            .config
            .read()
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

#[derive(serde::Serialize)]
struct FilePreview {
    path: String,
    name: String,
    size_bytes: u64,
    modified: Option<u64>,
    mime_hint: String,
    content: Option<String>,
    is_image: bool,
    is_directory: bool,
}

#[tauri::command]
async fn preview_file(path: String) -> Result<FilePreview, VantaError> {
    let p = std::path::Path::new(&path);
    if !p.exists() {
        return Err("Path does not exist".into());
    }

    let meta = std::fs::metadata(&path)
        .map_err(|e| format!("Cannot read metadata: {}", e))?;
    let name = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    let size_bytes = meta.len();
    let modified = meta.modified().ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs());
    let is_directory = meta.is_dir();

    let ext = p.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
    let image_exts = ["png", "jpg", "jpeg", "gif", "bmp", "webp", "svg", "ico"];
    let is_image = image_exts.contains(&ext.as_str());

    let mime_hint = match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "bmp" => "image/bmp",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "pdf" => "application/pdf",
        "rs" => "text/x-rust",
        "py" => "text/x-python",
        "js" | "mjs" => "text/javascript",
        "ts" => "text/typescript",
        "json" => "application/json",
        "toml" => "text/x-toml",
        "yaml" | "yml" => "text/x-yaml",
        "md" => "text/markdown",
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "sh" | "bash" | "zsh" => "text/x-shellscript",
        "txt" | "log" => "text/plain",
        "xml" => "text/xml",
        "csv" => "text/csv",
        _ => if is_directory { "inode/directory" } else { "application/octet-stream" },
    }.to_string();

    // Read text content for non-images, cap at 4KB
    let content = if !is_image && !is_directory && size_bytes < 512_000 {
        let text_types = ["text/", "application/json", "text/x-"];
        let is_text = text_types.iter().any(|t| mime_hint.starts_with(t))
            || mime_hint == "application/json";
        if is_text {
            std::fs::read_to_string(&path).ok().map(|s| {
                if s.len() > 4096 { format!("{}…", &s[..4096]) } else { s }
            })
        } else { None }
    } else if is_directory {
        // List directory entries (up to 50)
        let mut entries: Vec<String> = Vec::new();
        if let Ok(dir) = std::fs::read_dir(&path) {
            for entry in dir.take(50).flatten() {
                let n = entry.file_name().to_string_lossy().to_string();
                let suffix = if entry.path().is_dir() { "/" } else { "" };
                entries.push(format!("{}{}", n, suffix));
            }
        }
        entries.sort();
        Some(entries.join("\n"))
    } else { None };

    Ok(FilePreview { path, name, size_bytes, modified, mime_hint, content, is_image, is_directory })
}

#[tauri::command]
async fn open_spotify_mini_player(app_handle: tauri::AppHandle) -> Result<(), VantaError> {
    use tauri::WebviewUrl;

    if let Some(existing) = app_handle.get_webview_window("spotify-mini") {
        existing.set_focus().map_err(|e| format!("Focus failed: {}", e))?;
        return Ok(());
    }

    let url = WebviewUrl::App("/".into());
    let builder = tauri::WebviewWindowBuilder::new(
        &app_handle,
        "spotify-mini",
        url,
    )
    .title("Spotify Mini Player")
    .inner_size(320.0, 180.0)
    .min_inner_size(220.0, 120.0)
    .resizable(true)
    .decorations(false)
    .transparent(true)
    .always_on_top(true)
    .skip_taskbar(false);

    builder.build().map_err(|e| format!("Failed to create mini player window: {}", e))?;

    // Hint tiling WMs (Hyprland) to float this window
    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_millis(200));
        let _ = std::process::Command::new("hyprctl")
            .args(["dispatch", "togglefloating", "title:Spotify Mini Player"])
            .output();
    });
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run(start_hidden: bool, open_clipboard: bool) {
    env_logger::init();
    let startup_unclean = mark_startup_in_progress();

    if let Err(e) = clipboard::init_db() {
        log::error!("Failed to init clipboard DB: {}", e);
    }
    clipboard::start_watcher();

    let mut vanta_config = config::load_or_create_default();

    if let Ok(report) = config::migrate_config_on_disk() {
        if report.config_updated {
            log::info!(
                "Config contract migrated: schema {} -> {}, workflows {} -> {}",
                report.schema_from,
                report.schema_to,
                report.workflows_schema_from,
                report.workflows_schema_to,
            );
            vanta_config = config::load_or_create_default();
        }
    }

    let extension_migration = extensions::migrate_extension_manifests();
    if extension_migration.updated > 0 || !extension_migration.failed.is_empty() {
        log::info!(
            "Extension manifest migration: scanned={} updated={} failed={}",
            extension_migration.scanned,
            extension_migration.updated,
            extension_migration.failed.len()
        );
    }

    if (vanta_config.window.width - 800.0).abs() < f64::EPSILON
        && (vanta_config.window.height - 600.0).abs() < f64::EPSILON
    {
        vanta_config.window.width = 680.0;
        vanta_config.window.height = 420.0;
        let _ = vanta_config.save_with_source("migration");
    }

    if clamp_window_size(&mut vanta_config.window) {
        let _ = vanta_config.save_with_source("migration");
    }

    log::info!(
        "Using config at {:?} with window size {}x{}",
        config::config_path(),
        vanta_config.window.width,
        vanta_config.window.height
    );

    let hotkey_str = vanta_config.general.hotkey.clone();

    let apps: Vec<AppEntry> = Vec::new();
    let discovered_extensions: Vec<ExtensionEntry> = Vec::new();

    let history = if let Some(config_dir) = dirs::config_dir() {
        let vanta_dir = config_dir.join("vanta");
        if !vanta_dir.exists() {
            let _ = std::fs::create_dir_all(&vanta_dir);
        }
        History::load_or_create(&vanta_dir)
    } else {
        History::new()
    };

    let file_index: files::FileIndex = std::sync::Arc::new(Mutex::new(files::FileIndexState::default()));
    let macro_jobs = load_jobs_from_disk();

    let app_state = AppState {
        apps: Mutex::new(apps),
        config: RwLock::new(vanta_config),
        extensions_cache: Mutex::new(discovered_extensions),
        history: Mutex::new(history),
        file_index: file_index.clone(),
        macro_jobs: Mutex::new(macro_jobs),
        canceled_jobs: Mutex::new(HashSet::new()),
        startup_unclean: Mutex::new(startup_unclean),
    };

    let mut builder = tauri::Builder::default();

    let start_clipboard_flag = open_clipboard;
    if start_clipboard_flag {
        builder = builder.on_page_load(|window, _payload| {
            println!("[vanta][clipboard] on_page_load emit");
            let _ = window.emit("open_clipboard", ());
        });
    }

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(
             |app, args, _cwd| {
                println!("Single instance triggered with args: {:?}", args);
                let lower_args: Vec<String> = args.iter().map(|s| s.to_lowercase()).collect();
                if lower_args.contains(&"--clipboard".to_string()) || lower_args.contains(&"-c".to_string()) {
                    println!("Opening clipboard mode (single instance)");
                    if let Some(win) = app.get_webview_window("main") {
                        let dims = current_window_dims(app);
                        let _ = win.set_size(tauri::LogicalSize::new(dims.0, dims.1));
                        let _ = win.set_always_on_top(true);
                        let _ = win.center();
                        let _ = window::show_window(&win);
                        println!("[vanta][clipboard] single-instance emit");
                        emit_clipboard_with_retries(&win);
                    }
                } else {
                    println!("Toggling window");
                    let dims = current_window_dims(app);
                    let _ = window::toggle_window(app, Some(dims));
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
            get_config_audit,
            validate_config_schema,
            factory_reset_config,
            save_config,
            get_profiles,
            switch_profile,
            export_profile,
            import_profile,
            search,
            search_v3,
            save_query_history,
            get_query_history,
            launch_app,
            system_action,
            rescan_apps,
            hide_window,
            show_window,
            get_ui_state,
            set_ui_last_view,
            get_extensions,
            get_extension_bundle,
            get_extension_styles,
            extensions::create_extension_template,
            extensions::extension_fetch,
            extensions::extension_shell_execute,
            extensions::extension_storage_get,
            extensions::extension_storage_set,
            store::fetch_store_registry,
            store::check_store_extension_updates,
            store::submit_extension_rating,
            store::install_store_extension,
            store::rollback_store_extension,
            store::uninstall_extension,
            community::submit_community_feedback,
            community::vote_roadmap_item,
            community::get_community_feedback_summary,
            community::get_popular_workflows_feed,
            community::install_popular_workflow,
            community::export_community_snippet,
            community::import_community_snippet,
            get_workflows,
            get_command_templates,
            save_command_template,
            delete_command_template,
            list_workflow_secret_names,
            set_workflow_secret,
            delete_workflow_secret,
            dry_run_macro,
            run_macro,
            start_macro_job,
            list_macro_jobs,
            cancel_macro_job,
            retry_macro_job,
            get_suggestions,
            get_suggestions_v3,
            run_contract_migration,
            get_search_diagnostics,
            get_health_dashboard,
            get_usage_analytics,
            create_support_bundle,
            get_recovery_hints,
            get_clipboard_history,
            delete_clipboard_item,
            toggle_clipboard_pin,
            open_external_link,
            open_path,
            reveal_in_file_manager,
            open_with_editor,
            preview_file,
            open_spotify_mini_player,
            permissions::get_permission_decision,
            permissions::set_permission_decision,
            permissions::get_audit_events,
            get_apps,
            themes::get_installed_themes,
            themes::resize_window_for_theme,
            rebuild_file_index,
        ])
        .setup(move |app| {
            let app_handle = app.handle().clone();
            mark_startup_complete();

            if let Some(win) = app.get_webview_window("main") {
                let _ = window::init_window(&win, &app_handle);

                let mut width_height = current_window_dims(&app_handle);
                let mut restore_position: Option<(f64, f64)> = None;
                {
                    let persisted = load_ui_state();
                    if let (Some(w), Some(h)) = (persisted.window_width, persisted.window_height) {
                        let mut wc = config::WindowConfig { width: w, height: h };
                        let _ = clamp_window_size(&mut wc);
                        width_height = (wc.width, wc.height);
                    }
                    if let (Some(x), Some(y)) = (persisted.window_x, persisted.window_y) {
                        restore_position = Some((x, y));
                    }
                }

                println!(
                    "[vanta][window] applying size {}x{} from {:?}",
                    width_height.0,
                    width_height.1,
                    config::config_path()
                );
                let _ = win.set_size(LogicalSize::new(width_height.0, width_height.1));
                if let Some((x, y)) = restore_position {
                    let _ = win.set_position(LogicalPosition::new(x, y));
                } else {
                    let _ = win.center();
                }

                let win_for_events = win.clone();
                let _ = win.on_window_event(move |event| {
                    use tauri::WindowEvent;
                    if matches!(event, WindowEvent::Resized(_) | WindowEvent::Moved(_)) {
                        persist_window_state(&win_for_events);
                    }
                });

                if open_clipboard {
                    let _ = win.set_always_on_top(true);
                    let _ = window::show_window(&win);
                    println!("[vanta][clipboard] startup emit");
                    emit_clipboard_with_retries(&win);
                } else if start_hidden {
                    let _ = window::hide_window(&win);
                } else {
                    let _ = window::show_window(&win);
                }
            }

            if open_clipboard {
                let handle_clone = app_handle.clone();
                tauri::async_runtime::spawn(async move {
                    sleep(Duration::from_millis(450)).await;
                    if let Some(win) = handle_clone.get_webview_window("main") {
                        println!("[vanta][clipboard] delayed emit");
                        let _ = win.emit("open_clipboard", ());
                    }
                });
            }

            themes::seed_default_theme(&app_handle);

            {
                let scheduler_handle = app_handle.clone();
                tauri::async_runtime::spawn(async move {
                    run_scheduled_workflows_loop(scheduler_handle).await;
                });
            }

            #[cfg(desktop)]
            {
                use tauri_plugin_global_shortcut::{
                    GlobalShortcutExt, ShortcutState, Shortcut,
                };
                use std::str::FromStr;

                let mut shortcuts = Vec::new();
                let mut config_shortcut: Option<Shortcut> = None;
                let mut clipboard_shortcut: Option<Shortcut> = None;

                if let Ok(s) = Shortcut::from_str(&hotkey_str) {
                    config_shortcut = Some(s);
                    shortcuts.push(s);
                } else {
                    log::error!("Invalid config hotkey: {}", hotkey_str);
                }

                let clipboard_hotkey_str = "Super+V";
                if let Ok(s) = Shortcut::from_str(clipboard_hotkey_str) {
                    clipboard_shortcut = Some(s);
                    shortcuts.push(s);
                } else {
                    log::error!("Invalid clipboard hotkey: {}", clipboard_hotkey_str);
                }

                let cfg_sc = config_shortcut.clone();
                let clip_sc = clipboard_shortcut.clone();

                let plugin = tauri_plugin_global_shortcut::Builder::new()
                    .with_handler(move |app, sc, event| {
                        if event.state() == ShortcutState::Pressed {
                            if let Some(ref cfg) = cfg_sc {
                                if sc == cfg {
                                    let dims = current_window_dims(app);
                                    let _ = window::toggle_window(app, Some(dims));
                                    return;
                                }
                            }
                            if let Some(ref clip) = clip_sc {
                                if sc == clip {
                                    if let Some(win) = app.get_webview_window("main") {
                                        let dims = current_window_dims(app);
                                        let _ = win.set_size(tauri::LogicalSize::new(dims.0, dims.1));
                                        let _ = win.set_always_on_top(true);
                                        let _ = win.center();
                                        let _ = window::show_window(&win);
                                        println!("[vanta][clipboard] hotkey emit");
                                        emit_clipboard_with_retries(&win);
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

            let handle_for_watcher = app_handle.clone();
            std::thread::spawn(move || {
                config::watch_config(handle_for_watcher);
            });

            let handle_for_scanner = app_handle.clone();
            std::thread::spawn(move || {
                scanner::watch_desktop_entries(handle_for_scanner);
            });

            let handle_for_extensions = app_handle.clone();
            std::thread::spawn(move || {
                extensions::watch_extensions(handle_for_extensions);
            });

            // Initial app + extension scans in background
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

                    let exts = extensions::scan_extensions();
                    let seeds: Vec<(String, Vec<Capability>)> = exts
                        .iter()
                        .map(|e| (e.manifest.name.clone(), e.manifest.permissions.clone()))
                        .collect();
                    if let Err(err) = permissions::seed_missing_decisions(&seeds) {
                        log::warn!("Failed to seed permissions for extensions: {}", err);
                    }
                    log::info!("Initial extension scan complete: {} extensions", exts.len());
                    if let Some(state) = handle_for_initial_scan.try_state::<AppState>() {
                        if let Ok(mut cached_exts) = state.extensions_cache.lock() {
                            *cached_exts = exts.clone();
                        }
                    }
                    let _ = handle_for_initial_scan.emit("extensions-changed", &exts);
                });
            }

            // Build file index in background
            {
                let index_clone = app_handle.state::<AppState>().file_index.clone();
                let handle_for_index = app_handle.clone();
                let files_config = {
                    let state = app_handle.state::<AppState>();
                    let cfg = match state.config.read() {
                        Ok(cfg) => cfg.files.clone(),
                        Err(_) => crate::config::FilesConfig::default(),
                    };
                    cfg
                };
                std::thread::spawn(move || {
                    log::info!("Building file index...");
                    let index_state = files::build_index(&files_config);
                    log::info!("File index ready: {} entries", index_state.entries.len());
                    if let Ok(mut guard) = index_clone.lock() {
                        *guard = index_state.clone();
                    }
                    if let Some(state) = handle_for_index.try_state::<AppState>() {
                        if let Ok(mut cfg) = state.config.write() {
                            cfg.files.indexed_at = index_state.indexed_at;
                            let _ = cfg.save_with_source("indexer");
                        }
                    }
                });
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Vanta");
}

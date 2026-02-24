use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::{BufRead, Read};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use std::time::Duration;

use jsonschema::{Draft, JSONSchema};
use serde_json::json;

use crate::config;
use crate::permissions::{self, Capability, Decision};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScriptAction {
    #[serde(rename = "type")]
    pub action_type: ActionType,
    pub value: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActionType {
    Copy,
    Open,
    Run,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScriptItem {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<ScriptAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badge: Option<String>,
    #[serde(default = "default_urgency")]
    pub urgency: Urgency,
}

fn default_urgency() -> Urgency {
    Urgency::Normal
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Urgency {
    Low,
    Normal,
    Critical,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScriptOutput {
    pub items: Vec<ScriptItem>,
}

static SCRIPT_OUTPUT_SCHEMA: OnceLock<Result<JSONSchema, String>> = OnceLock::new();

#[derive(Debug)]
pub enum PermissionError {
    Deny,
    NeedsPrompt { missing_caps: Vec<Capability> },
}

pub fn check_and_record_permissions(
    script_id: &str,
    requested_caps: &[Capability],
) -> Result<(), PermissionError> {
    if requested_caps.is_empty() {
        return Ok(());
    }

    let response = permissions::get_decision_for(script_id, requested_caps);

    match response.decision {
        Decision::Allow if response.missing_caps.is_empty() => Ok(()),
        Decision::Allow | Decision::Ask => Err(PermissionError::NeedsPrompt {
            missing_caps: response.missing_caps,
        }),
        Decision::Deny => {
            for capability in requested_caps {
                if let Err(err) = permissions::record_block_event(script_id, capability.clone(), None) {
                    log::warn!("Failed to record block event for {}: {}", script_id, err);
                }
            }
            Err(PermissionError::Deny)
        }
    }
}

// Metadata scraped from script file headers.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScriptEntry {
    pub keyword: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub path: String,
    #[serde(default)]
    pub capabilities: Vec<Capability>,
}

fn scripts_dir() -> PathBuf {
    let dir = config::config_dir().join("scripts");
    // Ensure it exists
    if !dir.exists() {
        if let Err(e) = fs::create_dir_all(&dir) {
            log::warn!("Could not create scripts directory: {}", e);
        }
    }
    dir
}

// Finds executable scripts in the config dir.
pub fn scan_scripts() -> Vec<ScriptEntry> {
    let dir = scripts_dir();
    if !dir.exists() {
        return Vec::new();
    }

    let read_dir = match fs::read_dir(&dir) {
        Ok(rd) => rd,
        Err(e) => {
            log::warn!("Could not read scripts dir: {}", e);
            return Vec::new();
        }
    };

    let mut entries = Vec::new();

    for entry in read_dir.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        // Check if the file is executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(meta) = path.metadata() {
                if meta.permissions().mode() & 0o111 == 0 {
                    continue; // Not executable
                }
            }
        }

        // Extract keyword from filename (filename without extension)
        let keyword = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        if keyword.is_empty() {
            continue;
        }

        // Parse optional vanta: metadata from first 5 lines
        let (name, description, icon, capabilities) = parse_script_metadata(&path);

        entries.push(ScriptEntry {
            keyword,
            name,
            description,
            icon,
            path: path.to_string_lossy().to_string(),
            capabilities,
        });
    }

    entries.sort_by(|a, b| a.keyword.cmp(&b.keyword));
    log::info!("Discovered {} scripts", entries.len());
    entries
}

// Reads metadata like name/icon from the first few lines of the script.
fn parse_script_metadata(
    path: &Path,
) -> (Option<String>, Option<String>, Option<String>, Vec<Capability>) {
    let mut name = None;
    let mut description = None;
    let mut icon = None;
    let mut capabilities: Vec<Capability> = Vec::new();

    let file = match fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return (name, description, icon, capabilities),
    };

    let reader = std::io::BufReader::new(file);
    for (i, line) in reader.lines().enumerate() {
        if i >= 5 {
            break;
        }
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        let trimmed = line.trim();

        // Look for # vanta:key=value or // vanta:key=value
        let content = if let Some(rest) = trimmed.strip_prefix('#') {
            rest.trim()
        } else if let Some(rest) = trimmed.strip_prefix("//") {
            rest.trim()
        } else {
            continue;
        };

        if let Some(rest) = content.strip_prefix("vanta:") {
            if let Some((key, value)) = rest.split_once('=') {
                let key = key.trim();
                let value = value.trim();
                match key {
                    "name" => name = Some(value.to_string()),
                    "description" => description = Some(value.to_string()),
                    "icon" => icon = Some(value.to_string()),
                    "capabilities" => {
                        for cap_str in value.split(',').map(|s| s.trim().to_lowercase()) {
                            let cap = match cap_str.as_str() {
                                "network" => Some(Capability::Network),
                                "shell" => Some(Capability::Shell),
                                "filesystem" | "fs" | "file" => Some(Capability::Filesystem),
                                _ => None,
                            };
                            if let Some(cap) = cap {
                                if !capabilities.contains(&cap) {
                                    capabilities.push(cap);
                                }
                            } else if !cap_str.is_empty() {
                                log::warn!(
                                    "Unknown capability '{}' in script metadata for {}",
                                    cap_str,
                                    path.display()
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    (name, description, icon, capabilities)
}

// Executes the script. Enforces timeout and 1MB output cap to keep things sane.
pub fn execute_script(
    keyword: &str,
    args: &str,
    timeout_ms: u64,
    requested_caps: &[Capability],
) -> Result<ScriptOutput, String> {
    let start = std::time::Instant::now();
    let dir = scripts_dir();

    if let Err(err) = check_and_record_permissions(keyword, requested_caps) {
        return match err {
            PermissionError::NeedsPrompt { missing_caps } => {
                let payload = json!({
                    "script_id": keyword,
                    "missing_caps": missing_caps,
                    "requested_caps": requested_caps,
                });
                Err(format!("PERMISSION_NEEDED:{}", payload))
            }
            PermissionError::Deny => {
                let payload = json!({
                    "script_id": keyword,
                    "requested_caps": requested_caps,
                });
                Err(format!("PERMISSION_DENIED:{}", payload))
            }
        };
    }

    if let Some(ms) = simulate_timeout_ms() {
        log::warn!("Simulating timeout for '{}' after {}ms", keyword, ms);
        std::thread::sleep(Duration::from_millis(ms));
        return Err(format!("Simulated timeout after {}ms", ms));
    }

    if let Some(target) = simulate_error_keyword() {
        if target == keyword {
            log::warn!("Simulating error for '{}'", keyword);
            return Err("Simulated script error".to_string());
        }
    }

    // Find the script file by keyword
    let script_path = find_script_by_keyword(&dir, keyword)?;

    log::info!("Executing script: {} args='{}'", keyword, args);

    // Build the command
    let mut cmd = Command::new(&script_path);
    if !args.is_empty() {
        let parsed_args = shell_words::split(args)
            .map_err(|e| format!("Invalid script args for '{}': {}", keyword, e))?;
        for arg in parsed_args {
            cmd.arg(arg);
        }
    }
    cmd.stdin(std::process::Stdio::null());
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    // Spawn process
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to execute script '{}': {}", keyword, e))?;

    // Drain stdout/stderr concurrently to avoid child blocking on full pipes.
    let stdout_reader = child.stdout.take().map(|mut s| {
        std::thread::spawn(move || {
            let mut buf = Vec::new();
            let _ = std::io::Read::take(&mut s, 1_048_576).read_to_end(&mut buf);
            buf
        })
    });

    let stderr_reader = child.stderr.take().map(|mut s| {
        std::thread::spawn(move || {
            let mut buf = Vec::new();
            let _ = std::io::Read::take(&mut s, 1_048_576).read_to_end(&mut buf);
            buf
        })
    });

    // Wait with timeout
    let timeout = Duration::from_millis(timeout_ms);
    let status = match child.wait_timeout(timeout) {
        Ok(Some(status)) => status,
        Ok(None) => {
            // Timeout — kill the process
            let _ = child.kill();
            let _ = child.wait();
            log::warn!("Script '{}' timed out after {}ms", keyword, timeout_ms);
            return Err(format!(
                "Script '{}' timed out after {}ms",
                keyword, timeout_ms
            ));
        }
        Err(e) => {
            let _ = child.kill();
            return Err(format!("Error waiting for script '{}': {}", keyword, e));
        }
    };

    let stdout = stdout_reader
        .and_then(|h| h.join().ok())
        .map(|buf| String::from_utf8_lossy(&buf).to_string())
        .unwrap_or_default();

    let stderr = stderr_reader
        .and_then(|h| h.join().ok())
        .map(|buf| String::from_utf8_lossy(&buf).to_string())
        .unwrap_or_default();

    if !status.success() {
        let msg = if stderr.is_empty() {
            format!("Script '{}' exited with code {:?}", keyword, status.code())
        } else {
            format!(
                "Script '{}' error: {}",
                keyword,
                stderr.lines().next().unwrap_or("")
            )
        };
        return Err(msg);
    }

    if stdout.trim().is_empty() {
        return Err(format!("Script '{}' produced no output", keyword));
    }

    // Parse JSON output
    let output: ScriptOutput = serde_json::from_str(stdout.trim()).map_err(|e| {
        log::warn!(
            "Script '{}' output invalid JSON: {} — raw: {}",
            keyword,
            e,
            &stdout[..stdout.len().min(200)]
        );
        format!(
            "Invalid JSON output from '{}'. Run the script manually to debug.",
            keyword
        )
    })?;

    let elapsed = start.elapsed();
    log::info!(
        "Script '{}' completed: {} items in {:?}",
        keyword,
        output.items.len(),
        elapsed
    );

    Ok(output)
}

fn build_schema_validator() -> Result<JSONSchema, String> {
    let schema = script_output_schema();

    JSONSchema::options()
        .with_draft(Draft::Draft7)
        .compile(&schema)
        .map_err(|e| e.to_string())
}

fn script_output_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "required": ["items"],
        "properties": {
            "items": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["title", "urgency"],
                    "properties": {
                        "title": {"type": "string", "minLength": 1},
                        "subtitle": {"type": "string"},
                        "icon": {"type": "string"},
                        "badge": {"type": "string"},
                        "urgency": {"type": "string", "enum": ["low", "normal", "critical"]},
                        "action": {
                            "type": "object",
                            "required": ["type", "value"],
                            "properties": {
                                "type": {"type": "string", "enum": ["copy", "open", "run"]},
                                "value": {"type": "string", "minLength": 1}
                            },
                            "additionalProperties": false
                        }
                    },
                    "additionalProperties": false
                }
            }
        },
        "additionalProperties": false
    })
}

pub fn validate_script_output(output: &ScriptOutput) -> Result<(), String> {
    let validator = SCRIPT_OUTPUT_SCHEMA
        .get_or_init(|| build_schema_validator())
        .as_ref()
        .map_err(|e| format!("Failed to prepare script schema: {}", e))?;

    let value = serde_json::to_value(output)
        .map_err(|e| format!("Could not serialize script output: {}", e))?;

    let result = validator.validate(&value);

    match result {
        Ok(()) => Ok(()),
        Err(errors) => {
            let msgs: Vec<String> = errors.map(|err| err.to_string()).collect();
            Err(msgs.join("; "))
        }
    }
}

/// Find a script file by its keyword (filename without extension).
fn find_script_by_keyword(dir: &Path, keyword: &str) -> Result<PathBuf, String> {
    if !dir.exists() {
        return Err(format!("Scripts directory does not exist"));
    }

    let read_dir =
        fs::read_dir(dir).map_err(|e| format!("Cannot read scripts directory: {}", e))?;

    for entry in read_dir.flatten() {
        let path = entry.path();
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            if stem == keyword && path.is_file() {
                return Ok(path);
            }
        }
    }

    Err(format!("Script '{}' not found", keyword))
}

/// Watch the scripts directory and emit `scripts-changed` events.
pub fn watch_scripts(app_handle: tauri::AppHandle) {
    use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
    use std::sync::mpsc;
    use tauri::Emitter;

    let dir = scripts_dir();
    if !dir.exists() {
        log::warn!("Scripts directory does not exist, skipping watcher");
        return;
    }

    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();

    let mut watcher: RecommendedWatcher = match Watcher::new(
        tx,
        notify::Config::default().with_poll_interval(Duration::from_secs(2)),
    ) {
        Ok(w) => w,
        Err(e) => {
            log::error!("Failed to create scripts watcher: {}", e);
            return;
        }
    };

    if let Err(e) = watcher.watch(&dir, RecursiveMode::NonRecursive) {
        log::error!("Failed to watch scripts dir: {}", e);
        return;
    }

    log::info!("Watching scripts directory: {}", dir.display());

    let mut last_scan = std::time::Instant::now() - Duration::from_millis(600);

    for event in rx {
        match event {
            Ok(ev) => {
                let is_modify = matches!(
                    ev.kind,
                    EventKind::Create(_) | EventKind::Remove(_) | EventKind::Modify(_)
                );

                // Debounce: coalesce event bursts while keeping updates responsive.
                if is_modify && last_scan.elapsed() > Duration::from_millis(600) {
                    last_scan = std::time::Instant::now();

                    let scripts = scan_scripts();
                    // Seed permission store so newly added scripts prompt on first run.
                    let seeds: Vec<(String, Vec<Capability>)> = scripts
                        .iter()
                        .map(|s| (s.keyword.clone(), s.capabilities.clone()))
                        .collect();
                    if let Err(err) = permissions::seed_missing_decisions(&seeds) {
                        log::warn!("Failed to seed permissions for new scripts: {}", err);
                    }
                    log::info!("Scripts re-scanned: {} scripts", scripts.len());
                    let _ = app_handle.emit("scripts-changed", &scripts);
                }
            }
            Err(e) => {
                log::error!("Scripts watcher error: {}", e);
            }
        }
    }
}

// Extension trait to add timeout support to std::process::Child.
trait ChildExt {
    fn wait_timeout(
        &mut self,
        timeout: Duration,
    ) -> Result<Option<std::process::ExitStatus>, std::io::Error>;
}

fn simulate_timeout_ms() -> Option<u64> {
    env::var("VANTA_SIMULATE_TIMEOUT")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
}

fn simulate_error_keyword() -> Option<String> {
    env::var("VANTA_SIMULATE_ERROR").ok()
}

impl ChildExt for std::process::Child {
    fn wait_timeout(
        &mut self,
        timeout: Duration,
    ) -> Result<Option<std::process::ExitStatus>, std::io::Error> {
        let start = std::time::Instant::now();
        let poll_interval = Duration::from_millis(50);

        loop {
            match self.try_wait()? {
                Some(status) => return Ok(Some(status)),
                None => {
                    if start.elapsed() >= timeout {
                        return Ok(None);
                    }
                    std::thread::sleep(poll_interval);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_script_output() {
        let json = r#"{"items": [{"title": "Hello", "subtitle": "World", "action": {"type": "copy", "value": "hello"}}]}"#;
        let output: ScriptOutput = serde_json::from_str(json).unwrap();
        assert_eq!(output.items.len(), 1);
        assert_eq!(output.items[0].title, "Hello");
        assert!(output.items[0].action.is_some());
    }

    #[test]
    fn test_parse_script_output_minimal() {
        let json = r#"{"items": [{"title": "Minimal"}]}"#;
        let output: ScriptOutput = serde_json::from_str(json).unwrap();
        assert_eq!(output.items.len(), 1);
        assert_eq!(output.items[0].title, "Minimal");
        assert!(output.items[0].action.is_none());
        assert!(output.items[0].badge.is_none());
    }

    #[test]
    fn test_parse_script_output_with_badge_and_urgency() {
        let json = r#"{"items": [{"title": "CPU", "badge": "90%", "urgency": "critical"}]}"#;
        let output: ScriptOutput = serde_json::from_str(json).unwrap();
        assert_eq!(output.items[0].badge.as_deref(), Some("90%"));
        assert!(matches!(output.items[0].urgency, Urgency::Critical));
    }

    #[test]
    fn validate_script_output_accepts_minimal() {
        let output = ScriptOutput {
            items: vec![ScriptItem {
                title: "Hello".to_string(),
                subtitle: None,
                icon: None,
                action: None,
                badge: None,
                urgency: Urgency::Normal,
            }],
        };

        assert!(validate_script_output(&output).is_ok());
    }

    #[test]
    fn validate_script_output_rejects_empty_title() {
        let output = ScriptOutput {
            items: vec![ScriptItem {
                title: "".to_string(),
                subtitle: None,
                icon: None,
                action: Some(ScriptAction {
                    action_type: ActionType::Copy,
                    value: "value".to_string(),
                }),
                badge: None,
                urgency: Urgency::Normal,
            }],
        };

        assert!(validate_script_output(&output).is_err());
    }

    #[test]
    fn test_invalid_json_errors() {
        let result = serde_json::from_str::<ScriptOutput>("not json");
        assert!(result.is_err());
    }

    #[test]
    fn simulate_timeout_env_parses_number() {
        env::set_var("VANTA_SIMULATE_TIMEOUT", "250");
        assert_eq!(simulate_timeout_ms(), Some(250));
        env::remove_var("VANTA_SIMULATE_TIMEOUT");
    }

    #[test]
    fn simulate_error_env_reads_keyword() {
        env::set_var("VANTA_SIMULATE_ERROR", "demo");
        assert_eq!(simulate_error_keyword().as_deref(), Some("demo"));
        env::remove_var("VANTA_SIMULATE_ERROR");
    }
}

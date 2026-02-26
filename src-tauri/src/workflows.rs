use std::collections::HashMap;

use regex::Regex;
use serde::Serialize;
use serde_json::json;
use tauri::State;

use crate::config::{MacroArg, MacroStep, WorkflowMacro};
use crate::permissions::{self, Capability, Decision};
use crate::scripts::{self, ScriptEntry, ScriptOutput};
use crate::{launcher, AppState};

#[derive(Debug, Clone, Serialize)]
pub struct MacroDryRunStep {
    pub index: usize,
    pub kind: String,
    pub command: String,
    pub args: Vec<String>,
    pub capabilities: Vec<Capability>,
    pub decision: Decision,
    pub missing_caps: Vec<Capability>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MacroDryRunResult {
    pub macro_id: String,
    pub ready: bool,
    pub errors: Vec<String>,
    pub steps: Vec<MacroDryRunStep>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MacroRunStepResult {
    pub index: usize,
    pub kind: String,
    pub command: String,
    pub args: Vec<String>,
    pub capabilities: Vec<Capability>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<ScriptOutput>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MacroRunResult {
    pub macro_id: String,
    pub steps: Vec<MacroRunStepResult>,
}

#[derive(Debug, Clone)]
enum ResolvedKind {
    Script { script: String },
    System { command: String },
}

#[derive(Debug, Clone)]
struct ResolvedStep {
    id: String,
    kind: ResolvedKind,
    args: Vec<String>,
    capabilities: Vec<Capability>,
}

pub fn list_macros(state: &State<'_, AppState>) -> Result<Vec<WorkflowMacro>, String> {
    let cfg = state
        .config
        .lock()
        .map_err(|_| "Failed to access config".to_string())?;
    Ok(cfg.workflows.macros.clone())
}

pub fn dry_run_macro(
    macro_id: &str,
    provided_args: HashMap<String, String>,
    state: &State<'_, AppState>,
) -> Result<MacroDryRunResult, String> {
    let macro_def = fetch_macro(state, macro_id)?;
    let arg_map = resolve_args(&macro_def.args, &provided_args)?;
    let resolved_steps = resolve_steps(&macro_def, &arg_map, state)?;

    let mut steps = Vec::new();
    let mut errors = Vec::new();

    for (idx, step) in resolved_steps.iter().enumerate() {
        let decision = permissions::get_decision_for(&step.id, &step.capabilities);

        steps.push(MacroDryRunStep {
            index: idx,
            kind: match step.kind {
                ResolvedKind::Script { .. } => "script".to_string(),
                ResolvedKind::System { .. } => "system".to_string(),
            },
            command: match &step.kind {
                ResolvedKind::Script { script } => script.clone(),
                ResolvedKind::System { command } => command.clone(),
            },
            args: step.args.clone(),
            capabilities: step.capabilities.clone(),
            decision: decision.decision.clone(),
            missing_caps: decision.missing_caps.clone(),
        });

        if decision.decision == Decision::Deny {
            errors.push(format!(
                "Permission denied for step {} ({}).",
                idx,
                step.id
            ));
        }
    }

    let ready = errors.is_empty()
        && steps.iter().all(|s| s.decision != Decision::Deny)
        && steps.iter().all(|s| s.missing_caps.is_empty());

    Ok(MacroDryRunResult {
        macro_id: macro_def.id,
        ready,
        errors,
        steps,
    })
}

pub fn run_macro(
    macro_id: &str,
    provided_args: HashMap<String, String>,
    state: &State<'_, AppState>,
    app_handle: &tauri::AppHandle,
) -> Result<MacroRunResult, String> {
    let macro_def = fetch_macro(state, macro_id)?;
    let arg_map = resolve_args(&macro_def.args, &provided_args)?;
    let resolved_steps = resolve_steps(&macro_def, &arg_map, state)?;

    let (timeout_ms, strict_json) = {
        let cfg = state
            .config
            .lock()
            .map_err(|_| "Failed to access config".to_string())?;
        (cfg.scripts.timeout_ms, cfg.scripts.strict_json)
    };

    let mut steps = Vec::new();

    for (idx, step) in resolved_steps.into_iter().enumerate() {
        match step.kind {
            ResolvedKind::Script { ref script } => {
                let args_str = shell_words::join(step.args.clone());
                let output = scripts::execute_script(
                    script,
                    &args_str,
                    timeout_ms,
                    &step.capabilities,
                )?;

                if strict_json {
                    scripts::validate_script_output(&output)
                        .map_err(|e| format!("Script output failed validation: {}", e))?;
                }

                steps.push(MacroRunStepResult {
                    index: idx,
                    kind: "script".to_string(),
                    command: script.clone(),
                    args: step.args,
                    capabilities: step.capabilities,
                    status: "ok".to_string(),
                    output: Some(output),
                });
            }
            ResolvedKind::System { ref command } => {
                match scripts::check_and_record_permissions(&step.id, &step.capabilities) {
                    Ok(()) => {}
                    Err(scripts::PermissionError::NeedsPrompt { missing_caps }) => {
                        let payload = json!({
                            "script_id": step.id,
                            "missing_caps": missing_caps,
                            "requested_caps": step.capabilities,
                        });
                        return Err(format!("PERMISSION_NEEDED:{}", payload));
                    }
                    Err(scripts::PermissionError::Deny) => {
                        let payload = json!({
                            "script_id": step.id,
                            "requested_caps": step.capabilities,
                        });
                        return Err(format!("PERMISSION_DENIED:{}", payload));
                    }
                }

                let exec = shell_words::join(
                    std::iter::once(command.clone()).chain(step.args.clone().into_iter()),
                );
                launcher::launch(&exec, Some(app_handle)).map_err(|e| format!("{}", e))?;

                steps.push(MacroRunStepResult {
                    index: idx,
                    kind: "system".to_string(),
                    command: command.clone(),
                    args: step.args,
                    capabilities: step.capabilities,
                    status: "ok".to_string(),
                    output: None,
                });
            }
        }
    }

    Ok(MacroRunResult {
        macro_id: macro_def.id,
        steps,
    })
}

fn fetch_macro(state: &State<'_, AppState>, macro_id: &str) -> Result<WorkflowMacro, String> {
    let cfg = state
        .config
        .lock()
        .map_err(|_| "Failed to access config".to_string())?;

    cfg.workflows
        .macros
        .iter()
        .find(|m| m.id == macro_id)
        .cloned()
        .ok_or_else(|| format!("Macro '{}' not found", macro_id))
        .and_then(|m| {
            if m.enabled {
                Ok(m)
            } else {
                Err(format!("Macro '{}' is disabled", macro_id))
            }
        })
}

fn resolve_args(
    args_def: &[MacroArg],
    provided: &HashMap<String, String>,
) -> Result<HashMap<String, String>, String> {
    let mut resolved = HashMap::new();

    for arg in args_def {
        if let Some(val) = provided.get(&arg.name) {
            resolved.insert(arg.name.clone(), val.clone());
        } else if let Some(default_val) = &arg.default_value {
            resolved.insert(arg.name.clone(), default_val.clone());
        } else if arg.required {
            return Err(format!("Missing required arg '{}'", arg.name));
        }
    }

    Ok(resolved)
}

fn render_tokens(tokens: &[String], arg_map: &HashMap<String, String>) -> Result<Vec<String>, String> {
    let re = Regex::new(r"\{([A-Za-z0-9_]+)\}").unwrap();
    let mut rendered = Vec::new();

    for token in tokens {
        let mut last = 0;
        let mut output = String::new();

        for caps in re.captures_iter(token) {
            let m = caps.get(0).unwrap();
            output.push_str(&token[last..m.start()]);
            let key = caps.get(1).unwrap().as_str();
            let val = arg_map
                .get(key)
                .ok_or_else(|| format!("Missing value for placeholder '{}'", key))?;
            output.push_str(val);
            last = m.end();
        }

        output.push_str(&token[last..]);
        rendered.push(output);
    }

    Ok(rendered)
}

fn resolve_steps(
    macro_def: &WorkflowMacro,
    arg_map: &HashMap<String, String>,
    state: &State<'_, AppState>,
) -> Result<Vec<ResolvedStep>, String> {
    let mut steps = Vec::new();

    for (idx, step) in macro_def.steps.iter().enumerate() {
        match step {
            MacroStep::Script {
                script,
                args,
                capabilities,
            } => {
                let rendered_args = render_tokens(args, arg_map)?;
                let caps = resolve_script_capabilities(script, capabilities, state);
                steps.push(ResolvedStep {
                    id: format!("macro:{}:script:{}", macro_def.id, script),
                    kind: ResolvedKind::Script {
                        script: script.clone(),
                    },
                    args: rendered_args,
                    capabilities: caps,
                });
            }
            MacroStep::System {
                command,
                args,
                capabilities,
            } => {
                require_system_capabilities(capabilities)?;
                let rendered_args = render_tokens(args, arg_map)?;
                steps.push(ResolvedStep {
                    id: format!("macro:{}:system:{}:{}", macro_def.id, command, idx),
                    kind: ResolvedKind::System {
                        command: command.clone(),
                    },
                    args: rendered_args,
                    capabilities: capabilities.clone(),
                });
            }
        }
    }

    Ok(steps)
}

fn resolve_script_capabilities(
    script: &str,
    explicit: &[Capability],
    state: &State<'_, AppState>,
) -> Vec<Capability> {
    if !explicit.is_empty() {
        return explicit.to_vec();
    }

    let from_cache = state
        .scripts_cache
        .lock()
        .map(|cache| find_script_caps(cache.as_slice(), script))
        .unwrap_or_default();
    if !from_cache.is_empty() {
        return from_cache;
    }

    let scanned = scripts::scan_scripts();
    if let Ok(mut cache) = state.scripts_cache.lock() {
        *cache = scanned.clone();
    }

    find_script_caps(&scanned, script)
}

fn find_script_caps(cache: &[ScriptEntry], keyword: &str) -> Vec<Capability> {
    cache
        .iter()
        .find(|s| s.keyword.eq_ignore_ascii_case(keyword))
        .map(|s| s.capabilities.clone())
        .unwrap_or_default()
}

fn require_system_capabilities(caps: &[Capability]) -> Result<(), String> {
    if caps.is_empty() {
        Err("System steps must declare at least one capability".to_string())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_tokens_replaces_placeholders() {
        let map = HashMap::from([("name".to_string(), "world".to_string())]);
        let tokens = vec!["hello {name}".to_string(), "{name}!".to_string()];
        let rendered = render_tokens(&tokens, &map).unwrap();
        assert_eq!(rendered, vec!["hello world".to_string(), "world!".to_string()]);
    }

    #[test]
    fn render_tokens_missing_placeholder_errors() {
        let map = HashMap::new();
        let tokens = vec!["{missing}".to_string()];
        assert!(render_tokens(&tokens, &map).is_err());
    }

    #[test]
    fn system_steps_require_capabilities() {
        let caps: Vec<Capability> = Vec::new();
        assert!(require_system_capabilities(&caps).is_err());
    }
}
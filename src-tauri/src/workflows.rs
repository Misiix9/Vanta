use std::collections::HashMap;
use std::sync::LazyLock;

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::State;

use crate::config::{MacroArg, MacroStep, WorkflowMacro};
use crate::errors::VantaError;
use crate::extensions;
use crate::permissions::{self, Capability, Decision};
use crate::{launcher, AppState};

/// Compiled once; matches `{AlphaNumeric_Key}` placeholders in workflow tokens.
static TOKEN_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{([A-Za-z0-9_]+)\}").expect("invalid token regex"));

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroRunStepResult {
    pub index: usize,
    pub kind: String,
    pub command: String,
    pub args: Vec<String>,
    pub capabilities: Vec<Capability>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroRunResult {
    pub macro_id: String,
    pub steps: Vec<MacroRunStepResult>,
}

#[derive(Debug, Clone)]
enum ResolvedKind {
    Extension { ext_id: String, command: String },
    System { command: String },
}

#[derive(Debug, Clone)]
struct ResolvedStep {
    id: String,
    kind: ResolvedKind,
    args: Vec<String>,
    capabilities: Vec<Capability>,
}

pub fn list_macros(state: &State<'_, AppState>) -> Result<Vec<WorkflowMacro>, VantaError> {
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
) -> Result<MacroDryRunResult, VantaError> {
    let macro_def = fetch_macro(state, macro_id)?;
    let arg_map = resolve_args(&macro_def.args, &provided_args)?;
    let resolved_steps = resolve_steps(&macro_def, &arg_map)?;

    let mut steps = Vec::new();
    let mut errors = Vec::new();

    for (idx, step) in resolved_steps.iter().enumerate() {
        let decision = permissions::get_decision_for(&step.id, &step.capabilities);

        steps.push(MacroDryRunStep {
            index: idx,
            kind: match step.kind {
                ResolvedKind::Extension { .. } => "extension".to_string(),
                ResolvedKind::System { .. } => "system".to_string(),
            },
            command: match &step.kind {
                ResolvedKind::Extension { ext_id, command } => {
                    format!("{}:{}", ext_id, command)
                }
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
) -> Result<MacroRunResult, VantaError> {
    let macro_def = fetch_macro(state, macro_id)?;
    let arg_map = resolve_args(&macro_def.args, &provided_args)?;
    let resolved_steps = resolve_steps(&macro_def, &arg_map)?;

    let mut steps = Vec::new();

    for (idx, step) in resolved_steps.into_iter().enumerate() {
        match step.kind {
            ResolvedKind::Extension {
                ref ext_id,
                ref command,
            } => {
                match extensions::check_extension_permissions(&step.id, &step.capabilities) {
                    Ok(()) => {}
                    Err(extensions::PermissionError::NeedsPrompt { missing_caps }) => {
                        let payload = json!({
                            "script_id": step.id,
                            "missing_caps": missing_caps,
                            "requested_caps": step.capabilities,
                        });
                        return Err(format!("PERMISSION_NEEDED:{}", payload).into());
                    }
                    Err(extensions::PermissionError::Deny) => {
                        let payload = json!({
                            "script_id": step.id,
                            "requested_caps": step.capabilities,
                        });
                        return Err(format!("PERMISSION_DENIED:{}", payload).into());
                    }
                }

                log::info!(
                    "Macro step {}: extension {}:{}",
                    idx,
                    ext_id,
                    command
                );

                steps.push(MacroRunStepResult {
                    index: idx,
                    kind: "extension".to_string(),
                    command: format!("{}:{}", ext_id, command),
                    args: step.args,
                    capabilities: step.capabilities,
                    status: "ok".to_string(),
                });
            }
            ResolvedKind::System { ref command } => {
                match extensions::check_extension_permissions(&step.id, &step.capabilities) {
                    Ok(()) => {}
                    Err(extensions::PermissionError::NeedsPrompt { missing_caps }) => {
                        let payload = json!({
                            "script_id": step.id,
                            "missing_caps": missing_caps,
                            "requested_caps": step.capabilities,
                        });
                        return Err(format!("PERMISSION_NEEDED:{}", payload).into());
                    }
                    Err(extensions::PermissionError::Deny) => {
                        let payload = json!({
                            "script_id": step.id,
                            "requested_caps": step.capabilities,
                        });
                        return Err(format!("PERMISSION_DENIED:{}", payload).into());
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
                });
            }
        }
    }

    Ok(MacroRunResult {
        macro_id: macro_def.id,
        steps,
    })
}

pub fn run_macro_blocking(
    macro_id: &str,
    provided_args: HashMap<String, String>,
    state: &State<'_, AppState>,
    app_handle: &tauri::AppHandle,
) -> Result<MacroRunResult, VantaError> {
    let macro_def = fetch_macro(state, macro_id)?;
    let arg_map = resolve_args(&macro_def.args, &provided_args)?;
    let resolved_steps = resolve_steps(&macro_def, &arg_map)?;

    let mut steps = Vec::new();

    for (idx, step) in resolved_steps.into_iter().enumerate() {
        match step.kind {
            ResolvedKind::Extension {
                ref ext_id,
                ref command,
            } => {
                match extensions::check_extension_permissions(&step.id, &step.capabilities) {
                    Ok(()) => {}
                    Err(extensions::PermissionError::NeedsPrompt { missing_caps }) => {
                        let payload = json!({
                            "script_id": step.id,
                            "missing_caps": missing_caps,
                            "requested_caps": step.capabilities,
                        });
                        return Err(format!("PERMISSION_NEEDED:{}", payload).into());
                    }
                    Err(extensions::PermissionError::Deny) => {
                        let payload = json!({
                            "script_id": step.id,
                            "requested_caps": step.capabilities,
                        });
                        return Err(format!("PERMISSION_DENIED:{}", payload).into());
                    }
                }

                steps.push(MacroRunStepResult {
                    index: idx,
                    kind: "extension".to_string(),
                    command: format!("{}:{}", ext_id, command),
                    args: step.args,
                    capabilities: step.capabilities,
                    status: "ok".to_string(),
                });
            }
            ResolvedKind::System { ref command } => {
                match extensions::check_extension_permissions(&step.id, &step.capabilities) {
                    Ok(()) => {}
                    Err(extensions::PermissionError::NeedsPrompt { missing_caps }) => {
                        let payload = json!({
                            "script_id": step.id,
                            "missing_caps": missing_caps,
                            "requested_caps": step.capabilities,
                        });
                        return Err(format!("PERMISSION_NEEDED:{}", payload).into());
                    }
                    Err(extensions::PermissionError::Deny) => {
                        let payload = json!({
                            "script_id": step.id,
                            "requested_caps": step.capabilities,
                        });
                        return Err(format!("PERMISSION_DENIED:{}", payload).into());
                    }
                }

                launcher::launch_blocking(command, &step.args, Some(app_handle))?;

                steps.push(MacroRunStepResult {
                    index: idx,
                    kind: "system".to_string(),
                    command: command.clone(),
                    args: step.args,
                    capabilities: step.capabilities,
                    status: "ok".to_string(),
                });
            }
        }
    }

    Ok(MacroRunResult {
        macro_id: macro_def.id,
        steps,
    })
}

fn fetch_macro(state: &State<'_, AppState>, macro_id: &str) -> Result<WorkflowMacro, VantaError> {
    let cfg = state
        .config
        .lock()
        .map_err(|_| "Failed to access config".to_string())?;

    Ok(cfg.workflows
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
        })?)
}

fn resolve_args(
    args_def: &[MacroArg],
    provided: &HashMap<String, String>,
) -> Result<HashMap<String, String>, VantaError> {
    let mut resolved = HashMap::new();

    for arg in args_def {
        if let Some(val) = provided.get(&arg.name) {
            resolved.insert(arg.name.clone(), val.clone());
        } else if let Some(default_val) = &arg.default_value {
            resolved.insert(arg.name.clone(), default_val.clone());
        } else if arg.required {
            return Err(format!("Missing required arg '{}'", arg.name).into());
        }
    }

    Ok(resolved)
}

fn render_tokens(
    tokens: &[String],
    arg_map: &HashMap<String, String>,
) -> Result<Vec<String>, VantaError> {
    let mut rendered = Vec::new();

    for token in tokens {
        let mut last = 0;
        let mut output = String::new();

        for caps in TOKEN_RE.captures_iter(token) {
            // Group 0 (full match) and group 1 (key) are guaranteed present
            // by the regex structure within captures_iter.
            let m = caps.get(0).expect("capture group 0 missing in TOKEN_RE");
            output.push_str(&token[last..m.start()]);
            let key = caps.get(1).expect("capture group 1 missing in TOKEN_RE").as_str();
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
) -> Result<Vec<ResolvedStep>, VantaError> {
    let mut steps = Vec::new();

    for (idx, step) in macro_def.steps.iter().enumerate() {
        match step {
            MacroStep::Extension {
                ext_id,
                command,
                args,
                capabilities,
            } => {
                let rendered_args = render_tokens(args, arg_map)?;
                steps.push(ResolvedStep {
                    id: format!("macro:{}:ext:{}:{}", macro_def.id, ext_id, command),
                    kind: ResolvedKind::Extension {
                        ext_id: ext_id.clone(),
                        command: command.clone(),
                    },
                    args: rendered_args,
                    capabilities: capabilities.clone(),
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

fn require_system_capabilities(caps: &[Capability]) -> Result<(), VantaError> {
    if caps.is_empty() {
        Err("System steps must declare at least one capability".into())
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
        assert_eq!(
            rendered,
            vec!["hello world".to_string(), "world!".to_string()]
        );
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

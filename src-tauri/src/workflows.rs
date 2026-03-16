use std::collections::HashMap;
use std::process::Command;
use std::sync::LazyLock;
use std::time::{Duration, Instant};

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::State;

use crate::config::{
    MacroArg, MacroStep, StepErrorHandling, TimeoutBehavior, WorkflowCondition, WorkflowMacro,
};
use crate::errors::VantaError;
use crate::extensions;
use crate::permissions::{self, Capability, Decision};
use crate::{launcher, AppState};

/// Compiled once; matches `{AlphaNumeric_Key}` placeholders in workflow tokens.
static TOKEN_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{([A-Za-z0-9_.]+)\}").expect("invalid token regex"));

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
    Condition { label: String },
    Workflow { macro_id: String },
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

fn build_macro_catalog(macros: Vec<WorkflowMacro>) -> HashMap<String, WorkflowMacro> {
    macros
        .into_iter()
        .map(|m| (m.id.clone(), m))
        .collect::<HashMap<_, _>>()
}

pub fn dry_run_macro(
    macro_id: &str,
    provided_args: HashMap<String, String>,
    state: &State<'_, AppState>,
) -> Result<MacroDryRunResult, VantaError> {
    let macro_def = fetch_macro(state, macro_id)?;
    let macro_catalog = build_macro_catalog(list_macros(state)?);
    let arg_map = resolve_args(&macro_def.args, &provided_args)?;
    let resolved_steps = resolve_steps(&macro_def, &arg_map, Some(&macro_catalog))?;

    let mut steps = Vec::new();
    let mut errors = Vec::new();

    for (idx, step) in resolved_steps.iter().enumerate() {
        let decision = permissions::get_decision_for(&step.id, &step.capabilities);

        steps.push(MacroDryRunStep {
            index: idx,
            kind: match step.kind {
                ResolvedKind::Extension { .. } => "extension".to_string(),
                ResolvedKind::System { .. } => "system".to_string(),
                ResolvedKind::Condition { .. } => "condition".to_string(),
                ResolvedKind::Workflow { .. } => "workflow".to_string(),
            },
            command: match &step.kind {
                ResolvedKind::Extension { ext_id, command } => {
                    format!("{}:{}", ext_id, command)
                }
                ResolvedKind::System { command } => command.clone(),
                ResolvedKind::Condition { label } => label.clone(),
                ResolvedKind::Workflow { macro_id } => format!("workflow:{}", macro_id),
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
    if contains_conditional_steps(&macro_def.steps) || contains_workflow_steps(&macro_def.steps) {
        return Err(
            "Advanced workflow steps require macro job execution (blocking runner).".into(),
        );
    }
    let arg_map = resolve_args(&macro_def.args, &provided_args)?;
    let resolved_steps = resolve_steps(&macro_def, &arg_map, None)?;

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
            ResolvedKind::Condition { .. } => {
                return Err(
                    "Advanced workflow steps require macro job execution (blocking runner)."
                        .into(),
                );
            }
            ResolvedKind::Workflow { .. } => {
                return Err(
                    "Advanced workflow steps require macro job execution (blocking runner)."
                        .into(),
                );
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
    _app_handle: &tauri::AppHandle,
) -> Result<MacroRunResult, VantaError> {
    let macro_def = fetch_macro(state, macro_id)?;
    let macro_catalog = build_macro_catalog(list_macros(state)?);
    let arg_map = resolve_args(&macro_def.args, &provided_args)?;
    let mut value_map = arg_map.clone();
    let started = Instant::now();
    let mut call_stack = vec![macro_def.id.clone()];

    let mut steps = Vec::new();
    execute_steps_blocking(
        &macro_def.id,
        &macro_def.steps,
        "root",
        &mut value_map,
        &mut steps,
        started,
        macro_def.timeout_ms,
        &macro_def.timeout_behavior,
        &macro_catalog,
        &mut call_stack,
    )?;

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
    value_map: &HashMap<String, String>,
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
            let val = value_map
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

fn render_token(token: &str, value_map: &HashMap<String, String>) -> Result<String, VantaError> {
    Ok(render_tokens(&[token.to_string()], value_map)?
        .into_iter()
        .next()
        .unwrap_or_default())
}

fn step_output_placeholder(index: usize) -> String {
    format!("step.{}.output", index + 1)
}

enum CommandRunResult<T> {
    Completed(T),
    TimedOut,
}

fn execute_system_blocking_capture(
    command: &str,
    args: &[String],
    timeout_ms: Option<u64>,
) -> Result<CommandRunResult<String>, VantaError> {
    let mut child = Command::new(command)
        .args(args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to run '{}' in blocking mode: {}", command, e))?;

    if let Some(ms) = timeout_ms {
        let deadline = Instant::now() + Duration::from_millis(ms);
        loop {
            if child
                .try_wait()
                .map_err(|e| format!("Failed waiting for '{}' in blocking mode: {}", command, e))?
                .is_some()
            {
                break;
            }

            if Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                return Ok(CommandRunResult::TimedOut);
            }

            std::thread::sleep(Duration::from_millis(10));
        }
    }

    let output = child
        .wait_with_output()
        .map_err(|e| format!("Failed collecting output for '{}': {}", command, e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(CommandRunResult::Completed(stdout))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!(
            "Blocking command '{}' failed with status {:?}: {}",
            command,
            output.status.code(),
            stderr
        )
        .into())
    }
}

fn execute_system_blocking_exit_code(
    command: &str,
    args: &[String],
    timeout_ms: Option<u64>,
) -> Result<CommandRunResult<i32>, VantaError> {
    let mut child = Command::new(command)
        .args(args)
        .stdin(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to run '{}' in blocking mode: {}", command, e))?;

    if let Some(ms) = timeout_ms {
        let deadline = Instant::now() + Duration::from_millis(ms);
        loop {
            if let Some(status) = child
                .try_wait()
                .map_err(|e| format!("Failed waiting for '{}' in blocking mode: {}", command, e))?
            {
                return Ok(CommandRunResult::Completed(status.code().unwrap_or(-1)));
            }

            if Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                return Ok(CommandRunResult::TimedOut);
            }

            std::thread::sleep(Duration::from_millis(10));
        }
    }

    let status = child
        .wait()
        .map_err(|e| format!("Failed waiting for '{}' in blocking mode: {}", command, e))?;
    Ok(CommandRunResult::Completed(status.code().unwrap_or(-1)))
}

fn ensure_permissions(step_id: &str, capabilities: &[Capability]) -> Result<(), VantaError> {
    match extensions::check_extension_permissions(step_id, capabilities) {
        Ok(()) => Ok(()),
        Err(extensions::PermissionError::NeedsPrompt { missing_caps }) => {
            let payload = json!({
                "script_id": step_id,
                "missing_caps": missing_caps,
                "requested_caps": capabilities,
            });
            Err(format!("PERMISSION_NEEDED:{}", payload).into())
        }
        Err(extensions::PermissionError::Deny) => {
            let payload = json!({
                "script_id": step_id,
                "requested_caps": capabilities,
            });
            Err(format!("PERMISSION_DENIED:{}", payload).into())
        }
    }
}

fn contains_conditional_steps(steps: &[MacroStep]) -> bool {
    for step in steps {
        if let MacroStep::If {
            then_steps,
            else_steps,
            ..
        } = step
        {
            if contains_conditional_steps(then_steps) || contains_conditional_steps(else_steps) {
                return true;
            }
            return true;
        }
    }
    false
}

fn contains_workflow_steps(steps: &[MacroStep]) -> bool {
    for step in steps {
        match step {
            MacroStep::Workflow { .. } => return true,
            MacroStep::If {
                then_steps,
                else_steps,
                ..
            } => {
                if contains_workflow_steps(then_steps) || contains_workflow_steps(else_steps) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

fn describe_condition(condition: &WorkflowCondition) -> String {
    match condition {
        WorkflowCondition::StepOutputContains { step, value } => {
            format!("step.{}.output contains '{}'", step, value)
        }
        WorkflowCondition::StepOutputEquals { step, value } => {
            format!("step.{}.output == '{}'", step, value)
        }
        WorkflowCondition::SystemCommandExitCode {
            command,
            equals,
            ..
        } => format!("{} exit_code == {}", command, equals),
    }
}

fn evaluate_condition(
    macro_id: &str,
    path: &str,
    condition: &WorkflowCondition,
    value_map: &HashMap<String, String>,
) -> Result<(bool, Vec<String>, Vec<Capability>), VantaError> {
    match condition {
        WorkflowCondition::StepOutputContains { step, value } => {
            if *step == 0 {
                return Err("Condition step index must be 1-based".into());
            }
            let needle = render_token(value, value_map)?;
            let key = step_output_placeholder(step - 1);
            let haystack = value_map
                .get(&key)
                .ok_or_else(|| format!("Condition references missing {}", key))?
                .clone();
            Ok((haystack.contains(&needle), vec![needle], Vec::new()))
        }
        WorkflowCondition::StepOutputEquals { step, value } => {
            if *step == 0 {
                return Err("Condition step index must be 1-based".into());
            }
            let expected = render_token(value, value_map)?;
            let key = step_output_placeholder(step - 1);
            let actual = value_map
                .get(&key)
                .ok_or_else(|| format!("Condition references missing {}", key))?
                .clone();
            Ok((actual == expected, vec![expected], Vec::new()))
        }
        WorkflowCondition::SystemCommandExitCode {
            command,
            args,
            equals,
            capabilities,
        } => {
            require_system_capabilities(capabilities)?;
            let rendered_cmd = render_token(command, value_map)?;
            let rendered_args = render_tokens(args, value_map)?;
            let step_id = format!("macro:{}:if:{}:exit_code", macro_id, path);
            ensure_permissions(&step_id, capabilities)?;
            match execute_system_blocking_exit_code(&rendered_cmd, &rendered_args, None)? {
                CommandRunResult::Completed(code) => {
                    Ok((code == *equals, rendered_args, capabilities.clone()))
                }
                CommandRunResult::TimedOut => {
                    Err(format!("Condition command '{}' timed out", rendered_cmd).into())
                }
            }
        }
    }
}

struct StepExecutionResult {
    kind: String,
    command: String,
    args: Vec<String>,
    capabilities: Vec<Capability>,
    status: String,
    output: String,
}

fn step_error_policy(step: &MacroStep) -> &StepErrorHandling {
    match step {
        MacroStep::Extension { on_error, .. }
        | MacroStep::System { on_error, .. }
        | MacroStep::If { on_error, .. }
        | MacroStep::Workflow { on_error, .. } => on_error,
    }
}

fn step_preview(
    step: &MacroStep,
    value_map: &HashMap<String, String>,
) -> Result<(String, String, Vec<String>, Vec<Capability>), VantaError> {
    match step {
        MacroStep::Extension {
            ext_id,
            command,
            args,
            capabilities,
            ..
        } => Ok((
            "extension".to_string(),
            format!("{}:{}", ext_id, command),
            render_tokens(args, value_map)?,
            capabilities.clone(),
        )),
        MacroStep::System {
            command,
            args,
            capabilities,
            ..
        } => Ok((
            "system".to_string(),
            command.clone(),
            render_tokens(args, value_map)?,
            capabilities.clone(),
        )),
        MacroStep::If { condition, .. } => {
            let (cond_args, cond_caps) = resolve_condition_preview(condition, value_map)?;
            Ok((
                "condition".to_string(),
                describe_condition(condition),
                cond_args,
                cond_caps,
            ))
        }
        MacroStep::Workflow { macro_id, args, .. } => {
            let mut rendered = args
                .iter()
                .map(|(k, v)| {
                    render_token(v, value_map).map(|rv| format!("{}={}", k, rv))
                })
                .collect::<Result<Vec<_>, _>>()?;
            rendered.sort();
            Ok((
                "workflow".to_string(),
                format!("workflow:{}", macro_id),
                rendered,
                Vec::new(),
            ))
        }
    }
}

fn execute_single_step(
    macro_id: &str,
    step: &MacroStep,
    step_path: &str,
    value_map: &mut HashMap<String, String>,
    steps: &mut Vec<MacroRunStepResult>,
    started: Instant,
    workflow_timeout_ms: Option<u64>,
    workflow_timeout_behavior: &TimeoutBehavior,
    macro_catalog: &HashMap<String, WorkflowMacro>,
    call_stack: &mut Vec<String>,
) -> Result<StepExecutionResult, VantaError> {
    match step {
        MacroStep::Extension {
            ext_id,
            command,
            args,
            capabilities,
            ..
        } => {
            let rendered_args = render_tokens(args, value_map)?;
            let step_id = format!("macro:{}:ext:{}", macro_id, step_path);
            ensure_permissions(&step_id, capabilities)?;
            Ok(StepExecutionResult {
                kind: "extension".to_string(),
                command: format!("{}:{}", ext_id, command),
                args: rendered_args,
                capabilities: capabilities.clone(),
                status: "ok".to_string(),
                output: String::new(),
            })
        }
        MacroStep::System {
            command,
            args,
            capabilities,
            timeout_ms,
            timeout_behavior,
            ..
        } => {
            require_system_capabilities(capabilities)?;
            let rendered_args = render_tokens(args, value_map)?;
            let step_id = format!("macro:{}:system:{}", macro_id, step_path);
            ensure_permissions(&step_id, capabilities)?;

            let captured = execute_system_blocking_capture(command, &rendered_args, *timeout_ms)?;
            let (status, output) = match captured {
                CommandRunResult::Completed(stdout) => ("ok".to_string(), stdout),
                CommandRunResult::TimedOut => {
                    let ms = timeout_ms.unwrap_or_default();
                    match timeout_behavior {
                        TimeoutBehavior::Abort => {
                            return Err(
                                format!("System step '{}' timed out after {}ms", command, ms)
                                    .into(),
                            );
                        }
                        TimeoutBehavior::Skip => {
                            (format!("timeout_skipped:{}ms", ms), String::new())
                        }
                    }
                }
            };
            Ok(StepExecutionResult {
                kind: "system".to_string(),
                command: command.clone(),
                args: rendered_args,
                capabilities: capabilities.clone(),
                status,
                output,
            })
        }
        MacroStep::If {
            condition,
            then_steps,
            else_steps,
            ..
        } => {
            let (matched, cond_args, cond_caps) =
                evaluate_condition(macro_id, step_path, condition, value_map)?;

            if matched {
                execute_steps_blocking(
                    macro_id,
                    then_steps,
                    &format!("{}.then", step_path),
                    value_map,
                    steps,
                    started,
                    workflow_timeout_ms,
                    workflow_timeout_behavior,
                    macro_catalog,
                    call_stack,
                )?;
            } else {
                execute_steps_blocking(
                    macro_id,
                    else_steps,
                    &format!("{}.else", step_path),
                    value_map,
                    steps,
                    started,
                    workflow_timeout_ms,
                    workflow_timeout_behavior,
                    macro_catalog,
                    call_stack,
                )?;
            }

            Ok(StepExecutionResult {
                kind: "condition".to_string(),
                command: describe_condition(condition),
                args: cond_args,
                capabilities: cond_caps,
                status: if matched {
                    "true".to_string()
                } else {
                    "false".to_string()
                },
                output: if matched {
                    "true".to_string()
                } else {
                    "false".to_string()
                },
            })
        }
        MacroStep::Workflow {
            macro_id: target_macro_id,
            args,
            ..
        } => {
            if call_stack.iter().any(|id| id == target_macro_id) {
                return Err(
                    format!(
                        "Workflow composition cycle detected: {} -> {}",
                        call_stack.join(" -> "),
                        target_macro_id
                    )
                    .into(),
                );
            }

            let target = macro_catalog
                .get(target_macro_id)
                .ok_or_else(|| format!("Composed workflow '{}' not found", target_macro_id))?;

            if !target.enabled {
                return Err(format!("Composed workflow '{}' is disabled", target_macro_id).into());
            }

            let mut provided_args = HashMap::new();
            for (k, v) in args.iter() {
                provided_args.insert(k.clone(), render_token(v, value_map)?);
            }

            let nested_arg_map = resolve_args(&target.args, &provided_args)?;
            let mut nested_value_map = nested_arg_map;

            call_stack.push(target_macro_id.clone());
            let nested_result = execute_steps_blocking(
                &target.id,
                &target.steps,
                &format!("{}.workflow.{}", step_path, target.id),
                &mut nested_value_map,
                steps,
                started,
                workflow_timeout_ms,
                workflow_timeout_behavior,
                macro_catalog,
                call_stack,
            );
            let _ = call_stack.pop();
            nested_result?;

            let mut arg_pairs = provided_args
                .into_iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>();
            arg_pairs.sort();

            Ok(StepExecutionResult {
                kind: "workflow".to_string(),
                command: format!("workflow:{}", target.id),
                args: arg_pairs,
                capabilities: Vec::new(),
                status: "ok".to_string(),
                output: target.id.clone(),
            })
        }
    }
}

fn execute_steps_blocking(
    macro_id: &str,
    steps_def: &[MacroStep],
    path: &str,
    value_map: &mut HashMap<String, String>,
    steps: &mut Vec<MacroRunStepResult>,
    started: Instant,
    workflow_timeout_ms: Option<u64>,
    workflow_timeout_behavior: &TimeoutBehavior,
    macro_catalog: &HashMap<String, WorkflowMacro>,
    call_stack: &mut Vec<String>,
) -> Result<(), VantaError> {
    for (idx, step) in steps_def.iter().enumerate() {
        if let Some(limit_ms) = workflow_timeout_ms {
            if started.elapsed() >= Duration::from_millis(limit_ms) {
                match workflow_timeout_behavior {
                    TimeoutBehavior::Abort => {
                        return Err(format!("Workflow timed out after {}ms", limit_ms).into());
                    }
                    TimeoutBehavior::Skip => {
                        steps.push(MacroRunStepResult {
                            index: steps.len(),
                            kind: "workflow".to_string(),
                            command: "workflow-timeout".to_string(),
                            args: Vec::new(),
                            capabilities: Vec::new(),
                            status: format!("timeout_skipped_remaining:{}ms", limit_ms),
                        });
                        return Ok(());
                    }
                }
            }
        }

        let step_path = format!("{}.{}", path, idx);
        let policy = step_error_policy(step).clone();

        let mut attempt = 0usize;
        let mut success: Option<StepExecutionResult> = None;
        let mut last_err: Option<VantaError> = None;

        while attempt <= policy.retry_count as usize {
            match execute_single_step(
                macro_id,
                step,
                &step_path,
                value_map,
                steps,
                started,
                workflow_timeout_ms,
                workflow_timeout_behavior,
                macro_catalog,
                call_stack,
            ) {
                Ok(result) => {
                    success = Some(result);
                    break;
                }
                Err(err) => {
                    last_err = Some(err);
                    if attempt == policy.retry_count as usize {
                        break;
                    }
                    attempt += 1;
                    continue;
                }
            }
        }

        if let Some(result) = success {
            let step_index = steps.len();
            let status = if attempt > 0 && result.status == "ok" {
                format!("ok:retry-{}", attempt)
            } else {
                result.status
            };
            steps.push(MacroRunStepResult {
                index: step_index,
                kind: result.kind,
                command: result.command,
                args: result.args,
                capabilities: result.capabilities,
                status,
            });
            value_map.insert(step_output_placeholder(step_index), result.output);
        } else {
            let err = last_err.unwrap_or_else(|| "Workflow step failed".into());
            if policy.skip_on_failure {
                let (kind, command, args, capabilities) =
                    step_preview(step, value_map).unwrap_or_else(|_| {
                        (
                            "unknown".to_string(),
                            step_path.clone(),
                            Vec::new(),
                            Vec::new(),
                        )
                    });
                let step_index = steps.len();
                steps.push(MacroRunStepResult {
                    index: step_index,
                    kind,
                    command,
                    args,
                    capabilities,
                    status: format!("skipped_on_failure: {}", err),
                });
                value_map.insert(step_output_placeholder(step_index), String::new());
            } else {
                if !policy.finally_steps.is_empty() {
                    execute_steps_blocking(
                        macro_id,
                        &policy.finally_steps,
                        &format!("{}.finally", step_path),
                        value_map,
                        steps,
                        started,
                        workflow_timeout_ms,
                        workflow_timeout_behavior,
                        macro_catalog,
                        call_stack,
                    )?;
                }
                return Err(err);
            }
        }

        if !policy.finally_steps.is_empty() {
            execute_steps_blocking(
                macro_id,
                &policy.finally_steps,
                &format!("{}.finally", step_path),
                value_map,
                steps,
                started,
                workflow_timeout_ms,
                workflow_timeout_behavior,
                macro_catalog,
                call_stack,
            )?;
        }
    }

    Ok(())
}

fn resolve_steps(
    macro_def: &WorkflowMacro,
    arg_map: &HashMap<String, String>,
    macro_catalog: Option<&HashMap<String, WorkflowMacro>>,
) -> Result<Vec<ResolvedStep>, VantaError> {
    let mut steps = Vec::new();
    let mut call_stack = vec![macro_def.id.clone()];

    resolve_steps_recursive(
        &macro_def.id,
        &macro_def.steps,
        arg_map,
        "root",
        &mut steps,
        macro_catalog,
        &mut call_stack,
    )?;

    Ok(steps)
}

fn resolve_steps_recursive(
    macro_id: &str,
    steps_def: &[MacroStep],
    arg_map: &HashMap<String, String>,
    path: &str,
    out: &mut Vec<ResolvedStep>,
    macro_catalog: Option<&HashMap<String, WorkflowMacro>>,
    call_stack: &mut Vec<String>,
) -> Result<(), VantaError> {
    for (idx, step) in steps_def.iter().enumerate() {
        let step_path = format!("{}.{}", path, idx);
        match step {
            MacroStep::Extension {
                ext_id,
                command,
                args,
                capabilities,
                on_error,
                ..
            } => {
                let rendered_args = render_tokens(args, arg_map)?;
                out.push(ResolvedStep {
                    id: format!("macro:{}:ext:{}", macro_id, step_path),
                    kind: ResolvedKind::Extension {
                        ext_id: ext_id.clone(),
                        command: command.clone(),
                    },
                    args: rendered_args,
                    capabilities: capabilities.clone(),
                });
                resolve_steps_recursive(
                    macro_id,
                    &on_error.finally_steps,
                    arg_map,
                    &format!("{}.finally", step_path),
                    out,
                    macro_catalog,
                    call_stack,
                )?;
            }
            MacroStep::System {
                command,
                args,
                capabilities,
                on_error,
                ..
            } => {
                require_system_capabilities(capabilities)?;
                let rendered_args = render_tokens(args, arg_map)?;
                out.push(ResolvedStep {
                    id: format!("macro:{}:system:{}", macro_id, step_path),
                    kind: ResolvedKind::System {
                        command: command.clone(),
                    },
                    args: rendered_args,
                    capabilities: capabilities.clone(),
                });
                resolve_steps_recursive(
                    macro_id,
                    &on_error.finally_steps,
                    arg_map,
                    &format!("{}.finally", step_path),
                    out,
                    macro_catalog,
                    call_stack,
                )?;
            }
            MacroStep::If {
                condition,
                then_steps,
                else_steps,
                on_error,
            } => {
                let (args, capabilities) = resolve_condition_preview(condition, arg_map)?;
                out.push(ResolvedStep {
                    id: format!("macro:{}:if:{}", macro_id, step_path),
                    kind: ResolvedKind::Condition {
                        label: describe_condition(condition),
                    },
                    args,
                    capabilities,
                });
                resolve_steps_recursive(
                    macro_id,
                    then_steps,
                    arg_map,
                    &format!("{}.then", step_path),
                    out,
                    macro_catalog,
                    call_stack,
                )?;
                resolve_steps_recursive(
                    macro_id,
                    else_steps,
                    arg_map,
                    &format!("{}.else", step_path),
                    out,
                    macro_catalog,
                    call_stack,
                )?;
                resolve_steps_recursive(
                    macro_id,
                    &on_error.finally_steps,
                    arg_map,
                    &format!("{}.finally", step_path),
                    out,
                    macro_catalog,
                    call_stack,
                )?;
            }
            MacroStep::Workflow {
                macro_id: target_macro_id,
                args,
                on_error,
            } => {
                let mut rendered_args = args
                    .iter()
                    .map(|(k, v)| render_token(v, arg_map).map(|rv| format!("{}={}", k, rv)))
                    .collect::<Result<Vec<_>, _>>()?;
                rendered_args.sort();

                out.push(ResolvedStep {
                    id: format!("macro:{}:workflow:{}", macro_id, step_path),
                    kind: ResolvedKind::Workflow {
                        macro_id: target_macro_id.clone(),
                    },
                    args: rendered_args,
                    capabilities: Vec::new(),
                });

                if let Some(catalog) = macro_catalog {
                    if call_stack.iter().any(|id| id == target_macro_id) {
                        return Err(
                            format!(
                                "Workflow composition cycle detected in dry-run: {} -> {}",
                                call_stack.join(" -> "),
                                target_macro_id
                            )
                            .into(),
                        );
                    }

                    if let Some(target) = catalog.get(target_macro_id) {
                        let mut provided_args = HashMap::new();
                        for (k, v) in args.iter() {
                            provided_args.insert(k.clone(), render_token(v, arg_map)?);
                        }
                        let nested_arg_map = resolve_args(&target.args, &provided_args)?;

                        call_stack.push(target_macro_id.clone());
                        let nested_result = resolve_steps_recursive(
                            &target.id,
                            &target.steps,
                            &nested_arg_map,
                            &format!("{}.workflow.{}", step_path, target.id),
                            out,
                            macro_catalog,
                            call_stack,
                        );
                        let _ = call_stack.pop();
                        nested_result?;
                    }
                }

                resolve_steps_recursive(
                    macro_id,
                    &on_error.finally_steps,
                    arg_map,
                    &format!("{}.finally", step_path),
                    out,
                    macro_catalog,
                    call_stack,
                )?;
            }
        }
    }

    Ok(())
}

fn resolve_condition_preview(
    condition: &WorkflowCondition,
    arg_map: &HashMap<String, String>,
) -> Result<(Vec<String>, Vec<Capability>), VantaError> {
    match condition {
        WorkflowCondition::StepOutputContains { value, .. }
        | WorkflowCondition::StepOutputEquals { value, .. } => {
            let rendered_value = render_token(value, arg_map)?;
            Ok((vec![rendered_value], Vec::new()))
        }
        WorkflowCondition::SystemCommandExitCode {
            command,
            args,
            capabilities,
            ..
        } => {
            require_system_capabilities(capabilities)?;
            let mut rendered = Vec::new();
            rendered.push(render_token(command, arg_map)?);
            rendered.extend(render_tokens(args, arg_map)?);
            Ok((rendered, capabilities.clone()))
        }
    }
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
    fn render_tokens_fails_on_missing_placeholder() {
        let map = HashMap::new();
        let tokens = vec!["{missing}".to_string()];
        assert!(render_tokens(&tokens, &map).is_err());
    }

    #[test]
    fn render_tokens_supports_step_output_placeholders() {
        let map = HashMap::from([("step.1.output".to_string(), "done".to_string())]);
        let tokens = vec!["status={step.1.output}".to_string()];
        let rendered = render_tokens(&tokens, &map).unwrap();
        assert_eq!(rendered, vec!["status=done".to_string()]);
    }

    #[test]
    fn system_steps_require_capabilities() {
        let caps: Vec<Capability> = Vec::new();
        assert!(require_system_capabilities(&caps).is_err());
    }

    #[test]
    fn detects_conditional_steps_recursively() {
        let steps = vec![MacroStep::If {
            condition: WorkflowCondition::StepOutputEquals {
                step: 1,
                value: "ok".to_string(),
            },
            then_steps: vec![MacroStep::System {
                command: "echo".to_string(),
                args: vec!["yes".to_string()],
                capabilities: vec![Capability::Shell],
                on_error: StepErrorHandling::default(),
                timeout_ms: None,
                timeout_behavior: TimeoutBehavior::Abort,
            }],
            else_steps: Vec::new(),
            on_error: StepErrorHandling::default(),
        }];

        assert!(contains_conditional_steps(&steps));
    }

    #[test]
    fn evaluates_step_output_contains_condition() {
        let mut map = HashMap::new();
        map.insert("step.1.output".to_string(), "hello world".to_string());

        let condition = WorkflowCondition::StepOutputContains {
            step: 1,
            value: "world".to_string(),
        };

        let (matched, _args, _caps) = evaluate_condition("m", "root.0", &condition, &map).unwrap();
        assert!(matched);
    }

    #[test]
    fn resolve_steps_includes_finally_cleanup_steps() {
        let macro_def = WorkflowMacro {
            id: "m".to_string(),
            name: "Macro".to_string(),
            description: None,
            args: Vec::new(),
            enabled: true,
            timeout_ms: None,
            timeout_behavior: TimeoutBehavior::Abort,
            schedule: None,
            steps: vec![MacroStep::System {
                command: "echo".to_string(),
                args: vec!["one".to_string()],
                capabilities: vec![Capability::Shell],
                on_error: StepErrorHandling {
                    retry_count: 0,
                    skip_on_failure: false,
                    finally_steps: vec![MacroStep::System {
                        command: "echo".to_string(),
                        args: vec!["cleanup".to_string()],
                        capabilities: vec![Capability::Shell],
                        on_error: StepErrorHandling::default(),
                        timeout_ms: None,
                        timeout_behavior: TimeoutBehavior::Abort,
                    }],
                },
                timeout_ms: None,
                timeout_behavior: TimeoutBehavior::Abort,
            }],
        };

        let arg_map = HashMap::new();
        let resolved = resolve_steps(&macro_def, &arg_map, None).unwrap();
        assert_eq!(resolved.len(), 2);
    }

    #[test]
    fn resolve_steps_expands_composed_workflow() {
        let child = WorkflowMacro {
            id: "child".to_string(),
            name: "Child".to_string(),
            description: None,
            args: Vec::new(),
            steps: vec![MacroStep::System {
                command: "echo".to_string(),
                args: vec!["child".to_string()],
                capabilities: vec![Capability::Shell],
                on_error: StepErrorHandling::default(),
                timeout_ms: None,
                timeout_behavior: TimeoutBehavior::Abort,
            }],
            enabled: true,
            timeout_ms: None,
            timeout_behavior: TimeoutBehavior::Abort,
            schedule: None,
        };

        let parent = WorkflowMacro {
            id: "parent".to_string(),
            name: "Parent".to_string(),
            description: None,
            args: Vec::new(),
            steps: vec![MacroStep::Workflow {
                macro_id: "child".to_string(),
                args: HashMap::new(),
                on_error: StepErrorHandling::default(),
            }],
            enabled: true,
            timeout_ms: None,
            timeout_behavior: TimeoutBehavior::Abort,
            schedule: None,
        };

        let catalog = HashMap::from([
            (parent.id.clone(), parent.clone()),
            (child.id.clone(), child.clone()),
        ]);

        let resolved = resolve_steps(&parent, &HashMap::new(), Some(&catalog)).unwrap();
        assert_eq!(resolved.len(), 2);
        assert!(matches!(resolved[0].kind, ResolvedKind::Workflow { .. }));
        assert!(matches!(resolved[1].kind, ResolvedKind::System { .. }));
    }

    #[test]
    fn resolve_steps_detects_composition_cycle() {
        let cyc = WorkflowMacro {
            id: "cyc".to_string(),
            name: "Cycle".to_string(),
            description: None,
            args: Vec::new(),
            steps: vec![MacroStep::Workflow {
                macro_id: "cyc".to_string(),
                args: HashMap::new(),
                on_error: StepErrorHandling::default(),
            }],
            enabled: true,
            timeout_ms: None,
            timeout_behavior: TimeoutBehavior::Abort,
            schedule: None,
        };

        let catalog = HashMap::from([(cyc.id.clone(), cyc.clone())]);
        assert!(resolve_steps(&cyc, &HashMap::new(), Some(&catalog)).is_err());
    }
}

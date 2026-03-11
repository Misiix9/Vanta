use crate::errors::VantaError;
use std::process::{Command, Stdio};
use std::env;

#[cfg(not(test))]
fn spawn_cmd(cmd: &str, args: &[String]) -> Result<(), std::io::Error> {
    Command::new(cmd)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
}

#[cfg(test)]
fn spawn_cmd(_cmd: &str, _args: &[String]) -> Result<(), std::io::Error> {
    Ok(())
}

fn try_commands(commands: Vec<(&str, Vec<String>)>) -> Result<(), VantaError> {
    let mut errors = Vec::new();

    for (cmd, args) in commands {
        match spawn_cmd(cmd, &args) {
            Ok(_) => return Ok(()),
            Err(e) => errors.push(format!("{}: {}", cmd, e)),
        }
    }

    Err(format!("All commands failed: {}", errors.join("; ")).into())
}

// Handles .desktop Exec placeholders (like %u, %F) so we don't pass garbage to the shell.
pub fn launch(exec: &str, _app_handle: Option<&tauri::AppHandle>) -> Result<(), VantaError> {
    let start = std::time::Instant::now();

    // Check for window actions
    if exec.starts_with("focus:") {
        return focus_window(exec.trim_start_matches("focus:"));
    }

    if exec.starts_with("close-window:") {
        return close_window(exec.trim_start_matches("close-window:"));
    }

    if exec.starts_with("minimize-window:") {
        return minimize_window(exec.trim_start_matches("minimize-window:"));
    }

    if exec.starts_with("move-window-current:") {
        return move_window_to_current_workspace(exec.trim_start_matches("move-window-current:"));
    }

    let cleaned = strip_field_codes(exec);

    if cleaned.is_empty() {
        return Err("Empty exec command after parsing".into());
    }

    let parts = shell_words::split(&cleaned)
        .map_err(|e| format!("Invalid exec syntax '{}': {}", cleaned, e))?;

    let program = parts
        .first()
        .ok_or_else(|| "Empty exec command after parsing".to_string())?;

    log::info!("Launching program: {}", program);

    let mut command = Command::new(program);
    if parts.len() > 1 {
        command.args(&parts[1..]);
    }

    command
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to spawn '{}': {}", cleaned, e))?;

    let elapsed = start.elapsed();
    log::debug!("Launch took {:?}", elapsed);

    Ok(())
}

pub fn launch_blocking(
    command: &str,
    args: &[String],
    _app_handle: Option<&tauri::AppHandle>,
) -> Result<(), VantaError> {
    let status = Command::new(command)
        .args(args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|e| format!("Failed to run '{}' in blocking mode: {}", command, e))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "Blocking command '{}' failed with status {:?}",
            command,
            status.code()
        ).into())
    }
}

pub fn system_action(action: &str) -> Result<(), VantaError> {
    let normalized = action.to_lowercase();

    match normalized.as_str() {
        "sleep" => try_commands(vec![("systemctl", vec!["suspend".into()])]),
        "shutdown" => try_commands(vec![("systemctl", vec!["poweroff".into()])]),
        "restart" | "reboot" => try_commands(vec![("systemctl", vec!["reboot".into()])]),
        "lock" => try_commands(vec![
            ("loginctl", vec!["lock-session".into()]),
            ("loginctl", vec!["lock-sessions".into()]),
        ]),
        "logout" | "log-out" => {
            let mut candidates: Vec<(&str, Vec<String>)> = Vec::new();

            if let Ok(session) = env::var("XDG_SESSION_ID") {
                candidates.push(("loginctl", vec!["terminate-session".into(), session]));
            }

            if let Ok(user) = env::var("USER") {
                candidates.push(("loginctl", vec!["terminate-user".into(), user]));
            }

            candidates.push(("loginctl", vec!["kill-session".into(), "self".into()]));

            try_commands(candidates)
        }
        "bios" | "firmware" | "uefi" => try_commands(vec![
            (
                "systemctl",
                vec!["reboot".into(), "--firmware-setup".into()],
            ),
        ]),
        other => Err(format!("Unsupported system action: {}", other).into()),
    }
}

// Removes those pesky %u %F codes that freedesktop specs use.
fn strip_field_codes(exec: &str) -> String {
    let mut result = String::with_capacity(exec.len());
    let mut chars = exec.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '%' {
            if let Some(&next) = chars.peek() {
                match next {
                    'u' | 'U' | 'f' | 'F' | 'd' | 'D' | 'n' | 'N' | 'i' | 'c' | 'k' | 'v' | 'm' => {
                        chars.next(); // consume the field code char
                        continue;
                    }
                    '%' => {
                        chars.next();
                        result.push('%'); // literal %
                        continue;
                    }
                    _ => {}
                }
            }
        }
        result.push(ch);
    }

    let mut cleaned = result.trim().to_string();
    while cleaned.contains("  ") {
        cleaned = cleaned.replace("  ", " ");
    }
    cleaned
}

fn focus_window(address: &str) -> Result<(), VantaError> {
    let mut ok = false;

    if let Err(e) = spawn_cmd(
        crate::windows::hyprctl_path().as_str(),
        &["dispatch".into(), "focuswindow".into(), format!("address:{}", address)],
    ) {
        log::warn!("Hyprland focus failed: {}", e);
    } else {
        ok = true;
    }

    if let Err(e) = spawn_cmd("swaymsg", &[format!("[con_id={}] focus", address)]) {
        log::warn!("Sway focus failed: {}", e);
    } else {
        ok = true;
    }

    // X11 fallback (wmctrl). Requires window id (hex) as address.
    if let Err(e) = spawn_cmd("wmctrl", &["-i".into(), "-a".into(), address.to_string()]) {
        log::warn!("wmctrl focus failed: {}", e);
    } else {
        ok = true;
    }

    if ok {
        Ok(())
    } else {
        Err("Failed to focus window".into())
    }
}

fn close_window(address: &str) -> Result<(), VantaError> {
    let mut ok = false;

    if let Err(e) = spawn_cmd(
        "hyprctl",
        &["dispatch".into(), "closewindow".into(), format!("address:{}", address)],
    ) {
        log::warn!("Hyprland close failed: {}", e);
    } else {
        ok = true;
    }

    if let Err(e) = spawn_cmd("swaymsg", &[format!("[con_id={}] kill", address)]) {
        log::warn!("Sway close failed: {}", e);
    } else {
        ok = true;
    }

    if let Err(e) = spawn_cmd("wmctrl", &["-i".into(), "-c".into(), address.to_string()]) {
        log::warn!("X11 close failed: {}", e);
    } else {
        ok = true;
    }

    if ok {
        Ok(())
    } else {
        Err("Failed to close window".into())
    }
}

fn minimize_window(address: &str) -> Result<(), VantaError> {
    let mut ok = false;

    // Sway equivalent for minimizing/hiding the container.
    if let Err(e) = spawn_cmd("swaymsg", &[format!("[con_id={}] move scratchpad", address)]) {
        log::warn!("Sway minimize failed: {}", e);
    } else {
        ok = true;
    }

    // X11 fallback.
    if let Err(e) = spawn_cmd(
        "wmctrl",
        &[
            "-i".into(),
            "-r".into(),
            address.to_string(),
            "-b".into(),
            "add,hidden".into(),
        ],
    ) {
        log::warn!("X11 minimize failed: {}", e);
    } else {
        ok = true;
    }

    if ok {
        Ok(())
    } else {
        Err("Minimize not supported on this compositor".into())
    }
}

fn move_window_to_current_workspace(address: &str) -> Result<(), VantaError> {
    let mut ok = false;

    // Sway supports moving focused/target container to the active workspace.
    if let Err(e) = spawn_cmd(
        "swaymsg",
        &[format!("[con_id={}] move container to workspace current", address)],
    ) {
        log::warn!("Sway workspace move failed: {}", e);
    } else {
        ok = true;
    }

    if ok {
        Ok(())
    } else {
        Err("Move-to-workspace is not supported on this compositor".into())
    }
}

/// Launches a terminal emulator and runs the provided shell command.
/// Tries common terminals plus $TERMINAL; falls back to an error if none spawn.
pub fn launch_terminal_command(command: &str) -> Result<(), VantaError> {
    let wrapped = format!(
        "{}; echo \"\\nPress Enter to close\"; read -r _",
        command
    );

    let mut candidates: Vec<(String, Vec<String>)> = Vec::new();

    if let Ok(term_raw) = env::var("TERMINAL") {
        // Reject absurdly long values or those containing null bytes to avoid
        // resource exhaustion during shell-word parsing.
        if term_raw.len() <= 256 && !term_raw.contains('\0') {
            let parsed =
                shell_words::split(&term_raw).unwrap_or_else(|_| vec![term_raw.clone()]);
            if let Some((bin, rest)) = parsed.split_first() {
                let mut args: Vec<String> = rest.iter().map(|s| s.to_string()).collect();
                args.extend([
                    "-e".to_string(),
                    "bash".to_string(),
                    "-lc".to_string(),
                    wrapped.clone(),
                ]);
                candidates.push((bin.to_string(), args));
            }
        }
    }

    let presets: [(&str, &[&str]); 7] = [
        ("kitty", &["sh", "-c"]),
        ("alacritty", &["-e", "bash", "-lc"]),
        ("wezterm", &["start", "--", "bash", "-lc"]),
        ("gnome-terminal", &["--", "bash", "-lc"]),
        ("konsole", &["-e", "bash", "-lc"]),
        ("xterm", &["-e", "bash", "-lc"]),
        ("foot", &["-e", "sh", "-c"]),
    ];

    for (bin, base_args) in presets {
        let mut args: Vec<String> = base_args.iter().map(|s| s.to_string()).collect();
        args.push(wrapped.clone());
        candidates.push((bin.to_string(), args));
    }

    for (bin, args) in candidates {
        let spawn_result = Command::new(&bin)
            .args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();

        if spawn_result.is_ok() {
            return Ok(());
        }
    }

    Err("No terminal emulator found to run 'vanta doctor'".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_field_codes_basic() {
        assert_eq!(strip_field_codes("firefox %u"), "firefox");
        assert_eq!(strip_field_codes("code %F"), "code");
        assert_eq!(strip_field_codes("nautilus %U"), "nautilus");
    }

    #[test]
    fn test_strip_field_codes_no_codes() {
        assert_eq!(strip_field_codes("htop"), "htop");
        assert_eq!(strip_field_codes("alacritty -e vim"), "alacritty -e vim");
    }

    #[test]
    fn test_strip_field_codes_literal_percent() {
        assert_eq!(strip_field_codes("echo %%"), "echo %");
    }

    #[test]
    fn test_strip_field_codes_multiple() {
        assert_eq!(strip_field_codes("cmd %f --flag %U"), "cmd --flag");
    }

    #[test]
    fn routes_focus_close_minimize() {
        // With test stub, commands won't spawn; ensure routing surfaces as Ok
        assert!(launch("focus:abc", None).is_ok());
        assert!(launch("close-window:abc", None).is_ok());
        assert!(launch("minimize-window:abc", None).is_ok());
        assert!(launch("move-window-current:abc", None).is_ok());
    }

    #[test]
    fn blocking_launch_invalid_command_errors() {
        let err = launch_blocking("definitely-not-a-real-command", &[], None).unwrap_err();
        assert!(err.contains("Failed to run"));
    }
}

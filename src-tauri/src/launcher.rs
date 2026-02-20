use std::process::Command;
use tauri::Emitter;

// Handles .desktop Exec placeholders (like %u, %F) so we don't pass garbage to the shell.
pub fn launch(exec: &str, app_handle: Option<&tauri::AppHandle>) -> Result<(), String> {
    let start = std::time::Instant::now();

    if exec.starts_with("install:") {
        let url = exec.trim_start_matches("install:").to_string();
        if url.is_empty() {
            return Err("No URL provided".to_string());
        }

        if let Some(handle) = app_handle {
            let _ = handle.emit(
                "download_status",
                serde_json::json!({"status": "downloading"}),
            );
            let app_handle_clone = handle.clone();
            std::thread::spawn(move || {
                if let Err(e) = crate::store::download_script(&url) {
                    log::error!("Failed to install script: {}", e);
                    let _ = app_handle_clone.emit(
                        "download_status",
                        serde_json::json!({"status": "failed", "error": e.to_string()}),
                    );
                } else {
                    log::info!("Script downloaded and installed successfully.");
                    let _ = app_handle_clone
                        .emit("download_status", serde_json::json!({"status": "success"}));
                }
            });
            return Ok(());
        } else {
            if let Err(e) = crate::store::download_script(&url) {
                log::error!("Failed to install script: {}", e);
                return Err(e);
            } else {
                log::info!("Script downloaded and installed successfully.");
                return Ok(());
            }
        }
    }

    // Check for window focus action
    if exec.starts_with("focus:") {
        let address = exec.trim_start_matches("focus:");
        // Try Hyprland focus
        if let Err(e) = Command::new("hyprctl")
            .arg("dispatch")
            .arg("focuswindow")
            .arg(format!("address:{}", address))
            .spawn()
        {
            log::warn!("Hyprland focus failed: {}", e);
        }

        // Try Sway focus (address is con_id)
        if let Err(e) = Command::new("swaymsg")
            .arg(format!("[con_id={}] focus", address))
            .spawn()
        {
            log::warn!("Sway focus failed: {}", e);
        }

        return Ok(());
    }

    let cleaned = strip_field_codes(exec);

    if cleaned.is_empty() {
        return Err("Empty exec command after parsing".to_string());
    }

    log::info!("Launching via sh -c: {}", cleaned);

    Command::new("sh")
        .arg("-c")
        .arg(&cleaned)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to spawn '{}': {}", cleaned, e))?;

    let elapsed = start.elapsed();
    log::debug!("Launch took {:?}", elapsed);

    Ok(())
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
}

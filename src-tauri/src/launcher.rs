use std::process::Command;

// Handles .desktop Exec placeholders (like %u, %F) so we don't pass garbage to the shell.
pub fn launch(exec: &str) -> Result<(), String> {
    let start = std::time::Instant::now();

    // Check if it's a file path first (for file search results)
    let path = std::path::Path::new(exec);
    if path.exists() && path.is_file() {
        open::that(exec).map_err(|e| format!("Failed to open file: {}", e))?;
        return Ok(());
    }

    // Check for window focus action
    if exec.starts_with("focus:") {
        let address = exec.trim_start_matches("focus:");
        // Try Hyprland focus
        let _ = Command::new("hyprctl")
            .arg("dispatch")
            .arg("focuswindow")
            .arg(format!("address:{}", address))
            .spawn();

        // Try Sway focus (address is con_id)
        let _ = Command::new("swaymsg")
            .arg(format!("[con_id={}] focus", address))
            .spawn();

        return Ok(());
    }

    let cleaned = strip_field_codes(exec);

    if cleaned.is_empty() {
        return Err("Empty exec command after parsing".to_string());
    }

    let parts: Vec<&str> = cleaned.split_whitespace().collect();
    let (cmd, args) = parts.split_first().ok_or("Invalid exec command")?;

    log::info!("Launching: {} {:?}", cmd, args);

    Command::new(cmd)
        .args(args.iter())
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to spawn '{}': {}", cmd, e))?;

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

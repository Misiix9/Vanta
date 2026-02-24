use crate::config;
use crate::scripts;
use crate::DoctorArgs;
use std::env;
use std::fs;
use std::path::Path;

/// Runs `vanta doctor`, performing static checks and schema validation against installed scripts.
/// Returns an error if any script fails validation.
pub fn run(args: DoctorArgs) -> Result<(), String> {
    let strict_schema = args.strict;

    let config = config::load_or_create_default();
    let script_timeout = config.scripts.timeout_ms;

    let entries = scripts::scan_scripts();
    if entries.is_empty() {
        println!("No scripts found in ~/.config/vanta/scripts. Add scripts to validate.");
        return Ok(());
    }

    let mut failures = 0u32;

    for entry in entries {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        let _timeout_guard = args
            .simulate_timeout
            .map(|ms| EnvGuard::new("VANTA_SIMULATE_TIMEOUT", ms.to_string()));

        let _error_guard = args
            .simulate_error
            .as_ref()
            .and_then(|target| {
                if target == &entry.keyword {
                    Some(EnvGuard::new("VANTA_SIMULATE_ERROR", target.clone()))
                } else {
                    None
                }
            });

        let path = Path::new(&entry.path);
        if !path.exists() {
            errors.push("Script path does not exist".to_string());
        }

        if !path.is_file() {
            errors.push("Script path is not a file".to_string());
        }

        // Ensure executable bit on Unix.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(meta) = fs::metadata(path) {
                if meta.permissions().mode() & 0o111 == 0 {
                    errors.push("Script is not executable".to_string());
                }
            }
        }

        // Keyword must match filename stem.
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            if stem != entry.keyword {
                errors.push(format!(
                    "Keyword '{}' does not match filename '{}'. Expected stem to match keyword.",
                    entry.keyword, stem
                ));
            }
        }

        if entry.name.is_none() {
            warnings.push("Missing vanta:name metadata".to_string());
        }
        if entry.description.is_none() {
            warnings.push("Missing vanta:description metadata".to_string());
        }

        // Attempt to execute the script to validate JSON structure.
        match scripts::execute_script(&entry.keyword, "", script_timeout, &[]) {
            Ok(output) => match scripts::validate_script_output(&output) {
                Ok(()) => {}
                Err(msg) => {
                    if strict_schema {
                        errors.push(format!("Schema validation failed: {}", msg));
                    } else {
                        warnings.push(format!("Schema validation warning: {}", msg));
                    }
                }
            },
            Err(e) => errors.push(format!("Execution failed: {}", e)),
        }

        if errors.is_empty() && warnings.is_empty() {
            println!("✔ {} — OK", entry.keyword);
        } else {
            println!("✖ {}", entry.keyword);
            for err in &errors {
                println!("  [error] {}", err);
            }
            for warn in &warnings {
                println!("  [warn ] {}", warn);
            }
        }

        if !errors.is_empty() {
            failures += 1;
        }
    }

    if failures > 0 {
        Err(format!("{} script(s) failed validation", failures))
    } else {
        println!("All scripts validated successfully.");
        Ok(())
    }
}

struct EnvGuard {
    key: &'static str,
    previous: Option<String>,
    active: bool,
}

impl EnvGuard {
    fn new(key: &'static str, value: String) -> Self {
        let previous = env::var(key).ok();
        env::set_var(key, value);
        Self {
            key,
            previous,
            active: true,
        }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        if !self.active {
            return;
        }

        if let Some(ref prev) = self.previous {
            env::set_var(self.key, prev);
        } else {
            env::remove_var(self.key);
        }
    }
}
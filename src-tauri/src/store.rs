use std::fs;
use std::io::copy;
use std::io::Cursor;
use std::io::Read;
use std::time::Duration;
use zip::ZipArchive;

pub fn download_script(url: &str) -> Result<(), String> {
    log::info!("Fetching script from: {}", url);
    const MAX_DOWNLOAD_BYTES: u64 = 25 * 1024 * 1024; // 25 MB
    const DOWNLOAD_TIMEOUT_SECS: u64 = 20;

    let config_dir = dirs::config_dir().ok_or("No config dir")?;
    let scripts_dir = config_dir.join("vanta").join("scripts");
    if !scripts_dir.exists() {
        fs::create_dir_all(&scripts_dir).map_err(|e| e.to_string())?;
    }

    // --- LOCAL FILE HANDLING ---
    let local_path = std::path::Path::new(url);
    if local_path.exists() {
        log::info!("Installing from local path: {:?}", local_path);

        let ext = local_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        // If it's a known archive, extract it
        if ext.eq_ignore_ascii_case("zip") {
            // Very basic zip extraction for local files (same logic as remote)
            let file = fs::File::open(local_path).map_err(|e| e.to_string())?;
            let mut archive = ZipArchive::new(file)
                .map_err(|e| format!("Failed to read local archive: {}", e))?;

            let mut extracted = 0;
            for i in 0..archive.len() {
                let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
                let outpath = match file.enclosed_name() {
                    Some(path) => path.to_owned(),
                    None => continue,
                };

                let file_name = outpath.file_name().unwrap_or_default();
                if file.name().ends_with('/') || file_name.is_empty() {
                    continue;
                }

                let out_file_path = scripts_dir.join(file_name);
                let mut outfile = fs::File::create(&out_file_path).map_err(|e| e.to_string())?;
                copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
                log::info!("Extracted {:?}", out_file_path);
                extracted += 1;

                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = fs::metadata(&out_file_path)
                        .map_err(|e| e.to_string())?
                        .permissions();
                    perms.set_mode(0o755);
                    fs::set_permissions(&out_file_path, perms).map_err(|e| e.to_string())?;
                }
            }
            if extracted == 0 {
                return Err("No files extracted from the local archive.".to_string());
            }
            return Ok(());
        } else {
            // Just copy the single script file
            let file_name = local_path.file_name().ok_or("Invalid file name")?;
            let is_css = ext.eq_ignore_ascii_case("css");
            let target_dir = if is_css {
                config_dir.join("vanta").join("themes")
            } else {
                scripts_dir.clone()
            };
            if is_css && !target_dir.exists() {
                fs::create_dir_all(&target_dir).map_err(|e| e.to_string())?;
            }

            let out_file_path = target_dir.join(file_name);
            fs::copy(local_path, &out_file_path)
                .map_err(|e| format!("Failed to copy local file: {}", e))?;
            log::info!("Copied {:?}", out_file_path);

            if !is_css {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = fs::metadata(&out_file_path)
                        .map_err(|e| e.to_string())?
                        .permissions();
                    perms.set_mode(0o755);
                    fs::set_permissions(&out_file_path, perms).map_err(|e| e.to_string())?;
                }
            }
            return Ok(());
        }
    }

    // --- REMOTE DOWNLOAD HANDLING ---
    let is_remote_css = url.ends_with(".css");

    let download_url = if !is_remote_css && url.contains("github.com") && !url.ends_with(".zip") {
        format!("{}/archive/refs/heads/main.zip", url.trim_end_matches('/'))
    } else {
        url.to_string()
    };

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(DOWNLOAD_TIMEOUT_SECS))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    let response = client
        .get(&download_url)
        .send()
        .map_err(|e| format!("Download failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Failed to download: {}", response.status()));
    }

    if let Some(content_len) = response.content_length() {
        if content_len > MAX_DOWNLOAD_BYTES {
            return Err(format!(
                "Download too large: {} bytes (max {} bytes)",
                content_len, MAX_DOWNLOAD_BYTES
            ));
        }
    }

    let mut bytes = Vec::new();
    response
        .take(MAX_DOWNLOAD_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("Failed to read download body: {}", e))?;

    if bytes.len() as u64 > MAX_DOWNLOAD_BYTES {
        return Err(format!(
            "Download exceeded max size of {} bytes",
            MAX_DOWNLOAD_BYTES
        ));
    }

    // If it's a direct CSS file, drop it down immediately
    if is_remote_css {
        let themes_dir = config_dir.join("vanta").join("themes");
        if !themes_dir.exists() {
            fs::create_dir_all(&themes_dir).map_err(|e| e.to_string())?;
        }
        let file_name = url.split('/').last().unwrap_or("theme.css");
        let out_file_path = themes_dir.join(file_name);
        fs::write(&out_file_path, &bytes).map_err(|e| e.to_string())?;
        log::info!("Downloaded theme to {:?}", out_file_path);
        return Ok(());
    }

    let reader = Cursor::new(bytes);
    let mut archive = ZipArchive::new(reader).map_err(|e| e.to_string())?;

    let mut extracted = 0;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        let outpath = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };

        let file_name = outpath.file_name().unwrap_or_default();
        if file.name().ends_with('/') || file_name.is_empty() {
            continue;
        }

        let out_file_path = scripts_dir.join(file_name);
        let mut outfile = fs::File::create(&out_file_path).map_err(|e| e.to_string())?;
        copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
        log::info!("Extracted {:?}", out_file_path);
        extracted += 1;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&out_file_path)
                .map_err(|e| e.to_string())?
                .permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&out_file_path, perms).map_err(|e| e.to_string())?;
        }
    }

    if extracted == 0 {
        return Err("No files extracted from the archive.".to_string());
    }

    Ok(())
}

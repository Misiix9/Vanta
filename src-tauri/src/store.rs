use crate::bundles::{validate_bundle_archive, BundleManifest, ValidatedBundle};
use log::info;
use reqwest::blocking::Client;
use std::fs::{self, File};
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use std::time::Duration;

const MAX_DOWNLOAD_BYTES: u64 = 25 * 1024 * 1024; // 25 MB
const DOWNLOAD_TIMEOUT_SECS: u64 = 20;

fn config_root() -> Result<PathBuf, String> {
    if let Ok(dir) = std::env::var("VANTA_CONFIG_DIR") {
        if !dir.is_empty() {
            return Ok(PathBuf::from(dir));
        }
    }

    dirs::config_dir().ok_or_else(|| "No config dir".to_string())
}

fn ensure_dir(path: &Path) -> Result<(), String> {
    if !path.exists() {
        fs::create_dir_all(path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn set_exec_perms(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path).map_err(|e| e.to_string())?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn persist_manifest(manifest: &BundleManifest, scripts_dir: &Path) -> Result<(), String> {
    let manifest_path = scripts_dir.join(format!("{}.manifest.json", manifest.name));
    let json = serde_json::to_vec_pretty(manifest).map_err(|e| e.to_string())?;
    fs::write(&manifest_path, json).map_err(|e| e.to_string())
}

fn install_validated_bundle(bundle: ValidatedBundle, scripts_dir: &Path) -> Result<(), String> {
    for file in &bundle.files {
        let dest = scripts_dir.join(&file.relative_path);
        if let Some(parent) = dest.parent() {
            ensure_dir(parent)?;
        }
        fs::copy(&file.staging_path, &dest)
            .map_err(|e| format!("Failed to copy {}: {}", file.relative_path, e))?;
        set_exec_perms(&dest)?;
    }

    persist_manifest(&bundle.manifest, scripts_dir)?;
    Ok(())
}

pub fn download_script(url: &str) -> Result<(), String> {
    info!("Fetching script from: {}", url);

    let config_root = config_root()?;
    let vanta_root = config_root.join("vanta");
    let scripts_dir = vanta_root.join("scripts");
    ensure_dir(&scripts_dir)?;

    // --- LOCAL FILE HANDLING ---
    let local_path = Path::new(url);
    if local_path.exists() {
        let ext = local_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();

        if ext == "css" {
            let themes_dir = vanta_root.join("themes");
            ensure_dir(&themes_dir)?;
            let file_name = local_path.file_name().ok_or("Invalid file name")?;
            let out = themes_dir.join(file_name);
            fs::copy(local_path, &out).map_err(|e| format!("Failed to copy local theme: {}", e))?;
            return Ok(());
        }

        if ext == "zip" {
            let file = File::open(local_path).map_err(|e| e.to_string())?;
            let reader = std::io::BufReader::new(file);
            let bundle = validate_bundle_archive(reader).map_err(|e| e.to_string())?;
            install_validated_bundle(bundle, &scripts_dir)?;
            return Ok(());
        }

        return Err("Local installs must be a signed .zip bundle or a .css theme".to_string());
    }

    // --- REMOTE DOWNLOAD HANDLING ---
    let is_remote_css = url.to_ascii_lowercase().ends_with(".css");
    let download_url = if !is_remote_css && url.contains("github.com") && !url.ends_with(".zip") {
        format!("{}/archive/refs/heads/main.zip", url.trim_end_matches('/'))
    } else {
        url.to_string()
    };

    let client = Client::builder()
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

    if is_remote_css {
        let themes_dir = vanta_root.join("themes");
        ensure_dir(&themes_dir)?;
        let file_name = url.split('/').last().unwrap_or("theme.css");
        let out_file_path = themes_dir.join(file_name);
        fs::write(&out_file_path, &bytes).map_err(|e| e.to_string())?;
        return Ok(());
    }

    let reader = Cursor::new(bytes);
    let bundle = validate_bundle_archive(reader).map_err(|e| e.to_string())?;
    install_validated_bundle(bundle, &scripts_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{SigningKey, Signer};
    use httpmock::prelude::*;
    use sha2::{Digest, Sha256};
    use std::env;
    use std::io::Write as _;
    use serial_test::serial;
    use tempfile::tempdir;
    use zip::write::FileOptions;
    use zip::ZipWriter;

    fn set_test_config_dir(dir: &Path) {
        env::set_var("VANTA_CONFIG_DIR", dir.to_string_lossy().to_string());
    }

    fn signing_key() -> SigningKey {
        SigningKey::from_bytes(&[7u8; 32])
    }

    fn sha256_hex(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }

    fn build_bundle(file_name: &str, contents: &[u8]) -> Vec<u8> {
        let sha = sha256_hex(contents);
        let manifest = serde_json::json!({
            "name": "demo",
            "version": "1.0.0",
            "files": [ {"path": file_name, "sha256": sha} ]
        });

        let manifest_str = manifest.to_string();
        let sig = signing_key().sign(manifest_str.as_bytes()).to_bytes();

        let mut buf = Vec::new();
        {
            let cursor = Cursor::new(&mut buf);
            let mut writer = ZipWriter::new(cursor);
            let options = FileOptions::default();

            writer.start_file("manifest.json", options).unwrap();
            writer.write_all(manifest_str.as_bytes()).unwrap();

            writer.start_file("manifest.sig", options).unwrap();
            writer.write_all(&sig).unwrap();

            writer.start_file(file_name, options).unwrap();
            writer.write_all(contents).unwrap();

            writer.finish().unwrap();
        }

        buf
    }

    #[test]
    #[serial]
    fn installs_local_bundle_to_config_dir() {
        let temp = tempdir().unwrap();
        set_test_config_dir(temp.path());

        let bundle_bytes = build_bundle("script.sh", b"echo hi");
        let bundle_path = temp.path().join("bundle.zip");
        fs::write(&bundle_path, &bundle_bytes).unwrap();

        download_script(bundle_path.to_str().unwrap()).expect("install should succeed");

        let installed = temp
            .path()
            .join("vanta")
            .join("scripts")
            .join("script.sh");
        assert!(installed.exists());
    }

    #[test]
    #[serial]
    fn installs_remote_bundle() {
        let temp = tempdir().unwrap();
        set_test_config_dir(temp.path());

        let server = MockServer::start();
        let bundle_bytes = build_bundle("run.sh", b"echo remote");
        server.mock(|when, then| {
            when.method(GET).path("/bundle.zip");
            then.status(200)
                .header("content-type", "application/zip")
                .body(bundle_bytes.clone());
        });

        download_script(&format!("{}/bundle.zip", server.base_url()))
            .expect("remote install should succeed");

        let installed = temp
            .path()
            .join("vanta")
            .join("scripts")
            .join("run.sh");
        assert!(installed.exists());
    }

    #[test]
    #[serial]
    fn rejects_bad_signature() {
        let temp = tempdir().unwrap();
        set_test_config_dir(temp.path());

        let mut buf = Vec::new();
        {
            let cursor = Cursor::new(&mut buf);
            let mut writer = ZipWriter::new(cursor);
            let options = FileOptions::default();
            writer.start_file("manifest.json", options).unwrap();
            writer.write_all(b"{}" as &[u8]).unwrap();
            writer.start_file("manifest.sig", options).unwrap();
            writer.write_all(&[0u8; 64]).unwrap();
            writer.finish().unwrap();
        }
        let bundle_path = temp.path().join("invalid.zip");
        fs::write(&bundle_path, &buf).unwrap();

        let result = download_script(bundle_path.to_str().unwrap());
        assert!(result.is_err());
    }

    #[test]
    #[serial]
    fn installs_local_css_theme() {
        let temp = tempdir().unwrap();
        set_test_config_dir(temp.path());

        let css_path = temp.path().join("theme.css");
        fs::write(&css_path, b"body { background: #000; }").unwrap();

        download_script(css_path.to_str().unwrap()).expect("css install should succeed");

        let installed = temp
            .path()
            .join("vanta")
            .join("themes")
            .join("theme.css");
        assert!(installed.exists());
    }
}

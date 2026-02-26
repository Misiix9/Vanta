use base64::prelude::*;
use ed25519_dalek::{Signature, VerifyingKey, Verifier};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Seek, Write};
use std::path::PathBuf;
use tempfile::TempDir;
use zip::ZipArchive;

static BUNDLE_PUBKEY_BASE64: &str = include_str!("../resources/bundle_pubkey.txt");
static BUNDLE_PUBKEY: OnceCell<VerifyingKey> = OnceCell::new();

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BundleFile {
    pub path: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BundleManifest {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub source_url: Option<String>,
    pub created_at: Option<String>,
    pub files: Vec<BundleFile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedFile {
    pub relative_path: String,
    pub staging_path: PathBuf,
    pub sha256: String,
}

#[derive(Debug)]
pub struct ValidatedBundle {
    pub manifest: BundleManifest,
    pub staging: TempDir,
    pub files: Vec<ValidatedFile>,
}
#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum BundleError {
    #[error("Manifest file missing")]
    MissingManifest,
    #[error("Missing manifest signature")]
    MissingSignature,
    #[error("Invalid public key")]
    InvalidPublicKey,
    #[error("Signature verification failed")]
    InvalidSignature,
    #[error("Manifest parse failed: {0}")]
    ManifestParse(String),
    #[error("Unsupported manifest version")]
    UnsupportedVersion,
    #[error("Bundle read failed: {0}")]
    BundleRead(String),
    #[error("Path traversal detected: {0}")]
    PathTraversal(String),
    #[error("File missing: {0}")]
    FileMissing(String),
    #[error("Hash mismatch for {path}")]
    HashMismatch { path: String },
    #[error("Internal error: {0}")]
    Internal(String),
}

fn load_pubkey() -> Result<VerifyingKey, BundleError> {
    BUNDLE_PUBKEY
        .get_or_try_init(|| {
            let raw = BUNDLE_PUBKEY_BASE64.trim();
            let decoded: [u8; 32] = BASE64_STANDARD
                .decode(raw)
                .map_err(|_| BundleError::InvalidPublicKey)?
                .try_into()
                .map_err(|_| BundleError::InvalidPublicKey)?;
            VerifyingKey::from_bytes(&decoded)
                .map_err(|_| BundleError::InvalidPublicKey)
        })
        .cloned()
}

/// Verify manifest bytes and signature, returning the parsed manifest.
pub fn verify_manifest(manifest_bytes: &[u8], sig_bytes: &[u8]) -> Result<BundleManifest, BundleError> {
    if sig_bytes.is_empty() {
        return Err(BundleError::MissingSignature);
    }

    let manifest: BundleManifest = serde_json::from_slice(manifest_bytes)
        .map_err(|e| BundleError::ManifestParse(e.to_string()))?;

    if manifest.files.is_empty() {
        return Err(BundleError::ManifestParse("files must not be empty".to_string()));
    }

    if manifest.version.trim().is_empty() {
        return Err(BundleError::UnsupportedVersion);
    }

    let pubkey = load_pubkey()?;
    let signature = Signature::from_slice(sig_bytes).map_err(|_| BundleError::InvalidSignature)?;
    pubkey
        .verify(manifest_bytes, &signature)
        .map_err(|_| BundleError::InvalidSignature)?;

    Ok(manifest)
}

/// Validate a bundle archive (zip) by verifying manifest signature, blocking
/// traversal, hashing files, and staging them into a temporary directory.
pub fn validate_bundle_archive<R: Read + Seek>(reader: R) -> Result<ValidatedBundle, BundleError> {
    let mut archive = ZipArchive::new(reader).map_err(|e| BundleError::BundleRead(e.to_string()))?;

    let mut manifest_bytes: Option<Vec<u8>> = None;
    let mut sig_bytes: Option<Vec<u8>> = None;
    let staging = TempDir::new().map_err(|e: std::io::Error| BundleError::Internal(e.to_string()))?;
    let staging_root = staging.path().to_path_buf();
    let mut files: Vec<ValidatedFile> = Vec::new();

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| BundleError::BundleRead(e.to_string()))?;

        let name = file.name().to_string();
        if name.ends_with('/') {
            continue; // skip directories
        }

        if name == "manifest.json" {
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)
                .map_err(|e| BundleError::BundleRead(e.to_string()))?;
            manifest_bytes = Some(buf);
            continue;
        }

        if name == "manifest.sig" {
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)
                .map_err(|e| BundleError::BundleRead(e.to_string()))?;
            sig_bytes = Some(buf);
            continue;
        }

        let enclosed = file
            .enclosed_name()
            .ok_or_else(|| BundleError::PathTraversal(name.clone()))?
            .to_path_buf();

        if enclosed.is_absolute() {
            return Err(BundleError::PathTraversal(name));
        }

        let rel = enclosed.to_string_lossy().to_string();
        let out_path = staging_root.join(&rel);
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| BundleError::Internal(e.to_string()))?;
        }

        let mut data = Vec::new();
        file.read_to_end(&mut data)
            .map_err(|e| BundleError::BundleRead(e.to_string()))?;
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let sha = hex::encode(hasher.finalize());

        let mut outfile = File::create(&out_path).map_err(|e| BundleError::Internal(e.to_string()))?;
        outfile
            .write_all(&data)
            .map_err(|e| BundleError::Internal(e.to_string()))?;

        files.push(ValidatedFile {
            relative_path: rel,
            staging_path: out_path,
            sha256: sha,
        });
    }

    let manifest_bytes = manifest_bytes.ok_or(BundleError::MissingManifest)?;
    let sig_bytes = sig_bytes.ok_or(BundleError::MissingSignature)?;
    let manifest = verify_manifest(&manifest_bytes, &sig_bytes)?;

    for expected in &manifest.files {
        if let Some(found) = files.iter().find(|f| f.relative_path == expected.path) {
            if found.sha256.to_lowercase() != expected.sha256.to_lowercase() {
                return Err(BundleError::HashMismatch {
                    path: expected.path.clone(),
                });
            }
        } else {
            return Err(BundleError::FileMissing(expected.path.clone()));
        }
    }

    Ok(ValidatedBundle {
        manifest,
        staging,
        files,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{SigningKey, Signer, VerifyingKey};
    use std::io::Cursor;

    fn signing_key() -> SigningKey {
        SigningKey::from_bytes(&[7u8; 32])
    }

    fn ensure_pubkey_set(verifying: &VerifyingKey) {
        let _ = BUNDLE_PUBKEY.set(verifying.clone());
    }

    fn sha256_hex(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }

    fn make_manifest(files: &[(&str, &str)]) -> (String, Vec<u8>, VerifyingKey) {
        let signing = signing_key();
        let verifying = signing.verifying_key();
        let files_json: Vec<BundleFile> = files
            .iter()
            .map(|(path, sha)| BundleFile {
                path: path.to_string(),
                sha256: sha.to_string(),
            })
            .collect();

        let manifest = BundleManifest {
            name: "demo".to_string(),
            version: "1.0.0".to_string(),
            description: Some("test".to_string()),
            source_url: None,
            created_at: None,
            files: files_json,
        };

        let manifest_json = serde_json::to_string(&manifest).unwrap();
        let sig = signing.sign(manifest_json.as_bytes()).to_bytes().to_vec();
        (manifest_json, sig, verifying)
    }

    fn build_zip(manifest: &str, sig: &[u8], files: &[(&str, &[u8])]) -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let cursor = Cursor::new(&mut buf);
            let mut writer = zip::ZipWriter::new(cursor);
            let options = zip::write::FileOptions::default();

            writer.start_file("manifest.json", options).unwrap();
            writer.write_all(manifest.as_bytes()).unwrap();

            writer.start_file("manifest.sig", options).unwrap();
            writer.write_all(sig).unwrap();

            for &(name, data) in files {
                writer.start_file(name, options).unwrap();
                writer.write_all(data).unwrap();
            }

            writer.finish().unwrap();
        }
        buf
    }

    #[test]
    fn verifies_manifest_signature() {
        let (manifest_json, sig, verifying) = make_manifest(&[("script.sh", "deadbeef")]);
        ensure_pubkey_set(&verifying);
        let result = verify_manifest(manifest_json.as_bytes(), &sig);
        assert!(result.is_ok(), "verify_manifest failed: {:?}", result);
    }

    #[test]
    fn validates_bundle_archive_success() {
        let script = b"echo hi";
        let sha = sha256_hex(script);
        let (manifest_json, sig, verifying) = make_manifest(&[("script.sh", &sha)]);
        ensure_pubkey_set(&verifying);
        let zip_bytes = build_zip(&manifest_json, &sig, &[("script.sh", script)]);

        let validated = validate_bundle_archive(Cursor::new(zip_bytes)).expect("bundle should validate");
        assert_eq!(validated.manifest.name, "demo");
        assert_eq!(validated.files.len(), 1);
        assert!(validated.files[0].staging_path.exists());
    }

    #[test]
    fn rejects_missing_signature() {
        let script = b"echo hi";
        let sha = sha256_hex(script);
        let (manifest_json, _sig, verifying) = make_manifest(&[("script.sh", &sha)]);
        ensure_pubkey_set(&verifying);
        let mut zip_bytes = Vec::new();
        {
            let cursor = Cursor::new(&mut zip_bytes);
            let mut writer = zip::ZipWriter::new(cursor);
            let options = zip::write::FileOptions::default();
            writer.start_file("manifest.json", options).unwrap();
            writer.write_all(manifest_json.as_bytes()).unwrap();
            writer.start_file("script.sh", options).unwrap();
            writer.write_all(script).unwrap();
            writer.finish().unwrap();
        }

        let result = validate_bundle_archive(Cursor::new(zip_bytes));
        assert!(matches!(result, Err(BundleError::MissingSignature)));
    }

    #[test]
    fn rejects_signature_mismatch() {
        let script = b"echo hi";
        let sha = sha256_hex(script);
        let (manifest_json, _sig, verifying) = make_manifest(&[("script.sh", &sha)]);
        ensure_pubkey_set(&verifying);
        let bad_sig = vec![0u8; 64];
        let zip_bytes = build_zip(&manifest_json, &bad_sig, &[("script.sh", script)]);

        let result = validate_bundle_archive(Cursor::new(zip_bytes));
        assert!(matches!(result, Err(BundleError::InvalidSignature)));
    }

    #[test]
    fn rejects_hash_mismatch() {
        let script = b"echo hi";
        let sha = sha256_hex(b"wrong");
        let (manifest_json, sig, verifying) = make_manifest(&[("script.sh", &sha)]);
        ensure_pubkey_set(&verifying);
        let zip_bytes = build_zip(&manifest_json, &sig, &[("script.sh", script)]);

        let result = validate_bundle_archive(Cursor::new(zip_bytes));
        assert!(matches!(result, Err(BundleError::HashMismatch { .. })));
    }

    #[test]
    fn rejects_path_traversal() {
        let script = b"echo hi";
        let sha = sha256_hex(script);
        let (manifest_json, sig, verifying) = make_manifest(&[("../evil.sh", &sha)]);
        ensure_pubkey_set(&verifying);
        let zip_bytes = build_zip(&manifest_json, &sig, &[("../evil.sh", script)]);

        let result = validate_bundle_archive(Cursor::new(zip_bytes));
        assert!(matches!(result, Err(BundleError::PathTraversal(_))));
    }
}

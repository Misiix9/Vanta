use base64::prelude::*;
use ed25519_dalek::{Signature, VerifyingKey, Verifier};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};

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

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum BundleError {
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

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signature, SigningKey, Signer, VerifyingKey};

    #[test]
    fn parses_manifest_and_verifies_signature() {
        // Deterministic key for fixtures
        let seed = [7u8; 32];
        let signing = SigningKey::from_bytes(&seed);
        let verifying: VerifyingKey = signing.verifying_key();
        let manifest_json = r#"{
            "name": "demo",
            "version": "1.0.0",
            "description": "test bundle",
            "files": [
                {"path": "script.sh", "sha256": "deadbeef"}
            ]
        }"#;

        let sig: Signature = signing.sign(manifest_json.as_bytes());
        let sig_bytes = sig.to_bytes();

        // Install test pubkey into global so verification uses our deterministic key
        let _ = BUNDLE_PUBKEY.set(verifying.clone());

        let result = verify_manifest(manifest_json.as_bytes(), &sig_bytes);
        assert!(result.is_ok(), "verify_manifest failed: {:?} (pk {:?})", result, verifying.to_bytes());
        let parsed = result.unwrap();
        assert_eq!(parsed.name, "demo");
        assert_eq!(parsed.version, "1.0.0");
    }
}

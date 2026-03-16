use std::collections::HashMap;

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use rand::RngCore;
use serde::{Deserialize, Serialize};

use crate::config;
use crate::errors::VantaError;

const SECRET_MASK: &str = "********";

#[derive(Clone, Debug, Serialize, Deserialize)]
struct EncryptedSecretsFile {
    version: u32,
    nonce_b64: String,
    ciphertext_b64: String,
}

fn key_path() -> std::path::PathBuf {
    config::config_dir().join("workflow-secrets.key")
}

fn secrets_path() -> std::path::PathBuf {
    config::config_dir().join("workflow-secrets.json")
}

fn ensure_secret_name(name: &str) -> Result<(), VantaError> {
    if name.is_empty() {
        return Err("Secret name cannot be empty".into());
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        return Err("Secret name can only contain letters, numbers, '-', '_' and '.'".into());
    }
    Ok(())
}

fn read_or_create_key() -> Result<[u8; 32], VantaError> {
    let path = key_path();
    if path.exists() {
        let raw = std::fs::read(&path)
            .map_err(|e| format!("Failed to read secrets key: {}", e))?;
        if raw.len() != 32 {
            return Err("Invalid secrets key length".into());
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(&raw);
        return Ok(key);
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create secrets directory: {}", e))?;
    }

    let mut key = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut key);
    std::fs::write(&path, key)
        .map_err(|e| format!("Failed to write secrets key: {}", e))?;
    Ok(key)
}

fn load_plain_map() -> Result<HashMap<String, String>, VantaError> {
    let path = secrets_path();
    if !path.exists() {
        return Ok(HashMap::new());
    }

    let raw = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read secrets store: {}", e))?;
    if raw.trim().is_empty() {
        return Ok(HashMap::new());
    }

    let encrypted: EncryptedSecretsFile = serde_json::from_str(&raw)
        .map_err(|e| format!("Failed to parse secrets store: {}", e))?;

    let key = read_or_create_key()?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| format!("Failed to initialize secrets cipher: {}", e))?;

    let nonce_bytes = B64
        .decode(encrypted.nonce_b64)
        .map_err(|e| format!("Invalid secrets nonce encoding: {}", e))?;
    if nonce_bytes.len() != 12 {
        return Err("Invalid secrets nonce length".into());
    }
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = B64
        .decode(encrypted.ciphertext_b64)
        .map_err(|e| format!("Invalid secrets ciphertext encoding: {}", e))?;

    let plain = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|_| "Failed to decrypt secrets store".to_string())?;

    let map = serde_json::from_slice::<HashMap<String, String>>(&plain)
        .map_err(|e| format!("Failed to decode secrets payload: {}", e))?;
    Ok(map)
}

fn save_plain_map(map: &HashMap<String, String>) -> Result<(), VantaError> {
    let key = read_or_create_key()?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| format!("Failed to initialize secrets cipher: {}", e))?;

    let mut nonce_bytes = [0u8; 12];
    rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let plaintext = serde_json::to_vec(map)
        .map_err(|e| format!("Failed to serialize secrets payload: {}", e))?;
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|_| "Failed to encrypt secrets payload".to_string())?;

    let payload = EncryptedSecretsFile {
        version: 1,
        nonce_b64: B64.encode(nonce_bytes),
        ciphertext_b64: B64.encode(ciphertext),
    };

    let out = serde_json::to_string_pretty(&payload)
        .map_err(|e| format!("Failed to encode secrets file: {}", e))?;
    let path = secrets_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create secrets directory: {}", e))?;
    }
    std::fs::write(path, out).map_err(|e| format!("Failed to write secrets store: {}", e))?;
    Ok(())
}

pub fn set_secret(name: &str, value: &str) -> Result<(), VantaError> {
    ensure_secret_name(name)?;
    let mut map = load_plain_map()?;
    map.insert(name.to_string(), value.to_string());
    save_plain_map(&map)
}

pub fn delete_secret(name: &str) -> Result<bool, VantaError> {
    ensure_secret_name(name)?;
    let mut map = load_plain_map()?;
    let removed = map.remove(name).is_some();
    save_plain_map(&map)?;
    Ok(removed)
}

pub fn list_secret_names() -> Result<Vec<String>, VantaError> {
    let mut names = load_plain_map()?.into_keys().collect::<Vec<_>>();
    names.sort();
    Ok(names)
}

pub fn secret_tokens_plain() -> Result<HashMap<String, String>, VantaError> {
    let map = load_plain_map()?;
    Ok(map
        .into_iter()
        .map(|(k, v)| (format!("secret.{}", k), v))
        .collect())
}

pub fn secret_tokens_masked() -> Result<HashMap<String, String>, VantaError> {
    let names = list_secret_names()?;
    let mut out = HashMap::new();
    for name in names {
        out.insert(format!("secret.{}", name), SECRET_MASK.to_string());
    }
    Ok(out)
}

pub fn secret_values_plain() -> Result<Vec<String>, VantaError> {
    Ok(load_plain_map()?.into_values().collect())
}

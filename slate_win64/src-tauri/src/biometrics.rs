use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand::rngs::OsRng;
use std::fs;
use std::path::PathBuf;

use crate::filesystem::get_slate_dir;

fn get_vault_config_path() -> PathBuf {
    get_slate_dir().join("vault_config.json")
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct VaultConfig {
    /// Argon2id PHC string (`$argon2id$...`)
    pub password_hash: String,
}

pub fn validate_master_password(password: &str) -> Result<(), String> {
    if password.len() < 8 {
        return Err("Master password must be at least 8 characters".to_string());
    }
    Ok(())
}

pub fn config_contains_recovery_wrap() -> Result<bool, String> {
    let config_path = get_vault_config_path();
    if !config_path.exists() {
        return Ok(false);
    }
    let json = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read vault config: {}", e))?;
    let value: serde_json::Value = serde_json::from_str(&json)
        .map_err(|e| format!("Failed to parse vault config: {}", e))?;
    Ok(json_has_recovery_wrap(&value))
}

fn json_has_recovery_wrap(value: &serde_json::Value) -> bool {
    let Some(obj) = value.as_object() else {
        return false;
    };
    ["recovery_hash", "wrap_salt", "wrap_nonce", "wrapped_file_key"]
        .iter()
        .any(|key| obj.get(*key).is_some_and(|item| !item.is_null()))
}

pub fn set_master_password(password: &str) -> Result<(), String> {
    validate_master_password(password)?;

    let config_path = get_vault_config_path();
    let hashed = hash_master_password(password)?;

    if config_path.exists() {
        let json = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read vault config: {}", e))?;
        let value: serde_json::Value = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse vault config: {}", e))?;
        if json_has_recovery_wrap(&value) {
            return Err(
                "Refusing to rewrite vault config that contains a recovery wrap".to_string(),
            );
        }
    }

    let config = VaultConfig {
        password_hash: hashed,
    };

    let json = serde_json::to_string(&config)
        .map_err(|e| format!("Failed to serialize vault config: {}", e))?;

    fs::write(config_path, json).map_err(|e| format!("Failed to write vault config: {}", e))?;

    Ok(())
}

pub fn verify_master_password(password: &str) -> Result<bool, String> {
    let config_path = get_vault_config_path();
    if !config_path.exists() {
        return Ok(false);
    }

    let json = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read vault config: {}", e))?;

    let config: VaultConfig = serde_json::from_str(&json)
        .map_err(|e| format!("Failed to parse vault config: {}", e))?;

    // Migrate legacy unsalted SHA-256 hashes to Argon2 on successful login
    if is_legacy_sha256_hash(&config.password_hash) {
        let legacy_ok = verify_legacy_sha256(password, &config.password_hash);
        if legacy_ok {
            set_master_password(password)?;
        }
        return Ok(legacy_ok);
    }

    let parsed = PasswordHash::new(&config.password_hash)
        .map_err(|e| format!("Invalid stored password hash: {}", e))?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

pub fn is_vault_initialized() -> bool {
    get_vault_config_path().exists()
}

fn hash_master_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| format!("Failed to hash master password: {}", e))?;
    Ok(hash.to_string())
}

fn is_legacy_sha256_hash(stored: &str) -> bool {
    stored.len() == 64 && stored.chars().all(|c| c.is_ascii_hexdigit())
}

fn verify_legacy_sha256(password: &str, stored_hex: &str) -> bool {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    let result = hasher.finalize();
    let computed: String = result.iter().map(|b| format!("{:02x}", b)).collect();
    // Constant-time compare
    if computed.len() != stored_hex.len() {
        return false;
    }
    let mut diff = 0u8;
    for (a, b) in computed.bytes().zip(stored_hex.bytes()) {
        diff |= a ^ b;
    }
    diff == 0
}

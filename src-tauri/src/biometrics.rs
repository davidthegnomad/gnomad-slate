use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand::rngs::OsRng;
use std::fs;
use std::path::{Path, PathBuf};

use crate::crypto::{self, RecoveryWrapRecord};
use crate::filesystem::get_slate_dir;

fn config_path_in(dir: &Path) -> PathBuf {
    dir.join("vault_config.json")
}

fn get_vault_config_path() -> PathBuf {
    config_path_in(&get_slate_dir())
}

pub fn vault_config_path() -> PathBuf {
    get_vault_config_path()
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct VaultConfig {
    /// Argon2id PHC string (`$argon2id$...`)
    pub password_hash: String,
    /// Argon2id verifier for the recovery key. Absent on vaults created before recovery keys.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recovery_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wrap_salt: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wrap_nonce: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wrapped_file_key: Option<String>,
    /// SHA-256 hex of the ciphertext this config was committed with.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ciphertext_sha256: Option<String>,
}

impl VaultConfig {
    pub fn password_only(password_hash: String) -> Self {
        Self {
            password_hash,
            recovery_hash: None,
            wrap_salt: None,
            wrap_nonce: None,
            wrapped_file_key: None,
            ciphertext_sha256: None,
        }
    }

    pub fn set_recovery(&mut self, recovery_hash: String, record: &RecoveryWrapRecord) {
        self.recovery_hash = Some(recovery_hash);
        self.apply_wrap(record);
    }

    pub fn apply_wrap(&mut self, record: &RecoveryWrapRecord) {
        self.wrap_salt = Some(crypto::encode_hex(&record.salt));
        self.wrap_nonce = Some(crypto::encode_hex(&record.nonce));
        self.wrapped_file_key = Some(crypto::encode_hex(&record.wrapped_key));
    }

    /// `Ok(None)` when this vault was created without a recovery wrap.
    pub fn wrap_record(&self) -> Result<Option<RecoveryWrapRecord>, String> {
        let has_any = self.recovery_hash.is_some()
            || self.wrap_salt.is_some()
            || self.wrap_nonce.is_some()
            || self.wrapped_file_key.is_some();
        if !has_any {
            return Ok(None);
        }
        let salt_hex = self
            .wrap_salt
            .as_deref()
            .ok_or("Vault recovery data is incomplete")?;
        let nonce_hex = self
            .wrap_nonce
            .as_deref()
            .ok_or("Vault recovery data is incomplete")?;
        let wrapped_hex = self
            .wrapped_file_key
            .as_deref()
            .ok_or("Vault recovery data is incomplete")?;
        if self.recovery_hash.is_none() {
            return Err("Vault recovery data is incomplete".to_string());
        }
        Ok(Some(RecoveryWrapRecord {
            salt: crypto::decode_hex_array(salt_hex)?,
            nonce: crypto::decode_hex_array(nonce_hex)?,
            wrapped_key: crypto::decode_hex(wrapped_hex)?,
        }))
    }
}

pub fn validate_master_password(password: &str) -> Result<(), String> {
    if password.len() < 8 {
        return Err("Master password must be at least 8 characters".to_string());
    }
    Ok(())
}

pub fn set_master_password(password: &str) -> Result<(), String> {
    set_master_password_in(&get_slate_dir(), password)
}

fn set_master_password_in(dir: &Path, password: &str) -> Result<(), String> {
    validate_master_password(password)?;
    let hashed = hash_verifier(password)?;

    let mut config = if is_vault_initialized_in(dir) {
        read_vault_config_at(&config_path_in(dir))?
    } else {
        VaultConfig::password_only(String::new())
    };
    config.password_hash = hashed;
    save_vault_config_at(&config_path_in(dir), &config)
}

pub fn verify_master_password(password: &str) -> Result<bool, String> {
    verify_master_password_in(&get_slate_dir(), password)
}

pub fn verify_master_password_in(dir: &Path, password: &str) -> Result<bool, String> {
    if !is_vault_initialized_in(dir) {
        return Ok(false);
    }

    let config = read_vault_config_at(&config_path_in(dir))?;

    // Migrate legacy unsalted SHA-256 hashes to Argon2 on successful login
    if is_legacy_sha256_hash(&config.password_hash) {
        let legacy_ok = verify_legacy_sha256(password, &config.password_hash);
        if legacy_ok {
            set_master_password_in(dir, password)?;
        }
        return Ok(legacy_ok);
    }

    verify_verifier(password, &config.password_hash)
}

pub fn is_vault_initialized_in(dir: &Path) -> bool {
    config_path_in(dir).exists()
}

pub fn is_vault_initialized() -> bool {
    is_vault_initialized_in(&get_slate_dir())
}

pub fn load_vault_config_in(dir: &Path) -> Result<VaultConfig, String> {
    if !is_vault_initialized_in(dir) {
        return Err("Vault is not initialized".to_string());
    }
    read_vault_config_at(&config_path_in(dir))
}

pub fn load_vault_config() -> Result<VaultConfig, String> {
    load_vault_config_in(&get_slate_dir())
}

pub(crate) fn read_vault_config_at(path: &Path) -> Result<VaultConfig, String> {
    let json = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read vault config: {}", e))?;
    serde_json::from_str(&json).map_err(|e| format!("Failed to parse vault config: {}", e))
}

fn save_vault_config_at(config_path: &Path, config: &VaultConfig) -> Result<(), String> {
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create vault directory: {}", e))?;
    }
    let json = serde_json::to_string(config)
        .map_err(|e| format!("Failed to serialize vault config: {}", e))?;
    let tmp = config_path.with_extension("json.tmp");
    fs::write(&tmp, json).map_err(|e| format!("Failed to write vault config: {}", e))?;
    fs::rename(&tmp, config_path).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        format!("Failed to finalize vault config: {}", e)
    })
}

pub fn save_vault_config(config: &VaultConfig) -> Result<(), String> {
    save_vault_config_at(&get_vault_config_path(), config)
}

pub fn hash_verifier(secret: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(secret.as_bytes(), &salt)
        .map_err(|e| format!("Failed to hash master password: {}", e))?;
    Ok(hash.to_string())
}

pub fn verify_verifier(secret: &str, phc: &str) -> Result<bool, String> {
    let parsed = PasswordHash::new(phc).map_err(|e| format!("Invalid stored password hash: {}", e))?;
    Ok(Argon2::default()
        .verify_password(secret.as_bytes(), &parsed)
        .is_ok())
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

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_config(json: &str) -> VaultConfig {
        match serde_json::from_str(json) {
            Ok(config) => config,
            Err(err) => panic!("config json: {err}"),
        }
    }

    #[test]
    fn legacy_config_has_no_recovery_wrap() {
        let legacy = parse_config(r#"{"password_hash":"abc"}"#);
        assert!(legacy.recovery_hash.is_none());
        assert!(matches!(legacy.wrap_record(), Ok(None)));
    }

    #[test]
    fn incomplete_recovery_config_errors() {
        let partial = parse_config(r#"{"password_hash":"abc","wrap_salt":"0011"}"#);
        let message = match partial.wrap_record() {
            Err(message) => message,
            Ok(_) => panic!("expected incomplete recovery data"),
        };
        assert_eq!(message, "Vault recovery data is incomplete");
    }

    #[test]
    fn wrap_fields_roundtrip_without_storing_key() {
        let record = RecoveryWrapRecord {
            salt: [7u8; 16],
            nonce: [9u8; 12],
            wrapped_key: vec![1, 2, 3, 4],
        };
        let mut config = VaultConfig::password_only("hash".to_string());
        config.set_recovery("recovery-hash".to_string(), &record);
        let json = match serde_json::to_string(&config) {
            Ok(json) => json,
            Err(err) => panic!("serialize config: {err}"),
        };
        assert!(!json.contains("recovery-key"));
        let back = parse_config(&json);
        let parsed = match back.wrap_record() {
            Ok(Some(record)) => record,
            Ok(None) => panic!("missing wrap"),
            Err(err) => panic!("wrap parse failed: {err}"),
        };
        assert_eq!(back.recovery_hash.as_deref(), Some("recovery-hash"));
        assert_eq!(parsed.salt, record.salt);
        assert_eq!(parsed.nonce, record.nonce);
        assert_eq!(parsed.wrapped_key, record.wrapped_key);
    }

    #[test]
    fn recovery_verifier_rejects_wrong_key() {
        let key = crypto::generate_recovery_key();
        let hashed = hash_verifier(&key).unwrap();
        assert!(verify_verifier(&key, &hashed).unwrap());
        let wrong = crypto::generate_recovery_key();
        assert!(!verify_verifier(&wrong, &hashed).unwrap());
    }
}

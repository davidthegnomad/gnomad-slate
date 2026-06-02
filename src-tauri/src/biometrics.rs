use std::fs;
use std::path::PathBuf;
use crate::filesystem::get_slate_dir;

/// Returns the secure configuration file path: `<slate_dir>/vault_config.json`
fn get_vault_config_path() -> PathBuf {
    get_slate_dir().join("vault_config.json")
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct VaultConfig {
    pub password_hash: String, // Store SHA-256 or Argon2 hash of master password
    pub biometrics_enabled: bool,
}

/// Sets or changes the master password
pub fn set_master_password(password: &str) -> Result<(), String> {
    let config_path = get_vault_config_path();
    let hashed = sha256_hash(password);
    
    let config = VaultConfig {
        password_hash: hashed,
        biometrics_enabled: true,
    };
    
    let json = serde_json::to_string(&config)
        .map_err(|e| format!("Failed to serialize vault config: {}", e))?;
        
    fs::write(config_path, json)
        .map_err(|e| format!("Failed to write vault config: {}", e))?;
        
    Ok(())
}

/// Verifies if a master password matches the registered one
pub fn verify_master_password(password: &str) -> Result<bool, String> {
    let config_path = get_vault_config_path();
    if !config_path.exists() {
        // Vault is not setup yet. Automatically allow if no password is set, or return false
        return Ok(false);
    }
    
    let json = fs::read_to_string(config_path)
        .map_err(|e| format!("Failed to read vault config: {}", e))?;
        
    let config: VaultConfig = serde_json::from_str(&json)
        .map_err(|e| format!("Failed to parse vault config: {}", e))?;
        
    let hashed = sha256_hash(password);
    Ok(config.password_hash == hashed)
}

/// Checks if a master password has already been registered
pub fn is_vault_initialized() -> bool {
    get_vault_config_path().exists()
}

/// Simple helper to SHA-256 hash a string
fn sha256_hash(input: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    result.iter().map(|b| format!("{:02x}", b)).collect::<String>()
}

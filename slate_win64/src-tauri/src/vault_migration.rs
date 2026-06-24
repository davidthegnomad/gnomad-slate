//! Vault unlock planning and post-unlock migration (v1/v2 → v3).

use zeroize::Zeroizing;

use crate::crypto;
use crate::vault_io;

const SALT_LEN: usize = 16;

/// Result of reading and classifying an on-disk vault before session open.
pub struct UnlockPlan {
    pub file_salt: [u8; SALT_LEN],
    pub json: Zeroizing<Vec<u8>>,
    pub rotate_to_v3: bool,
    pub persist_after_legacy: bool,
}

/// Read vault file, decrypt, and determine migration flags.
pub fn plan_unlock(password: &str) -> Result<UnlockPlan, String> {
    let vault_file = vault_io::vault_file_path();
    let encrypted_data = if vault_file.exists() {
        std::fs::read(&vault_file).map_err(|e| format!("Failed to read secure vault: {}", e))?
    } else {
        Vec::new()
    };

    let json = if encrypted_data.is_empty() {
        Zeroizing::new(b"[]".to_vec())
    } else {
        Zeroizing::new(crypto::decrypt_vault(&encrypted_data, password)?)
    };

    let file_salt = if encrypted_data.is_empty() || crypto::is_legacy_vault_format(&encrypted_data) {
        crypto::generate_vault_salt()
    } else {
        crypto::vault_file_kdf_salt(&encrypted_data)?
    };

    let rotate_to_v3 =
        !encrypted_data.is_empty() && crypto::is_v2_vault_format(&encrypted_data);
    let persist_after_legacy =
        !encrypted_data.is_empty() && crypto::is_legacy_vault_format(&encrypted_data);

    Ok(UnlockPlan {
        file_salt,
        json,
        rotate_to_v3,
        persist_after_legacy,
    })
}

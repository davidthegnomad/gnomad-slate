//! Vault unlock planning and post-unlock migration (v1 → v3).
//! v2 ciphertext is not rewritten. This build cannot rewrap the file key and
//! keep a recovery wrap in the same commit, so a v2 file stays v2.

use zeroize::Zeroizing;

use crate::crypto;
use crate::vault_io;

const SALT_LEN: usize = 16;

pub const V2_REWRITE_REFUSED: &str =
    "Refusing to rewrite a v2 vault; this build cannot preserve the recovery wrap";

/// True when persisting would turn a v2 packet into another format.
pub fn v2_rewrite_refused(packet: &[u8]) -> bool {
    crypto::is_v2_vault_format(packet)
}

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

    // v2 is decrypted with its existing key and left on disk. Rotating it to v3
    // would change the file key and strand any recovery wrap stored beside it.
    let rotate_to_v3 = false;
    let persist_after_legacy =
        !encrypted_data.is_empty() && crypto::is_legacy_vault_format(&encrypted_data);

    Ok(UnlockPlan {
        file_salt,
        json,
        rotate_to_v3,
        persist_after_legacy,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v2_packet_is_not_rewritten() {
        assert!(v2_rewrite_refused(&[0x02, 0, 1, 2]));
        assert!(!v2_rewrite_refused(&[0x03, 0, 1, 2]));
        assert!(!v2_rewrite_refused(&[]));
    }
}

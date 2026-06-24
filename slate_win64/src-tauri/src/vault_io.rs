//! Encrypted vault file I/O — separate from IPC command routing.

use std::fs::{self, OpenOptions};
use std::io::Write;

use crate::app_state::AppState;
use crate::filesystem;
use crate::vault_session;

pub fn vault_file_path() -> std::path::PathBuf {
    filesystem::get_slate_dir().join("vault.enc")
}

/// Writes encrypted vault bytes atomically on Windows and Unix.
///
/// Windows `rename` fails when the destination exists, so we truncate-in-place
/// when `vault.enc` already exists, otherwise write via a same-directory `.part`
/// file and rename once.
pub fn write_encrypted_vault(data: &[u8]) -> Result<(), String> {
    let path = vault_file_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create vault directory: {}", e))?;
    }

    if path.exists() {
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&path)
            .map_err(|e| format!("Failed to open secure vault for update: {}", e))?;
        file.write_all(data)
            .map_err(|e| format!("Failed to write secure vault: {}", e))?;
        file.sync_all()
            .map_err(|e| format!("Failed to sync secure vault: {}", e))?;
        return Ok(());
    }

    let parent = path
        .parent()
        .ok_or_else(|| "Vault directory has no parent".to_string())?;
    let tmp = parent.join("vault.enc.part");
    fs::write(&tmp, data).map_err(|e| format!("Failed to write secure vault: {}", e))?;
    fs::rename(&tmp, &path).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        format!("Failed to finalize secure vault: {}", e)
    })
}

pub fn persist_vault(state: &AppState) -> Result<(), String> {
    let encrypted = vault_session::with_session(&state.vault_session, |session| {
        session.encrypt_secrets()
    })?;
    write_encrypted_vault(&encrypted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vault_file_path_ends_with_vault_enc() {
        let path = vault_file_path();
        assert_eq!(path.file_name().and_then(|n| n.to_str()), Some("vault.enc"));
    }
}

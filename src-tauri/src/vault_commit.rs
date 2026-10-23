use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use zeroize::Zeroize;

use crate::biometrics::{self, VaultConfig};
use crate::crypto::{self, VAULT_FILE_MISSING};
use crate::vault_session;

/// Shown when `vault.enc` exists and `vault_config.json` does not.
pub(crate) const VAULT_WIPE_REQUIRED: &str =
    "An encrypted vault file is left without its config. Initializing would destroy it.";

const LEFT_UNCHANGED: &str = "Left the existing vault files unchanged.";

#[derive(Debug)]
pub(crate) enum Reconcile {
    Done,
    /// A pending config was not applied. Live `vault.enc` and `vault_config.json` were not written.
    PendingMismatch(String),
}

pub(crate) struct InitializedVault {
    pub recovery_key: String,
}

fn enc_path(dir: &Path) -> PathBuf {
    dir.join("vault.enc")
}

fn enc_next_path(dir: &Path) -> PathBuf {
    dir.join("vault.enc.next")
}

fn config_path(dir: &Path) -> PathBuf {
    dir.join("vault_config.json")
}

fn config_next_path(dir: &Path) -> PathBuf {
    dir.join("vault_config.json.next")
}

fn ciphertext_sha256(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(bytes);
    crypto::encode_hex(digest.as_slice())
}

fn pending_mismatch(detail: &str) -> Reconcile {
    Reconcile::PendingMismatch(format!("{detail} {LEFT_UNCHANGED}"))
}

/// Stage ciphertext and config beside the live pair, then rename ciphertext first.
/// Live files stay put until both side files are synced. A failed config write
/// removes `vault.enc.next` and does not replace `vault.enc`.
pub(crate) fn commit_vault_pair(
    dir: &Path,
    packet: &[u8],
    config: &mut VaultConfig,
) -> Result<(), String> {
    if packet.is_empty() {
        return Err(VAULT_FILE_MISSING.to_string());
    }

    config.ciphertext_sha256 = Some(ciphertext_sha256(packet));
    let config_bytes = serde_json::to_vec(config)
        .map_err(|e| format!("Failed to serialize vault config: {}", e))?;

    let enc_next = enc_next_path(dir);
    let config_next = config_next_path(dir);
    write_durable(&enc_next, packet)?;
    if let Err(e) = write_durable(&config_next, &config_bytes) {
        let _ = remove_file_if_present(&enc_next);
        return Err(e);
    }

    let enc = enc_path(dir);
    let config_file = config_path(dir);
    if let Err(e) = fs::rename(&enc_next, &enc) {
        let _ = remove_file_if_present(&config_next);
        let _ = remove_file_if_present(&enc_next);
        return Err(format!("Failed to replace vault file: {}", e));
    }
    if let Err(e) = sync_dir(dir) {
        return Err(format!(
            "Vault file was replaced but the directory sync failed: {}. Restart the app to finish saving the vault config.",
            e
        ));
    }
    if let Err(e) = fs::rename(&config_next, &config_file) {
        return Err(format!(
            "Vault file was replaced but the config was not: {}. Restart the app to finish saving the vault config.",
            e
        ));
    }
    sync_dir(dir)?;
    Ok(())
}

/// Finish or roll back a commit that stopped between the two renames.
pub(crate) fn reconcile_vault_commit(dir: &Path) -> Result<Reconcile, String> {
    let enc_next = enc_next_path(dir);
    let config_next = config_next_path(dir);
    let enc_next_exists = enc_next.exists();
    let config_next_exists = config_next.exists();

    if enc_next_exists && config_next_exists {
        // Live pair was never replaced. Drop the staged pair, config.next first
        // so a crash cannot leave a config that startup would apply.
        remove_file_if_present(&config_next)?;
        remove_file_if_present(&enc_next)?;
        sync_dir(dir)?;
        return Ok(Reconcile::Done);
    }

    if config_next_exists {
        return finish_config_next(dir, &config_next);
    }

    if enc_next_exists {
        remove_file_if_present(&enc_next)?;
        sync_dir(dir)?;
    }

    Ok(Reconcile::Done)
}

fn finish_config_next(dir: &Path, config_next: &Path) -> Result<Reconcile, String> {
    let staged_bytes = match fs::read(config_next) {
        Ok(bytes) => bytes,
        Err(e) => {
            return Ok(pending_mismatch(&format!(
                "Pending vault config could not be read ({}).",
                e
            )));
        }
    };
    let staged: VaultConfig = match serde_json::from_slice(&staged_bytes) {
        Ok(config) => config,
        Err(_) => {
            return Ok(pending_mismatch(
                "Pending vault config could not be read.",
            ));
        }
    };
    let Some(expected) = staged.ciphertext_sha256.as_deref() else {
        return Ok(pending_mismatch(
            "Pending vault config has no ciphertext hash.",
        ));
    };

    let enc = enc_path(dir);
    let live = match fs::read(&enc) {
        Ok(bytes) if !bytes.is_empty() => bytes,
        Ok(_) => {
            return Ok(pending_mismatch(
                "Pending vault config does not match vault.enc.",
            ));
        }
        Err(_) => {
            return Ok(pending_mismatch(
                "Pending vault config does not match vault.enc.",
            ));
        }
    };
    if ciphertext_sha256(&live) != expected {
        return Ok(pending_mismatch(
            "Pending vault config does not match vault.enc.",
        ));
    }

    fs::rename(config_next, config_path(dir))
        .map_err(|e| format!("Failed to finish vault config: {}", e))?;
    sync_dir(dir)?;
    Ok(Reconcile::Done)
}

fn acknowledge_pending(dir: &Path) -> Result<(), String> {
    match reconcile_vault_commit(dir)? {
        Reconcile::Done | Reconcile::PendingMismatch(_) => Ok(()),
    }
}

pub(crate) fn require_vault_ciphertext(dir: &Path) -> Result<Vec<u8>, String> {
    let path = enc_path(dir);
    if !path.exists() {
        return Err(VAULT_FILE_MISSING.to_string());
    }
    let bytes = fs::read(&path).map_err(|e| format!("Failed to read secure vault: {}", e))?;
    if bytes.is_empty() {
        return Err(VAULT_FILE_MISSING.to_string());
    }
    Ok(bytes)
}

pub(crate) fn initialize_vault_in(
    dir: &Path,
    password: &str,
    wipe_existing: bool,
    state: &super::AppState,
) -> Result<InitializedVault, String> {
    acknowledge_pending(dir)?;
    biometrics::validate_master_password(password)?;
    if biometrics::is_vault_initialized_in(dir) {
        return Err("Vault is already initialized".to_string());
    }
    if enc_path(dir).exists() && !wipe_existing {
        return Err(VAULT_WIPE_REQUIRED.to_string());
    }

    let recovery_key = crypto::generate_recovery_key();
    let file_key = crypto::derive_vault_file_key(password)?;
    let recovery_hash = biometrics::hash_verifier(&recovery_key)?;
    let wrap = crypto::create_recovery_wrap(&file_key, &recovery_key)?;
    let mut config = VaultConfig::password_only(biometrics::hash_verifier(password)?);
    config.set_recovery(recovery_hash, &wrap);
    let packet = crypto::encrypt_vault(b"[]", &file_key)?;

    if enc_path(dir).exists() && !wipe_existing {
        return Err(VAULT_WIPE_REQUIRED.to_string());
    }
    commit_vault_pair(dir, &packet, &mut config)?;
    vault_session::open_session_with_json(&state.vault_session, password, b"[]")?;
    if let Ok(mut limiter) = state.unlock_limiter.lock() {
        limiter.record_success();
    }
    Ok(InitializedVault { recovery_key })
}

/// Issues a new recovery key for the unlocked session file key and stores only the wrap.
/// The returned key is the only copy. It is not written to disk.
pub(crate) fn issue_recovery_key_in(dir: &Path, state: &super::AppState) -> Result<String, String> {
    let mut file_key = vault_session::with_session(&state.vault_session, |session| {
        Ok(session.copy_file_key())
    })?;
    let issued = (|| {
        let recovery_key = crypto::generate_recovery_key();
        let recovery_hash = biometrics::hash_verifier(&recovery_key)?;
        let wrap = crypto::create_recovery_wrap(&file_key, &recovery_key)?;
        let mut config = biometrics::load_vault_config_in(dir)?;
        config.set_recovery(recovery_hash, &wrap);
        biometrics::save_vault_config_in(dir, &config)?;
        Ok(recovery_key)
    })();
    file_key.zeroize();
    issued
}

pub(crate) fn unlock_vault_in(
    dir: &Path,
    password: &str,
    state: &super::AppState,
) -> Result<(), String> {
    acknowledge_pending(dir)?;
    if biometrics::is_vault_initialized_in(dir) {
        let _ = require_vault_ciphertext(dir)?;
    }

    {
        let mut limiter = state
            .unlock_limiter
            .lock()
            .map_err(|_| "Rate limiter lock poisoned".to_string())?;
        limiter.check_allowed()?;
    }

    if !biometrics::verify_master_password_in(dir, password)? {
        if let Ok(mut limiter) = state.unlock_limiter.lock() {
            limiter.record_failure();
        }
        return Err("Invalid password".to_string());
    }

    let encrypted_data = require_vault_ciphertext(dir)?;
    let decrypted_bytes = crypto::decrypt_vault(&encrypted_data, password)?;
    let was_legacy = crypto::is_legacy_vault_format(&encrypted_data);
    vault_session::open_session_with_json(&state.vault_session, password, &decrypted_bytes)?;

    if was_legacy {
        let encrypted = vault_session::with_session(&state.vault_session, |session| {
            session.encrypt_secrets()
        })?;
        replace_ciphertext_only(dir, &encrypted)?;
    }

    if let Ok(mut limiter) = state.unlock_limiter.lock() {
        limiter.record_success();
    }
    Ok(())
}

pub(crate) fn recover_vault_in(
    dir: &Path,
    recovery_key: &str,
    new_password: &str,
    state: &super::AppState,
) -> Result<(), String> {
    acknowledge_pending(dir)?;
    if biometrics::is_vault_initialized_in(dir) {
        let _ = require_vault_ciphertext(dir)?;
    }

    biometrics::validate_master_password(new_password)?;
    let recovery_key = crypto::normalize_recovery_key(recovery_key)?;

    {
        let mut limiter = state
            .unlock_limiter
            .lock()
            .map_err(|_| "Rate limiter lock poisoned".to_string())?;
        limiter.check_allowed()?;
    }

    let mut config = biometrics::load_vault_config_in(dir)?;
    let wrap = config
        .wrap_record()?
        .ok_or_else(|| "This vault has no recovery key".to_string())?;
    let recovery_hash = config
        .recovery_hash
        .clone()
        .ok_or_else(|| "This vault has no recovery key".to_string())?;
    if !biometrics::verify_verifier(&recovery_key, &recovery_hash)? {
        if let Ok(mut limiter) = state.unlock_limiter.lock() {
            limiter.record_failure();
        }
        return Err("Invalid recovery key".to_string());
    }

    let encrypted_data = require_vault_ciphertext(dir)?;
    let rotated = crypto::rotate_file_key_with_recovery(
        &encrypted_data,
        &wrap,
        &recovery_key,
        new_password,
    )?;
    let _parsed =
        vault_session::VaultSession::from_password_and_json(new_password, &rotated.plaintext)?;

    config.password_hash = biometrics::hash_verifier(new_password)?;
    config.apply_wrap(&rotated.wrap);
    commit_vault_pair(dir, &rotated.packet, &mut config)?;
    vault_session::open_session_with_json(&state.vault_session, new_password, &rotated.plaintext)?;

    if let Ok(mut limiter) = state.unlock_limiter.lock() {
        limiter.record_success();
    }
    Ok(())
}

fn replace_ciphertext_only(dir: &Path, packet: &[u8]) -> Result<(), String> {
    if packet.is_empty() {
        return Err(VAULT_FILE_MISSING.to_string());
    }
    let tmp = dir.join("vault.enc.tmp");
    write_durable(&tmp, packet)?;
    fs::rename(&tmp, enc_path(dir)).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        format!("Failed to finalize secure vault: {}", e)
    })?;
    let _ = sync_dir(dir);
    Ok(())
}

fn write_durable(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create vault directory: {}", e))?;
    }
    let mut file = File::create(path)
        .map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;
    file.write_all(bytes)
        .map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;
    file.sync_all()
        .map_err(|e| format!("Failed to sync {}: {}", path.display(), e))?;
    if let Some(parent) = path.parent() {
        sync_dir(parent)?;
    }
    Ok(())
}

fn sync_dir(dir: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::io::AsRawFd;
        let file = File::open(dir)
            .map_err(|e| format!("Failed to open vault directory for sync: {}", e))?;
        // Safety: `file` is an open directory descriptor owned for this call.
        let rc = unsafe { fsync(file.as_raw_fd()) };
        if rc != 0 {
            return Err(format!(
                "Failed to sync vault directory: {}",
                std::io::Error::last_os_error()
            ));
        }
        Ok(())
    }
    #[cfg(not(unix))]
    {
        let _ = dir;
        Ok(())
    }
}

#[cfg(unix)]
extern "C" {
    fn fsync(fd: i32) -> i32;
}

fn remove_file_if_present(path: &Path) -> Result<(), String> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("Failed to remove {}: {}", path.display(), e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::{Arc, Mutex};

    struct TempVault {
        path: PathBuf,
    }

    impl TempVault {
        fn new(label: &str) -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let id = NEXT.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "gnomad-slate-vault-{label}-{}-{id}",
                std::process::id()
            ));
            fs::create_dir_all(&path).unwrap_or_else(|e| panic!("temp dir: {e}"));
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempVault {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn test_state() -> super::super::AppState {
        super::super::AppState {
            clipboard_history: Arc::new(Mutex::new(crate::clipboard::ClipboardHistory::new(1))),
            clipboard_filter_enabled: Arc::new(Mutex::new(true)),
            vault_session: Arc::new(Mutex::new(None)),
            unlock_limiter: Arc::new(Mutex::new(crate::unlock_limiter::UnlockRateLimiter::new())),
        }
    }

    fn read_bytes(path: &Path) -> Vec<u8> {
        fs::read(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
    }

    fn names(dir: &Path) -> Vec<String> {
        let mut found = Vec::new();
        let entries = fs::read_dir(dir).unwrap_or_else(|e| panic!("read dir: {e}"));
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                found.push(name.to_string());
            }
        }
        found.sort();
        found
    }

    #[test]
    fn failed_config_write_leaves_original_vault_bytes() {
        let dir = TempVault::new("config-fail");
        let original = b"original-vault-ciphertext-do-not-replace";
        let original_config = br#"{"password_hash":"original-hash"}"#;
        fs::write(enc_path(dir.path()), original).unwrap();
        fs::write(config_path(dir.path()), original_config).unwrap();
        fs::create_dir(config_next_path(dir.path())).unwrap();

        let mut config = VaultConfig::password_only("replacement-hash".to_string());
        let result = commit_vault_pair(dir.path(), b"new-ciphertext-not-written-live", &mut config);

        assert!(result.is_err());
        assert_eq!(read_bytes(&enc_path(dir.path())), original);
        assert_eq!(read_bytes(&config_path(dir.path())), original_config);
        assert!(!enc_next_path(dir.path()).exists());
    }

    #[test]
    fn missing_vault_file_on_unlock_and_recover_creates_nothing() {
        let state = test_state();
        let config = br#"{"password_hash":"not-a-real-hash"}"#;

        let missing = TempVault::new("missing-enc");
        fs::write(config_path(missing.path()), config).unwrap();
        let unlock_err = unlock_vault_in(missing.path(), "password1", &state).unwrap_err();
        assert_eq!(unlock_err, VAULT_FILE_MISSING);
        let recover_err = recover_vault_in(missing.path(), "abcd", "password1", &state).unwrap_err();
        assert_eq!(recover_err, VAULT_FILE_MISSING);
        assert_eq!(names(missing.path()), vec!["vault_config.json".to_string()]);
        assert_eq!(read_bytes(&config_path(missing.path())), config);

        let empty = TempVault::new("empty-enc");
        fs::write(config_path(empty.path()), config).unwrap();
        fs::write(enc_path(empty.path()), b"").unwrap();
        let unlock_empty = unlock_vault_in(empty.path(), "password1", &state).unwrap_err();
        assert_eq!(unlock_empty, VAULT_FILE_MISSING);
        let recover_empty = recover_vault_in(empty.path(), "abcd", "password1", &state).unwrap_err();
        assert_eq!(recover_empty, VAULT_FILE_MISSING);
        assert_eq!(read_bytes(&enc_path(empty.path())), b"");
        assert_eq!(read_bytes(&config_path(empty.path())), config);
        assert!(!enc_next_path(empty.path()).exists());
        assert!(!config_next_path(empty.path()).exists());

        let real = TempVault::new("nonempty-enc");
        let ciphertext = b"this-is-real-ciphertext";
        fs::write(config_path(real.path()), config).unwrap();
        fs::write(enc_path(real.path()), ciphertext).unwrap();
        let unlock_real = unlock_vault_in(real.path(), "password1", &state).unwrap_err();
        assert_ne!(unlock_real, VAULT_FILE_MISSING);
        assert!(!unlock_real.contains("missing or empty"));
        assert_eq!(read_bytes(&enc_path(real.path())), ciphertext);
        assert!(!enc_next_path(real.path()).exists());
    }

    #[test]
    fn initialize_without_wipe_keeps_orphan_vault_and_wipe_replaces_it() {
        let orphan = b"orphan-vault-ciphertext";
        let keep = TempVault::new("no-wipe");
        fs::write(enc_path(keep.path()), orphan).unwrap();
        let state = test_state();
        let err = match initialize_vault_in(keep.path(), "test-password", false, &state) {
            Err(message) => message,
            Ok(_) => panic!("initialize without wipe must fail"),
        };
        assert_eq!(err, VAULT_WIPE_REQUIRED);
        assert_eq!(read_bytes(&enc_path(keep.path())), orphan);
        assert!(!config_path(keep.path()).exists());
        assert!(!enc_next_path(keep.path()).exists());
        assert!(!config_next_path(keep.path()).exists());

        let wipe = TempVault::new("wipe");
        fs::write(enc_path(wipe.path()), orphan).unwrap();
        let created = initialize_vault_in(wipe.path(), "test-password", true, &state)
            .unwrap_or_else(|e| panic!("wipe initialize: {e}"));
        assert!(created.recovery_key.len() > 8);
        let replaced = read_bytes(&enc_path(wipe.path()));
        assert_ne!(replaced, orphan);
        assert!(!replaced.is_empty());
        assert!(config_path(wipe.path()).exists());
        assert!(!enc_next_path(wipe.path()).exists());
        assert!(!config_next_path(wipe.path()).exists());
        let saved = biometrics::load_vault_config_in(wipe.path()).unwrap();
        assert_eq!(
            saved.ciphertext_sha256.as_deref(),
            Some(ciphertext_sha256(&replaced).as_str())
        );
    }

    #[test]
    fn startup_with_both_next_files_deletes_them() {
        let dir = TempVault::new("both-next");
        let live_enc = b"live-ciphertext";
        let live_config = br#"{"password_hash":"live-hash"}"#;
        fs::write(enc_path(dir.path()), live_enc).unwrap();
        fs::write(config_path(dir.path()), live_config).unwrap();
        fs::write(enc_next_path(dir.path()), b"staged-ciphertext").unwrap();
        let staged_hash = ciphertext_sha256(live_enc);
        let staged = format!(
            r#"{{"password_hash":"staged-hash","ciphertext_sha256":"{staged_hash}"}}"#
        );
        fs::write(config_next_path(dir.path()), &staged).unwrap();

        let result = reconcile_vault_commit(dir.path()).unwrap();
        assert!(matches!(result, Reconcile::Done));
        assert_eq!(read_bytes(&enc_path(dir.path())), live_enc);
        assert_eq!(read_bytes(&config_path(dir.path())), live_config);
        assert!(!enc_next_path(dir.path()).exists());
        assert!(!config_next_path(dir.path()).exists());
    }

    #[test]
    fn startup_with_matching_config_next_finishes_rename() {
        let dir = TempVault::new("config-next");
        let live_enc = b"current-ciphertext-bytes";
        fs::write(enc_path(dir.path()), live_enc).unwrap();
        fs::write(config_path(dir.path()), br#"{"password_hash":"old-hash"}"#).unwrap();
        let staged_hash = ciphertext_sha256(live_enc);
        let staged = format!(
            r#"{{"password_hash":"new-hash","ciphertext_sha256":"{staged_hash}"}}"#
        );
        fs::write(config_next_path(dir.path()), &staged).unwrap();
        assert!(!enc_next_path(dir.path()).exists());

        let result = reconcile_vault_commit(dir.path()).unwrap();
        assert!(matches!(result, Reconcile::Done));
        assert_eq!(read_bytes(&enc_path(dir.path())), live_enc);
        assert_eq!(read_bytes(&config_path(dir.path())), staged.as_bytes());
        assert!(!config_next_path(dir.path()).exists());
    }

    #[test]
    fn startup_with_mismatched_config_next_leaves_live_files() {
        let dir = TempVault::new("mismatch");
        let live_enc = b"current-ciphertext-bytes";
        let live_config = br#"{"password_hash":"old-hash"}"#;
        fs::write(enc_path(dir.path()), live_enc).unwrap();
        fs::write(config_path(dir.path()), live_config).unwrap();
        let staged = br#"{"password_hash":"other-hash","ciphertext_sha256":"deadbeef"}"#;
        fs::write(config_next_path(dir.path()), staged).unwrap();

        let result = reconcile_vault_commit(dir.path()).unwrap();
        assert!(matches!(result, Reconcile::PendingMismatch(_)));
        assert_eq!(read_bytes(&enc_path(dir.path())), live_enc);
        assert_eq!(read_bytes(&config_path(dir.path())), live_config);
        assert_eq!(read_bytes(&config_next_path(dir.path())), staged);
    }
}

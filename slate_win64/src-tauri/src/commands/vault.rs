use crate::app_state::AppState;
use crate::biometrics;
use crate::clipboard;
use crate::filesystem;
use crate::vault_io::persist_vault;
use crate::vault_migration;
use crate::vault_session::{self, SecretMeta};

#[tauri::command]
pub fn is_vault_initialized() -> bool {
    biometrics::is_vault_initialized()
}

#[tauri::command]
pub fn is_vault_unlocked(state: tauri::State<AppState>) -> bool {
    vault_session::is_unlocked(&state.vault_session)
}

#[tauri::command]
pub fn initialize_vault(password: String, state: tauri::State<AppState>) -> Result<(), String> {
    let vault_file = crate::vault_io::vault_file_path();
    if vault_file.exists() {
        let bytes = std::fs::read(&vault_file)
            .map_err(|e| format!("Failed to read secure vault: {}", e))?;
        if vault_migration::v2_rewrite_refused(&bytes) {
            return Err(vault_migration::V2_REWRITE_REFUSED.to_string());
        }
    }
    if biometrics::config_contains_recovery_wrap()? {
        return Err(vault_migration::V2_REWRITE_REFUSED.to_string());
    }
    biometrics::validate_master_password(&password)?;
    biometrics::set_master_password(&password)?;
    let file_salt = crate::crypto::generate_vault_salt();
    vault_session::open_session_with_json(&state.vault_session, &password, file_salt, b"[]")?;
    persist_vault(&state)?;
    if let Ok(mut limiter) = state.unlock_limiter.lock() {
        limiter.record_success();
    }
    Ok(())
}

/// Unlock matrix (see `vault_migration::plan_unlock`):
/// - empty vault → new v3 salt
/// - v1 legacy → decrypt, new salt, persist as v3
/// - v2 fixed salt → decrypt with the existing key and do not rewrite the file
/// - v3 → use salt from file header
#[tauri::command]
pub async fn unlock_vault(password: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    {
        let mut limiter = state
            .unlock_limiter
            .lock()
            .map_err(|_| "Rate limiter lock poisoned".to_string())?;
        limiter.check_allowed()?;
    }

    if !biometrics::verify_master_password(&password)? {
        if let Ok(mut limiter) = state.unlock_limiter.lock() {
            limiter.record_failure();
        }
        return Err("Invalid password".to_string());
    }

    let password_for_disk = password.clone();
    let plan = tauri::async_runtime::spawn_blocking(move || {
        vault_migration::plan_unlock(&password_for_disk)
    })
        .await
        .map_err(|e| format!("Unlock task failed: {}", e))??;

    vault_session::open_session_with_json(
        &state.vault_session,
        &password,
        plan.file_salt,
        &plan.json,
    )?;

    if plan.rotate_to_v3 {
        return Err(vault_migration::V2_REWRITE_REFUSED.to_string());
    } else if plan.persist_after_legacy {
        persist_vault(&state)?;
    }

    if let Ok(mut limiter) = state.unlock_limiter.lock() {
        limiter.record_success();
    }

    Ok(())
}

#[tauri::command]
pub fn lock_vault(state: tauri::State<AppState>) {
    vault_session::close_session(&state.vault_session);
}

#[tauri::command]
pub fn list_secret_metadata(state: tauri::State<AppState>) -> Result<Vec<SecretMeta>, String> {
    vault_session::with_session(&state.vault_session, |session| Ok(session.list_metadata()))
}

#[tauri::command]
pub fn reveal_secret_value(index: usize, state: tauri::State<AppState>) -> Result<String, String> {
    let value = vault_session::with_session(&state.vault_session, |session| {
        session.get_value(index)
    })?;
    Ok(value.to_string())
}

#[tauri::command]
pub fn copy_secret_to_clipboard(index: usize, state: tauri::State<AppState>) -> Result<(), String> {
    let value = vault_session::with_session(&state.vault_session, |session| {
        session.get_value(index)
    })?;
    clipboard::copy_and_clear_after(value.to_string())
}

#[tauri::command]
pub fn add_secret(
    name: String,
    path: String,
    value: String,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    filesystem::validate_secret_path(&path)?;
    vault_session::with_session_mut(&state.vault_session, |session| {
        session.add_secret(name, path, value)
    })?;
    persist_vault(&state)
}

#[tauri::command]
pub fn delete_secret(index: usize, state: tauri::State<AppState>) -> Result<(), String> {
    vault_session::with_session_mut(&state.vault_session, |session| {
        session.delete_secret(index)
    })?;
    persist_vault(&state)
}

pub mod biometrics;
pub mod clipboard;
pub mod clipboard_filter;
pub mod crypto;
pub mod filesystem;
pub mod pasteboard_macos;
pub mod unlock_limiter;
pub mod vault_session;

use std::sync::{Arc, Mutex};
use arboard::Clipboard;
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

use crate::clipboard::{ClipboardEntry, ClipboardHistory};
use crate::filesystem::Note;
use crate::unlock_limiter::UnlockRateLimiterState;
use crate::vault_session::{SecretMeta, VaultSessionState};

pub struct AppState {
    pub clipboard_history: Arc<Mutex<ClipboardHistory>>,
    pub clipboard_filter_enabled: Arc<Mutex<bool>>,
    pub vault_session: Arc<VaultSessionState>,
    pub unlock_limiter: Arc<UnlockRateLimiterState>,
}

fn vault_file_path() -> std::path::PathBuf {
    filesystem::get_slate_dir().join("vault.enc")
}

fn write_encrypted_vault(data: &[u8]) -> Result<(), String> {
    std::fs::write(vault_file_path(), data)
        .map_err(|e| format!("Failed to write secure vault: {}", e))
}

fn persist_vault(state: &AppState) -> Result<(), String> {
    let encrypted = vault_session::with_session(&state.vault_session, |session| {
        session.encrypt_secrets()
    })?;
    write_encrypted_vault(&encrypted)
}

#[tauri::command]
fn get_clipboard_history(state: tauri::State<AppState>) -> Vec<ClipboardEntry> {
    if let Ok(hist) = state.clipboard_history.lock() {
        hist.get_entries()
    } else {
        Vec::new()
    }
}

#[tauri::command]
fn get_clipboard_filter_enabled(state: tauri::State<AppState>) -> bool {
    state
        .clipboard_filter_enabled
        .lock()
        .map(|g| *g)
        .unwrap_or(true)
}

#[tauri::command]
fn set_clipboard_filter_enabled(enabled: bool, state: tauri::State<AppState>) {
    if let Ok(mut guard) = state.clipboard_filter_enabled.lock() {
        *guard = enabled;
    }
}

#[tauri::command]
fn list_notes() -> Result<Vec<Note>, String> {
    filesystem::list_notes()
}

#[tauri::command]
fn read_note(filename: String) -> Result<String, String> {
    filesystem::read_note(&filename)
}

#[tauri::command]
fn write_note(title: String, content: String, is_ephemeral: bool) -> Result<Note, String> {
    filesystem::write_note(&title, &content, is_ephemeral)
}

#[tauri::command]
fn delete_note(filename: String) -> Result<(), String> {
    filesystem::delete_note(&filename)
}

#[tauri::command]
fn archive_notes() -> Result<usize, String> {
    filesystem::archive_stale_ephemeral_notes()
}

#[tauri::command]
fn is_vault_initialized() -> bool {
    biometrics::is_vault_initialized()
}

#[tauri::command]
fn is_vault_unlocked(state: tauri::State<AppState>) -> bool {
    vault_session::is_unlocked(&state.vault_session)
}

#[tauri::command]
fn initialize_vault(password: String, state: tauri::State<AppState>) -> Result<(), String> {
    biometrics::validate_master_password(&password)?;
    biometrics::set_master_password(&password)?;
    vault_session::open_session_with_json(&state.vault_session, &password, b"[]")?;
    persist_vault(&state)?;
    if let Ok(mut limiter) = state.unlock_limiter.lock() {
        limiter.record_success();
    }
    Ok(())
}

#[tauri::command]
fn unlock_vault(password: String, state: tauri::State<AppState>) -> Result<(), String> {
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

    let vault_file = vault_file_path();
    let encrypted_data = if vault_file.exists() {
        std::fs::read(&vault_file).map_err(|e| format!("Failed to read secure vault: {}", e))?
    } else {
        Vec::new()
    };

    let decrypted_bytes = if encrypted_data.is_empty() {
        b"[]".to_vec()
    } else {
        crypto::decrypt_vault(&encrypted_data, &password)?
    };

    let was_legacy =
        !encrypted_data.is_empty() && crypto::is_legacy_vault_format(&encrypted_data);

    vault_session::open_session_with_json(&state.vault_session, &password, &decrypted_bytes)?;

    if was_legacy {
        persist_vault(&state)?;
    }

    if let Ok(mut limiter) = state.unlock_limiter.lock() {
        limiter.record_success();
    }

    Ok(())
}

#[tauri::command]
fn lock_vault(state: tauri::State<AppState>) {
    vault_session::close_session(&state.vault_session);
}

#[tauri::command]
fn list_secret_metadata(state: tauri::State<AppState>) -> Result<Vec<SecretMeta>, String> {
    vault_session::with_session(&state.vault_session, |session| Ok(session.list_metadata()))
}

#[tauri::command]
fn reveal_secret_value(index: usize, state: tauri::State<AppState>) -> Result<String, String> {
    let value = vault_session::with_session(&state.vault_session, |session| {
        session.get_value(index)
    })?;
    // IPC requires an owned String; caller (UI) should clear display on hide/lock.
    Ok(value.to_string())
}

#[tauri::command]
fn copy_secret_to_clipboard(index: usize, state: tauri::State<AppState>) -> Result<(), String> {
    let value = vault_session::with_session(&state.vault_session, |session| {
        session.get_value(index)
    })?;

    let mut clipboard = Clipboard::new()
        .map_err(|e| format!("Clipboard unavailable: {}", e))?;
    clipboard
        .set_text(value.to_string())
        .map_err(|e| format!("Failed to copy to clipboard: {}", e))?;
    // `value` (Zeroizing<String>) drops here and overwrites its heap buffer.

    Ok(())
}

#[tauri::command]
fn add_secret(
    name: String,
    path: String,
    value: String,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    vault_session::with_session_mut(&state.vault_session, |session| {
        session.add_secret(name, path, value)
    })?;
    persist_vault(&state)
}

#[tauri::command]
fn delete_secret(index: usize, state: tauri::State<AppState>) -> Result<(), String> {
    vault_session::with_session_mut(&state.vault_session, |session| {
        session.delete_secret(index)
    })?;
    persist_vault(&state)
}

fn toggle_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if let Ok(visible) = window.is_visible() {
            if visible {
                let _ = window.hide();
            } else {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let clipboard_history = Arc::new(Mutex::new(ClipboardHistory::new(15)));
    let clipboard_filter_enabled = Arc::new(Mutex::new(true));
    clipboard::start_clipboard_monitor(
        clipboard_history.clone(),
        clipboard_filter_enabled.clone(),
    );

    let vault_session = Arc::new(Mutex::new(None));
    let unlock_limiter = Arc::new(Mutex::new(unlock_limiter::UnlockRateLimiter::new()));

    let shortcut = "Option+Space";

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(AppState {
            clipboard_history,
            clipboard_filter_enabled,
            vault_session,
            unlock_limiter,
        })
        .setup(move |app| {
            let handle = app.handle().clone();

            use tauri_plugin_global_shortcut::ShortcutState;

            match shortcut.parse::<Shortcut>() {
                Ok(shortcut_parsed) => {
                    let _ = app.global_shortcut().on_shortcut(
                        shortcut_parsed,
                        move |_app, _shortcut, event| {
                            if event.state() == ShortcutState::Pressed {
                                toggle_window(&handle);
                            }
                        },
                    );
                }
                Err(e) => {
                    eprintln!("Gnomad Slate: failed to register global shortcut: {e}");
                }
            }

            if let Some(window) = app.get_webview_window("main") {
                let w = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::Focused(false) = event {
                        let _ = w.hide();
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_clipboard_history,
            get_clipboard_filter_enabled,
            set_clipboard_filter_enabled,
            list_notes,
            read_note,
            write_note,
            delete_note,
            archive_notes,
            is_vault_initialized,
            is_vault_unlocked,
            initialize_vault,
            unlock_vault,
            lock_vault,
            list_secret_metadata,
            reveal_secret_value,
            copy_secret_to_clipboard,
            add_secret,
            delete_secret,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

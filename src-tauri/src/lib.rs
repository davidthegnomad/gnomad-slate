pub mod biometrics;
pub mod clipboard;
pub mod clipboard_filter;
pub mod crypto;
pub mod filesystem;
#[cfg(target_os = "linux")]
pub mod linux_display;
#[cfg(target_os = "linux")]
pub mod linux_tray;
pub mod logging;
pub mod macos_app;
pub mod menu;
pub mod pasteboard_macos;
pub mod shortcuts;
pub mod unlock_limiter;
pub mod vault_session;

use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(target_os = "macos")]
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex};
use arboard::Clipboard;
use tauri::{AppHandle, Emitter, Manager, RunEvent};
use tauri_plugin_global_shortcut::ShortcutState;

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

#[derive(Serialize)]
pub struct AppVersionInfo {
    pub version: String,
    pub channel: String,
    pub display_version: String,
}

#[tauri::command]
fn get_app_version_info() -> AppVersionInfo {
    let channel = if cfg!(target_os = "macos") {
        "beta"
    } else {
        "alpha"
    };

    let version = env!("CARGO_PKG_VERSION").to_string();
    let display_version = if cfg!(target_os = "linux") {
        format!("v{version}")
    } else {
        format!("v2.1 {channel}")
    };

    AppVersionInfo {
        version,
        channel: channel.to_string(),
        display_version,
    }
}

fn vault_file_path() -> std::path::PathBuf {
    filesystem::get_slate_dir().join("vault.enc")
}

fn write_encrypted_vault(data: &[u8]) -> Result<(), String> {
    let path = vault_file_path();
    let tmp = path.with_extension("enc.tmp");
    std::fs::write(&tmp, data).map_err(|e| format!("Failed to write secure vault: {}", e))?;
    std::fs::rename(&tmp, &path).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        format!("Failed to finalize secure vault: {}", e)
    })
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
fn get_summon_shortcut_label() -> String {
    shortcuts::summon_shortcut().to_string()
}

/// Legacy command — prefer `confirm_quit` which locks the vault and exits atomically.
#[tauri::command]
fn prepare_for_quit(state: tauri::State<AppState>) {
    vault_session::close_session(&state.vault_session);
}

/// Called by the frontend after save/cleanup. Backend locks vault and exits (no window.destroy IPC race).
#[tauri::command]
fn confirm_quit(app: AppHandle, state: tauri::State<AppState>) {
    if QUIT_IN_PROGRESS.swap(true, Ordering::SeqCst) {
        tracing::debug!("confirm_quit ignored — already quitting");
        return;
    }
    vault_session::close_session(&state.vault_session);
    tracing::info!("application exit confirmed");
    app.exit(0);
    // Global shortcut plugin can keep the event loop alive after app.exit on Linux.
    #[cfg(target_os = "linux")]
    std::process::exit(0);
}

#[tauri::command]
fn copy_text_to_clipboard(text: String) -> Result<(), String> {
    let mut clipboard = Clipboard::new()
        .map_err(|e| format!("Clipboard unavailable: {}", e))?;
    clipboard
        .set_text(text)
        .map_err(|e| format!("Failed to copy to clipboard: {}", e))?;
    Ok(())
}

#[tauri::command]
fn write_note(
    title: String,
    content: String,
    is_ephemeral: bool,
    previous_filename: Option<String>,
) -> Result<Note, String> {
    filesystem::write_note(
        &title,
        &content,
        is_ephemeral,
        previous_filename.as_deref(),
    )
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

static INTENTIONALLY_HIDDEN: AtomicBool = AtomicBool::new(false);
static QUIT_IN_PROGRESS: AtomicBool = AtomicBool::new(false);
#[cfg(target_os = "macos")]
static APP_WAS_ACTIVE: AtomicBool = AtomicBool::new(true);

pub(crate) fn emit_quit_requested(app: &AppHandle) {
    if QUIT_IN_PROGRESS.load(Ordering::SeqCst) {
        return;
    }
    let _ = app.emit("app-quit-requested", ());
}

fn show_main_window(app: &AppHandle) {
    INTENTIONALLY_HIDDEN.store(false, Ordering::SeqCst);
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn hide_main_window(app: &AppHandle, intentional: bool) {
    if intentional {
        INTENTIONALLY_HIDDEN.store(true, Ordering::SeqCst);
    }
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

pub(crate) fn menu_show_main_window(app: &AppHandle) {
    show_main_window(app);
}

pub(crate) fn menu_hide_main_window(app: &AppHandle) {
    hide_main_window(app, true);
}

fn toggle_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        match window.is_visible() {
            Ok(true) => hide_main_window(app, true),
            _ => show_main_window(app),
        }
    }
}

fn maybe_show_on_app_activate(app: &AppHandle) {
    #[cfg(not(target_os = "macos"))]
    {
        let _ = app;
        return;
    }
    #[cfg(target_os = "macos")]
    {
        let active = crate::macos_app::application_is_active();
        let was_active = APP_WAS_ACTIVE.swap(active, Ordering::SeqCst);

        if !active || was_active || INTENTIONALLY_HIDDEN.load(Ordering::SeqCst) {
            return;
        }

        if let Some(window) = app.get_webview_window("main") {
            if window.is_visible().unwrap_or(true) == false {
                show_main_window(app);
            }
        }
    }
}

fn summon_shortcut() -> &'static str {
    shortcuts::summon_shortcut()
}

fn register_global_shortcut(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let shortcut = summon_shortcut();
    app.plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_shortcuts([shortcut])?
            .with_handler(|app, _shortcut, event| {
                if event.state() == ShortcutState::Pressed {
                    toggle_window(app);
                }
            })
            .build(),
    )?;
    tracing::info!("registered global shortcut {shortcut}");
    Ok(())
}

#[cfg(target_os = "macos")]
fn attach_hide_on_blur(window: &tauri::WebviewWindow) {
    let hide_generation = Arc::new(AtomicU64::new(0));
    let window_for_events = window.clone();

    window_for_events.clone().on_window_event(move |event| {
        match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                let _ = window_for_events.emit("window-close-requested", ());
            }
            tauri::WindowEvent::Focused(false) => {
                let generation = hide_generation.fetch_add(1, Ordering::SeqCst) + 1;
                let w = window_for_events.clone();
                let hide_generation = hide_generation.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(200));
                    if hide_generation.load(Ordering::SeqCst) != generation {
                        return;
                    }
                    if w.is_focused().unwrap_or(true) {
                        return;
                    }
                    let _ = w.hide();
                });
            }
            tauri::WindowEvent::Focused(true) => {
                hide_generation.fetch_add(1, Ordering::SeqCst);
            }
            _ => {}
        }
    });
}

#[cfg(not(target_os = "macos"))]
fn attach_window_close_handler(_window: &tauri::WebviewWindow) {
    // Native close is handled via getCurrentWindow().onCloseRequested in the frontend (Linux).
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    logging::init();

    let clipboard_history = Arc::new(Mutex::new(ClipboardHistory::new(15)));
    let clipboard_filter_enabled = Arc::new(Mutex::new(true));
    clipboard::start_clipboard_monitor(
        clipboard_history.clone(),
        clipboard_filter_enabled.clone(),
    );

    let vault_session = Arc::new(Mutex::new(None));
    let unlock_limiter = Arc::new(Mutex::new(unlock_limiter::UnlockRateLimiter::new()));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .manage(AppState {
            clipboard_history,
            clipboard_filter_enabled,
            vault_session,
            unlock_limiter,
        })
        .setup(move |app| {
            if let Err(e) = menu::install_app_menu(&app.handle()) {
                tracing::warn!("failed to install app menu: {e}");
            }

            #[cfg(desktop)]
            if let Err(e) = register_global_shortcut(&app.handle()) {
                tracing::warn!("failed to register global shortcut: {e}");
            }

            #[cfg(target_os = "linux")]
            if let Err(e) = linux_tray::install_tray(&app.handle()) {
                tracing::warn!("failed to install system tray: {e}");
            }

            if let Some(window) = app.get_webview_window("main") {
                #[cfg(target_os = "macos")]
                attach_hide_on_blur(&window);
                #[cfg(not(target_os = "macos"))]
                attach_window_close_handler(&window);
            }

            Ok(())
        })
        .on_menu_event(|app, event| {
            menu::handle_menu_event(app, event.id().0.as_str());
        })
        .invoke_handler(tauri::generate_handler![
            get_app_version_info,
            get_summon_shortcut_label,
            prepare_for_quit,
            confirm_quit,
            copy_text_to_clipboard,
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
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            if let RunEvent::MainEventsCleared = event {
                maybe_show_on_app_activate(app_handle);
            }
            if let RunEvent::ExitRequested { api, .. } = event {
                api.prevent_exit();
                emit_quit_requested(app_handle);
            }
        });
}

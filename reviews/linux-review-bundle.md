# Linux code bundle

### src-tauri/src/menu/linux.rs
```
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Manager};
use tauri_plugin_opener::OpenerExt;

use super::{emit_to_frontend, focus_main_window};

pub fn install(app: &AppHandle) -> tauri::Result<()> {
    let new_draft = MenuItem::with_id(
        app,
        "new_draft",
        "New Scratchpad Draft",
        true,
        Some("CmdOrCtrl+N"),
    )?;
    let new_saved = MenuItem::with_id(
        app,
        "new_saved_note",
        "New Saved Note",
        true,
        Some("CmdOrCtrl+Shift+N"),
    )?;
    let save_note = MenuItem::with_id(
        app,
        "save_note",
        "Save",
        true,
        Some("CmdOrCtrl+S"),
    )?;
    let save_permanent = MenuItem::with_id(
        app,
        "save_permanent",
        "Save as Permanent Note",
        true,
        Some("CmdOrCtrl+Shift+S"),
    )?;
    let delete_note = MenuItem::with_id(app, "delete_note", "Delete Current Note", true, None::<&str>)?;
    let archive_stale =
        MenuItem::with_id(app, "archive_stale", "Archive Stale Scratchpad Notes", true, None::<&str>)?;
    let quit = PredefinedMenuItem::quit(app, Some("Quit Gnomad Slate"))?;

    let file_submenu = Submenu::with_items(
        app,
        "File",
        true,
        &[
            &new_draft,
            &new_saved,
            &save_note,
            &save_permanent,
            &PredefinedMenuItem::separator(app)?,
            &delete_note,
            &archive_stale,
            &PredefinedMenuItem::separator(app)?,
            &quit,
        ],
    )?;

    let edit_submenu = Submenu::with_items(
        app,
        "Edit",
        true,
        &[
            &PredefinedMenuItem::cut(app, None)?,
            &PredefinedMenuItem::copy(app, None)?,
            &PredefinedMenuItem::paste(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::select_all(app, None)?,
        ],
    )?;

    let view_edit = MenuItem::with_id(app, "view_edit", "Edit Mode", true, None::<&str>)?;
    let view_split = MenuItem::with_id(app, "view_split", "Split Mode", true, None::<&str>)?;
    let view_preview = MenuItem::with_id(app, "view_preview", "Preview Mode", true, None::<&str>)?;
    let command_palette = MenuItem::with_id(
        app,
        "command_palette",
        "Command Palette",
        true,
        Some("CmdOrCtrl+P"),
    )?;
    let clipboard_history = MenuItem::with_id(
        app,
        "clipboard_history",
        "Clipboard History",
        true,
        Some("CmdOrCtrl+Shift+V"),
    )?;
    let toggle_line_numbers =
        MenuItem::with_id(app, "toggle_line_numbers", "Show Line Numbers", true, None::<&str>)?;
    let theme_light = MenuItem::with_id(app, "theme_light", "Light Theme", true, None::<&str>)?;
    let theme_dark = MenuItem::with_id(app, "theme_dark", "Dark Theme", true, None::<&str>)?;
    let theme_system = MenuItem::with_id(app, "theme_system", "System Theme", true, None::<&str>)?;

    let view_submenu = Submenu::with_items(
        app,
        "View",
        true,
        &[
            &view_edit,
            &view_split,
            &view_preview,
            &PredefinedMenuItem::separator(app)?,
            &command_palette,
            &clipboard_history,
            &PredefinedMenuItem::separator(app)?,
            &toggle_line_numbers,
            &PredefinedMenuItem::separator(app)?,
            &theme_light,
            &theme_dark,
            &theme_system,
        ],
    )?;

    let window_minimize = MenuItem::with_id(app, "window_minimize", "Minimize", true, None::<&str>)?;
    let window_hide = MenuItem::with_id(app, "window_hide", "Hide Window", true, Some("Alt+Space"))?;
    let window_show = MenuItem::with_id(app, "window_show", "Show Window", true, None::<&str>)?;
    let window_close = MenuItem::with_id(app, "window_close", "Close Window", true, None::<&str>)?;

    let window_submenu = Submenu::with_items(
        app,
        "Window",
        true,
        &[
            &window_minimize,
            &window_hide,
            &window_show,
            &PredefinedMenuItem::separator(app)?,
            &window_close,
        ],
    )?;

    let help_guide = MenuItem::with_id(app, "help_guide", "Gnomad Slate Help", true, Some("F1"))?;
    let contact_support =
        MenuItem::with_id(app, "contact_support", "Contact Support…", true, None::<&str>)?;
    let about = MenuItem::with_id(app, "about", "About Gnomad Slate", true, None::<&str>)?;
    let check_updates =
        MenuItem::with_id(app, "check_updates", "Check for Updates…", true, None::<&str>)?;

    let help_submenu = Submenu::with_items(
        app,
        "Help",
        true,
        &[
            &help_guide,
            &contact_support,
            &PredefinedMenuItem::separator(app)?,
            &about,
            &check_updates,
        ],
    )?;

    let root = Menu::with_items(
        app,
        &[
            &file_submenu,
            &edit_submenu,
            &view_submenu,
            &window_submenu,
            &help_submenu,
        ],
    )?;

    app.set_menu(root)?;
    Ok(())
}

pub fn handle_event(app: &AppHandle, event_id: &str) {
    match event_id {
        "window_minimize" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.minimize();
            }
        }
        "window_hide" => {
            crate::menu_hide_main_window(app);
        }
        "window_show" => {
            crate::menu_show_main_window(app);
        }
        "window_close" => {
            emit_to_frontend(app, "window-close-requested");
        }
        "contact_support" => {
            let _ = app.opener().open_url(
                "mailto:david@gnomadstudio.org?subject=Gnomad%20Slate%20Linux%20Support",
                None::<&str>,
            );
            focus_main_window(app);
        }
        "about" => emit_to_frontend(app, "menu-about"),
        "check_updates" => emit_to_frontend(app, "menu-check-updates"),
        "new_draft" => emit_to_frontend(app, "menu-new-draft"),
        "new_saved_note" => emit_to_frontend(app, "menu-new-saved-note"),
        "save_note" => emit_to_frontend(app, "menu-save"),
        "save_permanent" => emit_to_frontend(app, "menu-save-permanent"),
        "delete_note" => emit_to_frontend(app, "menu-delete-note"),
        "archive_stale" => emit_to_frontend(app, "menu-archive-stale"),
        "view_edit" => emit_to_frontend(app, "menu-view-edit"),
        "view_split" => emit_to_frontend(app, "menu-view-split"),
        "view_preview" => emit_to_frontend(app, "menu-view-preview"),
        "command_palette" => emit_to_frontend(app, "menu-command-palette"),
        "clipboard_history" => emit_to_frontend(app, "menu-clipboard-history"),
        "toggle_line_numbers" => emit_to_frontend(app, "menu-toggle-line-numbers"),
        "theme_light" => emit_to_frontend(app, "menu-theme-light"),
        "theme_dark" => emit_to_frontend(app, "menu-theme-dark"),
        "theme_system" => emit_to_frontend(app, "menu-theme-system"),
        "help_guide" => emit_to_frontend(app, "menu-help"),
        _ => {}
    }
}

```

### src-tauri/src/menu/mod.rs
```
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
mod fallback;

use tauri::{AppHandle, Emitter, Manager};

pub fn install_app_menu(app: &AppHandle) -> tauri::Result<()> {
    #[cfg(target_os = "linux")]
    return linux::install(app);
    #[cfg(target_os = "macos")]
    return macos::install(app);
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    return fallback::install(app);
}

pub fn handle_menu_event(app: &AppHandle, event_id: &str) {
    #[cfg(target_os = "linux")]
    linux::handle_event(app, event_id);
    #[cfg(target_os = "macos")]
    macos::handle_event(app, event_id);
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    fallback::handle_event(app, event_id);
}

pub(crate) fn focus_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

pub(crate) fn emit_to_frontend(app: &AppHandle, event: &str) {
    let _ = app.emit(event, ());
    focus_main_window(app);
}

```

### src-tauri/src/lib.rs
```
pub mod biometrics;
pub mod clipboard;
pub mod clipboard_filter;
pub mod crypto;
pub mod filesystem;
pub mod macos_app;
pub mod menu;
pub mod pasteboard_macos;
pub mod unlock_limiter;
pub mod vault_session;

use serde::Serialize;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
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
fn get_summon_shortcut_label() -> String {
    "Alt+Space".to_string()
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
static APP_WAS_ACTIVE: AtomicBool = AtomicBool::new(true);

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

fn register_global_shortcut(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    app.plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_shortcuts(["Alt+Space"])?
            .with_handler(|app, _shortcut, event| {
                if event.state() == ShortcutState::Pressed {
                    toggle_window(app);
                }
            })
            .build(),
    )?;
    eprintln!("Gnomad Slate: registered global shortcut Alt+Space (Option+Space on macOS)");
    Ok(())
}

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

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .manage(AppState {
            clipboard_history,
            clipboard_filter_enabled,
            vault_session,
            unlock_limiter,
        })
        .setup(move |app| {
            if let Err(e) = menu::install_app_menu(&app.handle()) {
                eprintln!("Gnomad Slate: failed to install app menu: {e}");
            }

            #[cfg(desktop)]
            if let Err(e) = register_global_shortcut(&app.handle()) {
                eprintln!("Gnomad Slate: failed to register global shortcut: {e}");
            }

            if let Some(window) = app.get_webview_window("main") {
                attach_hide_on_blur(&window);
            }

            Ok(())
        })
        .on_menu_event(|app, event| {
            menu::handle_menu_event(app, event.id().0.as_str());
        })
        .invoke_handler(tauri::generate_handler![
            get_app_version_info,
            get_summon_shortcut_label,
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
        });
}

```

### src-tauri/tauri.conf.json
```
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Gnomad Slate",
  "version": "2.1.0-beta.2",
  "identifier": "com.gnomadstudio.gnomad-slate",
  "build": {
    "beforeDevCommand": "npm run dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "npm run build",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "label": "main",
        "title": "Gnomad Slate",
        "width": 1120,
        "height": 640,
        "minWidth": 960,
        "minHeight": 520,
        "theme": "Dark",
        "backgroundColor": "#0d0d10"
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "createUpdaterArtifacts": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "macOS": {
      "minimumSystemVersion": "10.15"
    },
    "windows": {
      "webviewInstallMode": {
        "type": "downloadBootstrapper"
      }
    }
  },
  "plugins": {
    "updater": {
      "pubkey": "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDFBQTkyNUU2QzQyQjdEQwpSV1RjdDBKc1hwS3FBYlNCU1RjaXR6N2thWVhiYTRMUHFQZnM3b25FcTM2eklPQnVZQUNiODd3Swo=",
      "endpoints": [
        "https://github.com/davidthegnomad/gnomad-slate/releases/latest/download/latest.json"
      ]
    }
  }
}

```

### src-tauri/capabilities/default.json
```
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for the main window",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "opener:default",
    "global-shortcut:default",
    "updater:default",
    "process:default",
    "os:default"
  ]
}

```

### src-tauri/Cargo.toml
```
[package]
name = "tauri-app"
version = "2.1.0-beta.2"
description = "Gnomad Slate — desktop scratchpad and encrypted secrets vault"
authors = ["Gnomad Studio"]
edition = "2021"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[lib]
# The `_lib` suffix may seem redundant but it is necessary
# to make the lib name unique and wouldn't conflict with the bin name.
# This seems to be only an issue on Windows, see https://github.com/rust-lang/cargo/issues/8519
name = "tauri_app_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-opener = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
aes-gcm = "0.10.3"
argon2 = "0.5.3"
rand = "0.8"
arboard = "3.6.1"
dirs = "6.0.0"
tauri-plugin-global-shortcut = "2.3.2"
sha2 = "0.11.0"
zeroize = { version = "1.8", features = ["zeroize_derive"] }
tauri-plugin-updater = { version = "2.10.1", features = ["rustls-tls"] }
tauri-plugin-process = "2.3.1"
tauri-plugin-os = "2"

[target.'cfg(target_os = "macos")'.dependencies]
objc2 = "0.5"
objc2-foundation = "0.2"
objc2-app-kit = { version = "0.3", features = ["NSPasteboard", "NSRunningApplication"] }


```

### src/App.tsx
```
import { useState, useEffect, useRef, useCallback, useMemo, type ButtonHTMLAttributes } from "react";
import { invoke, listen, getPlatformWindow, LogicalSize, quitDesktopApp } from "./utils/ipc";
import { AboutDialog } from "./components/AboutDialog";
import { HelpDialog } from "./components/HelpDialog";
import UnsavedChangesModal from "./components/UnsavedChangesModal";
import { useLinuxMenuBridge } from "./hooks/useLinuxMenuBridge";
import { renderMarkdownToHtml } from "./utils/markdown";
import { modKeyLabel, modShortcut } from "./utils/modKeyLabel";
import { isLinuxDesktop } from "./utils/platform";
import { resolveTheme, type ThemePreference } from "./utils/theme";
import "./App.css";

interface Note {
  filename: string;
  title: string;
  content: string;
  is_ephemeral: boolean;
  last_modified: number;
}

interface ClipboardEntry {
  text: string;
  timestamp: number;
}

interface SecretMeta {
  index: number;
  name: string;
  path: string;
}

interface AppVersionInfo {
  version: string;
  channel: string;
  display_version: string;
}

const NOTE_SAVE_DEBOUNCE_MS = 400;

function TipButton({
  tip,
  tipBelow = false,
  className,
  children,
  ...rest
}: ButtonHTMLAttributes<HTMLButtonElement> & { tip: string; tipBelow?: boolean }) {
  return (
    <button
      {...rest}
      className={`has-tooltip${tipBelow ? " tooltip-below" : ""}${className ? ` ${className}` : ""}`}
      data-tooltip={tip}
    >
      {children}
    </button>
  );
}

function invokeErrorMessage(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  return String(error);
}

function App() {
  // --- Notes State ---
  const [notes, setNotes] = useState<Note[]>([]);
  const [selectedNote, setSelectedNote] = useState<Note | null>(null);
  const [editorTitle, setEditorTitle] = useState("");
  const [editorContent, setEditorContent] = useState("");
  const [isEphemeral, setIsEphemeral] = useState(true);
  const [searchQuery, setSearchQuery] = useState("");
  const [activeTab, setActiveTab] = useState<"all" | "scratch" | "vault" | "secrets">("scratch");
  const [editorView, setEditorView] = useState<"edit" | "preview" | "split">("edit");

  // --- Clipboard Capsule State ---
  const [showClipboard, setShowClipboard] = useState(false);
  const [clipboardHistory, setClipboardHistory] = useState<ClipboardEntry[]>([]);
  const [selectedClipIdx, setSelectedClipIdx] = useState(0);

  // --- Command Palette State ---
  const [showPalette, setShowPalette] = useState(false);
  const [paletteQuery, setPaletteQuery] = useState("");
  const [selectedPaletteIdx, setSelectedPaletteIdx] = useState(0);

  // --- Secure Vault (Vault-X) State ---
  const [isVaultInitialized, setIsVaultInitialized] = useState(false);
  const [isVaultUnlocked, setIsVaultUnlocked] = useState(false);
  const [vaultPassword, setVaultPassword] = useState("");
  const [secretMetadata, setSecretMetadata] = useState<SecretMeta[]>([]);
  const [revealedIndex, setRevealedIndex] = useState<number | null>(null);
  const [revealedValue, setRevealedValue] = useState("");
  
  // Add Secret Form
  const [showAddSecret, setShowAddSecret] = useState(false);
  const [newSecretName, setNewSecretName] = useState("");
  const [newSecretPath, setNewSecretPath] = useState("");
  const [newSecretValue, setNewSecretValue] = useState("");

  const [showAbout, setShowAbout] = useState(false);
  const [showHelp, setShowHelp] = useState(false);
  const [aboutAutoCheck, setAboutAutoCheck] = useState(false);
  const [appVersionLabel, setAppVersionLabel] = useState("v2.1 beta");
  const [isLinux] = useState(() => isLinuxDesktop());
  const [showLineNumbers, setShowLineNumbers] = useState(() => {
    return localStorage.getItem("gnomad-line-numbers") === "true";
  });
  const [closePromptOpen, setClosePromptOpen] = useState(false);
  const pendingWindowClose = useRef(false);

  const editorRef = useRef<HTMLTextAreaElement>(null);
  const saveDebounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const editorTitleRef = useRef(editorTitle);
  const editorContentRef = useRef(editorContent);
  const isEphemeralRef = useRef(isEphemeral);
  const activeTabRef = useRef(activeTab);

  const [saveFeedback, setSaveFeedback] = useState<string | null>(null);
  const saveFeedbackTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    editorTitleRef.current = editorTitle;
  }, [editorTitle]);

  useEffect(() => {
    editorContentRef.current = editorContent;
  }, [editorContent]);

  useEffect(() => {
    isEphemeralRef.current = isEphemeral;
  }, [isEphemeral]);

  useEffect(() => {
    activeTabRef.current = activeTab;
  }, [activeTab]);

  // --- Theme State ---
  const [theme, setTheme] = useState<ThemePreference>(() => {
    return (localStorage.getItem("gnomad-theme") as ThemePreference) || "dark";
  });

  const [clipboardFilterEnabled, setClipboardFilterEnabled] = useState(() => {
    return localStorage.getItem("gnomad-clipboard-filter") !== "false";
  });

  const previewHtml = useMemo(
    () => renderMarkdownToHtml(editorContent),
    [editorContent]
  );

  useEffect(() => {
    localStorage.setItem("gnomad-theme", theme);

    const applyTheme = () => {
      const active = resolveTheme(
        theme,
        window.matchMedia("(prefers-color-scheme: dark)").matches
      );
      if (active === "light") {
        document.documentElement.setAttribute("data-theme", "light");
      } else {
        document.documentElement.removeAttribute("data-theme");
      }
      const win = getPlatformWindow();
      void win.setTheme(active);
      void win.setBackgroundColor(active === "light" ? "#f3f4f6" : "#0d0d10");
    };

    applyTheme();

    if (theme === "system") {
      const media = window.matchMedia("(prefers-color-scheme: dark)");
      const listener = () => applyTheme();
      media.addEventListener("change", listener);
      return () => media.removeEventListener("change", listener);
    }
  }, [theme]);

  useEffect(() => {
    localStorage.setItem("gnomad-clipboard-filter", String(clipboardFilterEnabled));
    void invoke("set_clipboard_filter_enabled", { enabled: clipboardFilterEnabled });
  }, [clipboardFilterEnabled]);

  useEffect(() => {
    void invoke<boolean>("get_clipboard_filter_enabled")
      .then((enabled) => setClipboardFilterEnabled(enabled))
      .catch(() => undefined);
  }, []);

  useEffect(() => {
    if (isLinux) {
      document.documentElement.setAttribute("data-platform", "linux");
    }
  }, [isLinux]);

  useEffect(() => {
    if (isLinux) {
      localStorage.setItem("gnomad-line-numbers", String(showLineNumbers));
    }
  }, [isLinux, showLineNumbers]);

  useEffect(() => {
    void invoke<AppVersionInfo>("get_app_version_info")
      .then((info) => setAppVersionLabel(info.display_version))
      .catch(() => undefined);
  }, []);

  useEffect(() => {
    void (async () => {
      const win = getPlatformWindow();
      const size = await win.innerSize();
      if (size.width < 960) {
        await win.setSize(new LogicalSize(1120, 640));
      }
    })();
  }, []);

  useEffect(() => {
    return () => {
      if (saveFeedbackTimerRef.current) {
        clearTimeout(saveFeedbackTimerRef.current);
      }
    };
  }, []);

  // --- Palette Commands ---
  const paletteCommands = [
    { name: "Format JSON", desc: "Prettifies and validates raw JSON in the editor", shortcut: `${modKeyLabel()}+F` },
    { name: "Prettify Markdown", desc: "Cleans list structure and header spacing", shortcut: `${modKeyLabel()}+L` },
    { name: "Copy as Raw HTML", desc: "Compiles Markdown to sanitized HTML and copies", shortcut: `${modKeyLabel()}+Shift+H` },
    { name: "Create Daily Note", desc: "Creates or appends to daily journal note", shortcut: "⌘D" },
    { name: "Append to Daily Note", desc: "Appends selected text directly to today's log", shortcut: "⌘⌥D" },
    { name: "Export selection to GitHub Gist", desc: "Publishes selected text as an online gist", shortcut: "⌘G" },
    { name: "Lock / Unlock Secret Vault", desc: "Toggles Vault-X secure credentials lock", shortcut: "⌘⌥L" },
    { name: "Switch to Light Theme", desc: "Sets theme to a crisp white minimalist style", shortcut: "" },
    { name: "Switch to Dark Theme", desc: "Sets theme to deep obsidian neon style", shortcut: "" },
    { name: "Switch to System Theme", desc: "Matches system preferences dynamically", shortcut: "" },
  ];

  // --- Load Initial Notes & Check Vault ---
  useEffect(() => {
    loadNotes();
    checkVaultStatus();
    
    // Auto-run ephemeral sweep on launch
    invoke("archive_notes").then((count) => {
      if (count && (count as number) > 0) {
        console.log(`Auto-archived ${count} stale temporary notes.`);
        loadNotes();
      }
    });
  }, []);

  const loadNotes = async () => {
    try {
      const allNotes = await invoke<Note[]>("list_notes");
      setNotes(allNotes);
      
      // Select first note if none is selected
      if (allNotes.length > 0 && !selectedNote) {
        selectNote(allNotes[0]);
      }
    } catch (e) {
      console.error("Failed to load notes:", e);
    }
  };

  const checkVaultStatus = async () => {
    try {
      const initialized = await invoke<boolean>("is_vault_initialized");
      const unlocked = await invoke<boolean>("is_vault_unlocked");
      setIsVaultInitialized(initialized);
      setIsVaultUnlocked(unlocked);
      if (unlocked) {
        await loadSecretMetadata();
      }
    } catch (e) {
      console.error(e);
    }
  };

  const loadSecretMetadata = async () => {
    try {
      const meta = await invoke<SecretMeta[]>("list_secret_metadata");
      setSecretMetadata(meta);
    } catch (e) {
      console.error("Failed to load secret metadata:", e);
      setSecretMetadata([]);
    }
  };

  const clearRevealedSecret = () => {
    setRevealedIndex(null);
    setRevealedValue("");
  };

  const selectNote = async (note: Note) => {
    await flushPendingSave();
    setSelectedNote(note);
    setEditorTitle(note.title);
    // Strip ephemeral comment if visual presentation
    let content = note.content;
    if (content.startsWith("<!-- ephemeral -->\n")) {
      content = content.replace("<!-- ephemeral -->\n", "");
    }
    setEditorContent(content);
    setIsEphemeral(note.is_ephemeral);
  };

  const showSaveFeedback = useCallback((message: string) => {
    setSaveFeedback(message);
    if (saveFeedbackTimerRef.current) {
      clearTimeout(saveFeedbackTimerRef.current);
    }
    saveFeedbackTimerRef.current = setTimeout(() => {
      setSaveFeedback(null);
      saveFeedbackTimerRef.current = null;
    }, 1500);
  }, []);

  const saveNoteNow = useCallback(
    async (options?: { promoteToSaved?: boolean }) => {
      if (activeTabRef.current === "secrets") {
        return;
      }

      if (saveDebounceRef.current) {
        clearTimeout(saveDebounceRef.current);
        saveDebounceRef.current = null;
      }

      const title = editorTitleRef.current.trim() || "Untitled note";
      const content = editorContentRef.current;
      const promote = options?.promoteToSaved ?? false;
      const ephemeralState = promote ? false : isEphemeralRef.current;

      if (!editorTitleRef.current.trim()) {
        setEditorTitle(title);
        editorTitleRef.current = title;
      }

      try {
        const previousFilename = selectedNote?.filename ?? null;
        const saved = await invoke<Note>("write_note", {
          title,
          content,
          isEphemeral: ephemeralState,
          previousFilename,
        });

        setNotes((prev) => {
          const filtered = prev.filter((n) => n.filename !== saved.filename);
          if (previousFilename && previousFilename !== saved.filename) {
            return [saved, ...filtered.filter((n) => n.filename !== previousFilename)];
          }
          return [saved, ...filtered];
        });

        setSelectedNote(saved);
        setIsEphemeral(saved.is_ephemeral);
        isEphemeralRef.current = saved.is_ephemeral;
        showSaveFeedback(promote || !saved.is_ephemeral ? "Saved" : "Draft saved");
      } catch (e) {
        console.error("Save failed:", e);
        showSaveFeedback("Save failed");
      }
    },
    [showSaveFeedback, selectedNote]
  );

  const createNewDraft = useCallback(async () => {
    await saveNoteNow();

    const draftTitle = `Note_${new Date().toLocaleTimeString().replace(/\s/g, "_")}`;
    setEditorTitle(draftTitle);
    setEditorContent("");
    setIsEphemeral(true);
    setActiveTab("scratch");
    editorTitleRef.current = draftTitle;
    editorContentRef.current = "";
    isEphemeralRef.current = true;
    activeTabRef.current = "scratch";

    try {
      const saved = await invoke<Note>("write_note", {
        title: draftTitle,
        content: "",
        isEphemeral: true,
        previousFilename: null,
      });
      setNotes((prev) => {
        const filtered = prev.filter((n) => n.filename !== saved.filename);
        return [saved, ...filtered];
      });
      setSelectedNote(saved);
    } catch (e) {
      console.error("Failed to create draft:", e);
    }
  }, [saveNoteNow]);

  const createNewSavedNote = useCallback(async () => {
    await saveNoteNow();

    const title = `Note_${new Date().toISOString().slice(0, 10)}`;
    setEditorTitle(title);
    setEditorContent("");
    setIsEphemeral(false);
    setActiveTab("vault");
    editorTitleRef.current = title;
    editorContentRef.current = "";
    isEphemeralRef.current = false;
    activeTabRef.current = "vault";

    try {
      const saved = await invoke<Note>("write_note", {
        title,
        content: "",
        isEphemeral: false,
        previousFilename: null,
      });
      setNotes((prev) => {
        const filtered = prev.filter((n) => n.filename !== saved.filename);
        return [saved, ...filtered];
      });
      setSelectedNote(saved);
    } catch (e) {
      console.error("Failed to create saved note:", e);
    }
  }, [saveNoteNow]);

  const deleteCurrentNote = useCallback(async () => {
    if (!selectedNote) {
      return;
    }
    try {
      await invoke("delete_note", { filename: selectedNote.filename });
      setNotes((prev) => prev.filter((n) => n.filename !== selectedNote.filename));
      setSelectedNote(null);
      setEditorTitle("");
      setEditorContent("");
    } catch (e) {
      console.error(e);
    }
  }, [selectedNote]);

  const closeApp = useCallback(async () => {
    if (isLinux) {
      await quitDesktopApp();
      return;
    }
    const win = getPlatformWindow();
    await win.destroy();
  }, [isLinux]);

  const handleEditorShortcut = useCallback(
    (e: React.KeyboardEvent) => {
      const isCmdOrCtrl = e.metaKey || e.ctrlKey;
      if (!isCmdOrCtrl) {
        return;
      }

      if (!e.shiftKey && (e.key === "s" || e.key === "S")) {
        e.preventDefault();
        void saveNoteNow({ promoteToSaved: isEphemeralRef.current });
        return;
      }

      if (!e.shiftKey && (e.key === "n" || e.key === "N")) {
        e.preventDefault();
        void createNewDraft();
      }
    },
    [saveNoteNow, createNewDraft]
  );

  const saveActiveNote = useCallback(
    async (overrideContent?: string, overrideTitle?: string, forceVault?: boolean) => {
      const titleToSave = overrideTitle !== undefined ? overrideTitle : editorTitle;
      const contentToSave = overrideContent !== undefined ? overrideContent : editorContent;
      const ephemeralState = forceVault !== undefined ? !forceVault : isEphemeral;

      if (!titleToSave.trim()) return;

      try {
        const previousFilename = selectedNote?.filename ?? null;
        const saved = await invoke<Note>("write_note", {
          title: titleToSave,
          content: contentToSave,
          isEphemeral: ephemeralState,
          previousFilename,
        });

        setNotes((prev) => {
          const filtered = prev.filter((n) => n.filename !== saved.filename);
          if (previousFilename && previousFilename !== saved.filename) {
            return [saved, ...filtered.filter((n) => n.filename !== previousFilename)];
          }
          return [saved, ...filtered];
        });

        setSelectedNote(saved);
        setIsEphemeral(saved.is_ephemeral);
      } catch (e) {
        console.error("Autosave failed:", e);
      }
    },
    [editorTitle, editorContent, isEphemeral, selectedNote]
  );

  const flushPendingSave = useCallback(async () => {
    if (saveDebounceRef.current) {
      clearTimeout(saveDebounceRef.current);
      saveDebounceRef.current = null;
      await saveActiveNote();
    }
  }, [saveActiveNote]);

  const scheduleSaveNote = useCallback(
    (overrideContent?: string, overrideTitle?: string, forceVault?: boolean) => {
      if (saveDebounceRef.current) {
        clearTimeout(saveDebounceRef.current);
      }
      saveDebounceRef.current = setTimeout(() => {
        saveDebounceRef.current = null;
        void saveActiveNote(overrideContent, overrideTitle, forceVault);
      }, NOTE_SAVE_DEBOUNCE_MS);
    },
    [saveActiveNote]
  );

  useEffect(() => {
    return () => {
      if (saveDebounceRef.current) {
        clearTimeout(saveDebounceRef.current);
      }
    };
  }, []);

  // --- Delete Note Function ---
  const deleteNote = async (filename: string, e: React.MouseEvent) => {
    e.stopPropagation();
    try {
      await invoke("delete_note", { filename });
      setNotes((prev) => prev.filter((n) => n.filename !== filename));
      if (selectedNote?.filename === filename) {
        setSelectedNote(null);
        setEditorTitle("");
        setEditorContent("");
      }
    } catch (e) {
      console.error(e);
    }
  };

  const lineNumberRows = useMemo(() => {
    const lineCount = Math.max(1, editorContent.split("\n").length);
    return Array.from({ length: lineCount }, (_, index) => index + 1);
  }, [editorContent]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void listen("window-close-requested", () => {
      if (saveDebounceRef.current) {
        pendingWindowClose.current = true;
        setClosePromptOpen(true);
        return;
      }
      void (async () => {
        await saveNoteNow();
        await closeApp();
      })();
    }).then((fn) => {
      unlisten = fn;
    });
    return () => {
      unlisten?.();
    };
  }, [saveNoteNow, closeApp]);

  // --- Keydown Event Handling for Universal Hotkeys ---
  useEffect(() => {
    const isEditableTarget = (target: EventTarget | null): boolean => {
      if (!(target instanceof HTMLElement)) return false;
      const tag = target.tagName;
      return tag === "INPUT" || tag === "TEXTAREA" || target.isContentEditable;
    };

    const handleKeyDown = (e: KeyboardEvent) => {
      const isCmdOrCtrl = e.metaKey || e.ctrlKey;

      // F1: Help (Linux)
      if (isLinux && e.key === "F1") {
        e.preventDefault();
        setShowHelp(true);
        return;
      }

      // Esc: Dismiss overlays
      if (e.key === "Escape") {
        setShowClipboard(false);
        setShowPalette(false);
        return;
      }

      // ⌘S: Save current note (promotes scratchpad drafts to Saved)
      if (isCmdOrCtrl && !e.shiftKey && (e.key === "s" || e.key === "S")) {
        e.preventDefault();
        e.stopPropagation();
        if (activeTabRef.current !== "secrets") {
          void saveNoteNow({ promoteToSaved: isEphemeralRef.current });
        }
        return;
      }

      // ⌘N: New scratchpad draft
      if (isCmdOrCtrl && !e.shiftKey && (e.key === "n" || e.key === "N")) {
        e.preventDefault();
        e.stopPropagation();
        if (activeTabRef.current !== "secrets") {
          void createNewDraft();
        }
        return;
      }

      // ⌘P: Open Command Palette (not while typing in a field — allow select-all etc.)
      if (
        isCmdOrCtrl &&
        e.key.toLowerCase() === "p" &&
        !isEditableTarget(e.target)
      ) {
        e.preventDefault();
        setPaletteQuery("");
        setSelectedPaletteIdx(0);
        setShowPalette((prev) => !prev);
        setShowClipboard(false);
      }

      // ⌘⇧V: Open Clipboard History Capsule
      if (isCmdOrCtrl && e.shiftKey && e.key.toLowerCase() === "v") {
        e.preventDefault();
        loadClipboardHistory();
        setSelectedClipIdx(0);
        setShowClipboard((prev) => !prev);
        setShowPalette(false);
      }
    };

    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [saveNoteNow, createNewDraft, isLinux]);

  // --- Clipboard History Fetch ---
  const loadClipboardHistory = async () => {
    try {
      // In the browser extension, capture the active clipboard text first
      if (typeof window !== "undefined" && !(window as any).__TAURI_INTERNALS__ && typeof navigator !== "undefined" && navigator.clipboard?.readText) {
        try {
          const text = await navigator.clipboard.readText();
          if (text && text.trim()) {
            const chromeApi = (globalThis as { chrome?: typeof chrome }).chrome;
            const runtime = chromeApi?.runtime;
            if (runtime?.sendMessage) {
              await new Promise<void>((resolve) => {
                runtime.sendMessage!({ type: "record-clipboard", text }, () => resolve());
              });
            }
          }
        } catch (clipErr) {
          // silent ignore if permission not yet granted
        }
      }

      const history = await invoke<ClipboardEntry[]>("get_clipboard_history");
      setClipboardHistory(history);
    } catch (e) {
      console.error(e);
    }
  };

  const handlePasteFromClipboard = (text: string) => {
    // Inject text at editor cursor
    if (editorRef.current) {
      const textarea = editorRef.current;
      const start = textarea.selectionStart;
      const end = textarea.selectionEnd;
      const val = textarea.value;
      const updatedVal = val.substring(0, start) + text + val.substring(end);
      setEditorContent(updatedVal);
      saveActiveNote(updatedVal);
    } else {
      setEditorContent((prev) => prev + "\n" + text);
    }
    setShowClipboard(false);
  };

  useLinuxMenuBridge({
    createNewDraft,
    createNewSavedNote,
    saveNoteNow,
    deleteCurrentNote,
    loadNotes,
    loadClipboardHistory,
    setEditorView,
    setShowPalette,
    setPaletteQuery,
    setSelectedPaletteIdx,
    setShowClipboard,
    setSelectedClipIdx,
    setShowLineNumbers,
    setTheme,
    setShowHelp,
    setAboutAutoCheck,
    setShowAbout,
  });

  // --- Command Palette Action Executions ---
  const executeCommand = async (commandName: string) => {
    setShowPalette(false);

    switch (commandName) {
      case "Switch to Light Theme":
        setTheme("light");
        break;
      case "Switch to Dark Theme":
        setTheme("dark");
        break;
      case "Switch to System Theme":
        setTheme("system");
        break;
      case "Format JSON":
        try {
          const parsed = JSON.parse(editorContent);
          const formatted = JSON.stringify(parsed, null, 2);
          setEditorContent(formatted);
          saveActiveNote(formatted);
        } catch (err) {
          alert("Error: Active note content does not contain valid JSON.");
        }
        break;

      case "Prettify Markdown": {
        // Basic markdown formatting cleanups
        let prettified = editorContent
          .split("\n")
          .map((line) => {
            if (line.startsWith("-") && !line.startsWith("- ")) {
              return "- " + line.substring(1);
            }
            return line;
          })
          .join("\n");
        setEditorContent(prettified);
        saveActiveNote(prettified);
        break;
      }

      case "Copy as Raw HTML": {
        const html = renderMarkdownToHtml(editorContent);
        try {
          await invoke("copy_text_to_clipboard", { text: html });
          alert("Markdown compiled to HTML and copied to clipboard.");
        } catch {
          alert("Could not copy HTML to clipboard.");
        }
        break;
      }

      case "Create Daily Note":
      case "Append to Daily Note": {
        const today = new Date().toISOString().split("T")[0];
        const title = `Daily_Note_${today}`;
        const newContent = commandName === "Create Daily Note" 
          ? `## Log: ${today}\n\n*   Active drafting logs...` 
          : `\n\n*   [Logged at ${new Date().toLocaleTimeString()}]: ${editorContent}`;

        try {
          await invoke("write_note", {
            title,
            content: newContent,
            isEphemeral: false,
            previousFilename: null,
          });
          loadNotes();
          alert(`Appended to ${title}!`);
        } catch (e) {
          console.error(e);
        }
        break;
      }

      case "Export selection to GitHub Gist":
        alert("Mock action: selection uploaded securely to your GitHub private Gists!");
        break;

      case "Lock / Unlock Secret Vault":
        setActiveTab("secrets");
        break;
    }
  };

  // --- Secure Vault-X Database Operations ---
  const handleInitVault = async () => {
    if (!vaultPassword) return;
    try {
      await invoke("initialize_vault", { password: vaultPassword });
      setIsVaultInitialized(true);
      setIsVaultUnlocked(true);
      setVaultPassword("");
      clearRevealedSecret();
      await loadSecretMetadata();
    } catch (e: unknown) {
      alert("Failed to initialize: " + invokeErrorMessage(e));
    }
  };

  const handleUnlockVault = async () => {
    if (!vaultPassword) return;
    try {
      await invoke("unlock_vault", { password: vaultPassword });
      setIsVaultUnlocked(true);
      setVaultPassword("");
      clearRevealedSecret();
      await loadSecretMetadata();
    } catch (e: unknown) {
      alert(invokeErrorMessage(e));
    }
  };

  const handleLockVault = async () => {
    await invoke("lock_vault");
    setIsVaultUnlocked(false);
    setVaultPassword("");
    setSecretMetadata([]);
    clearRevealedSecret();
  };

  const handleAddSecret = async () => {
    if (!newSecretName || !newSecretValue) return;
    try {
      await invoke("add_secret", {
        name: newSecretName,
        path: newSecretPath,
        value: newSecretValue,
      });
      setNewSecretName("");
      setNewSecretPath("");
      setNewSecretValue("");
      setShowAddSecret(false);
      clearRevealedSecret();
      await loadSecretMetadata();
    } catch (e: unknown) {
      alert("Failed to add secret: " + invokeErrorMessage(e));
    }
  };

  const handleDeleteSecret = async (index: number) => {
    try {
      await invoke("delete_secret", { index });
      clearRevealedSecret();
      await loadSecretMetadata();
    } catch (e: unknown) {
      alert("Failed to delete secret: " + invokeErrorMessage(e));
    }
  };

  const handleCopySecret = async (index: number, name: string) => {
    try {
      await invoke("copy_secret_to_clipboard", { index });
      alert(`${name} copied to clipboard!`);
    } catch (e: unknown) {
      alert("Copy failed: " + invokeErrorMessage(e));
    }
  };

  const handleToggleReveal = async (index: number) => {
    if (revealedIndex === index) {
      clearRevealedSecret();
      return;
    }
    try {
      const value = await invoke<string>("reveal_secret_value", { index });
      setRevealedIndex(index);
      setRevealedValue(value);
    } catch (e: unknown) {
      alert("Could not reveal secret: " + invokeErrorMessage(e));
    }
  };

  // --- Render Notes Filter list ---
  const filteredNotes = notes.filter((note) => {
    // Search filter
    const matchesSearch = note.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
      note.content.toLowerCase().includes(searchQuery.toLowerCase());
    
    // Tab filter
    if (activeTab === "scratch") return matchesSearch && note.is_ephemeral;
    if (activeTab === "vault") return matchesSearch && !note.is_ephemeral;
    return matchesSearch;
  });

  return (
    <div className="app-container">
      {/* 1. Left Sidebar */}
      <aside className="sidebar">
        <div className="sidebar-header">
          <h1 className="logo-text">
            <span style={{ fontSize: "18px" }}>✦</span> Gnomad Slate
          </h1>
          <span style={{ fontSize: "10px", color: "var(--text-muted)", fontStyle: "italic" }}>
            {appVersionLabel}
          </span>
        </div>

        <div className="search-box">
          <input
            type="text"
            className="search-input"
            placeholder="Search notes... (⌘P)"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
          />
        </div>

        <div className="tabs-row">
          <TipButton
            className={`tab-btn ${activeTab === "scratch" ? "active" : ""}`}
            tip="Quick notes that auto-archive after 48 hours"
            onClick={() => setActiveTab("scratch")}
          >
            Scratchpad
          </TipButton>
          <TipButton
            className={`tab-btn ${activeTab === "vault" ? "active" : ""}`}
            tip="Saved notes that stay until you delete them"
            onClick={() => setActiveTab("vault")}
          >
            Saved
          </TipButton>
          <TipButton
            className={`tab-btn ${activeTab === "secrets" ? "active" : ""}`}
            tip="Password-protected store for API keys and tokens"
            onClick={() => setActiveTab("secrets")}
          >
            Secrets
          </TipButton>
        </div>

        <div className="notes-list">
          {activeTab !== "secrets" && filteredNotes.map((note) => (
            <div
              key={note.filename}
              className={`note-item ${selectedNote?.filename === note.filename ? "selected" : ""}`}
              onClick={() => selectNote(note)}
            >
              <div className="note-item-header">
                <span className="note-title">{note.title}</span>
                <span className={`note-badge ${note.is_ephemeral ? "ephemeral" : "vault"}`}>
                  {note.is_ephemeral ? "Scratch" : "Saved"}
                </span>
              </div>
              <p className="note-preview">
                {note.content.replace("<!-- ephemeral -->\n", "").substring(0, 40) || "Empty note"}
              </p>
              <button
                className="delete-btn has-tooltip"
                data-tooltip="Delete this note permanently"
                onClick={(e) => deleteNote(note.filename, e)}
              >
                ✕
              </button>
            </div>
          ))}
          {activeTab !== "secrets" && filteredNotes.length === 0 && (
            <p style={{ padding: "16px", color: "var(--text-muted)", fontSize: "12px", textAlign: "center" }}>
              No notes found. Click New Draft to start writing.
            </p>
          )}
        </div>

        {/* Dynamic Sidebar footer command shortcut tips */}
        <div style={{ padding: "12px 16px", borderTop: "1px solid var(--border-color)", fontSize: "11px", color: "var(--text-muted)" }}>
          <div style={{ display: "flex", justifyContent: "space-between", marginBottom: "4px" }}>
            <span>{modShortcut("P")}</span> <span>Palette</span>
          </div>
          <div style={{ display: "flex", justifyContent: "space-between" }}>
            <span>{modShortcut("V", { shift: true })}</span> <span>Clipboard</span>
          </div>
          <div style={{ display: "flex", justifyContent: "space-between", marginBottom: "8px", fontSize: "10px" }}>
            <span>Alt+Space</span> <span>Summon</span>
          </div>

          <label className="settings-toggle-row has-tooltip" data-tooltip="Hide likely passwords and API keys from clipboard history">
            <input
              type="checkbox"
              checked={clipboardFilterEnabled}
              onChange={(e) => setClipboardFilterEnabled(e.target.checked)}
            />
            <span>Filter sensitive clipboard items</span>
          </label>

          <div className="theme-row">
            <TipButton
              className={`theme-toggle-btn ${theme === "light" ? "active" : ""}`}
              tip="Use a light background and dark text"
              onClick={() => setTheme("light")}
            >
              ☀️ Light
            </TipButton>
            <TipButton
              className={`theme-toggle-btn ${theme === "dark" ? "active" : ""}`}
              tip="Use a dark background — default for Slate"
              onClick={() => setTheme("dark")}
            >
              🌙 Dark
            </TipButton>
            <TipButton
              className={`theme-toggle-btn ${theme === "system" ? "active" : ""}`}
              tip="Match your Mac's light or dark mode"
              onClick={() => setTheme("system")}
            >
              🖥️ System
            </TipButton>
          </div>
        </div>
      </aside>

      {/* 2. Main Workstation Area */}
      <section style={{ height: "100%", overflow: "hidden" }}>
        {activeTab === "secrets" ? (
          <div className="vault-panel">
            {!isVaultUnlocked ? (
              <div className="lock-screen">
                <div className="lock-icon">🔒</div>
                <h2 style={{ marginBottom: "8px", fontSize: "16px", fontWeight: "600" }}>Vault-X Encrypted Secrets</h2>
                <p style={{ color: "var(--text-muted)", fontSize: "12px", marginBottom: "20px", textAlign: "center", maxWidth: "250px" }}>
                  {isVaultInitialized 
                    ? "Enter your master key credentials to unlock safe data."
                    : "Initialize a new secure AES-256 vault by setting a master key password."
                  }
                </p>
                <div className="vault-input-group">
                  <input
                    type="password"
                    className="vault-password-input"
                    placeholder="Enter Master Password"
                    value={vaultPassword}
                    onChange={(e) => setVaultPassword(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === "Enter") {
                        isVaultInitialized ? handleUnlockVault() : handleInitVault();
                      }
                    }}
                  />
                  <button
                    className="action-btn accent has-tooltip"
                    style={{ padding: "10px", justifyContent: "center" }}
                    data-tooltip={
                      isVaultInitialized
                        ? "Unlock your encrypted secrets with your master password"
                        : "Create a new encrypted vault with a master password"
                    }
                    onClick={() =>
                      isVaultInitialized ? handleUnlockVault() : handleInitVault()
                    }
                  >
                    {isVaultInitialized ? "Unlock Vault" : "Initialize Vault"}
                  </button>
                </div>
              </div>
            ) : (
              <div style={{ display: "flex", flexDirection: "column", height: "100%" }}>
                <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "16px" }}>
                  <div>
                    <h2 style={{ fontSize: "18px", fontWeight: "600" }}>Vault-X Credentials Registry</h2>
                    <p style={{ color: "var(--text-muted)", fontSize: "12px" }}>Highly secure offline keys, tokens, and SSH configurations</p>
                  </div>
                  <div style={{ display: "flex", gap: "8px" }}>
                    <TipButton
                      className="action-btn"
                      tip="Add an API key, token, or password to the encrypted vault"
                      onClick={() => setShowAddSecret(true)}
                    >
                      + Add Secret Key
                    </TipButton>
                    <TipButton
                      className="action-btn"
                      tip="Lock the vault and clear secrets from memory"
                      onClick={() => void handleLockVault()}
                    >
                      Lock Vault
                    </TipButton>
                  </div>
                </div>

                {showAddSecret && (
                  <div style={{ background: "rgba(255,255,255,0.03)", padding: "16px", borderRadius: "8px", border: "1px solid var(--border-color)", marginBottom: "16px" }}>
                    <h3 style={{ fontSize: "13px", fontWeight: "600", marginBottom: "12px" }}>Register Secure Credential</h3>
                    <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr 1fr", gap: "12px", marginBottom: "12px" }}>
                      <input
                        type="text"
                        placeholder="Key Name (e.g. OPENAI_KEY)"
                        className="search-input"
                        value={newSecretName}
                        onChange={(e) => setNewSecretName(e.target.value)}
                      />
                      <input
                        type="text"
                        placeholder="Project Mapping Path (Optional)"
                        className="search-input"
                        value={newSecretPath}
                        onChange={(e) => setNewSecretPath(e.target.value)}
                      />
                      <input
                        type="password"
                        placeholder="Secret Value / API Token"
                        className="search-input"
                        value={newSecretValue}
                        onChange={(e) => setNewSecretValue(e.target.value)}
                      />
                    </div>
                    <div style={{ display: "flex", gap: "8px", justifyContent: "flex-end" }}>
                      <TipButton className="action-btn" tip="Close the add-secret form without saving" onClick={() => setShowAddSecret(false)}>Cancel</TipButton>
                      <TipButton className="action-btn accent" tip="Encrypt and store this credential in the vault" onClick={() => void handleAddSecret()}>Save Securely</TipButton>
                    </div>
                  </div>
                )}

                <div className="vault-table-container">
                  <table className="vault-table">
                    <thead>
                      <tr>
                        <th>Secret Name</th>
                        <th>Project Mapping Path</th>
                        <th>Token / Key Value</th>
                        <th style={{ width: "120px" }}>Actions</th>
                      </tr>
                    </thead>
                    <tbody>
                      {secretMetadata.map((secret) => (
                        <tr key={`${secret.name}-${secret.index}`}>
                          <td style={{ fontWeight: "600" }}>{secret.name}</td>
                          <td style={{ fontStyle: "italic", color: "var(--text-muted)", fontSize: "11px" }}>
                            {secret.path || "Global / Unmapped"}
                          </td>
                          <td
                            className="key-mask has-tooltip"
                            data-tooltip="Click to copy this secret to your clipboard"
                            onClick={() => void handleCopySecret(secret.index, secret.name)}
                          >
                            {revealedIndex === secret.index
                              ? revealedValue
                              : "•••••••••••••••• (Click to Copy)"}
                          </td>
                          <td>
                            <div style={{ display: "flex", gap: "8px" }}>
                              <TipButton
                                className="tab-btn"
                                style={{ padding: "2px 6px" }}
                                tip={revealedIndex === secret.index ? "Hide the secret value on screen" : "Temporarily show the secret value on screen"}
                                onClick={() => void handleToggleReveal(secret.index)}
                              >
                                {revealedIndex === secret.index ? "Hide" : "Show"}
                              </TipButton>
                              <TipButton
                                className="tab-btn"
                                style={{ padding: "2px 6px", color: "#ef4444" }}
                                tip="Remove this secret from the vault permanently"
                                onClick={() => void handleDeleteSecret(secret.index)}
                              >
                                Delete
                              </TipButton>
                            </div>
                          </td>
                        </tr>
                      ))}
                      {secretMetadata.length === 0 && (
                        <tr>
                          <td colSpan={4} style={{ textAlign: "center", color: "var(--text-muted)", padding: "24px" }}>
                            No secrets registered yet. Click "+ Add Secret Key" to secure your first credential!
                          </td>
                        </tr>
                      )}
                    </tbody>
                  </table>
                </div>
              </div>
            )}
          </div>
        ) : (
          <div className="editor-container">
            <div className={`editor-header${isLinux ? " editor-header-linux" : ""}`}>
              {isLinux ? (
                <>
                  <div className="toolbar-actions toolbar-actions-linux">
                    {saveFeedback && (
                      <span className="save-feedback" aria-live="polite">
                        {saveFeedback}
                      </span>
                    )}
                    <div className="view-mode-row">
                      <TipButton
                        className={`tab-btn ${editorView === "edit" ? "active" : ""}`}
                        tipBelow
                        tip="Edit raw markdown in the text area"
                        onClick={() => setEditorView("edit")}
                      >
                        Edit
                      </TipButton>
                      <TipButton
                        className={`tab-btn ${editorView === "split" ? "active" : ""}`}
                        tipBelow
                        tip="See your markdown and preview side by side"
                        onClick={() => setEditorView("split")}
                      >
                        Split
                      </TipButton>
                      <TipButton
                        className={`tab-btn ${editorView === "preview" ? "active" : ""}`}
                        tipBelow
                        tip="View rendered markdown only"
                        onClick={() => setEditorView("preview")}
                      >
                        Preview
                      </TipButton>
                    </div>
                    <TipButton
                      className={`action-btn ${showLineNumbers ? "active-glow" : ""}`}
                      tipBelow
                      tip="Show or hide line numbers in the editor"
                      onClick={() => setShowLineNumbers((prev) => !prev)}
                    >
                      # Lines
                    </TipButton>
                    <TipButton
                      className={`action-btn ${isEphemeral ? "active-glow" : ""}`}
                      tipBelow
                      tip={
                        isEphemeral
                          ? `Scratchpad draft — auto-archives after 48 hours. Click or press ${modShortcut("S")} to save permanently.`
                          : "Saved permanently. Click to mark as a scratchpad draft again."
                      }
                      onClick={() => {
                        const nextVal = !isEphemeral;
                        setIsEphemeral(nextVal);
                        saveActiveNote(editorContent, editorTitle, !nextVal);
                      }}
                    >
                      {isEphemeral ? "✧ Scratchpad Draft" : "🔒 Saved Note"}
                    </TipButton>
                    <TipButton
                      className="action-btn"
                      tipBelow
                      tip={`Start a fresh scratchpad note (${modShortcut("N")})`}
                      onClick={() => void createNewDraft()}
                    >
                      New Draft
                    </TipButton>
                  </div>
                  <div className="editor-meta-row">
                    <div className="editor-title-container">
                      <span style={{ fontSize: "14px", color: isEphemeral ? "var(--accent-cyan)" : "var(--accent-color)" }}>●</span>
                      <input
                        type="text"
                        className="editor-title-input"
                        placeholder="Untitled note"
                        value={editorTitle}
                        onKeyDown={handleEditorShortcut}
                        onChange={(e) => {
                          setEditorTitle(e.target.value);
                          scheduleSaveNote(editorContent, e.target.value);
                        }}
                      />
                    </div>
                    <span className="editor-filename" title="Note file on disk">
                      {selectedNote?.filename ?? "unsaved.md"}
                    </span>
                  </div>
                </>
              ) : (
                <>
                  <div className="editor-title-container">
                    <span style={{ fontSize: "14px", color: isEphemeral ? "var(--accent-cyan)" : "var(--accent-color)" }}>●</span>
                    <input
                      type="text"
                      className="editor-title-input"
                      placeholder="Untitled note"
                      value={editorTitle}
                      onKeyDown={handleEditorShortcut}
                      onChange={(e) => {
                        setEditorTitle(e.target.value);
                        scheduleSaveNote(editorContent, e.target.value);
                      }}
                    />
                  </div>

                  <div className="toolbar-actions">
                    {saveFeedback && (
                      <span className="save-feedback" aria-live="polite">
                        {saveFeedback}
                      </span>
                    )}
                    <div className="view-mode-row">
                      <TipButton
                        className={`tab-btn ${editorView === "edit" ? "active" : ""}`}
                        tip="Edit raw markdown in the text area"
                        onClick={() => setEditorView("edit")}
                      >
                        Edit
                      </TipButton>
                      <TipButton
                        className={`tab-btn ${editorView === "split" ? "active" : ""}`}
                        tip="See your markdown and preview side by side"
                        onClick={() => setEditorView("split")}
                      >
                        Split
                      </TipButton>
                      <TipButton
                        className={`tab-btn ${editorView === "preview" ? "active" : ""}`}
                        tip="View rendered markdown only"
                        onClick={() => setEditorView("preview")}
                      >
                        Preview
                      </TipButton>
                    </div>
                    <TipButton
                      className={`action-btn ${isEphemeral ? "active-glow" : ""}`}
                      tipBelow
                      tip={
                        isEphemeral
                          ? "Scratchpad draft — auto-archives after 48 hours. Click or press ⌘S to save permanently."
                          : "Saved permanently. Click to mark as a scratchpad draft again."
                      }
                      onClick={() => {
                        const nextVal = !isEphemeral;
                        setIsEphemeral(nextVal);
                        saveActiveNote(editorContent, editorTitle, !nextVal);
                      }}
                    >
                      {isEphemeral ? "✧ Scratchpad Draft" : "🔒 Saved Note"}
                    </TipButton>
                    <TipButton
                      className="action-btn"
                      tipBelow
                      tip="Start a fresh scratchpad note (⌘N)"
                      onClick={() => void createNewDraft()}
                    >
                      New Draft
                    </TipButton>
                  </div>
                </>
              )}
            </div>

            <div
              className={`editor-body ${editorView === "split" ? "editor-body-split" : ""}`}
            >
              {(editorView === "edit" || editorView === "split") && (
                <div className={`editor-input-wrap${isLinux && showLineNumbers ? " editor-input-wrap-lined" : ""}`}>
                  {isLinux && showLineNumbers && (
                    <div className="line-numbers-gutter" aria-hidden="true">
                      {lineNumberRows.map((line) => (
                        <div key={line} className="line-number">
                          {line}
                        </div>
                      ))}
                    </div>
                  )}
                  <textarea
                    ref={editorRef}
                    className="text-textarea"
                    placeholder={
                      isLinux
                        ? `Start typing in Markdown… ${modShortcut("S")} save · ${modShortcut("N")} new draft · ${modShortcut("P")} palette`
                        : "Start typing in Markdown… ⌘S save · ⌘N new draft · ⌘P palette"
                    }
                    value={editorContent}
                    onKeyDown={handleEditorShortcut}
                    onChange={(e) => {
                      setEditorContent(e.target.value);
                      scheduleSaveNote(e.target.value, editorTitle);
                    }}
                    onScroll={
                      isLinux && showLineNumbers
                        ? (e) => {
                            const gutter = e.currentTarget.previousElementSibling;
                            if (gutter instanceof HTMLElement) {
                              gutter.scrollTop = e.currentTarget.scrollTop;
                            }
                          }
                        : undefined
                    }
                  />
                </div>
              )}
              {(editorView === "preview" || editorView === "split") && (
                <div
                  className="markdown-preview"
                  dangerouslySetInnerHTML={{ __html: previewHtml }}
                />
              )}
            </div>
          </div>
        )}
      </section>

      {/* 3. Extended Clipboard history Capsule Dialog */}
      {showClipboard && (
        <div className="overlay-backdrop" onClick={() => setShowClipboard(false)}>
          <div className="modal-box" onClick={(e) => e.stopPropagation()}>
            <div className="modal-header">
              <h3 className="modal-title">📋 Clipboard History Capsule</h3>
              <span className="palette-shortcut">Esc to close</span>
            </div>
            <div className="modal-body">
              {clipboardHistory.map((item, idx) => (
                <div
                  key={idx}
                  className={`clipboard-item ${idx === selectedClipIdx ? "selected" : ""}`}
                  onClick={() => handlePasteFromClipboard(item.text)}
                >
                  {item.text}
                </div>
              ))}
              {clipboardHistory.length === 0 && (
                <p style={{ color: "var(--text-muted)", textAlign: "center", padding: "20px" }}>
                  Clipboard history is empty. Copy text anywhere in your desktop!
                </p>
              )}
            </div>
          </div>
        </div>
      )}

      {/* 4. Visual Command Palette Dialog */}
      {showPalette && (
        <div className="overlay-backdrop" onClick={() => setShowPalette(false)}>
          <div className="modal-box" onClick={(e) => e.stopPropagation()} style={{ width: "500px" }}>
            <input
              type="text"
              autoFocus
              className="palette-search"
              placeholder="Type a notepad action command..."
              value={paletteQuery}
              onChange={(e) => setPaletteQuery(e.target.value)}
            />
            <div style={{ padding: "8px 0", maxHeight: "280px", overflowY: "auto" }}>
              {paletteCommands
                .filter(cmd => cmd.name.toLowerCase().includes(paletteQuery.toLowerCase()) || 
                              cmd.desc.toLowerCase().includes(paletteQuery.toLowerCase()))
                .map((cmd, idx) => (
                  <div
                    key={idx}
                    className={`palette-item ${idx === selectedPaletteIdx ? "selected" : ""}`}
                    onClick={() => executeCommand(cmd.name)}
                  >
                    <div>
                      <div style={{ fontSize: "13px", fontWeight: "500" }}>{cmd.name}</div>
                      <div style={{ fontSize: "11px", color: "var(--text-muted)" }}>{cmd.desc}</div>
                    </div>
                    <span className="palette-shortcut">{cmd.shortcut}</span>
                  </div>
                ))}
            </div>
          </div>
        </div>
      )}

      <AboutDialog
        open={showAbout}
        autoCheckUpdates={aboutAutoCheck}
        onClose={() => {
          setShowAbout(false);
          setAboutAutoCheck(false);
        }}
      />
      {isLinux && (
        <HelpDialog open={showHelp} onClose={() => setShowHelp(false)} />
      )}
      {closePromptOpen && (
        <UnsavedChangesModal
          noteTitle={editorTitle}
          onSave={() => {
            void (async () => {
              await flushPendingSave();
              setClosePromptOpen(false);
              await closeApp();
            })();
          }}
          onDiscard={() => {
            void (async () => {
              setClosePromptOpen(false);
              await closeApp();
            })();
          }}
          onCancel={() => {
            pendingWindowClose.current = false;
            setClosePromptOpen(false);
          }}
        />
      )}
    </div>
  );
}

export default App;

```

### src/App.css
```
@import url('https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700&family=JetBrains+Mono:wght@400;500&display=swap');

:root {
  --bg-app: rgba(13, 13, 16, 0.88);
  --bg-sidebar: rgba(20, 20, 25, 0.45);
  --border-color: rgba(255, 255, 255, 0.08);
  --border-focus: rgba(138, 92, 246, 0.4);
  --text-main: #f3f4f6;
  --text-muted: #9ca3af;
  --accent-color: #8b5cf6;
  --accent-glow: rgba(139, 92, 246, 0.25);
  --accent-cyan: #06b6d4;
  --accent-cyan-glow: rgba(6, 182, 212, 0.25);
  --bg-card: rgba(30, 30, 40, 0.5);
  --bg-hover: rgba(255, 255, 255, 0.05);
  --font-sans: 'Inter', -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  --font-mono: 'JetBrains Mono', monospace;
}

:root[data-theme="light"] {
  --bg-app: rgba(243, 244, 246, 0.93);
  --bg-sidebar: rgba(255, 255, 255, 0.65);
  --border-color: rgba(0, 0, 0, 0.08);
  --border-focus: rgba(124, 58, 237, 0.45);
  --text-main: #111827;
  --text-muted: #4b5563;
  --accent-color: #7c3aed;
  --accent-glow: rgba(124, 58, 237, 0.18);
  --accent-cyan: #0891b2;
  --accent-cyan-glow: rgba(8, 145, 178, 0.18);
  --bg-card: rgba(255, 255, 255, 0.85);
  --bg-hover: rgba(0, 0, 0, 0.04);
}

* {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

body {
  background: transparent;
  color: var(--text-main);
  font-family: var(--font-sans);
  font-size: 14px;
  overflow: hidden;
  height: 100vh;
  width: 100vw;
  user-select: none;
}

/* App Layout Grid */
.app-container {
  display: grid;
  grid-template-columns: 260px 1fr;
  height: 100vh;
  width: 100vw;
  background: var(--bg-app);
  backdrop-filter: blur(24px) saturate(190%);
  -webkit-backdrop-filter: blur(24px) saturate(190%);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  overflow: hidden;
  position: relative;
}

/* Sidebar Styling */
.sidebar {
  background: var(--bg-sidebar);
  border-right: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.sidebar-header {
  padding: 16px;
  border-bottom: 1px solid var(--border-color);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.logo-text {
  font-size: 16px;
  font-weight: 700;
  letter-spacing: 0.5px;
  background: linear-gradient(135deg, var(--accent-color), var(--accent-cyan));
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  display: flex;
  align-items: center;
  gap: 6px;
}

.search-box {
  padding: 12px 16px;
}

.search-input {
  width: 100%;
  background: rgba(0, 0, 0, 0.2);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  padding: 8px 12px;
  color: var(--text-main);
  font-family: var(--font-sans);
  font-size: 12px;
  outline: none;
  transition: all 0.2s ease;
  user-select: text;
  -webkit-user-select: text;
}

.search-input:focus {
  border-color: var(--border-focus);
  box-shadow: 0 0 0 2px var(--accent-glow);
}

.tabs-row {
  display: flex;
  padding: 0 16px 8px 16px;
  gap: 8px;
  border-bottom: 1px solid var(--border-color);
}

.tab-btn {
  background: transparent;
  border: none;
  color: var(--text-muted);
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 4px;
  transition: all 0.2s ease;
}

.tab-btn:hover {
  color: var(--text-main);
  background: var(--bg-hover);
}

.tab-btn.active {
  color: var(--text-main);
  background: rgba(139, 92, 246, 0.15);
  border: 1px solid rgba(139, 92, 246, 0.3);
}

.notes-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

/* Custom Scrollbar */
::-webkit-scrollbar {
  width: 6px;
}
::-webkit-scrollbar-track {
  background: transparent;
}
::-webkit-scrollbar-thumb {
  background: var(--border-color);
  border-radius: 3px;
}
::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.2);
}

.note-item {
  display: flex;
  flex-direction: column;
  padding: 10px 12px;
  border-radius: 6px;
  cursor: pointer;
  margin-bottom: 4px;
  position: relative;
  transition: all 0.2s ease;
  border: 1px solid transparent;
}

.note-item:hover {
  background: var(--bg-hover);
}

.note-item.selected {
  background: rgba(255, 255, 255, 0.06);
  border-color: rgba(255, 255, 255, 0.08);
}

.note-item-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 4px;
}

.note-title {
  font-size: 13px;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 170px;
}

.note-badge {
  font-size: 9px;
  padding: 1px 4px;
  border-radius: 3px;
  font-weight: 600;
}

.note-badge.ephemeral {
  background: rgba(6, 182, 212, 0.15);
  color: var(--accent-cyan);
  border: 1px solid rgba(6, 182, 212, 0.25);
}

.note-badge.vault {
  background: rgba(138, 92, 246, 0.15);
  color: var(--accent-color);
  border: 1px solid rgba(138, 92, 246, 0.25);
}

.note-preview {
  font-size: 11px;
  color: var(--text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.delete-btn {
  position: absolute;
  right: 12px;
  bottom: 10px;
  background: transparent;
  border: none;
  color: transparent;
  cursor: pointer;
  font-size: 12px;
  transition: all 0.2s ease;
}

.note-item:hover .delete-btn {
  color: #ef4444;
}

/* Editor Area Styling */
.editor-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
  background: rgba(0, 0, 0, 0.1);
}

.editor-header {
  padding: 12px 20px;
  border-bottom: 1px solid var(--border-color);
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
  background: rgba(10, 10, 15, 0.2);
  overflow: visible;
  position: relative;
  z-index: 20;
}

.editor-title-container {
  display: flex;
  align-items: center;
  gap: 12px;
  flex: 1 1 auto;
  min-width: 0;
}

.editor-title-input {
  background: transparent;
  border: none;
  font-size: 18px;
  font-weight: 600;
  color: var(--text-main);
  outline: none;
  width: 100%;
  max-width: 280px;
  min-width: 120px;
  font-family: var(--font-sans);
  user-select: text;
  -webkit-user-select: text;
}

.toolbar-actions {
  display: flex;
  gap: 8px;
  align-items: center;
  overflow: visible;
  flex: 0 0 auto;
}

.save-feedback {
  font-size: 11px;
  font-weight: 600;
  color: var(--accent-cyan);
  white-space: nowrap;
  padding-right: 4px;
}

.action-btn {
  background: var(--bg-hover);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  padding: 6px 12px;
  color: var(--text-main);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 6px;
  transition: all 0.2s ease;
  white-space: nowrap;
}

.action-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  border-color: rgba(255, 255, 255, 0.2);
}

.action-btn.active-glow {
  background: rgba(6, 182, 212, 0.1);
  border-color: var(--accent-cyan);
  box-shadow: 0 0 8px var(--accent-cyan-glow);
}

.action-btn.accent {
  background: var(--accent-color);
  border-color: var(--accent-color);
}

.action-btn.accent:hover {
  box-shadow: 0 0 12px var(--accent-glow);
}

.editor-body {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.editor-body-split {
  flex-direction: row;
}

.editor-body-split .text-textarea,
.editor-body-split .markdown-preview {
  flex: 1;
  width: 50%;
  min-width: 0;
}

.editor-body-split .editor-input-wrap {
  flex: 1;
  width: 50%;
  min-width: 0;
}

.editor-body-split .text-textarea {
  border-right: 1px solid var(--border-color);
}

.markdown-preview {
  flex: 1;
  overflow-y: auto;
  padding: 24px;
  color: var(--text-main);
  font-size: 14px;
  line-height: 1.65;
  user-select: text;
}

.markdown-preview h1,
.markdown-preview h2,
.markdown-preview h3 {
  margin: 1em 0 0.5em;
  font-weight: 600;
}

.markdown-preview h1 { font-size: 1.5rem; }
.markdown-preview h2 { font-size: 1.25rem; }
.markdown-preview p { margin-bottom: 0.75em; }

.markdown-preview code {
  font-family: var(--font-mono);
  background: var(--bg-hover);
  padding: 0.1em 0.35em;
  border-radius: 4px;
  font-size: 0.9em;
}

.markdown-preview pre {
  background: var(--bg-card);
  padding: 12px;
  border-radius: 8px;
  overflow-x: auto;
  margin-bottom: 1em;
}

.markdown-preview pre code {
  background: transparent;
  padding: 0;
}

.markdown-preview ul,
.markdown-preview ol {
  margin: 0 0 0.75em 1.25em;
}

.view-mode-row {
  display: flex;
  gap: 4px;
  margin-right: 8px;
}

.settings-toggle-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 10px 0 8px;
  font-size: 11px;
  color: var(--text-muted);
  cursor: pointer;
}

.settings-toggle-row input {
  accent-color: var(--accent-color);
}

.text-textarea {
  flex: 1;
  background: transparent;
  border: none;
  resize: none;
  outline: none;
  padding: 24px;
  color: var(--text-main);
  font-family: var(--font-sans);
  font-size: 14px;
  line-height: 1.6;
  user-select: text;
  -webkit-user-select: text;
}

/* Modals and Overlays Styling (Sleek Glassmorphic Dialogs) */
.overlay-backdrop {
  position: absolute;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background: rgba(10, 10, 15, 0.7);
  backdrop-filter: blur(12px);
  display: flex;
  justify-content: center;
  align-items: flex-start;
  padding-top: 10vh;
  z-index: 100;
  animation: fadeIn 0.15s ease-out;
}

.modal-box {
  background: rgba(20, 20, 28, 0.85);
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 12px;
  width: 550px;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.5), 0 0 2px var(--accent-glow);
  overflow: hidden;
  animation: slideDown 0.2s cubic-bezier(0.16, 1, 0.3, 1);
}

.modal-header {
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-color);
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: rgba(0, 0, 0, 0.2);
}

.modal-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-main);
}

.modal-body {
  padding: 20px;
  max-height: 380px;
  overflow-y: auto;
}

/* Clipboard History Overlay */
.clipboard-item {
  padding: 10px 14px;
  border-radius: 6px;
  cursor: pointer;
  font-family: var(--font-mono);
  font-size: 12px;
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid var(--border-color);
  margin-bottom: 8px;
  white-space: pre-wrap;
  word-break: break-all;
  max-height: 80px;
  overflow: hidden;
  text-overflow: ellipsis;
  transition: all 0.15s ease;
}

.clipboard-item:hover, .clipboard-item.selected {
  background: rgba(138, 92, 246, 0.1);
  border-color: var(--accent-color);
  box-shadow: 0 0 8px rgba(138, 92, 246, 0.15);
}

/* Command Palette Overlay */
.palette-search {
  width: 100%;
  background: rgba(0, 0, 0, 0.3);
  border: none;
  border-bottom: 1px solid var(--border-color);
  padding: 14px 20px;
  color: var(--text-main);
  font-family: var(--font-sans);
  font-size: 14px;
  outline: none;
}

.palette-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 20px;
  cursor: pointer;
  transition: all 0.1s ease;
}

.palette-item:hover, .palette-item.selected {
  background: var(--bg-hover);
  color: var(--accent-cyan);
}

.palette-shortcut {
  font-size: 10px;
  color: var(--text-muted);
  background: rgba(255, 255, 255, 0.08);
  padding: 2px 6px;
  border-radius: 4px;
}

/* Vault / Security Lock Styling */
.lock-screen {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  width: 100%;
  padding: 40px;
  background: rgba(10, 10, 15, 0.4);
}

.lock-icon {
  font-size: 48px;
  margin-bottom: 20px;
  background: linear-gradient(135deg, var(--accent-color), var(--accent-cyan));
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.vault-input-group {
  display: flex;
  flex-direction: column;
  gap: 12px;
  width: 280px;
}

.vault-password-input {
  background: rgba(0, 0, 0, 0.3);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  padding: 10px 14px;
  color: var(--text-main);
  text-align: center;
  font-size: 14px;
  outline: none;
  transition: all 0.2s ease;
  user-select: text;
  -webkit-user-select: text;
}

.vault-password-input:focus {
  border-color: var(--accent-color);
  box-shadow: 0 0 10px var(--accent-glow);
}

/* Vault Credentials Registry styling */
.vault-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 24px;
  overflow: hidden;
}

.vault-table-container {
  flex: 1;
  overflow-y: auto;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background: rgba(0, 0, 0, 0.2);
}

.vault-table {
  width: 100%;
  border-collapse: collapse;
  text-align: left;
}

.vault-table th, .vault-table td {
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-color);
}

.vault-table th {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  color: var(--text-muted);
  background: rgba(0, 0, 0, 0.3);
}

.vault-table td {
  font-size: 13px;
}

.key-mask {
  font-family: var(--font-mono);
  color: var(--text-muted);
  cursor: pointer;
  transition: color 0.15s ease;
}

.key-mask:hover {
  color: var(--text-main);
}

/* Theme Toggle Controls in Sidebar Footer */
.theme-row {
  display: flex;
  justify-content: space-between;
  gap: 4px;
  background: rgba(0, 0, 0, 0.08);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  padding: 2px;
  margin-top: 8px;
}

:root[data-theme="light"] .theme-row {
  background: rgba(0, 0, 0, 0.04);
}

.theme-toggle-btn {
  flex: 1;
  background: transparent;
  border: none;
  color: var(--text-muted);
  font-size: 10px;
  font-weight: 500;
  cursor: pointer;
  padding: 4px 2px;
  border-radius: 4px;
  transition: all 0.15s ease;
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 4px;
}

.theme-toggle-btn:hover {
  color: var(--text-main);
  background: var(--bg-hover);
}

.theme-toggle-btn.active {
  color: var(--text-main);
  background: var(--bg-card);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.15);
  border: 1px solid var(--border-color);
}

/* Hover tooltips */
.has-tooltip {
  position: relative;
}

.has-tooltip::after {
  content: attr(data-tooltip);
  position: absolute;
  bottom: calc(100% + 8px);
  left: 50%;
  transform: translateX(-50%);
  padding: 6px 10px;
  border-radius: 6px;
  background: rgba(20, 20, 28, 0.96);
  border: 1px solid var(--border-color);
  color: var(--text-main);
  font-size: 11px;
  font-weight: 400;
  line-height: 1.4;
  white-space: normal;
  width: max-content;
  max-width: 220px;
  text-align: center;
  pointer-events: none;
  opacity: 0;
  visibility: hidden;
  transition: opacity 0.15s ease, visibility 0.15s ease;
  z-index: 1000;
  box-shadow: 0 8px 20px rgba(0, 0, 0, 0.35);
}

:root[data-theme="light"] .has-tooltip::after {
  background: rgba(255, 255, 255, 0.98);
  box-shadow: 0 8px 20px rgba(0, 0, 0, 0.12);
}

.has-tooltip:hover::after,
.has-tooltip:focus-visible::after {
  opacity: 1;
  visibility: visible;
}

.has-tooltip.tooltip-below::after {
  bottom: auto;
  top: calc(100% + 8px);
}

.delete-btn.has-tooltip::after {
  bottom: calc(100% + 4px);
  right: 0;
  left: auto;
  transform: none;
}

.about-dialog {
  width: 420px;
}

.about-body {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.about-close-btn {
  background: transparent;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 14px;
  padding: 4px;
}

.about-close-btn:hover {
  color: var(--text-main);
}

.about-logo-row {
  display: flex;
  align-items: center;
  gap: 14px;
}

.about-logo-mark {
  font-size: 28px;
  background: linear-gradient(135deg, var(--accent-color), var(--accent-cyan));
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.about-app-name {
  font-size: 20px;
  font-weight: 700;
  margin-bottom: 2px;
}

.about-version {
  font-size: 14px;
  font-weight: 600;
  color: var(--accent-cyan);
}

.about-version-detail {
  font-size: 11px;
  color: var(--text-muted);
  margin-top: 2px;
}

.about-channel-note {
  font-size: 12px;
  color: var(--text-muted);
  line-height: 1.5;
}

.about-actions {
  display: flex;
  justify-content: flex-start;
}

.about-update-status {
  font-size: 12px;
  color: var(--text-main);
}

.about-credit {
  font-size: 12px;
  color: var(--text-muted);
  text-align: center;
  padding-top: 4px;
  border-top: 1px solid var(--border-color);
}

.about-link {
  background: none;
  border: none;
  padding: 0;
  color: var(--accent-color);
  cursor: pointer;
  font: inherit;
  text-decoration: underline;
}

.about-link:hover {
  color: var(--accent-cyan);
}

/* Animations */
@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes slideDown {
  from { transform: translateY(-20px); opacity: 0; }
  to { transform: translateY(0); opacity: 1; }
}

/* --- Linux desktop layout --- */
[data-platform="linux"] .editor-header-linux {
  flex-direction: column;
  align-items: stretch;
  gap: 10px;
}

[data-platform="linux"] .toolbar-actions-linux {
  flex-wrap: wrap;
  width: 100%;
}

[data-platform="linux"] .editor-meta-row {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}

[data-platform="linux"] .editor-filename {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-muted);
  padding-left: 22px;
  user-select: text;
}

.editor-input-wrap {
  display: flex;
  flex: 1;
  min-height: 0;
  min-width: 0;
  overflow: hidden;
}

.line-numbers-gutter {
  overflow: hidden;
  flex: 0 0 auto;
  padding: 24px 8px 24px 12px;
  font-family: var(--font-mono);
  font-size: 12px;
  line-height: 1.6;
  color: var(--text-muted);
  text-align: right;
  user-select: none;
  border-right: 1px solid var(--border-color);
  background: rgba(0, 0, 0, 0.15);
}

.line-number {
  height: 1.6em;
}

.help-dialog {
  width: min(560px, 92vw);
  max-height: 85vh;
}

.help-body {
  max-height: 60vh;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.help-intro {
  font-size: 13px;
  line-height: 1.5;
}

.help-section-title {
  font-size: 13px;
  font-weight: 600;
  margin-bottom: 6px;
}

.help-section-body {
  font-size: 12px;
  line-height: 1.55;
  color: var(--text-muted);
  white-space: pre-line;
}

```

### src/hooks/useLinuxMenuBridge.ts
```
import { useEffect, useRef } from "react";
import { invoke, listen } from "../utils/ipc";
import type { ThemePreference } from "../utils/theme";

type MenuBridgeOptions = {
  createNewDraft: () => Promise<void>;
  createNewSavedNote: () => Promise<void>;
  saveNoteNow: (options?: { promoteToSaved?: boolean }) => Promise<void>;
  deleteCurrentNote: () => Promise<void>;
  loadNotes: () => Promise<void>;
  loadClipboardHistory: () => Promise<void>;
  setEditorView: (view: "edit" | "preview" | "split") => void;
  setShowPalette: (open: boolean) => void;
  setPaletteQuery: (query: string) => void;
  setSelectedPaletteIdx: (idx: number) => void;
  setShowClipboard: (open: boolean) => void;
  setSelectedClipIdx: (idx: number) => void;
  setShowLineNumbers: (value: boolean | ((prev: boolean) => boolean)) => void;
  setTheme: (theme: ThemePreference) => void;
  setShowHelp: (open: boolean) => void;
  setAboutAutoCheck: (check: boolean) => void;
  setShowAbout: (open: boolean) => void;
};

/** Wire native Linux menu bar events to React state (no-op on other platforms). */
export function useLinuxMenuBridge(options: MenuBridgeOptions) {
  const optionsRef = useRef(options);
  optionsRef.current = options;

  useEffect(() => {
    const handlers: Array<Promise<() => void>> = [
      listen("menu-about", () => {
        optionsRef.current.setAboutAutoCheck(false);
        optionsRef.current.setShowAbout(true);
      }),
      listen("menu-check-updates", () => {
        optionsRef.current.setAboutAutoCheck(true);
        optionsRef.current.setShowAbout(true);
      }),
      listen("menu-new-draft", () => {
        void optionsRef.current.createNewDraft();
      }),
      listen("menu-new-saved-note", () => {
        void optionsRef.current.createNewSavedNote();
      }),
      listen("menu-save", () => {
        void optionsRef.current.saveNoteNow();
      }),
      listen("menu-save-permanent", () => {
        void optionsRef.current.saveNoteNow({ promoteToSaved: true });
      }),
      listen("menu-delete-note", () => {
        void optionsRef.current.deleteCurrentNote();
      }),
      listen("menu-archive-stale", () => {
        void invoke<number>("archive_notes").then((count) => {
          if (count > 0) {
            void optionsRef.current.loadNotes();
          }
        });
      }),
      listen("menu-view-edit", () => optionsRef.current.setEditorView("edit")),
      listen("menu-view-split", () => optionsRef.current.setEditorView("split")),
      listen("menu-view-preview", () => optionsRef.current.setEditorView("preview")),
      listen("menu-command-palette", () => {
        optionsRef.current.setPaletteQuery("");
        optionsRef.current.setSelectedPaletteIdx(0);
        optionsRef.current.setShowPalette(true);
        optionsRef.current.setShowClipboard(false);
      }),
      listen("menu-clipboard-history", () => {
        void optionsRef.current.loadClipboardHistory();
        optionsRef.current.setSelectedClipIdx(0);
        optionsRef.current.setShowClipboard(true);
        optionsRef.current.setShowPalette(false);
      }),
      listen("menu-toggle-line-numbers", () => {
        optionsRef.current.setShowLineNumbers((prev) => !prev);
      }),
      listen("menu-theme-light", () => optionsRef.current.setTheme("light")),
      listen("menu-theme-dark", () => optionsRef.current.setTheme("dark")),
      listen("menu-theme-system", () => optionsRef.current.setTheme("system")),
      listen("menu-help", () => optionsRef.current.setShowHelp(true)),
    ];

    return () => {
      void Promise.all(handlers).then((unsubs) => {
        unsubs.forEach((unsub) => unsub());
      });
    };
  }, []);
}

```

### src/components/HelpDialog.tsx
```
import { LINUX_HELP_SECTIONS, LINUX_SUPPORT_EMAIL } from "../content/linuxHelp";
import { openUrl } from "../utils/ipc";

interface HelpDialogProps {
  open: boolean;
  onClose: () => void;
}

export function HelpDialog({ open, onClose }: HelpDialogProps) {
  if (!open) {
    return null;
  }

  return (
    <div className="overlay-backdrop" onClick={onClose}>
      <div
        className="modal-box help-dialog"
        onClick={(e) => e.stopPropagation()}
        role="dialog"
        aria-labelledby="help-title"
        aria-modal="true"
      >
        <div className="modal-header">
          <h3 className="modal-title" id="help-title">
            Gnomad Slate — Help
          </h3>
          <button
            type="button"
            className="about-close-btn"
            aria-label="Close Help dialog"
            onClick={onClose}
          >
            ✕
          </button>
        </div>
        <div className="modal-body help-body">
          <p className="help-intro">
            Quick reference for the Linux desktop build. Need a human?{" "}
            <button
              type="button"
              className="about-link"
              onClick={() => void openUrl(`mailto:${LINUX_SUPPORT_EMAIL}`)}
            >
              {LINUX_SUPPORT_EMAIL}
            </button>
          </p>
          {LINUX_HELP_SECTIONS.map((section) => (
            <section key={section.title} className="help-section">
              <h4 className="help-section-title">{section.title}</h4>
              <p className="help-section-body">{section.body}</p>
            </section>
          ))}
        </div>
      </div>
    </div>
  );
}

```

### src/content/linuxHelp.ts
```
/** Detailed help copy shown in the Linux Help dialog. */
export const LINUX_HELP_SECTIONS = [
  {
    title: "Getting started",
    body: `Gnomad Slate is a desktop scratchpad for quick markdown notes, clipboard history, and Vault-X encrypted secrets.

Press Alt+Space from anywhere to summon or hide the window. Slate auto-hides when you click away — use Alt+Space or the Window menu to bring it back.`,
  },
  {
    title: "Notes",
    body: `Scratchpad notes are ephemeral and auto-archive after 48 hours. Saved notes live in Vaults until you delete them.

Ctrl+S saves the current note. If you're on a scratchpad draft, Ctrl+S promotes it to a saved note. Ctrl+N starts a new scratchpad draft.

Use Edit, Split, or Preview in the toolbar to change how you work with markdown.`,
  },
  {
    title: "Command palette & clipboard",
    body: `Ctrl+P opens the command palette for JSON formatting, markdown cleanup, theme switching, and more.

Ctrl+Shift+V opens the clipboard history capsule. Toggle "Filter sensitive clipboard items" in the sidebar to block likely API keys and tokens.`,
  },
  {
    title: "Vault-X secrets",
    body: `Open the Secrets tab to create or unlock an AES-256-GCM vault protected by an Argon2id master password.

Secrets never leave Rust memory in full — the UI only sees metadata until you copy or reveal a value. Lock the vault when you're done.`,
  },
  {
    title: "Data location",
    body: `Notes and vault data are stored locally at ~/.local/share/gnomad-slate/

Back up that folder to preserve your notes and encrypted secrets.`,
  },
  {
    title: "Support",
    body: `Questions, bugs, or feedback: david@gnomadstudio.org

Built with ❤️ by Gnomad Studio — https://gnomadstudio.org`,
  },
] as const;

export const LINUX_SUPPORT_EMAIL = "david@gnomadstudio.org";

```

### src/utils/platform.ts
```
import { type as osType } from "@tauri-apps/plugin-os";
import { isTauri } from "./ipc";

/** True when running the Tauri desktop build on Linux. */
export function isLinuxDesktop(): boolean {
  if (!isTauri()) return false;
  try {
    return osType() === "linux";
  } catch {
    return false;
  }
}

```

### src/utils/ipc.ts
```
import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { listen as tauriListen } from "@tauri-apps/api/event";
import { getCurrentWindow as getTauriWindow, LogicalSize as TauriLogicalSize } from "@tauri-apps/api/window";
import { openUrl as tauriOpenUrl } from "@tauri-apps/plugin-opener";
import { exit as tauriExit } from "@tauri-apps/plugin-process";
import { isLinuxDesktop } from "./platform";

/** Determine if running in the Tauri desktop environment */
export const isTauri = (): boolean => {
  return typeof window !== "undefined" && (window as any).__TAURI_INTERNALS__ !== undefined;
};

/** Platform-agnostic openUrl helper */
export async function openUrl(url: string): Promise<void> {
  if (isTauri()) {
    return tauriOpenUrl(url);
  } else {
    window.open(url, "_blank", "noopener,noreferrer");
  }
}

/** Chrome extension API when running outside Tauri (optional). */
function extensionChrome(): typeof chrome | undefined {
  return (globalThis as { chrome?: typeof chrome }).chrome;
}

/** Platform-agnostic invoke wrapper */
export async function invoke<T>(cmd: string, args?: Record<string, any>): Promise<T> {
  if (isTauri()) {
    return tauriInvoke<T>(cmd, args);
  } else {
    const chromeApi = extensionChrome();
    const runtime = chromeApi?.runtime;
    if (runtime?.sendMessage) {
      return new Promise<T>((resolve, reject) => {
        runtime.sendMessage!({ type: "tauri-command", cmd, args }, (response: unknown) => {
          if (runtime.lastError) {
            reject(new Error(runtime.lastError.message));
          } else if (
            response &&
            typeof response === "object" &&
            "error" in response &&
            (response as { error?: unknown }).error
          ) {
            reject((response as { error: unknown }).error);
          } else {
            resolve((response as { data?: T } | undefined)?.data as T);
          }
        });
      });
    }
    return Promise.reject(new Error("No platform IPC handler available"));
  }
}

/** Platform-agnostic event listener wrapper */
export function listen<T>(event: string, handler: (event: { payload: T }) => void): Promise<() => void> {
  if (isTauri()) {
    return tauriListen<T>(event, handler);
  } else {
    const chromeApi = extensionChrome();
    const onMessage = chromeApi?.runtime?.onMessage;
    if (onMessage) {
      const listener = (message: unknown) => {
        if (
          message &&
          typeof message === "object" &&
          (message as { type?: string; event?: string }).type === "tauri-event" &&
          (message as { event?: string }).event === event
        ) {
          handler({ payload: (message as { payload: T }).payload });
        }
      };
      onMessage.addListener(listener);
      return Promise.resolve(() => {
        onMessage.removeListener(listener);
      });
    }
    return Promise.resolve(() => {});
  }
}

/** Destroy the window and exit the process on Linux (global shortcut keeps app alive otherwise). */
export async function quitDesktopApp(): Promise<void> {
  if (!isTauri()) {
    window.close();
    return;
  }
  const win = getTauriWindow();
  await win.destroy();
  if (isLinuxDesktop()) {
    await tauriExit(0);
  }
}

/** Platform-agnostic window helper */
export const getPlatformWindow = () => {
  if (isTauri()) {
    const win = getTauriWindow();
    return {
      innerSize: () => win.innerSize(),
      setSize: (size: any) => win.setSize(size),
      setTheme: (theme: "light" | "dark") => win.setTheme(theme),
      setBackgroundColor: (color: string) => win.setBackgroundColor(color),
      destroy: () => win.destroy(),
    };
  } else {
    // Mock window methods for extension environment
    return {
      innerSize: () => Promise.resolve({ width: window.innerWidth, height: window.innerHeight }),
      setSize: async (_size: any) => {
        // Extensions cannot resize their own popups/sidepanels directly, so we do nothing
      },
      setTheme: async (_theme: "light" | "dark") => {
        // Themes are handled purely via document data-themes in CSS, so do nothing
      },
      setBackgroundColor: async (_color: string) => {
        // Handled via CSS, so do nothing
      },
      destroy: async () => {
        window.close();
      },
    };
  }
};

/** Mock LogicalSize that maps to Tauri's LogicalSize if available */
export class LogicalSize {
  width: number;
  height: number;

  constructor(width: number, height: number) {
    this.width = width;
    this.height = height;
    if (isTauri()) {
      return new TauriLogicalSize(width, height) as any;
    }
  }
}

```

### src/utils/modKeyLabel.ts
```
export function isMacOS(): boolean {
  if (typeof navigator === "undefined") return false;
  const platform = navigator.platform ?? "";
  const ua = navigator.userAgent ?? "";
  return /Mac|iPhone|iPod|iPad/i.test(platform) || /Mac OS X/i.test(ua);
}

export function modKeyLabel(): string {
  return isMacOS() ? "⌘" : "Ctrl";
}

export function formatShortcut(shortcut: string): string {
  if (isMacOS()) {
    return shortcut.replace(/\bCtrl\b/g, "⌘").replace(/\bCmd\b/g, "⌘");
  }
  return shortcut.replace(/\bCmd\b/g, "Ctrl");
}

export function modShortcut(key: string, extras?: { shift?: boolean; alt?: boolean }): string {
  const parts: string[] = [modKeyLabel()];
  if (extras?.shift) parts.push("Shift");
  if (extras?.alt) parts.push(isMacOS() ? "⌥" : "Alt");
  parts.push(key);
  return parts.join("+");
}

```

### scripts/install-linux-deps.sh
```
#!/usr/bin/env bash
# Install Tauri 2 Linux build dependencies (Fedora/Nobara or Debian/Ubuntu).
set -euo pipefail

if command -v dnf >/dev/null 2>&1; then
  echo "Installing via dnf (Fedora/Nobara)…"
  sudo dnf install -y \
    webkit2gtk4.1-devel \
    openssl-devel \
    curl \
    wget \
    file \
    libappindicator-gtk3-devel \
    librsvg2-devel \
    patchelf
  exit 0
fi

if command -v apt-get >/dev/null 2>&1; then
  echo "Installing via apt (Debian/Ubuntu)…"
  sudo apt-get update
  sudo apt-get install -y \
    libwebkit2gtk-4.1-dev \
    libappindicator3-dev \
    librsvg2-dev \
    patchelf \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev
  exit 0
fi

echo "Unsupported distro: install webkit2gtk-4.1 dev packages manually." >&2
exit 1

```

### package.json
```
{
  "name": "gnomad-slate",
  "private": true,
  "version": "2.1.0-beta.2",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "tauri": "tauri",
    "tauri:dev": "tauri dev",
    "tauri:dev:x11": "GDK_BACKEND=x11 WEBKIT_DISABLE_DMABUF_RENDERER=1 tauri dev",
    "tauri:build": "tauri build",
    "build:extension": "node extension-gemini/build.mjs",
    "tauri:build:ci": "tauri build",
    "docs:export": "node scripts/export-docs.mjs",
    "test": "vitest run",
    "test:watch": "vitest"
  },
  "dependencies": {
    "@tauri-apps/api": "^2",
    "@tauri-apps/plugin-opener": "^2",
    "@tauri-apps/plugin-os": "^2.3.2",
    "@tauri-apps/plugin-process": "^2.3.1",
    "@tauri-apps/plugin-updater": "^2.10.1",
    "marked": "^15.0.12",
    "react": "^19.1.0",
    "react-dom": "^19.1.0"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2",
    "@types/react": "^19.1.8",
    "@types/react-dom": "^19.1.6",
    "@vitejs/plugin-react": "^4.6.0",
    "jsdom": "^26.1.0",
    "typescript": "~5.8.3",
    "vite": "^7.0.4",
    "vitest": "^3.2.4"
  }
}

```

pub mod crypto;
pub mod filesystem;
pub mod clipboard;
pub mod biometrics;

use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

use crate::clipboard::{ClipboardEntry, ClipboardHistory};
use crate::filesystem::Note;

pub struct AppState {
    pub clipboard_history: Arc<Mutex<ClipboardHistory>>,
}

// --- Tauri Command Definitions ---

#[tauri::command]
fn get_clipboard_history(state: tauri::State<AppState>) -> Vec<ClipboardEntry> {
    if let Ok(hist) = state.clipboard_history.lock() {
        hist.get_entries()
    } else {
        Vec::new()
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
fn initialize_vault(password: String) -> Result<(), String> {
    biometrics::set_master_password(&password)?;
    // Create an empty encrypted secrets file initially
    save_vault("[]".to_string(), password)
}

#[tauri::command]
fn unlock_vault(password: String) -> Result<String, String> {
    // 1. Verify master password
    if !biometrics::verify_master_password(&password)? {
        return Err("Invalid password".to_string());
    }
    
    // 2. Read encrypted secrets file: `<slate_dir>/vault.enc`
    let vault_file = filesystem::get_slate_dir().join("vault.enc");
    if !vault_file.exists() {
        return Ok("[]".to_string()); // Return empty JSON array if no secrets saved yet
    }
    
    let encrypted_data = std::fs::read(vault_file)
        .map_err(|e| format!("Failed to read secure vault: {}", e))?;
        
    // 3. Decrypt data
    let decrypted_bytes = crypto::decrypt(&encrypted_data, &password)?;
    let decrypted_string = String::from_utf8(decrypted_bytes)
        .map_err(|_| "Decrypted payload is not valid UTF-8".to_string())?;
        
    Ok(decrypted_string)
}

#[tauri::command]
fn save_vault(decrypted_json: String, password: String) -> Result<(), String> {
    // 1. Verify master password
    if !biometrics::verify_master_password(&password)? {
        return Err("Invalid password (cannot lock vault)".to_string());
    }
    
    // 2. Encrypt JSON data
    let encrypted_data = crypto::encrypt(decrypted_json.as_bytes(), &password)?;
    
    // 3. Save to `<slate_dir>/vault.enc`
    let vault_file = filesystem::get_slate_dir().join("vault.enc");
    std::fs::write(vault_file, encrypted_data)
        .map_err(|e| format!("Failed to write secure vault: {}", e))?;
        
    Ok(())
}

// --- Window Toggling Helper ---

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

// --- Main Runner ---

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 1. Initialize thread-safe Clipboard history (keeps last 15 elements)
    let clipboard_history = Arc::new(Mutex::new(ClipboardHistory::new(15)));
    
    // 2. Start system clipboard monitoring in background
    clipboard::start_clipboard_monitor(clipboard_history.clone());
    
    // 3. Setup global shortcut plugin (Option+Space or Alt+Space)
    let shortcut = "Option+Space"; // Falls back gracefully on Linux to Alt+Space
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(AppState { clipboard_history })
        .setup(move |app| {
            // Register hotkey using global shortcut plugin
            let handle = app.handle().clone();
            
            use tauri_plugin_global_shortcut::ShortcutState;

            let shortcut_parsed = shortcut.parse::<Shortcut>().unwrap();
                
            let _ = app.global_shortcut().on_shortcut(shortcut_parsed, move |_app, _shortcut, event| {
                if event.state() == ShortcutState::Pressed {
                    toggle_window(&handle);
                }
            });

            // Automatically hide window when focus is lost (blur event)
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
            list_notes,
            read_note,
            write_note,
            delete_note,
            archive_notes,
            is_vault_initialized,
            initialize_vault,
            unlock_vault,
            save_vault
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

use arboard::Clipboard;

use crate::app_state::AppState;
use crate::clipboard::ClipboardEntry;

#[tauri::command]
pub fn get_clipboard_history(state: tauri::State<AppState>) -> Vec<ClipboardEntry> {
    if let Ok(hist) = state.clipboard_history.lock() {
        hist.get_entries()
    } else {
        Vec::new()
    }
}

#[tauri::command]
pub fn get_clipboard_filter_enabled(state: tauri::State<AppState>) -> bool {
    state
        .clipboard_filter_enabled
        .lock()
        .map(|g| *g)
        .unwrap_or(true)
}

#[tauri::command]
pub fn set_clipboard_filter_enabled(enabled: bool, state: tauri::State<AppState>) {
    if let Ok(mut guard) = state.clipboard_filter_enabled.lock() {
        *guard = enabled;
    }
}

#[tauri::command]
pub fn copy_text_to_clipboard(text: String) -> Result<(), String> {
    let mut clipboard = Clipboard::new()
        .map_err(|e| format!("Clipboard unavailable: {}", e))?;
    clipboard
        .set_text(text)
        .map_err(|e| format!("Failed to copy to clipboard: {}", e))?;
    Ok(())
}

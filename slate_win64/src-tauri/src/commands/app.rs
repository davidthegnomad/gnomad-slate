use tauri::AppHandle;

use crate::app_state::{AppState, AppVersionInfo};
use crate::shortcuts;
use crate::vault_session;
use crate::window;

#[tauri::command]
pub fn get_app_version_info() -> AppVersionInfo {
    let version = env!("CARGO_PKG_VERSION").to_string();
    let channel = "beta";
    let display_version = format!("v{version} (Windows)");

    AppVersionInfo {
        version,
        channel: channel.to_string(),
        display_version,
    }
}

#[tauri::command]
pub fn get_summon_shortcut_label() -> String {
    shortcuts::summon_shortcut().to_string()
}

/// Called by the frontend after save/cleanup. Backend locks vault and exits.
#[tauri::command]
pub fn confirm_quit(app: AppHandle, state: tauri::State<AppState>) {
    if window::mark_quit_in_progress(&app) {
        tracing::debug!("confirm_quit ignored — already quitting");
        return;
    }
    vault_session::close_session(&state.vault_session);
    tracing::info!("application exit confirmed");
    app.exit(0);
}

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

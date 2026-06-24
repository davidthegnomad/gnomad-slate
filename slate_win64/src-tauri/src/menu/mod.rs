mod windows;

use tauri::{AppHandle, Emitter};

use crate::window;

pub fn install_app_menu(app: &AppHandle) -> tauri::Result<()> {
    windows::install(app)
}

pub fn handle_menu_event(app: &AppHandle, event_id: &str) {
    windows::handle_event(app, event_id);
}

pub(crate) fn focus_main_window(app: &AppHandle) {
    window::focus_main_window(app);
}

pub(crate) fn emit_to_frontend(app: &AppHandle, event: &str) {
    let _ = app.emit(event, ());
    focus_main_window(app);
}

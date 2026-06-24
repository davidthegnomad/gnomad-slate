//! Main window visibility, close-to-tray, and global summon shortcut.

use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::ShortcutState;

use crate::app_state::AppState;
use crate::shortcuts;

pub fn is_quit_in_progress(app: &AppHandle) -> bool {
    app.state::<AppState>()
        .window_lifecycle
        .is_quit_in_progress()
}

pub fn mark_quit_in_progress(app: &AppHandle) -> bool {
    app.state::<AppState>()
        .window_lifecycle
        .mark_quit_in_progress()
}

pub fn emit_quit_requested(app: &AppHandle) {
    if is_quit_in_progress(app) {
        return;
    }
    let _ = app.emit("app-quit-requested", ());
}

pub fn show_main_window(app: &AppHandle) {
    let lifecycle = &app.state::<AppState>().window_lifecycle;
    lifecycle.set_intentionally_hidden(false);
    focus_main_window(app);
}

pub fn hide_main_window(app: &AppHandle, intentional: bool) {
    if intentional {
        app.state::<AppState>()
            .window_lifecycle
            .set_intentionally_hidden(true);
    }
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

pub fn menu_show_main_window(app: &AppHandle) {
    show_main_window(app);
}

pub fn menu_hide_main_window(app: &AppHandle) {
    hide_main_window(app, true);
}

pub fn focus_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn toggle_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        match window.is_visible() {
            Ok(true) => hide_main_window(app, true),
            _ => show_main_window(app),
        }
    }
}

pub fn register_global_shortcut(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let shortcut = shortcuts::summon_shortcut();
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

/// Windows: close-to-tray — X hides the window; File → Exit fully quits.
pub fn attach_window_close_handler(window: &tauri::WebviewWindow) {
    let app = window.app_handle().clone();
    let w = window.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            if is_quit_in_progress(&app) {
                return;
            }
            api.prevent_close();
            app.state::<AppState>()
                .window_lifecycle
                .set_intentionally_hidden(true);
            let _ = w.hide();
        }
    });
}

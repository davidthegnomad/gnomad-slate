pub mod app_state;
pub mod biometrics;
pub mod clipboard;
pub mod clipboard_filter;
pub mod commands;
pub mod crypto;
pub mod filesystem;
pub mod logging;
pub mod menu;
pub mod shortcuts;
pub mod unlock_limiter;
pub mod vault_io;
pub mod vault_migration;
pub mod vault_session;
pub mod window;
pub mod windows_clipboard;
pub mod windows_tray;

use std::sync::{Arc, Mutex};

use tauri::{Manager, RunEvent};

use crate::app_state::AppState;
use crate::clipboard::ClipboardHistory;
use crate::unlock_limiter::UnlockRateLimiter;
pub use crate::window::{emit_quit_requested, menu_hide_main_window, menu_show_main_window};

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
    let unlock_limiter = Arc::new(Mutex::new(UnlockRateLimiter::new()));

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
            window_lifecycle: crate::app_state::WindowLifecycle::new(),
        })
        .setup(move |app| {
            if let Err(e) = menu::install_app_menu(&app.handle()) {
                tracing::warn!("failed to install app menu: {e}");
            }

            #[cfg(desktop)]
            if let Err(e) = window::register_global_shortcut(&app.handle()) {
                tracing::warn!("failed to register global shortcut: {e}");
            }

            if let Err(e) = windows_tray::install_tray(&app.handle()) {
                tracing::warn!("failed to install system tray: {e}");
            }

            if let Some(window) = app.get_webview_window("main") {
                window::attach_window_close_handler(&window);
            }

            Ok(())
        })
        .on_menu_event(|app, event| {
            menu::handle_menu_event(app, event.id().0.as_str());
        })
        .invoke_handler(tauri::generate_handler![
            commands::app::get_app_version_info,
            commands::app::get_summon_shortcut_label,
            commands::app::confirm_quit,
            commands::clipboard::copy_text_to_clipboard,
            commands::clipboard::get_clipboard_history,
            commands::clipboard::get_clipboard_filter_enabled,
            commands::clipboard::set_clipboard_filter_enabled,
            commands::notes::list_notes,
            commands::notes::read_note,
            commands::notes::write_note,
            commands::notes::delete_note,
            commands::notes::archive_notes,
            commands::vault::is_vault_initialized,
            commands::vault::is_vault_unlocked,
            commands::vault::initialize_vault,
            commands::vault::unlock_vault,
            commands::vault::lock_vault,
            commands::vault::list_secret_metadata,
            commands::vault::reveal_secret_value,
            commands::vault::copy_secret_to_clipboard,
            commands::vault::add_secret,
            commands::vault::delete_secret,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            if let RunEvent::ExitRequested { api, .. } = event {
                if window::is_quit_in_progress(app_handle) {
                    return;
                }
                api.prevent_exit();
                emit_quit_requested(app_handle);
            }
        });
}

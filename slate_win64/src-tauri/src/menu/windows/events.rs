use tauri::{AppHandle, Manager};
use tauri_plugin_opener::OpenerExt;

use crate::emit_quit_requested;
use super::super::{emit_to_frontend, focus_main_window};

/// Routes native Windows menu item clicks to frontend events or window actions.
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
        "window_close" | "file_close" => {
            emit_to_frontend(app, "window-close-requested");
        }
        "file_quit" => emit_quit_requested(app),
        "contact_support" => {
            let _ = app.opener().open_url(
                "mailto:david@gnomadstudio.org?subject=Gnomad%20Slate%20Windows%20Support",
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

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
        Some("Ctrl+N"),
    )?;
    let new_saved = MenuItem::with_id(
        app,
        "new_saved_note",
        "New Saved Note",
        true,
        Some("Ctrl+Shift+N"),
    )?;
    let save_note = MenuItem::with_id(
        app,
        "save_note",
        "Save",
        true,
        Some("Ctrl+S"),
    )?;
    let save_permanent = MenuItem::with_id(
        app,
        "save_permanent",
        "Save as Permanent Note",
        true,
        Some("Ctrl+Shift+S"),
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
        Some("Ctrl+P"),
    )?;
    let clipboard_history = MenuItem::with_id(
        app,
        "clipboard_history",
        "Clipboard History",
        true,
        Some("Ctrl+Shift+V"),
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
    let window_hide = MenuItem::with_id(app, "window_hide", "Hide Window", true, Some("Super+Shift+Space"))?;
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

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::AppHandle;

use super::{emit_to_frontend, focus_main_window};

pub fn install(app: &AppHandle) -> tauri::Result<()> {
    let about = MenuItem::with_id(app, "about", "About Gnomad Slate", true, None::<&str>)?;
    let check_updates =
        MenuItem::with_id(app, "check_updates", "Check for Updates…", true, None::<&str>)?;

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
    let help_submenu = Submenu::with_items(app, "Help", true, &[&about, &check_updates])?;
    let root = Menu::with_items(app, &[&edit_submenu, &help_submenu])?;

    app.set_menu(root)?;
    Ok(())
}

pub fn handle_event(app: &AppHandle, event_id: &str) {
    match event_id {
        "about" => {
            emit_to_frontend(app, "menu-about");
        }
        "check_updates" => {
            emit_to_frontend(app, "menu-check-updates");
        }
        _ => {}
    }
}

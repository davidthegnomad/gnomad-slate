use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::AppHandle;

use super::{emit_to_frontend, focus_main_window};

pub fn install(app: &AppHandle) -> tauri::Result<()> {
    let about = MenuItem::with_id(app, "about", "About Gnomad Slate", true, None::<&str>)?;
    let check_updates =
        MenuItem::with_id(app, "check_updates", "Check for Updates…", true, None::<&str>)?;

    let app_submenu = Submenu::with_items(
        app,
        "Gnomad Slate",
        true,
        &[
            &about,
            &PredefinedMenuItem::separator(app)?,
            &check_updates,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::hide(app, None)?,
            &PredefinedMenuItem::hide_others(app, None)?,
            &PredefinedMenuItem::show_all(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::quit(app, None)?,
        ],
    )?;
    // No key equivalent: Cmd+Z must reach the focused field.
    let undo = MenuItem::with_id(app, "edit-undo", "Undo", true, None::<&str>)?;
    let redo = MenuItem::with_id(app, "edit-redo", "Redo", true, None::<&str>)?;
    let edit_submenu = Submenu::with_items(
        app,
        "Edit",
        true,
        &[
            &undo,
            &redo,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::cut(app, None)?,
            &PredefinedMenuItem::copy(app, None)?,
            &PredefinedMenuItem::paste(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::select_all(app, None)?,
        ],
    )?;
    let root = Menu::with_items(app, &[&app_submenu, &edit_submenu])?;

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
        "edit-undo" => {
            emit_to_frontend(app, "menu-undo");
        }
        "edit-redo" => {
            emit_to_frontend(app, "menu-redo");
        }
        _ => {}
    }
}

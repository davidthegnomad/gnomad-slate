use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Emitter, Manager};

pub fn install_app_menu(app: &AppHandle) -> tauri::Result<()> {
    let about = MenuItem::with_id(app, "about", "About Gnomad Slate", true, None::<&str>)?;
    let check_updates =
        MenuItem::with_id(app, "check_updates", "Check for Updates…", true, None::<&str>)?;

    #[cfg(target_os = "macos")]
    let root = {
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
        let edit_submenu = Submenu::with_items(
            app,
            "Edit",
            true,
            &[
                &PredefinedMenuItem::undo(app, None)?,
                &PredefinedMenuItem::redo(app, None)?,
                &PredefinedMenuItem::separator(app)?,
                &PredefinedMenuItem::cut(app, None)?,
                &PredefinedMenuItem::copy(app, None)?,
                &PredefinedMenuItem::paste(app, None)?,
                &PredefinedMenuItem::separator(app)?,
                &PredefinedMenuItem::select_all(app, None)?,
            ],
        )?;
        Menu::with_items(app, &[&app_submenu, &edit_submenu])?
    };

    #[cfg(not(target_os = "macos"))]
    let root = {
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
        Menu::with_items(app, &[&edit_submenu, &help_submenu])?
    };

    app.set_menu(root)?;
    Ok(())
}

pub fn handle_menu_event(app: &AppHandle, event_id: &str) {
    match event_id {
        "about" => {
            let _ = app.emit("menu-about", ());
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        "check_updates" => {
            let _ = app.emit("menu-check-updates", ());
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        _ => {}
    }
}

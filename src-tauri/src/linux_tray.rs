//! Linux system tray — show/hide/quit when the window is hidden.

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter};

use crate::{menu_hide_main_window, menu_show_main_window};

pub fn install_tray(app: &AppHandle) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "tray_show", "Show Gnomad Slate", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "tray_hide", "Hide Window", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "tray_quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &hide, &quit])?;

    let icon = match app.default_window_icon() {
        Some(i) => i.clone(),
        None => return Ok(()),
    };

    let _tray = TrayIconBuilder::with_id("gnomad-slate-tray")
        .icon(icon)
        .menu(&menu)
        .tooltip("Gnomad Slate")
        .on_menu_event(|app, event| match event.id().as_ref() {
            "tray_show" => menu_show_main_window(app),
            "tray_hide" => menu_hide_main_window(app),
            "tray_quit" => {
                let _ = app.emit("app-quit-requested", ());
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                menu_show_main_window(&app);
            }
        })
        .build(app)?;

    Ok(())
}

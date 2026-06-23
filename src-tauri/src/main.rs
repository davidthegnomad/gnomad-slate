// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    #[cfg(target_os = "linux")]
    tauri_app_lib::linux_display::apply_launch_env();
    tauri_app_lib::run()
}

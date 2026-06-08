//! macOS application activation helpers.

#[cfg(target_os = "macos")]
pub fn application_is_active() -> bool {
    use objc2_app_kit::NSRunningApplication;

    NSRunningApplication::currentApplication().isActive()
}

#[cfg(not(target_os = "macos"))]
pub fn application_is_active() -> bool {
    false
}

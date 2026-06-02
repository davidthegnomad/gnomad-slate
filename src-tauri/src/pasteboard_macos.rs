//! macOS pasteboard change detection without reading full clipboard text.

#[cfg(target_os = "macos")]
pub fn general_change_count() -> Option<isize> {
    use objc2_app_kit::NSPasteboard;
    let pasteboard = NSPasteboard::generalPasteboard();
    Some(pasteboard.changeCount())
}

#[cfg(not(target_os = "macos"))]
pub fn general_change_count() -> Option<isize> {
    None
}

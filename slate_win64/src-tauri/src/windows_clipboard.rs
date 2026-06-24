//! Windows clipboard change detection via GetClipboardSequenceNumber.
//! Provides an efficient way to detect clipboard changes without polling content.

/// Returns the Windows clipboard sequence number or None.
/// On Windows, GetClipboardSequenceNumber changes every time the clipboard content changes,
/// allowing us to avoid expensive polling when nothing has changed.
pub fn general_change_count() -> Option<isize> {
    #[cfg(target_os = "windows")]
    {
        unsafe {
            let count = winapi::um::winuser::GetClipboardSequenceNumber();
            Some(count as isize)
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        None
    }
}

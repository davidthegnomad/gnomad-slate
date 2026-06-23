//! WebKitGTK display workarounds for Linux desktop launches.

/// Apply before GTK/WebKit init (call from `main`).
///
/// App menu / `.desktop` launches use Wayland by default on Nobara/KDE/GNOME.
/// WebKitGTK often crashes with `Gdk Error 71 (Protocol error)` on Wayland;
/// forcing X11 matches the stable `tauri:dev:x11` path.
pub fn apply_launch_env() {
    if std::env::var_os("GDK_BACKEND").is_none() {
        std::env::set_var("GDK_BACKEND", "x11");
    }
    if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }
}

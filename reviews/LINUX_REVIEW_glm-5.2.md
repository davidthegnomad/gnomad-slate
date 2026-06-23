<!-- model: z-ai/glm-5.2 -->
<!-- generated: 2026-06-23T20:49:25.187158+00:00 -->

## Linux optimization findings — Rust / Tauri backend

### Critical (blocks good Linux desktop UX)
1. **`src-tauri/src/lib.rs`**: `maybe_show_on_app_activate` calls `crate::macos_app::application_is_active()` unconditionally inside the `MainEventsCleared` event loop. This leaks macOS-specific behavior into Linux, causing compilation failures or runtime panics. **Fix**: Gate the `macos_app` module and `maybe_show_on_app_activate` with `#[cfg(target_os = "macos")]`, or remove the call entirely from the Linux build path.
2. **`scripts/install-linux-deps.sh`**: Missing `libsoup3` development packages. WebKit2GTK 4.1 strictly requires it, and the build will fail on a fresh Nobara/Fedora install. **Fix**: Add `libsoup3-devel` to the `dnf` block and `libsoup-3.0-dev` to the `apt-get` block.
3. **`src-tauri/src/lib.rs`**: The global shortcut `Alt+Space` conflicts with default window menu shortcuts in GNOME, KDE, and most X11/Wayland window managers. It will likely fail to register or break native window controls. **Fix**: Change the default to a less intrusive combination like `Super+Space` or `Ctrl+Alt+Space`, and expose it as a user-configurable setting.

### High — Wayland/WebKit/GTK/Tauri lifecycle
- **Hide-on-blur (`src-tauri/src/lib.rs`)**: `attach_hide_on_blur` relies on `WindowEvent::Focused(false)` and a 200ms thread sleep. Wayland compositors (like Mutter) often do not send reliable focus events, especially when interacting with panels or other windows. This will cause the window to either refuse to hide or hide randomly. **Fix**: Remove the blur-based hiding for Wayland; rely on the global shortcut or a tray icon to toggle visibility instead.
- **Window close vs process exit (`src-tauri/src/lib.rs`)**: `CloseRequested` prevents the window from closing and emits to the frontend. If the frontend fails to handle this, the user has no way to quit the app except killing the process. Ensure the `Quit` menu item (`PredefinedMenuItem::quit` in `linux.rs`) actually calls `app.exit(0)` and isn't swallowed by the close-requested logic.
- **RPM Packaging (`tauri.conf.json`)**: `targets: "all"` builds RPM, DEB, and AppImage. The dependency `libappindicator-gtk3` is deprecated. **Fix**: Switch to `ayatana-appindicator3` if a tray icon is used, or remove the tray dependency entirely if not needed. Ensure the generated `.desktop` file includes `StartupWMClass=com.gnomadstudio.gnomad-slate` for proper taskbar grouping.

### Medium — performance & maintainability
- **`arboard` on Wayland (`src-tauri/src/lib.rs`)**: `arboard` sets the clipboard, but Wayland requires the owning process to stay alive to serve the clipboard contents. If the app hides immediately after copying a secret, the clipboard data may be lost. **Fix**: Consider using a clipboard manager daemon approach or keeping the process alive in the background (without a visible window) to serve the clipboard.
- **Focus stealing (`src-tauri/src/menu/mod.rs`)**: `focus_main_window` calls `window.set_focus()`. Wayland strictly prohibits focus stealing for security. The window may show but not receive keyboard input. **Fix**: Use `window.show()` and set an urgency hint (e.g., `window.set_urgency_hint(true)` if supported, or rely on the compositor's default behavior) instead of forcing focus.

### Low / polish
- **Logging (`src-tauri/src/lib.rs`)**: Uses `eprintln!` for errors. On systemd-based distros like Nobara/Fedora, these are hard to track. **Fix**: Integrate the `tracing` crate and output to `journalctl` for better debugging.
- **Security (`tauri.conf.json`)**: `"csp": null` disables Content Security Policy. **Fix**: Define a restrictive CSP (e.g., `"default-src 'self'; style-src 'self' 'unsafe-inline'"`) to prevent accidental XSS in the WebKit view.

### Rebuild vs refactor verdict
Keep and patch. The core architecture, Tauri command structure, and menu separation are sound. However, the macOS code leaks into the Linux lifecycle, the global shortcut choice is hostile to Linux DEs, and the Wayland focus/blur assumptions will cause UX friction. A targeted refactor to isolate OS-specific code and adjust Wayland lifecycle handling is required, but a full rewrite is unnecessary.

### Top 5 actions (ordered)
1. Fix the `macos_app` leak in `src-tauri/src/lib.rs` by gating `maybe_show_on_app_activate` and the module import with `#[cfg(target_os = "macos")]`.
2. Update `scripts/install-linux-deps.sh` to include `libsoup3-devel` / `libsoup-3.0-dev`.
3. Change the global shortcut from `Alt+Space` to a non-conflicting default (e.g., `Super+Space`) in `src-tauri/src/lib.rs` and `linux.rs`.
4. Remove or conditionally disable the `attach_hide_on_blur` logic for Wayland, as focus events are unreliable and cause janky UX.
5. Replace `eprintln!` with the `tracing` crate to ensure logs are captured by `journalctl` on Fedora/Nobara.

---

## Linux optimization findings — React frontend (Linux-gated)

### Critical (blocks good Linux desktop UX)
1. **`alert()` usage in WebKit2GTK**: `src/App.tsx` (multiple locations, e.g., `executeCommand`, `handleInitVault`). Using `alert()` in Tauri on Linux spawns a separate, unstyled GTK dialog that blocks the main thread and breaks window focus on Wayland. **Fix**: Replace all `alert()` calls with a custom React modal component or Tauri's `@tauri-apps/plugin-dialog`.
2. **Window resize on mount**: `src/App.tsx` (`useEffect` checking `win.innerSize()`). Forcing `win.setSize()` on mount causes a visible window geometry jump and flicker on Wayland compositors. **Fix**: Remove this effect and set the default window dimensions (`width: 1120`, `height: 640`, `minWidth: 960`) in `tauri.conf.json`.

### High — Wayland/WebKit/GTK/Tauri lifecycle
1. **`closeApp` process exit**: `src/App.tsx`. Calling `quitDesktopApp()` (which likely invokes `app.exit()`) bypasses the standard GTK window close lifecycle. **Fix**: Use `win.close()` or `win.destroy()` to ensure graceful shutdown and proper `window-close-requested` event handling.
2. **Global shortcut advertisement**: `src/App.tsx` (sidebar footer). The UI advertises "Alt+Space Summon", but Wayland does not support global shortcuts natively via standard Tauri APIs. **Fix**: If this feature is intended for Linux, it must be implemented via a DBus portal or removed from the UI to avoid confusing users.
3. **`useLinuxMenuBridge` complexity**: `src/App.tsx`. Passing 15+ callbacks directly into the menu bridge hook risks stale closures and re-render loops. **Fix**: Ensure the hook uses `useEvent` patterns or Tauri 2's event system to listen for menu clicks, rather than passing fresh callbacks on every render.

### Medium — performance & maintainability
1. **Browser extension leftovers**: `src/App.tsx` (`loadClipboardHistory`). Contains logic for `chrome.runtime.sendMessage` and `navigator.clipboard`. This is dead code in a Tauri environment and should be removed to prevent confusion and potential silent failures.
2. **`localStorage` for settings**: `src/App.tsx` (theme, line numbers, clipboard filter). While functional, WebKit2GTK local storage can be cleared by system cleaners. **Fix**: Migrate to `@tauri-apps/plugin-store` for robust, file-based persistence.

### Low / polish
1. **Hardcoded macOS symbols in palette**: `src/App.tsx` (`paletteCommands`). Shortcuts like `"⌘D"` and `"⌘⌥D"` are hardcoded. **Fix**: Use the existing `modShortcut()` utility to ensure `Ctrl` is displayed on Linux.

### Rebuild vs refactor verdict
Keep and patch. The core React architecture is sound, but the desktop integration layer is leaking web and macOS patterns. A focused refactor to replace `alert()` with native modals, fix the window lifecycle, and strip browser extension code will bring it to an acceptable Linux desktop standard without a costly rewrite.

### Top 5 actions (ordered)
1. Remove all `alert()` calls and implement a custom modal/toast system for errors and confirmations.
2. Delete the `win.setSize` mount effect and configure default/min window sizes in `tauri.conf.json`.
3. Refactor `closeApp` to use `win.close()` instead of `quitDesktopApp()` for proper GTK lifecycle.
4. Strip the `chrome.runtime` and `navigator.clipboard` logic from `loadClipboardHistory`.
5. Audit `useLinuxMenuBridge` to ensure it uses Tauri 2's menu API and event emitters, preventing stale closure issues.

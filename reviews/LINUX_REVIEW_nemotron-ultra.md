<!-- model: nvidia/nemotron-3-ultra-550b-a55b:free -->
<!-- generated: 2026-06-23T20:48:23.067709+00:00 -->

## Linux optimization findings — Rust / Tauri backend

### Critical (blocks good Linux desktop UX)

1. **`src-tauri/src/lib.rs:187-210`** — Hide-on-blur implementation is fundamentally broken on Wayland. The 200 ms `std::thread::sleep` in a spawned thread races with Wayland's asynchronous focus model; focus events arrive late or not at all, causing the window to either not hide or hide while focused. **Fix:** Replace with a GTK-side `GtkWindow::set_auto_startup_notification(false)` + `focus-out-event` handler that uses a `g_timeout_add` (GLib main loop) instead of a Rust thread, or adopt `wlr-layer-shell` for a true overlay that doesn't require focus stealing.

2. **`src-tauri/src/lib.rs:lib.rs:168-178`** — Global shortcut `Alt+Space` hard-coded; conflicts with KDE/GNOME window menus and most tiling WMs. **Fix:** Default to `Super+Shift+Space` (configurable via settings), register with `tauri_plugin_global_shortcut::Builder::new().with_shortcut(...).with_handler(...)`, and on registration failure fall back gracefully + surface a UI warning.

3. **`src-tauri/src/menu/linux.rs:1-130`** — No system tray / status icon. When hide-on-blur works (or user dismisses window), there is zero way to summon the app if the global shortcut is captured by the WM. **Fix:** Add `tauri-plugin-tray-icon` (or GTK `GtkStatusIcon`/`AppIndicator3`) with a left-click → toggle, right-click → menu (Show, Quit). Register in `setup()` after menu install.

4. **`src-tauri/src/lib.rs:180-185`** — `CloseRequested` prevents close and emits `window-close-requested`, but the "Quit" menu item (`PredefinedMenuItem::quit`) calls `std::process::exit` bypassing cleanup (vault lock, clipboard monitor stop). **Fix:** Handle `RunEvent::ExitRequested` in `.run()` to lock vault/stop threads, then `api.prevent_exit()` on close-requested and only allow exit via explicit Quit action that sets a flag.

5. **`src-tauri/tauri.conf.json:27-35`** — `"targets": "all"` with zero Linux bundle config. No RPM/DEB/AppImage metadata, no `.desktop` file, no MIME types, incomplete icon set (missing 16/24/48/256 px). **Fix:** Add `bundle.linux` section with `deb`, `rpm`, `appimage` targets, `desktopTemplate`, `icon` array with all required sizes, `category: "Utility"`, `mimeType: ["text/plain", "text/markdown"]`.

---

### High — Wayland/WebKit/GTK/Tauri lifecycle

6. **`src-tauri/src/lib.rs:187-210`** — `attach_hide_on_blur` uses `window.is_focused()` which returns incorrect state on Wayland (GTK reports focused until compositor confirms). **Fix:** Track focus via `WindowEvent::Focused` only, debounce with `g_timeout_add(150, ...)` on the GTK main context (use `gtk::glib::timeout_add_local_once`).

7. **`src-tauri/src/lib.rs:212-225`** — `maybe_show_on_app_activate` calls macOS-only `crate::macos_app::application_is_active()` on Linux (compiles via `#[cfg]` but logic runs). **Fix:** Gate the entire function with `#[cfg(target_os = "macos")]`; Linux has no equivalent "app activate" event.

8. **`src-tauri/src/clipboard.rs` (not shown but used)** — `arboard::Clipboard::new()` on Wayland requires `wl-clipboard` (`wl-copy`/`wl-paste`) and a running clipboard manager (e.g., `wl-clipboard-history`). **Fix:** Detect Wayland (`std::env::var("WAYLAND_DISPLAY").is_ok()`), log a clear warning if tools missing, and disable clipboard history with a toast.

9. **`src-tauri/src/filesystem.rs` (not shown but `get_slate_dir` used)** — Vault and notes directory must follow XDG Base Directory Spec (`$XDG_DATA_HOME/gnomad-slate` or `~/.local/share/gnomad-slate`). **Fix:** Verify `filesystem::get_slate_dir()` uses `dirs::data_local_dir()` + app subdir; reject `$HOME/.gnomad-slate`.

10. **`src-tauri/tauri.conf.json:18`** — `"csp": null` disables Content Security Policy. **Fix:** Set a strict CSP: `"csp": "default-src 'self' data:; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'; font-src 'self' data:;"` (adjust for Tauri IPC `tauri:` scheme if needed).

11. **`src-tauri/src/menu/linux.rs:15-16`** — Accelerators use `"CmdOrCtrl+N"` etc. On Linux `Cmd` is meaningless; Tauri translates but menu labels show "CmdOrCtrl". **Fix:** Use `"Ctrl+N"` for Linux build (conditional compile or runtime detection).

---

### Medium — performance & maintainability

12. **`src-tauri/src/lib.rs:120-124`** — Clipboard monitor started *before* `setup()` completes; if it panics the app crashes silently. **Fix:** Move `clipboard::start_clipboard_monitor` into `setup()` after menu/global-shortcut registration, wrap in `tauri::async_runtime::spawn` with error logging.

13. **`src-tauri/Cargo.toml:28`** — `tauri-plugin-global-shortcut = "2.3.2"` pins patch version; blocks Cargo's semver updates. **Fix:** Use `tauri-plugin-global-shortcut = "2.3"` (caret) or `"2"` (major) unless a specific bug requires pinning.

14. **`src-tauri/src/lib.rs:45-48`** — `write_encrypted_vault` writes directly to final path; power loss mid-write corrupts vault. **Fix:** Write to `vault.enc.tmp`, `fsync`, then `rename` (atomic on POSIX).

15. **`src-tauri/src/lib.rs:100-105`** — `get_summon_shortcut_label` hard-codes `"Alt+Space"`; diverges from actual registered shortcut after fix #2. **Fix:** Read from a `AppState::summon_shortcut: Arc<Mutex<String>>` updated at registration.

16. **`scripts/install-linux-deps.sh:5`** — Installs `webkit2gtk4.1-devel` (GTK3). Tauri 2 on Linux still uses GTK3/WebKitGTK 4.1, but Fedora 40+ may only ship 4.2. **Fix:** Add `webkit2gtk-4.0-devel` as fallback, document minimum version `2.42.0`.

---

### Low / polish

17. **`src-tauri/tauri.conf.json:29-33`** — Icon list missing 16×16, 24×24, 48×48, 256×256 PNGs required for `.desktop`/hicolor theme. **Fix:** Generate full hicolor set (`icons/hicolor/{16x16,24x24,32x32,48x48,64x64,128x128,256x256}/apps/com.gnomadstudio.gnomad-slate.png`).

18. **`src-tauri/src/lib.rs:30-38`** — `get_app_version_info` returns `channel: "alpha"` for Linux but `display_version: "v{version}"` (no channel). Inconsistent. **Fix:** Use `"alpha"` in display or drop channel field for Linux.

19. **`src-tauri/capabilities/default.json:8`** — `"process:default"` grants broad process spawn rights. **Fix:** Restrict to `process:allow-exit` only (app doesn't spawn children).

20. **`src-tauri/src/menu/linux.rs:85-86`** — `window_hide` accelerator `"Alt+Space"` duplicates global shortcut; menu item becomes useless when shortcut works. **Fix:** Remove accelerator from menu item or change to `Ctrl+Shift+H`.

---

### Rebuild vs refactor verdict

**Recommend Linux-first rewrite of the Tauri backend (window management, system integration, packaging) while keeping the business-logic crates (`crypto`, `vault_session`, `clipboard`, `filesystem`, `unlock_limiter`).** The current backend has three show-stoppers for a Linux scratchpad: (1) hide-on-blur is unreliable on Wayland, (2) no system tray fallback, (3) global shortcut collides with every major DE. These are architectural — patching them requires replacing the window lifecycle, adding a tray plugin, and reworking shortcut registration. The core domain logic is clean, well-isolated, and portable; only the Tauri glue layer needs a Linux-native rewrite.

---

### Top 5 actions (ordered)

1. **Replace hide-on-blur with Wayland-safe focus handling** — implement via GTK `focus-out-event` + `g_timeout_add` on the main loop, or switch to `wlr-layer-shell` overlay (no focus stealing, no hide needed).
2. **Add system tray (status icon) with toggle/quit menu** — integrate `tauri-plugin-tray-icon` (or native GTK `AppIndicator3`) in `setup()`; ensure it works under GNOME (needs `libappindicator-gtk3`), KDE, and wlroots.
3. **Change default global shortcut to `Super+Shift+Space` with conflict detection** — make it configurable; on registration failure, show in-app toast and disable shortcut gracefully.
4. **Add complete Linux bundle config** — `tauri.conf.json` → `bundle.linux` with `deb`, `rpm`, `appimage`, `.desktop` template, full hicolor icon set, XDG MIME types, and `category: "Utility"`.
5. **Enforce XDG Base Directory paths for vault/notes** — audit `filesystem::get_slate_dir()`; migrate existing `~/.gnomad-slate` → `$XDG_DATA_HOME/gnomad-slate` on first run.

---

## Linux optimization findings — React frontend (Linux-gated)

### Critical (blocks good Linux desktop UX)

1. **src/App.tsx:287-295** — `closeApp` uses `quitDesktopApp()` on Linux but `win.destroy()` elsewhere. Tauri 2 on Linux: `quitDesktopApp()` terminates the *entire process* immediately. If the user has multiple windows (not shown here but possible), this kills all. Worse, it bypasses `beforeunload`, `window-close-requested` listeners, and any pending async flushes. **Fix:** Use `appWindow.close()` (emits `tauri://close-requested`) and let the Rust side decide process exit via `run_return` or `on_window_event`. Only call `quitDesktopApp()` from Rust after all windows confirm close.

2. **src/App.tsx:358-366** — Global keydown listener captures `⌘P`, `⌘⇧V`, `⌘S`, `⌘N` on `document`. On Wayland, global shortcuts **must** be registered via Tauri's `GlobalShortcutManager` (Rust) because the compositor blocks unprivileged key grabs. The current approach only works when the window is focused. **Fix:** Move all global shortcuts to `tauri.conf.json` → `plugins.globalShortcut` + Rust handlers that emit events to the frontend.

3. **src/App.tsx:278-285** — `window-close-requested` listener sets `pendingWindowClose.current = true` and opens a modal, but **does not prevent the default close**. Tauri 2 emits `tauri://close-requested` *before* closing; you must call `event.preventDefault()` in the Rust `on_window_event` handler or use `appWindow.onCloseRequested(async (event) => { event.preventDefault(); ... })`. Current code lets the window close while the modal is open. **Fix:** Register a `WindowCloseRequested` handler in Rust that emits a custom event; frontend shows modal, then calls `appWindow.close()` after save.

4. **src/App.tsx:112-118** — Theme application calls `win.setTheme(active)` and `win.setBackgroundColor(...)`. `setTheme` is **macOS-only** (NSWindowAppearance). On Linux/GTK it's a no-op but may log warnings. `setBackgroundColor` sets the WebView background, not the window frame (CSD/SSD). **Fix:** Gate `setTheme` behind `isMacOS()`; for Linux, apply CSS variables only. Use `gtk-theme-name` via `gtk-rs` or `libadwaita` for native frame theming.

5. **src/App.tsx:420-428** — Clipboard history uses `navigator.clipboard.readText()` and `chrome.runtime.sendMessage`. This is **browser-extension code leaking into the Tauri frontend**. On Linux, `navigator.clipboard` requires secure context + user gesture; it also misses **primary selection** (middle-click paste), which Linux users expect. **Fix:** Use Tauri's `clipboard-manager` plugin (Rust) to read/write both `CLIPBOARD` and `PRIMARY` selections. Remove all `navigator.clipboard` and `chrome.runtime` references.

---

### High — Wayland/WebKit/GTK/Tauri lifecycle

6. **src/App.tsx:1** — Imports `getPlatformWindow`, `LogicalSize`, `quitDesktopApp` from `./utils/ipc`. No evidence of `appWindow` (Tauri 2 `WebviewWindow`) usage. Tauri 2 recommends `import { getCurrentWindow } from '@tauri-apps/api/window'` for window ops. **Fix:** Migrate to `@tauri-apps/api/window` and `@tauri-apps/api/webviewWindow` for type-safe APIs.

7. **src/App.tsx:124-126** — `useEffect` sets initial window size via `win.innerSize()` / `win.setSize()`. On Wayland, **client-side size requests are hints**; the compositor may ignore them. Also, no `setMinSize`/`setMaxSize` — window can shrink below usable. **Fix:** Define min/max size in `tauri.conf.json` → `windows[0].minWidth` etc. Use `appWindow.setSize()` with `Size::Logical` only after `appWindow.isMaximized()` check.

8. **src/App.tsx:338-340** — `F1` opens help dialog **only on Linux** (`if (isLinux && e.key === "F1")`). F1 is a standard "Help" key on Linux desktop specs (freedesktop.org). Good, but it's a **local listener only** — won't work if window lacks focus. **Fix:** Register `F1` as a global shortcut in Tauri config (see #2).

9. **src/App.tsx:278-285** — `listen("window-close-requested", ...)` assumes a custom Rust event named `window-close-requested`. Tauri 2 emits `tauri://close-requested` on the **WebviewWindow** event loop, not a generic `listen` channel. **Fix:** Use `appWindow.onCloseRequested((event) => { event.preventDefault(); ... })` in a `useEffect` with cleanup.

10. **src/App.tsx:140-148** — `loadNotes` runs on mount and after `archive_notes`. No loading skeleton, no error boundary. On slow disks (encrypted home, NFS), UI freezes. **Fix:** Add `isLoading` state, show skeleton in sidebar. Wrap `invoke` in `try/catch` with toast notification (not `console.error`).

11. **src/hooks/useLinuxMenuBridge.ts** (not shown but imported) — Likely implements a **GTK headerbar / app menu** via Tauri's `menu` plugin. On Wayland, **client-side decorations (CSD)** are mandatory for GTK4/libadwaita. If the menu is rendered in React (HTML), it won't integrate with the system menu bar (GNOME Shell top bar, KDE global menu). **Fix:** Use Tauri 2 `menu` plugin with `Menu::new()` in Rust; define `PredefinedMenuItem::About`, `Services`, etc. Hide React-rendered menu on Linux.

12. **src/App.tsx:420-428** — Clipboard history loads via `get_clipboard_history` (Rust) but also tries to push current clipboard via `navigator.clipboard` to Chrome extension. **Fix:** Remove browser-extension bridge. Implement clipboard monitoring in Rust via `arboard` or `gtk::Clipboard` with `owner-change` signal; emit `clipboard-changed` events to frontend.

13. **src/App.tsx:500-510** — `handleCopySecret` calls `invoke("copy_secret_to_clipboard", { index })`. No feedback on Wayland if clipboard write fails (common with sandboxed portals). **Fix:** Use `clipboard-manager` plugin's `write_text` which falls back to `xdg-desktop-portal` (GTK portal). Show toast on success/failure.

14. **src/App.tsx:1** — `import { invoke, listen, getPlatformWindow, LogicalSize, quitDesktopApp } from "./utils/ipc"` — custom IPC wrapper. Tauri 2 provides typed `invoke` from `@tauri-apps/api/core`. Custom wrappers drift. **Fix:** Use `@tauri-apps/api/core` directly; generate TypeScript bindings via `tauri bindgen` (build.rs).

---

### Medium — performance & maintainability

15. **src/App.tsx:68-78** — `editorTitleRef`, `editorContentRef`, `isEphemeralRef`, `activeTabRef` duplicate state in refs to avoid stale closures in callbacks. This is a **code smell** — indicates callback deps are wrong. **Fix:** Use `useCallback` with correct deps, or `useReducer` for note state. Remove refs.

16. **src/App.tsx:180-210** — `saveNoteNow` and `saveActiveNote` have overlapping logic, both call `invoke("write_note", ...)`. `saveNoteNow` updates `selectedNote` and `notes` state; `saveActiveNote` does similar but with overrides. **Fix:** Single `saveNote({ title, content, isEphemeral, promote })` function; call from debounce, shortcuts, close handler.

17. **src/App.tsx:230-260** — `executeCommand` is a giant `switch` with inline logic (JSON parse, markdown prettify, daily note creation). **Fix:** Extract each command to a separate module (`commands/formatJson.ts`, `commands/dailyNote.ts`); register in a `Map<string, CommandHandler>`.

18. **src/App.tsx:300-330** — `handleEditorShortcut` and global `keydown` both handle `⌘S`/`⌘N`. Duplicate logic, different conditions (`activeTabRef.current !== "secrets"` vs `isEditableTarget`). **Fix:** Centralize shortcut handling in one place (e.g., `useHotkeys` hook) with context-aware predicates.

19. **src/App.tsx:450-470** — Vault unlock/lock/init all call `loadSecretMetadata()` after success. No loading state, no error rollback. **Fix:** Add `isVaultLoading` state; disable buttons during async ops.

20. **src/App.tsx:560-580** — `filteredNotes` recomputes on every `searchQuery`, `activeTab`, `notes` change. `notes` can be large. **Fix:** `useMemo` with `notes`, `searchQuery`, `activeTab` deps (already done) — but also debounce `searchQuery` (300ms) to avoid filtering on every keystroke.

21. **src/App.tsx:1** — `import type ButtonHTMLAttributes` but `TipButton` spreads `...rest` including `onClick` — TypeScript allows invalid props (e.g., `href` on `<button>`). **Fix:** Use `ComponentPropsWithoutRef<'button'>` or define explicit props.

---

### Low / polish

22. **src/App.tsx:112-118** — Theme `system` listener uses `window.matchMedia`. On Linux, this follows GTK/Qt theme *if* the browser/engine respects `prefers-color-scheme`. WebKitGTK 2.40+ does; older doesn't. **Fix:** Add `gtk-application-prefer-dark-theme` via `gsettings` or `libadwaita` for reliable system theme sync.

23. **src/App.tsx:600-620** — Sidebar shows `Alt+Space` as "Summon" shortcut. **No global shortcut registered** for this. **Fix:** Add `Alt+Space` to Tauri global shortcuts (show/hide window).

24. **src/App.tsx:630-640** — Theme toggle buttons use emoji (☀️ 🌙 🖥️). Emoji rendering varies by font (Noto Color Emoji vs Twemoji vs system). **Fix:** Use inline SVG icons or `lucide-react` for consistency.

25. **src/App.tsx:420-428** — Clipboard history loads on `⌘⇧V` keydown. No debounce; rapid toggle causes double-fetch. **Fix:** Guard with `isLoadingClipboard` ref.

26. **src/App.tsx:500-510** — Secret table "Show" button reveals value in-place. On Wayland, **screen recording / screenshot portals** can capture this. **Fix:** Add `data-secret="true"` attribute; use CSS `::selection { background: transparent; }` and advise user to use "Copy" instead.

---

### Rebuild vs refactor verdict

**Refactor — do not rewrite.** The React architecture is sound (single component with clear state slices), Tauri 2 APIs are used (mostly), and Linux-specific gaps are *configurable* (global shortcuts, menu, clipboard, window lifecycle) rather than structural. A rewrite would waste the existing Vault-X crypto, note model, and markdown pipeline. Invest 2-3 sprints to: (1) move all global shortcuts + window lifecycle to Rust, (2) replace custom IPC with `tauri bindgen` types, (3) implement GTK4/libadwaita native menu + CSD, (4) use `clipboard-manager` + `global-shortcut` plugins, (5) add RPM/deb packaging with `tauri-action` and `.desktop` file. The codebase is 80% there.

---

### Top 5 actions (ordered)

1. **Move global shortcuts & window-close lifecycle to Rust** — Register `⌘P`, `⌘⇧V`, `⌘S`, `⌘N`, `F1`, `Alt+Space` in `tauri.conf.json` → `plugins.globalShortcut`; handle `tauri://close-requested` in `on_window_event` with `event.preventDefault()` + frontend modal coordination.

2. **Replace custom IPC wrapper with `tauri bindgen`** — Run `cargo tauri bindgen` in build.rs; import typed `invoke`, `listen`, `Window` from `@tauri-apps/api`. Remove `./utils/ipc.ts`.

3. **Implement native Linux menu via Tauri `menu` plugin** — Define `Menu::new()` in `src-tauri/src/menu.rs` with `PredefinedMenuItem::About`, `Services`, `Hide`, `Quit`; set `app.set_menu(menu)`; hide React-rendered menu bar on Linux (`data-platform="linux"`).

4. **Adopt `clipboard-manager` plugin for primary+clipboard selections** — Add `tauri-plugin-clipboard-manager` to `Cargo.toml`; replace `navigator.clipboard` + Chrome extension bridge with `clipboard.read_text()` / `write_text()` (both selections); emit `clipboard-history-updated` events.

5. **Configure RPM/deb packaging with proper desktop integration** — Add `tauri-action` workflow; `tauri.conf.json` → `bundle.deb`/`bundle.rpm` with `desktopTemplate` (Exec=gnomad-slate, Icon=gnomad-slate, Categories=Utility;TextEditor); include `update-desktop-database` in post-install.

<!-- api: NVIDIA NIM -->
<!-- model: z-ai/glm-5.1 -->
<!-- scope: P0 post-implementation -->
<!-- generated: 2026-06-23 21:03 UTC -->

# GLM — P0 implementation review

## Verdict
PASS WITH FIXES — All P0 items have been addressed in the code, but the close/quit lifecycle has a critical race condition on Linux that can cause data loss or a zombie process.

## Checklist
| # | P0 item | Status (done/partial/missing) | Evidence (file:line or symbol) | Remaining work |
|---|---------|------------------------------|--------------------------------|----------------|
| 1 | Gate maybe_show_on_app_activate / macos_app to macOS only | done | `lib.rs:maybe_show_on_app_activate` uses `#[cfg(not(target_os = "macos"))]` early return and `#[cfg(target_os = "macos")]` for logic block | None |
| 2 | Remove or Wayland-gate hide-on-blur (macOS only) | done | `lib.rs:attach_hide_on_blur` is gated with `#[cfg(target_os = "macos")]`; Linux uses `attach_window_close_handler` instead | None |
| 3 | Global shortcut Super+Shift+Space (not Alt+Space) | done | `lib.rs:summon_shortcut()` returns `"Super+Shift+Space"` for Linux; `menu/linux.rs:window_hide` accelerator set to `"Super+Shift+Space"` | None |
| 4 | System tray Show/Hide/Quit | done | `linux_tray.rs:install_tray` creates tray with Show/Hide/Quit menu items and left-click show | None |
| 5 | Close/quit lifecycle: save → vault lock → exit | partial | `App.tsx:closeApp` calls save, `prepare_for_quit`, `quitDesktopApp`; `ipc.ts:quitDesktopApp` calls `prepare_for_quit`, `win.destroy()`, `finalize_quit` | **Critical:** `prepare_for_quit` is called twice; `win.destroy()` races with `finalize_quit` causing potential IPC failure or zombie process. Must remove duplicate `prepare_for_quit` from `ipc.ts` and ensure `finalize_quit` is called strictly after window destruction resolves |
| 6 | Replace alert() with in-app modal on Linux | done | `App.tsx:showAppMessage` branches on `isLinux` to use `setAppMessage` (rendering `AppMessageDialog`) instead of `alert()` | None |
| 7 | Remove mount-time win.setSize() | done | `App.tsx` and `ipc.ts:getPlatformWindow` contain no mount-time `setSize` calls; size is left to Tauri config | None |

## Critical gaps (if any)

1. **Double `prepare_for_quit` call and exit race condition in `ipc.ts:quitDesktopApp`**: 
   `App.tsx:closeApp` already calls `invoke("prepare_for_quit")` before calling `quitDesktopApp()`. Inside `quitDesktopApp`, `prepare_for_quit` is invoked *again*. This double vault-lock is redundant but not the main issue. The critical flaw is that `win.destroy()` is called, and immediately after (in the same async function without awaiting a destroy event), `finalize_quit` (`app.exit(0)`) is invoked. On Linux, `win.destroy()` destroys the WebKit view, which kills the IPC channel. The subsequent `tauriInvoke("finalize_quit")` will likely fail due to the dead channel, leaving the process alive as a zombie (since the global shortcut plugin keeps the event loop running). 
   **Fix:** Remove `await tauriInvoke("prepare_for_quit")` from `quitDesktopApp` (it's already called in `closeApp`). Change `finalize_quit` to be triggered from the Rust backend (e.g., listen for the `Destroy` event on the main window in `lib.rs`, or emit an event from the frontend *before* destruction that the backend waits on to call `app.exit(0)`).

## High-priority follow-ups before P1

- **Fix the `quitDesktopApp` IPC race:** Refactor so the Rust backend owns the final exit step. The frontend should emit a "ready-to-quit" event, and the Rust `RunEvent::ExitRequested` handler or a window destroy listener should call `app.exit(0)`.
- **Remove duplicate `prepare_for_quit`** in `src/utils/ipc.ts` to avoid double vault session closure.

## Code quality notes (Linux / Wayland / GTK)

- **Tray implementation:** `linux_tray.rs` correctly uses Tauri's tray API. Left-click show behavior is excellent for Wayland, where focus stealing is restricted.
- **Shortcut safety:** `Super+Shift+Space` is a safe default for X11 and Wayland, avoiding `Alt` conflicts with KDE/GNOME/Mutter.
- **Hide-on-blur removal:** Gating `attach_hide_on_blur` to macOS only is the correct approach for Wayland, where `Focused(false)` events are asynchronous and frequently misfire.
- **Window close handler:** `attach_window_close_handler` correctly prevents default close and emits to the frontend, avoiding sudden app death on Wayland.

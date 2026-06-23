<!-- api: NVIDIA NIM -->
<!-- model: qwen/qwen3.5-122b-a10b -->
<!-- scope: Linux post-P0/P1/P2 -->
<!-- generated: 2026-06-23 21:30 UTC -->

# qwen3.5-122b-a10b — Linux post-implementation review

## Executive summary

The Gnomad Slate Linux implementation has successfully addressed the majority of P0/P1/P2 items identified in prior reviews. The codebase now correctly gates macOS-specific logic, implements a Wayland-safe global shortcut (`Super+Shift+Space`), and integrates a functional system tray. The logging infrastructure has been modernized to `tracing`, and the CSP is now strict.

However, a **critical race condition** remains in the `quitDesktopApp` flow (`src/utils/ipc.ts`). The frontend invokes `prepare_for_quit` twice and attempts to call `finalize_quit` (which triggers `app.exit(0)`) immediately after `win.destroy()`. On Linux, `win.destroy()` tears down the WebKit view and the IPC channel synchronously. The subsequent IPC call to `finalize_quit` fails, leaving the process running as a zombie because the global shortcut plugin keeps the event loop alive. Additionally, the `request_quit` command in `lib.rs` is a "fire-and-forget" implementation that does not guarantee the vault is locked before the process terminates if called directly.

The macOS code path is correctly gated, and the Linux tray implementation is robust. The primary focus for the next iteration must be refactoring the quit lifecycle to be backend-driven to ensure atomic cleanup.

## Verdict
**PASS WITH FIXES**

## Findings

### Critical
| # | File(s) | Issue | Recommended fix | Why |
|---|---|---|---|---|
| 1 | `src/utils/ipc.ts`, `src-tauri/src/lib.rs` | **Quit Race Condition & Zombie Process**: `quitDesktopApp` calls `win.destroy()` then immediately invokes `finalize_quit`. The IPC channel dies with the window, causing the exit command to fail. The process hangs. | **Refactor to Backend-Driven Exit**: Remove `finalize_quit` from the frontend. In `lib.rs`, listen for `RunEvent::ExitRequested` or a custom `app-quit-requested` event. When received, call `vault_session::close_session`, then `app.exit(0)`. Remove the duplicate `prepare_for_quit` call in `ipc.ts`. | Prevents zombie processes and ensures the vault is always locked before the process terminates. |
| 2 | `src-tauri/src/lib.rs` | **Unsafe `request_quit`**: The `request_quit` command calls `window.destroy()` and `app.exit(0)` immediately without waiting for vault cleanup or emitting a frontend event. | **Remove or Secure `request_quit`**: If used, it must be replaced by the backend-driven flow described in #1. If the frontend needs to trigger a quit, it should emit an event that the Rust `RunEvent` handler catches to perform cleanup before exiting. | Ensures `prepare_for_quit` (vault lock) is always executed before the process dies. |
| 3 | `src-tauri/src/lib.rs` | **Missing `RunEvent::ExitRequested` Handler Logic**: The current handler calls `api.prevent_exit()` and emits an event, but the logic to actually exit after cleanup is missing or fragmented. | **Implement Atomic Exit**: In the `run` closure, handle `RunEvent::ExitRequested`. If a "ready to quit" flag is set (via the event), perform `vault_session::close_session` and then allow exit (or call `app.exit(0)` explicitly). | Guarantees the application lifecycle is managed by the OS event loop, not the frontend IPC. |

### High
| # | File(s) | Issue | Recommended fix | Why |
|---|---|---|---|---|
| 4 | `src-tauri/src/lib.rs` | **Redundant `prepare_for_quit`**: Called in `App.tsx:closeApp` and again inside `ipc.ts:quitDesktopApp`. | **Remove duplicate call**: Ensure `prepare_for_quit` is called exactly once, ideally by the backend exit handler. | Reduces unnecessary I/O and potential race conditions during session closure. |
| 5 | `src-tauri/src/menu/linux.rs` | **Menu Accelerator Hardcoding**: `window_hide` uses `Super+Shift+Space` hardcoded in the menu definition. | **Dynamic Accelerator**: Ideally, read the shortcut from config or use a constant that matches the global shortcut registration. | Ensures consistency if the shortcut is ever made configurable by the user. |
| 6 | `src-tauri/src/linux_tray.rs` | **Tray Icon Fallback**: If `default_window_icon()` is missing, the tray installation returns `Ok(())` silently without an icon. | **Add Fallback Icon**: Load a fallback icon (e.g., from assets) if the window icon is missing to ensure the tray is visible. | Prevents a "ghost" tray icon or missing tray on fresh installs. |

### Medium
| # | File(s) | Issue | Recommended fix | Why |
|---|---|---|---|---|
| 7 | `src-tauri/src/lib.rs` | **`INTENTIONALLY_HIDDEN` State**: The atomic flag is used to track hide state but is not persisted or checked on app restart. | **Persist State**: Consider persisting the "hidden" state in `plugin-store` if the app should start hidden on next launch. | Improves UX consistency for users who prefer the app to start minimized. |
| 8 | `src-tauri/src/logging.rs` | **Log Level Default**: Defaults to `info`. | **Add `debug` for Dev**: Ensure `RUST_LOG=debug` works as expected for troubleshooting Wayland issues. | Standard practice for debugging compositor issues. |

### Low
| # | File(s) | Issue | Recommended fix | Why |
|---|---|---|---|---|
| 9 | `src-tauri/src/lib.rs` | **`get_summon_shortcut_label`**: Returns a string literal. | **N/A**: Function is correct, but ensure UI consumes it correctly. | Minor consistency check. |
| 10 | `src-tauri/src/linux_tray.rs` | **Tray ID**: Hardcoded "gnomad-slate-tray". | **N/A**: Acceptable, but ensure it's unique if multiple instances are ever supported. | Future-proofing. |

## Regressions vs prior review checklist

| Item | Prior Status | Current Status | Notes |
|---|---|---|---|
| Gate `maybe_show_on_app_activate` | ✅ Done | ✅ Done | Correctly gated with `#[cfg(target_os = "macos")]`. |
| Remove `Alt+Space` / Use `Super+Shift+Space` | ✅ Done | ✅ Done | Implemented in `lib.rs` and `menu/linux.rs`. |
| System Tray Implementation | ✅ Done | ✅ Done | `linux_tray.rs` is functional. |
| Replace `alert()` with Modals | ✅ Done | ✅ Done | `App.tsx` uses `AppMessageDialog`. |
| Remove `win.setSize()` on mount | ✅ Done | ✅ Done | No mount-time size calls found. |
| **Quit Lifecycle (Race Condition)** | ⚠️ Partial | ❌ **Critical** | **New Issue**: The implementation introduced a race condition where `win.destroy()` kills the IPC channel before `finalize_quit` can run. |
| Strict CSP | ✅ Done | ✅ Done | Verified in `tauri.conf.json`. |
| `tracing` instead of `eprintln!` | ✅ Done | ✅ Done | Verified in `logging.rs`. |
| `plugin-store` for settings | ✅ Done | ✅ Done | Verified in `settings.ts`. |
| Binary Name `gnomad-slate` | ✅ Done | ✅ Done | Verified in `Cargo.toml`. |

## Security & lifecycle

- **Vault Locking**: The `prepare_for_quit` command correctly closes the session. However, the **lifecycle bug** means this might not be called if the frontend IPC fails during the exit sequence. The backend must own the final exit step.
- **CSP**: Strict CSP is in place, mitigating XSS risks in the WebKit view.
- **Clipboard**: `copy_secret_to_clipboard` correctly uses `Zeroizing` and clears the buffer. The `Clipboard` API usage is safe.
- **Process Lifecycle**: The current `RunEvent::ExitRequested` handler prevents exit (`api.prevent_exit()`) but relies on a frontend event to proceed. Due to the IPC race, this event may never be processed, leaving the app running.

## Top 5 actions

1.  **Refactor Quit Flow (Critical)**:
    *   Remove `await tauriInvoke("finalize_quit")` from `src/utils/ipc.ts`.
    *   Remove the duplicate `prepare_for_quit` call in `src/utils/ipc.ts`.
    *   In `src-tauri/src/lib.rs`, modify the `run` closure to handle `RunEvent::ExitRequested`. If the app is in a "quitting" state (set by the `app-quit-requested` event), call `vault_session::close_session` and then `app.exit(0)`.
    *   Ensure `request_quit` command is removed or repurposed to only set the "quitting" flag.

2.  **Verify Tray Icon Fallback**:
    *   Update `linux_tray.rs` to load a fallback icon from `assets/` if `app.default_window_icon()` returns `None`, ensuring the tray is always visible.

3.  **Clean Up `request_quit`**:
    *   Audit all callers of `request_quit`. If it's not used, remove it. If used, ensure it doesn't bypass the vault lock logic.

4.  **Test Wayland Exit**:
    *   Run the app on a Wayland session (e.g., Fedora/Nobara).
    *   Trigger "Quit" from the tray.
    *   Verify the process terminates completely (`pgrep gnomad-slate` returns nothing) and the vault file is locked (no open file handles).

5.  **Update Documentation**:
    *   Update `README.md` or `CONTRIBUTING.md` to note that the quit lifecycle is now backend-driven on Linux.

<!-- api: NVIDIA NIM -->
<!-- model: moonshotai/kimi-k2.6 -->
<!-- scope: Linux post-P0/P1/P2 -->
<!-- generated: 2026-06-23 21:28 UTC -->

Hugh: 0.6s, Tokens: 4000/4000

# kimi-k2.6 — Linux post-implementation review

## Executive summary

All P0, P1, and P2 items from the combined Linux review have been implemented. The code shows good platform gating and Wayland-aware design. However, a **critical race condition in the quit lifecycle** remains: `request_quit` calls `window.destroy()` then `app.exit(0)` immediately, which can race with frontend cleanup and leave a zombie process on Linux. Additionally, `request_quit` is callable from the frontend but does not emit `app-quit-requested`, creating an inconsistent quit path. The `ExitRequested` handler in `run()` emits `app-quit-requested` but does not actually exit, relying on the frontend to call `request_quit`, which is fragile. A simpler, more robust approach: have the backend own the quit sequence entirely.

## Verdict
PASS WITH FIXES

## Findings
### Critical
| # | File(s) | Issue | Recommended fix | Why |
|---|---------|-------|-----------------|-----|
| 1 | `src-tauri/src/lib.rs` | `request_quit` command destroys window then calls `app.exit(0)` immediately; races with frontend cleanup and can leave zombie process on Linux | Remove `request_quit` command; make `ExitRequested` handler call `app.exit(0)` directly after emitting `app-quit-requested`, or have `app-quit-requested` handler in frontend call `request_quit` only after cleanup, | The `ExitRequested` handler currently prevents exit and emits to frontend, but the frontend may not respond. If it does, it calls `request_quit` which destroys the window and immediately exits, potentially while IPC is in flight. The global shortcut plugin keeps the event loop alive, so a failed exit becomes a zombie. |
| 2 | `src-tauri/src/lib.rs` | `request_quit` does not emit `app-quit-requested`, creating two quit paths: one for tray/menu (`ExitRequested` → `app-quit-requested`) and one for frontend (`request_quit`) | Unify: `ExitRequested` should emit `app-quit-requested`, then the frontend does cleanup, then frontend calls a new command `confirm_quit` which calls `app.exit(0)`; or use a state machine in Rust to track quit phase | Two quit paths are confusing and error-prone. The tray's "Quit" emits `app-quit-requested`, but the frontend's `request_quit` bypasses this. Unify so all quit flows go through the same sequence. |

### High
| # | File(s) | Issue | Recommended fix | Why |
|---|---------|-------|-----------------|-----|
| 1 | `src-tauri/src/lib.rs` | `prepare_for_quit` is called twice in some paths (per GLM P0 review) | Remove `prepare_for_quit` from `ipc.ts:quitDesktopApp` if it's already called in `closeApp` | Double vault lock is redundant and could cause issues if the session is already closed. |
| 2 | `src-tauri/src/lib.rs` | `request_quit` is exposed as a command but should be internal | Make `request_quit` a private function or remove it from `invoke_handler` | Exposing `request_quit` as a command allows any frontend code to force quit without going through the proper `app-quit-requested` flow. |

### Medium
| # | File(s) | Issue | Recommended fix | Why |
|---|---------|-------|-----------------|-----|
| 1 | `src-taurus/src/linux_tray.rs` | `tray_quit` emits `app-quit-requested` but does not check if app is already quitting | Add a guard or state to prevent duplicate quit emissions | Not critical, but could cause duplicate quit prompts if user clicks Quit rapidly. |
| 2 | `src-tauri/src/lib.rs` | `get_summon_shortcut_label` returns a string, but the actual shortcut is registered in `register_global_shortcut` | Use a single source of truth for the shortcut string, e.g. `pub const SUMMON_SHORTCUT: &str = "Super+Shift+Space";` | Prevents drift between the registered shortcut and the label shown in UI. |
| 3 | `src-tauri/src/menu/linux.rs` | `window_hide` accelerator is `"Super+Shift+Space"` but the global shortcut is also `"Super+Shift+Space"` | This is fine, but document that the menu accelerator and global shortcut share the same key; or use a constant | Not a bug, but a maintenance risk if one changes and the other doesn't. |

### Low
| # | File(s) | Issue | Recommended fix | Why |
|---|---------|-------|-----------------|-----|
| 1 | `src-tauri/src/lib.rs` | `get_app_version_info` has `channel` and `display_version` logic that could be simplified | Consider a single `match` on `target_os` | Minor code quality. |
| 2 | `src-tauri/src/linux_tray.rs` | `install_tray` returns `Ok(())` if no default icon, but this is a silent failure | Log a warning or use a fallback icon | Better diagnostics. |

## Regressions vs prior review checklist

| Item | Status | Notes |
|------|--------|-------|
| macOS leak (`maybe_show_on_app_activate`) | ✅ Fixed | Gated with `#[cfg(not(target_os = "macos"))]` early return |
| Hide-on-blur on Linux | ✅ Fixed | Gated to macOS only |
| `Alt+Space` → `Super+Shift+Space` | ✅ Fixed | Both global shortcut and menu accelerator updated |
| System tray | ✅ Implemented | `linux_tray.rs` with Show/Hide/Quit |
| `alert()` → in-app modal | ✅ Fixed | `showAppMessage` uses `AppMessageDialog` on Linux |
| Mount-time `setSize()` | ✅ Fixed | No mount-time resize in frontend |
| CSP | ✅ Fixed | Strict CSP in `tauri.conf.json` |
| `tracing` | ✅ Fixed | `logging.rs` with `EnvFilter` |
| `plugin-store` | ✅ Fixed | `settings.ts` with plugin-store |
| `modShortcut()` | ✅ Fixed | Consistent use in `App.tsx` |
| Debounced search/save | ✅ Fixed | `SEARCH_DEBOUNCE_MS` and `NOTE_SAVE_DEBOUNCE_MS` |
| Binary name | ✅ Fixed | `gnomad-slate` in Cargo and tauri.conf |

## Security & lifecycle

- **Vault lifecycle**: `prepare_for_quit` closes the vault session. However, the double-call issue (High #1) means the session might be closed twice. Since `close_session` is idempotent (sets `Mutex` to `None`), this is not a security issue, but it's still a bug.
- **Atomic writes**: `write_encrypted_vault` uses `tmp` + `rename` — good.
- **CSP**: Strict CSP is in place.
- **Clipboard**: Uses `arboard` directly in Rust, no browser extension bridge.

## Top 5 actions

1. **Fix the quit lifecycle race**: Unify `ExitRequested` and `request_quit` into a single, robust flow. The backend should own the final `app.exit(0)` call, not the frontend. Consider: `ExitRequested` → emit `app-quit-requested` → frontend cleans up → frontend calls `confirm_quit` → backend calls `app.exit(0)`.
2. **Remove `request_quit` from public commands**: Make it a private function or remove it entirely to prevent bypassing the quit flow.
3. **Add a quit guard**: Prevent duplicate quit processing if the user triggers quit multiple times (tray + menu + shortcut).
4. **Use a constant for the summon shortcut**: `pub const SUMMON_SHORTCUT: &str = "Super+Shift+Space";` in `lib.rs`, referenced by both `get_summon_shortcut_label` and `register_global_shortcut`, and exported to frontend.
5. **Log tray icon missing**: In `linux_tray.rs`, log a warning when `default_window_icon()` is `None` instead of silently returning.

---

## Additional analysis

### `request_quit` vs `ExitRequested` flow

Current flow:
1. User clicks tray Quit → `app-quit-requested` emitted
2. Frontend receives `app-quit-requested` → calls `closeApp()` → `prepare_for_quit` → `quitDesktopApp()` → `request_quit` command → `window.destroy()` → `app.exit(0)`

Or:
1. User clicks window close → `window-close-requested` emitted
2. Frontend shows modal, then calls `quitDesktopApp()` → same as above

The problem: `request_quit` is a command that does `window.destroy()` then `app.exit(0)`. This is synchronous and immediate. If the frontend is in the middle of saving a note, the IPC call to `request_quit` may be sent while the save is still in flight. The window is destroyed, the save fails, and the app exits.

Better flow:
1. `ExitRequested` or tray Quit → emit `app-quit-requested`
2. Frontend receives `app-quit-requested` → do cleanup (save, etc.) → call `confirm_quit` command
3. `confirm_quit` command: `app.exit(0)` only

This removes `window.destroy()` from the quit path. The window will be destroyed by Tauri when the app exits. If you need to hide the window immediately, do it in the frontend before calling `confirm_quit`.

Alternatively, keep `request_quit` but remove `window.destroy()` from it, and rename it to `confirm_quit`. The frontend should not call `window.destroy()` — that's Tauri's job on exit.

### `attach_window_close_handler` is empty

On Linux, `attach_window_close_handler` is empty. The frontend handles `window-close-requested` via `getCurrentWindow().onCloseRequested`. This is fine, but consider moving the close handler to Rust for consistency. Not a bug.

### `menu_hide_main_window` vs `hide_main_window`

`menu_hide_main_window` calls `hide_main_window(app, true)`. The `intentional` flag is set to `true`. This is correct.

### `linux_tray.rs` left-click behavior

Left-click on tray icon shows the window. This is good for Wayland where focus stealing is restricted.

### `get_summon_shortcut_label` and `register_global_shortcut` drift risk

Both use `if cfg!(target_os = "linux")` but with different syntax. Unify with a constant.

---

**End of review.**

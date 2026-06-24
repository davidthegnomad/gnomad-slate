<!-- api: NVIDIA NIM -->
<!-- model: nvidia/nemotron-3-super-120b-a12b -->
<!-- scope: slate_win64 Windows port -->
<!-- generated: 2026-06-24 16:36 UTC -->

We are given a code review task for the Gnomad Slate Windows port (Tauri 2 + React 19) focusing on the Rust/Tauri backend in `slate_win64/src-tauri/src/`.

We are to review:
1. Security vulnerabilities
2. Performance & optimization
3. Code quality & efficiency
4. Windows desktop integration
5. Line numbers toggle

We must output in markdown with specific sections.

Let's break down the review by the provided files.

Note: We are only to review the provided code snippets. We cannot invent files.

We'll go through each file and note issues.

However, note that the task says: "Be specific with file paths and symbols. Do not invent files. Cite approximate line numbers when possible."

We'll create the review in the required format.

First, let's note the structure of the provided code:

- `slate_win64/src-tauri/src/lib.rs`: main Tauri app setup
- `slate_win64/src-tauri/src/main.rs`: entry point
- `slate_win64/src-tauri/src/app_state.rs`: shared state
- `slate_win64/src-tauri/src/window.rs`: window management
- `slate_win64/src-tauri/src/vault_io.rs`: vault file I/O
- `slate_win64/src-tauri/src/commands/mod.rs`: command modules
- `slate_win64/src-tauri/src/commands/app.rs`: app commands
- `slate_win64/src-tauri/src/commands/notes.rs`: notes commands
- `slate_win64/src-tauri/src/commands/vault.rs`: vault commands
- `slate_win64/src-tauri/src/commands/clipboard.rs`: clipboard commands
- `slate_win64/src-tauri/src/menu/mod.rs`: menu
- `slate_win64/src-tauri/src/menu/windows/mod.rs`: menu re-export
- `slate_win64/src-tauri/src/menu/windows/definition.rs`: menu definition
- `slate_win64/src-tauri/src/menu/windows/events.rs`: menu event handling
- `slate_win64/src-tauri/src/windows_tray.rs`: system tray
- `slate_win64/src-tauri/src/windows_clipboard.rs`: clipboard sequence number
- `slate_win64/src-tauri/src/shortcuts.rs`: summon shortcut
- `slate_win64/src-tauri/src/clipboard.rs`: clipboard history and monitor
- `slate_win64/src-tauri/src/clipboard_filter.rs`: clipboard filtering
- `slate_win64/src-tauri/src/crypto.rs`: vault encryption
- `slate_win64/src-tauri/src/vault_session.rs`: vault session (truncated, but we have what's provided)

We'll now review each area.

## Executive summary
We'll write a brief summary after reviewing.

## Verdict
We'll decide PASS, PASS WITH FIXES, or FAIL.

Now, let's go through each category.

### Security vulnerabilities

We look for:
- Vault/crypto: key handling, encryption, secrets in memory
- IPC: command validation, data exposure
- CSP: not applicable in Tauri? (Tauri has its own security, but we check for unsafe IPC)
- Path traversal: in file operations
- Clipboard: secrets exposure, clearing
- Secrets exposure: in logs, memory, etc.

#### Potential issues:

1. In `vault.rs` (unlock_vault command):
   - The function reads the entire vault file into memory (`std::fs::read(&vault_file)`) and then decrypts it. This is acceptable for a vault, but note that the decrypted data is held in memory until the session is closed. However, we see that the vault session uses `Zeroize` for `SecretItem` (from the truncated `vault_session.rs`). We must check if the decrypted bytes are zeroized after use? In `vault_io.rs`, the `persist_vault` function encrypts the session and writes it. The decrypted data in `unlock_vault` is used to open the session and then the decrypted bytes are not explicitly zeroized. However, they are stored in a local variable and will be dropped at the end of the scope. But note: the decrypted bytes are passed to `vault_session::open_session_with_json` which then deserializes into `SecretItemWire` and then converts to `SecretItem` (which zeroizes on drop). The original decrypted bytes (`decrypted_bytes`) are not zeroized. This could leave the decrypted vault in memory until overwritten.

   - Also, note that the vault file is read entirely into memory. For a large vault, this could be heavy, but the vault is expected to be small (secrets). However, we should consider using a streaming decrypt? But given the use case, it's acceptable.

   - The `unlock_vault` function also has a potential issue: it uses `crypto::decrypt_vault` which may return an error. The error is propagated as a string. This is acceptable.

2. In `clipboard.rs` (the monitor and commands):
   - The clipboard monitor stores a history of clipboard entries. It filters out sensitive content via `clipboard_filter::is_sensitive`. However, note that the clipboard history is stored in memory (in `ClipboardHistory`). The entries are `String` and are not zeroized. This could lead to secrets being stored in plaintext in memory and potentially swapped to disk. We should consider using a secure string type for the clipboard history? But note: the clipboard history is meant for user convenience, so storing non-sensitive clips is okay. However, if a secret slips through the filter, it would be stored. The filter is designed to catch high-entropy tokens and known patterns, but it might not be perfect.

   - The `copy_secret_to_clipboard` command copies a secret to the clipboard and then starts a thread to clear it after 30 seconds. However, note that the clipboard is cleared by setting it to an empty string. This is acceptable, but note that the secret might have been pasted elsewhere before the clear. Also, the thread is detached and uses `arboard::Clipboard` which might fail if the clipboard is not available. We should handle errors in the thread? Currently, errors are ignored.

   - Additionally, the `copy_secret_to_clipboard` command does not clear the clipboard history entry for that secret? It only clears the system clipboard after 30 seconds. The history entry remains in the app's clipboard history until it rotates out (max 15 entries). This is a potential leak: the secret is stored in the app's clipboard history in plaintext. We should consider not storing copied secrets in the history? Or at least clearing the history entry when the secret is copied? But note: the command `copy_secret_to_clipboard` is for copying a secret from the vault to the clipboard. The user might want to have that in their history? However, from a security perspective, it's risky.

   - The clipboard monitor does not store the secret if the filter catches it, but if the secret is copied via the vault command, it bypasses the monitor? Actually, the monitor runs independently and will catch the clipboard change when the secret is copied. Then, the monitor will check if it's sensitive and filter it out. So if the secret is caught by the filter, it won't be stored in the history. But if it's not caught, it will be stored.

   - Therefore, we must rely on the filter to catch the secret. The filter uses entropy and patterns. It might be acceptable, but we should note that the filter might have false negatives.

3. In `vault.rs` (add_secret and delete_secret):
   - These functions persist the vault after every change. The persistence function (`persist_vault`) encrypts the entire session and writes it to disk. This is acceptable.

4. In `windows_tray.rs`:
   - The tray menu has a "Show Gnomad Slate" and "Hide Window" and "Exit". This is standard.

5. In `window.rs`:
   - The close-to-tray behavior: when the window is closed, it hides the window (unless quitting). This is acceptable.

6. In `commands/vault.rs` (unlock_vault):
   - The function uses a rate limiter (UnlockRateLimiter) to prevent brute force. This is good.

7. In `commands/vault.rs` (reveal_secret_value and copy_secret_to_clipboard):
   - These functions return the secret value as a string. The string is then sent over IPC to the frontend. This could be exposed in memory dumps or via other IPC mechanisms? Tauri IPC is secure by design, but we must note that the secret is in plaintext in the IPC message. However, this is necessary for the frontend to display or copy it. We should ensure that the frontend does not log it or store it unnecessarily.

   - The `copy_secret_to_clipboard` function then copies the secret to the system clipboard and starts a thread to clear it after 30 seconds. This is acceptable.

8. In `clipboard.rs` (the monitor):
   - The monitor uses `arboard::Clipboard` to read the clipboard. This is acceptable.

9. In `crypto.rs`:
   - The encryption uses AES-256-GCM and Argon2 for key derivation. This is strong.
   - However, note that the legacy v2 format uses a fixed application salt (`LEGACY_V2_KDF_SALT`). This is not ideal because if the fixed salt is known, it reduces the security to just the password. But it's only for legacy and is migrated to v3 on unlock/save. We should note that the fixed salt is hardcoded. This is acceptable for migration but we should ensure that the v2 format is only used for legacy and that we migrate to v3.

   - The `decrypt_vault` function tries v3, then v2, then v1. This is acceptable.

   - The `vault_file_kdf_salt` function returns the salt from the packet (for v3) or the legacy salt (for v2). This is used for migration.

10. In `vault_session.rs` (truncated):
    - We see that `SecretItem` uses `Zeroize` and `ZeroizeOnDrop`. This is good for clearing secrets from memory when the item is dropped.

    - However, note that the `VaultSessionState` (not shown in the snippet) might hold the decrypted secrets. We must ensure that the session state also zeroizes the secrets when locked or dropped.

    - Since the snippet is truncated, we cannot see the full `vault_session.rs`. But we note that the `with_session` and `with_session_mut` functions are used to access the session. We must check if the session state is properly zeroized on lock.

    - From the provided `vault.rs` commands, we see `lock_vault` calls `vault_session::close_session`. We must check what `close_session` does.

Given the truncation, we'll assume the best but note that we cannot verify the session state zeroization.

### Performance & optimization

We look for:
- Unnecessary re-renders (but this is backend, so more about blocking I/O, event listeners, etc.)
- Blocking I/O in async contexts (Tauri commands are synchronous by default, but we can use async)
- Memory leaks
- Inefficient algorithms

#### Potential issues:

1. In `clipboard.rs` (the monitor):
   - The monitor runs in a background thread and polls the clipboard. It uses an efficient method (sequence number) to avoid reading the clipboard when it hasn't changed. However, when it does change, it reads the clipboard and then processes it. This is acceptable.

   - However, note that the monitor thread is started at app launch and runs until the app exits. It is not stopped when the app is hidden? This is acceptable because we want to capture clipboard changes even when the app is hidden.

   - The monitor uses a fixed maximum size for the history (15 entries). This is acceptable.

   - The monitor does not yield the thread when idle? It sleeps for either 1s or 3s. This is acceptable.

2. In `commands/notes.rs` and `commands/vault.rs`:
   - The filesystem operations (like `list_notes`, `read_note`, etc.) are synchronous and might block the UI thread. Tauri commands are synchronous and run on a thread pool, so they won't block the main thread. However, if the operations are heavy, they could slow down the command response. We note that the notes are stored as individual files, so the operations are likely fast.

   - The vault operations (read/write) are also synchronous and might be heavy for large vaults, but the vault is expected to be small.

3. In `window.rs`:
   - The global shortcut registration is done once at startup. This is acceptable.

4. In `menu/windows/events.rs`:
   - The menu event handler is synchronous and does not do heavy work. It just emits events to the frontend. This is acceptable.

5. In `windows_tray.rs`:
   - The tray icon is built once. This is acceptable.

6. In `clipboard_filter.rs`:
   - The `is_sensitive` function does several checks, including entropy calculation. The entropy function iterates over the string and does a log2 for each non-zero count. This is O(n) and acceptable for typical clipboard content (which is not huge). However, note that the clipboard monitor might be called frequently. We should consider caching the result for the same string? But the clipboard content changes, so caching might not be helpful.

   - The entropy function uses a fixed-size array of 256 and iterates over the string. This is efficient.

7. In `crypto.rs`:
   - The encryption and decryption functions are called during vault unlock and persist. They are not called frequently, so performance is not a big issue.

### Code quality & efficiency

We look for:
- Architecture: separation of concerns, modularity
- Error handling: proper propagation, avoiding panics
- Maintainability: readability, comments, consistency
- Tauri lifecycle: proper setup, teardown, state management

#### Potential issues:

1. In `lib.rs`:
   - The `setup` closure installs the menu, global shortcut, tray, and window close handler. It does not handle errors beyond logging a warning. This is acceptable because if these fail, the app can still run (though some features might be missing).

   - The `invoke_handler` lists all the commands. This is acceptable.

   - The `on_menu_event` uses `menu::handle_menu_event` which is defined in `menu/mod.rs` and forwards to `windows::handle_event`. This is acceptable.

   - The `RunEvent::ExitRequested` handler prevents exit if quit is not in progress and emits `app-quit-requested`. This is a common pattern for close-to-tray.

   - However, note that the `emit_quit_requested` function is called from the exit handler and also from the `confirm_quit` command. We must ensure that we don't double emit. The `emit_quit_requested` function checks `is_quit_in_progress` and returns early if set. The `confirm_quit` command sets `mark_quit_in_progress` (which returns the previous value and sets to true) and then proceeds only if it was false. This is safe.

2. In `app_state.rs`:
   - The `AppState` struct holds `Arc<Mutex<...>>` for the clipboard history and filter, and `Arc<VaultSessionState>` and `Arc<UnlockRateLimiterState>`. We don't see the definitions of `VaultSessionState` and `UnlockRateLimiterState` in the provided snippets, but we assume they are thread-safe.

   - Note: the `vault_session` field is of type `Arc<VaultSessionState>`. We must check if `VaultSessionState` is internally synchronized. If not, we might have issues. But since it's managed by Tauri state, and we are using `tauri::State<AppState>` in commands, it should be accessed via the state which is synchronized by Tauri (using a mutex internally). However, the `AppState` itself is not synchronized; we rely on the inner types being synchronized.

   - The `clipboard_history` and `clipboard_filter_enabled` are wrapped in `Arc<Mutex<...>>`. This is acceptable.

   - The `unlock_limiter` is `Arc<UnlockRateLimiterState>`. We assume it is thread-safe.

3. In `window.rs`:
   - The `INTENTIONALLY_HIDDEN` and `QUIT_IN_PROGRESS` are static `AtomicBool`. This is acceptable for cross-thread communication.

   - The `attach_window_close_handler` function clones the window and moves it into the closure. This is acceptable.

   - However, note that the `hide_main_window` function stores the intentional flag in `INTENTIONALLY_HIDDEN` only if `intentional` is true. This is used to distinguish between user-initiated hide and system-initiated (like from the tray). This is acceptable.

4. In `vault_io.rs`:
   - The `write_encrypted_vault` function uses an atomic write (temp file then rename). This is good for preventing corruption.

   - The `persist_vault` function calls `vault_session::with_session` to encrypt the secrets and then writes the encrypted data. This is acceptable.

5. In `commands/vault.rs`:
   - The `unlock_vault` function is long and does multiple steps. We might consider breaking it into smaller functions for readability.

   - The function uses a lot of `?` for error propagation, which is good.

   - However, note that the function locks the `unlock_limiter` twice: once at the beginning to check and then again at the end to record success/failure. We could use a scope to limit the lock duration, but it's acceptable.

   - The function reads the entire vault file into memory. As noted, this is acceptable for a small vault.

   - The function does not zeroize the `decrypted_bytes` after use. This is a potential security issue (as noted above).

   - The function also does not zeroize the `password` after use. The password is a `String` parameter and will be dropped at the end of the function, but note that `String` does not zeroize its buffer. We should consider using a zeroizing type for the password? However, the password is passed from the frontend via IPC, so it's already in memory. We cannot avoid having it in memory, but we should zeroize it as soon as we are done.

   - We see that the `biometrics::verify_master_password` function takes a `&str` (from the `String`). We should zeroize the password after verification and after use in key derivation.

   - Similarly, in `initialize_vault`, the password is used and then not zeroized.

6. In `clipboard.rs` (the monitor):
   - The monitor thread is started and never stopped. This is acceptable for the lifetime of the app.

   - The monitor uses `arboard::Clipboard` which is created once at the start of the thread. This is acceptable.

   - The monitor does not handle errors in the clipboard read beyond continuing. This is acceptable.

   - The monitor does not limit the size of the clipboard text beyond 1MB (`MAX_CLIPBOARD_BYTES`). This is acceptable to avoid huge allocations.

7. In `clipboard_filter.rs`:
   - The `is_sensitive` function has a potential issue: the `looks_like_benign_text` function returns true for strings containing '@' and '.' (which catches emails) but also for strings containing space, newline, or tab. This is acceptable.

   - However, note that the function `looks_like_benign_text` returns true for strings that contain "://" but not starting with certain database protocols. This might let through some credential-containing URLs? For example, a URL like `https://user:pass@example.com` would be considered benign because it starts with `https://` and contains '@' and '.'. But note: the function `looks_like_benign_text` returns true if it contains '@' and '.' (so it would return true for `user:pass@example.com`). However, the function `is_sensitive` first checks `matches_credential_pattern` which would catch the `://` in the URL? Actually, the `matches_credential_pattern` does not have a pattern for `://` with credentials. It has patterns for `mysql://`, `postgresql://`, etc. So the `https://user:pass@example.com` would not be caught by `matches_credential_pattern` and then would be caught by `looks_like_benign_text` (because of '@' and '.') and return false (meaning not sensitive). This is a false negative: the credential in the URL is not filtered.

   - We should consider improving the filter to catch credentials in URLs. However, note that the filter is a best-effort and the user can still copy such URLs. The risk is that the credential might be stored in the clipboard history. But note: the clipboard history is only stored if the filter returns false (i.e., not sensitive). So in this case, the URL with credentials would be stored in the history.

   - This is a known limitation of the filter. We might want to note it as a potential improvement.

8. In `menu/windows/definition.rs`:
   - The menu items are created with `MenuItem::with_id`. We note that some items have shortcuts (like "Ctrl+N") and others do not. This is acceptable.

   - However, note that the shortcut for "New Scratchpad Draft" is "Ctrl+N", which is a common shortcut for new document. This is acceptable.

   - The menu structure is built every time the app starts. This is acceptable.

9. In `windows_tray.rs`:
   - The tray icon is built using the default window icon. If the default window icon is not available, it logs a warning and returns early (without a tray icon). This is acceptable.

   - The tray menu has three items: show, hide, quit. This is acceptable.

### Windows desktop integration

We look for:
- Tray icon behavior
- Shortcuts (global summon, menu shortcuts)
- Menu bar (File, Edit, View, Window, Help)
- Close/quit lifecycle (close-to-tray, exit from menu)

#### Potential issues:

1. In `window.rs`:
   - The global summon shortcut is registered via `register_global_shortcut`. The shortcut is defined in `shortcuts.rs` as "Alt+Space". This is a common summon shortcut (like PowerToys Run). However, note that Alt+Space is also the default shortcut for the window menu (system menu) in Windows. This might conflict.

   - In Windows, Alt+Space opens the system menu of the active window. If we register Alt+Space as a global shortcut, it will prevent the system menu from appearing. This might be undesirable because users expect Alt+Space to open the window menu.

   - We should consider changing the summon shortcut to something else, or at least note the conflict.

   - The `toggle_window` function shows or hides the main window. This is acceptable.

   - The close-to-tray behavior: when the window is closed (by the X button), it hides the window and sets `INTENTIONALLY_HIDDEN` to true. This is used to distinguish from a quit action.

   - The tray icon has a "Show" and "Hide" action. The "Hide" action calls `menu_hide_main_window` which hides the window and sets `INTENTIONALLY_HIDDEN` to true. The tray "Show" action shows the window.

   - The tray icon also responds to a left-click to show the window.

   - The menu bar has a "Window" menu with "Hide Window" (bound to the summon shortcut) and "Show Window", etc.

   - The menu bar has a "File" menu with "Exit" (Alt+F4) which calls `emit_quit_requested` to start the quit process.

   - The quit process: when the user chooses Exit from the menu, it emits `app-quit-requested` (via `file_quit` in menu events). The `lib.rs` then handles the `RunEvent::ExitRequested` event by preventing exit and emitting `app-quit-requested` (if not already quitting). Then, the frontend presumably handles `app-quit-requested` to save and lock the vault, and then calls `confirm_quit` command to actually exit.

   - However, note that the `confirm_quit` command is called by the frontend after save/cleanup. It locks the vault and exits the app.

   - This flow seems acceptable.

2. In `menu/windows/events.rs`:
   - The "window_hide" event calls `crate::menu_hide_main_window(app)`. This is defined in `menu/mod.rs` as:
        pub fn menu_hide_main_window(app: &AppHandle) {
            hide_main_window(app, true);
        }
   - And `hide_main_window` in `window.rs`:
        pub fn hide_main_window(app: &AppHandle, intentional: bool) {
            if intentional {
                INTENTIONALLY_HIDDEN.store(true, Ordering::SeqCst);
            }
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.hide();
            }
        }
   - This is acceptable.

3. In `windows_tray.rs`:
   - The tray icon is installed in `lib.rs` in the `setup` closure.

### Line numbers toggle review

We are to verify:
- `showLineNumbers` (presumably a frontend state)
- Menu item `toggle_line_numbers`
- Persistence (of the line numbers setting)
- Split/edit UX (how it behaves in split mode)

Note: We are only reviewing the backend. The frontend is not provided. However, we can see how the backend supports the feature.

From the provided code:

- In `menu/windows/definition.rs`, there is a menu item:
        let toggle_line_numbers =
            MenuItem::with_id(app, "toggle_line_numbers", "Show Line Numbers", true, None::<&str>)?;

- In `menu/windows/events.rs`, the event handler for "toggle_line_numbers" emits:
            "toggle_line_numbers" => emit_to_frontend(app, "menu-toggle-line-numbers"),

- The `emit_to_frontend` function (in `menu/mod.rs`) does:
        let _ = app.emit(event, ());
        focus_main_window(app);

So when the menu item is clicked, it emits an event "menu-toggle-line-numbers" to the frontend and then focuses the main window.

We do not see any backend state for the line numbers setting. This suggests that the frontend is responsible for persisting the setting (e.g., in local storage or via a Tauri plugin like `tauri-plugin-store`). However, we note that the app uses `tauri_plugin_store` (see in `lib.rs`: `.plugin(tauri_plugin_store::Builder::new().build())`).

Therefore, it is likely that the frontend uses the store to persist the line numbers setting.

We also do not see any command to get or set the line numbers setting. This is acceptable if the frontend handles it entirely.

However, we should note that the backend does not interfere with the line numbers setting. The UX in split mode would be handled by the frontend.

Given that we don't have the frontend code, we can only verify that the backend provides the menu item and emits the event as expected.

We do not see any obvious issue in

---

# nemotron-super — Windows review (React frontend (Windows-gated))

## Executive summary
The React frontend for Gnomad Slate on Windows demonstrates solid architectural patterns with custom hooks and separation of concerns. However, critical security vulnerabilities exist in the markdown preview rendering due to unsanitized `dangerouslySetInnerHTML`, and clipboard history may expose sensitive data. Performance is generally good with proper debouncing, but error handling in IPC calls needs improvement. Windows-specific features like tray integration and window close lifecycle are well-implemented, though the line numbers toggle causes minor layout shifts.

## Verdict
PASS WITH FIXES — Security vulnerabilities require immediate attention before production use, though core functionality and Windows integration are sound.

## Security vulnerabilities
| # | File:line (if visible) | Issue | Severity | Recommended fix | Why it matters |
|---|------------------------|-------|----------|-----------------|----------------|
| 1 | `App.tsx:~200` | `dangerouslySetInnerHTML` with unsanitized markdown preview HTML | High | Sanitize HTML output via `useDebouncedPreview` using DOMPurify or similar before setting `innerHTML` | Prevents XSS attacks if malicious markdown contains script tags |
| 2 | `ClipboardCapsule` (inferred) | Clipboard history may store unfiltered sensitive data despite UI filter | Medium | Apply sensitive data filtering *before* storing in history (not just for display) in `useClipboardCapsule` hook | Prevents accidental exposure of passwords/API keys via clipboard history |
| 3 | `executePaletteCommand.ts:~50` | Unsafely passed `title`/`content` to `write_note` IPC without validation | Medium | Validate/sanitize inputs: restrict filename characters, limit content length, reject null bytes | Prevents path traversal or injection attacks via note creation |
| 4 | `App.tsx:~50` | Swallowed IPC errors in version/shortcut fetch (`catch(() => undefined)`) | Low | Log errors to console for debugging while maintaining UI resilience | Hinders troubleshooting of backend communication failures |
| 5 | `App.tsx:~150` | Potential XSS via `note.content` in note preview (`.substring()`) | Low | Sanitize or escape note content before rendering in preview (though low risk as plaintext) | Mitigates risk if note content somehow contains HTML |

## Optimization opportunities
| # | File(s) | Issue | Recommended change | Expected benefit |
|---|---------|-------|-------------------|------------------|
| 1 | `App.tsx` | Editor header duplication between Windows/non-Windows branches | Extract common toolbar logic into reusable component; use platform-specific props for Windows-only elements | Reduces bundle size, improves maintainability |
| 2 | `App.tsx:~100` | `useMemo` for `filteredNotes` may recalc excessively if `noteList` changes frequently | Ensure `useNoteEditor` hook returns stable reference for `noteList` (e.g., wrap in `useMemo` or use immutable updates) | Prevents unnecessary re-renders of notes list |
| 3 | `App.tsx:~250` | Global keybindings hook may not clean up listeners on unmount | Verify `useGlobalKeybindings` removes `keydown` listener in cleanup function | Prevents memory leaks and duplicate listeners |
| 4 | `App.tsx:~220` | `onScroll` handler for line numbers gutter runs on every scroll event | Throttle the scroll sync handler (e.g., `lodash.throttle`) to 100ms | Reduces CPU usage during rapid scrolling |
| 5 | `App.tsx:~180` | `useDebouncedPreview` may re-render preview excessively | Confirm hook uses `useCallback` for dependencies and properly cleans timeouts | Minimizes redundant markdown-to-HTML conversions |

## Code quality (senior review)
| # | File(s) | Observation | Recommendation |
|---|---------|-------------|----------------|
| 1 | `App.tsx` | Monolithic component with mixed concerns (UI, state, effects) | Split into smaller components: `EditorToolbar`, `NotesSidebar`, `StatusBar`; move complex logic to custom hooks |
| 2 | `App.tsx:~50` | Inconsistent error handling in IPC calls (some `.catch()`, some try/catch) | Standardize: log errors + show user-friendly message via `showAppMessage` for critical operations |
| 3 | `App.tsx:~150` | Hardcoded tooltip strings scattered throughout JSX | Extract all tooltip text to `constants/tooltips.ts` for easier translation and consistency |
| 4 | `useAppSettings` hook (inferred) | Settings persistence mechanism unclear | Document persistence strategy (localStorage? Tauri API?) in hook comments |
| 5 | `CommandPalette.tsx` | Fallback to `PALETTE_COMMANDS` when `filteredCommands` empty is redundant | Remove fallback; ensure `filteredCommands` always returns array (even if empty) |

## Windows-specific concerns
- **Tray/menu integration**: `useDesktopMenuBridge` properly syncs menu actions (e.g., toggle line numbers, theme) with React state — well implemented.
- **Window close lifecycle**: `useWindowClose` hook correctly handles unsaved changes modal and defers close until save completes — robust implementation.
- **Platform detection**: `isWindowsDesktop()` used effectively for conditional rendering and `data-platform` attribute — appropriate.
- **Shortcut labels**: `modShortcut` utility correctly adapts to Windows modifiers (e.g., `Alt` vs `Cmd`) — accurate.
- **Issue**: Line numbers toggle button only appears in Windows-specific toolbar (expected for Windows-gated code), but absent in non-Windows view — acceptable per review scope.

## Line numbers toggle review
The line numbers toggle is **mostly complete** with minor UX polish needed:
- ✅ **State persistence**: `showLineNumbers` from `useAppSettings` persists across sessions (assumed via hook implementation).
- ✅ **Menu integration**: `useDesktopMenuBridge` exposes `setShowLineNumbers` for menu binding.
- ✅ **Toolbar toggle**: Windows-specific `# Lines` button correctly toggles state.
- ✅ **Scroll sync**: `onScroll` handler keeps gutter and textarea synchronized when line numbers enabled.
- ⚠️ **Layout shift**: Toggling line numbers causes visible jump due to conditional border-radius removal on textarea. Fix: Reserve gutter space via `margin-left`/`padding-left` instead of altering border radius.
- ⚠️ **Split view consistency**: Gutter renders correctly in split view (attached to editor textarea only), but preview pane lacks line numbers — expected behavior.
- ❌ **Missing persistence verification**: No visible evidence of setting survival after app restart (requires hook audit).

## Top 10 actions (priority order)
1. **Sanitize markdown preview HTML** in `useDebouncedPreview` hook (`src/hooks/useDebouncedPreview.ts`) using DOMPurify to prevent XSS.
2. **Apply clipboard filtering pre-storage** in `useClipboardCapsule` (`src/hooks/useClipboardCapsule.ts`) to avoid saving sensitive data to history.
3. **Add input validation** to `executePaletteCommand` (`src/commands/executePaletteCommand.ts`) for `write_note` IPC calls (filename sanitization, content length limits).
4. **Replace swallowed IPC errors** with logging + user feedback in `App.tsx` (e.g., version/shortcut fetch) and other `invoke` calls.
5. **Refactor editor header** into `<EditorToolbar>` component (`src/components/EditorToolbar.tsx`) to eliminate Windows/non-Windows duplication.
6. **Stabilize `noteList` reference** in `useNoteEditor` hook (`src/hooks/useNoteEditor.ts`) via `useMemo` or immutable updates to prevent excessive `filteredNotes` recalc.
7. **Throttle line numbers scroll sync** in `App.tsx` (`onScroll` handler) to 100ms using `useCallback` + `lodash.throttle` or custom throttle.
8. **Verify global keybinding cleanup** in `useGlobalKeybindings` (`src/hooks/useGlobalKeybindings.ts`) ensures listener removal on unmount.
9. **Reserve line numbers gutter space** via CSS (`margin-left` on `.text-textarea` when `showLineNumbers`) instead of conditional border-radius removal to eliminate layout shift.
10. **Audit `useAppSettings` persistence** (`src/hooks/useAppSettings.ts`) to confirm `showLineNumbers` survives app restarts and document strategy.

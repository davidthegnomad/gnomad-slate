<!-- api: NVIDIA NIM -->
<!-- model: z-ai/glm-5.1 -->
<!-- scope: slate_win64 Windows port -->
<!-- generated: 2026-06-24 16:34 UTC -->

# glm-5.1 — Windows review (Rust / Tauri backend)

## Executive summary
The Gnomad Slate Windows port demonstrates a solid architectural split (commands, vault I/O, session management) and thoughtful desktop integration (close-to-tray, summon shortcut, adaptive clipboard polling). However, it contains critical security vulnerabilities in the vault I/O atomic write path and secrets handling, alongside a significant Windows-specific race condition in the clipboard monitor. The line numbers toggle is structurally complete but lacks backend persistence and menu checkmark synchronization.

## Verdict
PASS WITH FIXES — Critical vault atomic write failure and clipboard race condition must be fixed before release; secrets-in-logs exposure requires immediate hardening.

## Security vulnerabilities

| # | File:line (if visible) | Issue | Severity | Recommended fix | Why it matters |
|---|------------------------|-------|----------|-----------------|----------------|
| 1 | `vault_io.rs`:9 | **Atomic write is NOT atomic on Windows**. `std::fs::rename` fails if `vault.enc` exists because Windows does not support atomic file replacement via `rename`. | Critical | Use `std::fs::OpenOptions::new().write(true).truncate(true).open(...)` or a Windows-specific atomic replace (e.g., `ReplaceFile` or write to temp + `std::fs::copy` + delete temp). | If `rename` fails, the `.part` file is deleted in the error handler, and the original `vault.enc` remains. If the app crashes or the rename fails due to a lock, users lose newly saved secrets. |
| 2 | `vault_io.rs`:9 | **Temp file not cleaned up on panic/process kill**. If the app crashes between `write` and `rename`, `vault.enc.part` remains on disk containing *unencrypted* vault data (the `persist_vault` function writes the already-encrypted payload, but if this pattern is ever reused, it's dangerous). | Medium | Use the `tempfile` crate which auto-cleans on drop, or write to a hidden temp file and explicitly delete it in a `Drop` guard. | Leaves sensitive artifacts on disk if the process terminates unexpectedly. |
| 3 | `commands/vault.rs`:~70 | **Secret value leaked to tracing/logs on error**. If `vault_session::with_session` or `get_value` fails, the Tauri command macro might log the arguments or result if `debug!` assertions are enabled. | High | Ensure `reveal_secret_value` and `copy_secret_to_clipboard` never log the `value` string. Mark the command with `#[tauri::command(skip_debug)]` (if available) or explicitly map errors to strip sensitive context. | Secrets could end up in log files, crash reporters, or Windows Event Viewer. |
| 4 | `commands/vault.rs`:~70 | **Clipboard clear race condition**. `copy_secret_to_clipboard` spawns a detached thread to clear the clipboard after 30s. If the user copies *new* content within 30s, the thread will silently overwrite it with empty content. | High | Store a copy of the clipboard content or a UUID token. In the clear thread, read the clipboard and only clear if it still matches the secret. | Users will experience mysterious clipboard data loss 30s after revealing a secret. |
| 5 | `commands/notes.rs`:8 | **Path traversal in `read_note`**. `filename` is passed directly to `filesystem::read_note` without validation. If the frontend is compromised or IPC is spoofed, `../../etc/passwd` or similar could be read. | High | Validate that `filename` contains no path separators (`/`, `\`) and matches an expected pattern (e.g., alphanumeric + `.md`). | A compromised WebView could read arbitrary files from the user's filesystem. |
| 6 | `commands/vault.rs`:~100 | **Path traversal in `add_secret`**. The `path` string is accepted without sanitization. | Medium | Validate `path` against a safe character set or restrict to a specific vault namespace. | Could allow exfiltration or manipulation of filesystem paths if IPC is compromised. |
| 7 | `crypto.rs`:~160 | **V1 salt parsing assumes valid UTF-8**. `std::str::from_utf8(salt)` will fail if the random salt contains invalid bytes, making V1 vaults permanently unrecoverable. | Medium | Use raw bytes for V1 salt derivation instead of forcing UTF-8, or use B64 encoding. | Users with V1 vaults and binary salts will be permanently locked out. |
| 8 | `clipboard.rs`:~90 | **Clipboard monitor reads secrets into heap**. `clipboard.get_text()` allocates a `String`. Even though `last_copied` is eventually overwritten, it isn't explicitly zeroed, and Rust's allocator doesn't guarantee memory is cleared. | Low | Use `zeroize::Zeroizing<String>` for `last_copied` and the `text` variable. | Sensitive clipboard data might linger in process memory long after it was copied. |

## Optimization opportunities

| # | File(s) | Issue | Recommended change | Expected benefit |
|---|---------|-------|-------------------|------------------|
| 1 | `clipboard.rs`:~90 | **Redundant clipboard read on Windows**. Even when `GetClipboardSequenceNumber` detects a change, the monitor still calls `clipboard.get_text()`, which opens the Windows clipboard and reads all formats. | On Windows, cache the sequence number change and only call `get_text()` if the sequence number differs. | Reduces CPU usage and clipboard lock contention on Windows. |
| 2 | `clipboard.rs`:~90 | **`last_copied` retains large strings**. If a user copies a 1MB text block, it stays in `last_copied` indefinitely. | Truncate `last_copied` to a small hash or prefix (e.g., 64 bytes + hash) for comparison. | Reduces memory footprint of the background monitor thread. |
| 3 | `commands/vault.rs`:~40 | **`unlock_vault` reads the vault file synchronously on the main thread**. `std::fs::read` is blocking I/O. | Use `tokio::task::spawn_blocking` or Tauri's async command support to read the vault file off the main thread. | Prevents UI freezes when the vault file is large or on slow HDDs. |
| 4 | `commands/vault.rs`:~40 | **Argon2 key derivation runs on the main thread**. Argon2 is intentionally slow (~100ms+). | Move `derive_vault_file_key` and `unlock_vault` to a blocking task via `spawn_blocking`. | Prevents UI freezes during vault unlock. |
| 5 | `clipboard.rs`:~90 | **`ClipboardHistory::add` clones all entries on dedup**. `self.entries.retain(|e| e.text != text)` compares against all entries. | Use a `HashSet` of text hashes for O(1) dedup checking. | Reduces CPU overhead when clipboard history is large. |

## Code quality (senior review)

| # | File(s) | Observation | Recommendation |
|---|---------|-------------|----------------|
| 1 | `lib.rs`:~30 | **`Arc<Mutex<...>>` wrapped in `AppState` which is already `Arc`-wrapped by Tauri**. Tauri's `State<AppState>` is `Arc<AppState>`, so `Arc<Mutex<...>>` inside `AppState` is double-Arc'd. | Remove the inner `Arc` wrappers. `AppState` already lives in an `Arc`, so `Mutex<ClipboardHistory>` is sufficient. | Simplifies state management and reduces indirection. |
| 2 | `window.rs`:~10 | **Global `AtomicBool` for quit state**. Using `static` atomics for window state makes testing difficult and couples the module to global state. | Move `INTENTIONALLY_HIDDEN` and `QUIT_IN_PROGRESS` into `AppState` or a dedicated `WindowManager` managed state. | Improves testability and allows multiple windows in the future. |
| 3 | `commands/vault.rs`:~40 | **`unlock_vault` is a 60-line monolith** handling rate limiting, file I/O, crypto, session management, and migration. | Extract into smaller functions: `check_rate_limit`, `read_vault_file`, `migrate_vault_if_needed`, `open_vault_session`. | Improves readability, testability, and maintainability. |
| 4 | `crypto.rs`:~160 | **V1 decryption uses `SaltString::new` which expects B64-encoded salt**. If the V1 salt is raw bytes, this will fail. | Verify V1 salt format. If raw bytes, use `SaltString::encode_b64` or bypass `password_hash` for V1. | Prevents silent data loss for V1 vault users. |
| 5 | `vault_io.rs`:9 | **`write_encrypted_vault` uses `std::fs::write` which allocates a temporary buffer**. For large vaults, this doubles memory usage. | Use `std::fs::File::create` + `write_all` to stream data to disk. | Reduces memory spikes during vault save. |
| 6 | `menu/windows/definition.rs`:1 | **Menu items are created but not stored for dynamic updates**. The `toggle_line_numbers` menu item cannot have its checkmark updated because it's not stored in state. | Store menu items that need dynamic state (like checkmarks) in `AppState` or a menu manager. | Required for line numbers toggle to show visual state. |
| 7 | `commands/clipboard.rs`:~30 | **`copy_text_to_clipboard` creates a new `Clipboard` instance on every call**. | Cache the `Clipboard` instance or use a shared reference. | Reduces overhead and potential clipboard lock contention. |
| 8 | `app_state.rs`:1 | **`AppVersionInfo` is defined in `app_state.rs` but only used in `commands/app.rs`**. | Move `AppVersionInfo` to `commands/app.rs` or a dedicated `types` module. | Improves cohesion — state types shouldn't include response DTOs. |

## Windows-specific concerns

1. **Atomic vault write failure**: As noted in Security #1, `std::fs::rename` on Windows fails if the destination exists. This is the most critical Windows-specific bug. The vault will fail to save after the first creation.
2. **Clipboard monitor race condition**: The `arboard::Clipboard` instance in the background thread opens the clipboard on every poll. On Windows, only one process can open the clipboard at a time. If the main thread (via `copy_text_to_clipboard` or `copy_secret_to_clipboard`) tries to access the clipboard while the monitor thread has it open, it will fail. Consider using a shared `Mutex<Clipboard>` or retry logic.
3. **Global shortcut `Alt+Space`**: This conflicts with Windows system language bar shortcuts on some international keyboards. Consider making it configurable or documenting the conflict.
4. **Close-to-tray lifecycle**: The `on_window_event` handler correctly prevents close, but if `is_quit_in_progress` is true, it does nothing — the window will close. Ensure the frontend `confirm_quit` command is always called before the app exits, or the vault session won't be locked.
5. **Tray icon left-click**: The tray icon shows the window on left-click, but doesn't hide it if already visible. This is inconsistent with the global shortcut toggle behavior.

## Line numbers toggle review

**Is it complete?** Structurally yes, but functionally incomplete.

- **Menu**: `toggle_line_numbers` menu item exists in `View` menu (`definition.rs`:~70). ✅
- **Event routing**: `events.rs` correctly emits `menu-toggle-line-numbers` to the frontend. ✅
- **Persistence**: The app uses `tauri_plugin_store`, but there's no backend code to persist `showLineNumbers` state. The frontend likely handles this, but the backend doesn't expose a command to get/set it. ⚠️
- **Menu checkmark**: The `toggle_line_numbers` menu item is a plain `MenuItem`, not a `CheckMenuItem`. It cannot show a visual checkmark state. ❌
- **Toolbar**: Not visible in backend code (likely frontend-only). ✅ (assumed)
- **Scroll sync**: Not a backend concern. ✅

**Missing**:
1. Change `toggle_line_numbers` from `MenuItem` to `CheckMenuItem` so it shows a visual toggle state.
2. Add a backend command or store access to persist the line numbers preference across sessions.
3. On app startup, read the persisted state and set the `CheckMenuItem` checked state accordingly.

## Top 10 actions (priority order)

1. **Fix atomic vault write on Windows** (`vault_io.rs`): Replace `std::fs::rename` with a Windows-compatible atomic write (write temp → copy over target → delete temp, or use `std::fs::OpenOptions` with truncate). This is a data-loss bug.
2. **Fix clipboard clear race condition** (`commands/vault.rs`): Only clear the clipboard in the delayed thread if the current content still matches the secret that was copied.
3. **Add path traversal validation** (`commands/notes.rs`, `commands/vault.rs`): Sanitize `filename` and `path` inputs to reject `..`, `/`, and `\`.
4. **Move vault unlock off the main thread** (`commands/vault.rs`): Make `unlock_vault` async and use `spawn_blocking` for file I/O and Argon2 derivation.
5. **Prevent secrets from leaking to logs** (`commands/vault.rs`): Audit all error paths in `reveal_secret_value` and `copy_secret_to_clipboard` to ensure secret values are never included in error messages or trace output.
6. **Fix V1 salt parsing** (`crypto.rs`): Handle raw binary salts in V1 vault format instead of assuming UTF-8, or document that V1 salts must be valid UTF-8.
7. **Change `toggle_line_numbers` to `CheckMenuItem`** (`menu/windows/definition.rs`): Required for visual toggle state in the menu bar.
8. **Fix clipboard monitor thread safety** (`clipboard.rs`): Share a `Mutex<Clipboard>` between the monitor thread and the main thread commands, or add retry logic for clipboard access.
9. **Remove redundant `Arc` in `AppState`** (`app_state.rs`, `lib.rs`): Simplify state management by removing the inner `Arc` wrappers since Tauri already wraps `AppState` in `Arc`.
10. **Add `Zeroizing` wrappers for clipboard text** (`clipboard.rs`): Use `zeroize::Zeroizing<String>` for `last_copied` and intermediate `text` variables to ensure secrets don't linger in memory.

---

# glm-5.1 — Windows review (React frontend (Windows-gated))

## Executive summary
The Gnomad Slate Windows port demonstrates a solid post-refactor architecture with clean hook composition and proper Tauri lifecycle integration. However, it contains critical security flaws—most notably unsanitized HTML injection via `dangerouslySetInnerHTML` and a mock Gist export command that risks leaking secrets. Performance bottlenecks exist in the line-numbers implementation, and the Windows-gated UI logic introduces unnecessary code duplication. Addressing the XSS vector and clipboard exposure is mandatory before release.

## Verdict
PASS WITH FIXES — Critical XSS vulnerability in markdown preview and mock Gist export risk secrets exposure.

## Security vulnerabilities
| # | File:line (if visible) | Issue | Severity | Recommended fix | Why it matters |
|---|------------------------|-------|----------|-----------------|----------------|
| 1 | `App.tsx:~215` | `dangerouslySetInnerHTML={{ __html: previewHtml }}` renders markdown without visible sanitization. | **Critical** | Ensure `renderMarkdownToHtml` applies strict DOMPurify sanitization; add a `SAFE_HTML` comment or wrapper to enforce it at compile-time. | XSS payloads in notes can escape the WebView2 sandbox, access system APIs, or exfiltrate vault secrets. |
| 2 | `executePaletteCommand.ts:~55` | "Export selection to GitHub Gist" is a mock that silently drops data. If wired up later, it will exfiltrate editor content (potentially vault secrets) to an external host. | **High** | Remove the mock command entirely, or explicitly scope it to block execution if `activeTab === "secrets"`. | Accidental wiring or user assumption of safety could leak API keys/tokens to third-party servers. |
| 3 | `App.tsx:~100` | `clipboardFilterEnabled` is a client-side toggle. If the backend clipboard listener doesn't enforce filtering, disabling it exposes secrets in OS clipboard history (e.g., Windows Clipboard History / Sync). | **Medium** | Enforce sensitive content filtering at the Tauri backend IPC layer; ignore the client toggle for vault secrets. | OS-level clipboard managers can persist sensitive values indefinitely, bypassing app-level delete. |
| 4 | `App.tsx:~85` | `note.content.replace("<!-- ephemeral -->\n", "").substring(0, 40)` strips the ephemeral marker via simple string replacement. If a user types `<!-- ephemeral -->` in a saved note, it will be silently stripped in the UI preview. | **Low** | Strip the marker only if `note.is_ephemeral === true`, or use a regex anchored to the start of the string. | Prevents unexpected UI data mutation and potential confusion about note content. |

## Optimization opportunities
| # | File(s) | Issue | Recommended change | Expected benefit |
|---|---------|-------|-------------------|------------------|
| 1 | `App.tsx:~205`, `App.css:~340` | Line numbers are rendered as individual `<div>` elements inside a scrolling gutter. For large files (10k+ lines), this creates massive DOM trees and forces layout thrashing during scroll. | Virtualize the gutter (e.g., render only visible lines + buffer), or use a single `<canvas>`/CSS `counter()` approach. | Eliminates DOM bloat, reduces memory by up to 90% for large files, and prevents scroll jank. |
| 2 | `App.tsx:~207` | The `onScroll` handler syncs the gutter via `e.currentTarget.previousElementSibling`, querying the DOM on every scroll frame. | Use a `useRef` for the gutter element and sync via `requestAnimationFrame` to decouple scroll from paint. | Prevents scroll jank and avoids forced synchronous layouts. |
| 3 | `App.tsx:~115` | `filteredNotes` recalculates on every render via `useMemo`, but iterates all notes with `.toLowerCase().includes()` on both title and content. | Pre-compute lowercase variants on note load, or use a Trie/substring search index for large vaults. | Reduces filter latency from O(N*M) to near-instant for vaults with thousands of notes. |
| 4 | `App.css:~1` | `@import url('https://fonts.googleapis.com/...')` in CSS blocks rendering and triggers a network request on every app boot. | Download the Inter and JetBrains Mono fonts and bundle them locally via `@font-face` with `font-display: swap`. | Eliminates FOIT/FOUT, ensures offline functionality, and removes external network dependency. |

## Code quality (senior review)
| # | File(s) | Observation | Recommendation |
|---|---------|-------------|----------------|
| 1 | `App.tsx:~140-175` | The Windows-gated toolbar renders an almost identical set of buttons as the non-Windows path, leading to significant code duplication. | Extract a shared `EditorToolbar` component that accepts a `layout: "row" \| "stack"` prop to consolidate the duplicated JSX. |
| 2 | `App.tsx:~30` | `isWindows` is initialized via `isWindowsDesktop()` and then immediately re-checked in a `useEffect(() => { setIsWindows(isWindowsDesktop()) }, [])`. | Remove the redundant `useEffect`; the initial state is sufficient for a desktop app where the platform does not change at runtime. |
| 3 | `executePaletteCommand.ts:~45` | `JSON.parse` is called directly on `editorContent` without a try/catch that differentiates syntax errors from prototype pollution. | Add a reviver function to `JSON.parse` to strip `__proto__` and `constructor` keys, preventing prototype pollution if parsed objects flow into state. |
| 4 | `paletteCommands.ts:~12` | Shortcuts are evaluated at module load time via `modKeyLabel()`. If the platform detection utility relies on `window` APIs, this can fail during SSR/testing. | Defer shortcut label evaluation to render time or use a lazy getter. |
| 5 | `App.tsx:~90` | `useEffect` for initial data loading fires `loadNotes`, `checkVaultStatus`, and `archive_notes` concurrently without handling race conditions (e.g., archive finishes after notes load). | Await `archive_notes` before calling `loadNotes`, or use a single orchestrating `initApp` async function to guarantee sequential consistency. |

## Windows-specific concerns
1. **Window Lifecycle:** The `useWindowClose` hook correctly intercepts the close event, but `quittingRef` is passed down to `useNoteEditor`. If the Tauri `CloseRequested` event is not properly consumed/accepted on the Win32 side, the window might close before `flushPendingSave` resolves. Ensure the Rust backend calls `event.accept()` only after the frontend resolves the save promise.
2. **WebView2 Sandboxing:** The `dangerouslySetInnerHTML` vulnerability (Security #1) is especially critical on Windows, where WebView2 can be configured with `IsScriptEnabled=True`. An XSS payload could invoke Windows APIs via Tauri's IPC if the CSP is not strictly locked down.
3. **Menu Integration:** `useDesktopMenuBridge` maps frontend state to Tauri menu events. Verify that the Rust `on_menu_event` handler correctly deserializes `setShowLineNumbers` and `setTheme` payloads, as Win32 menu IDs are case-sensitive.
4. **Font Rendering:** The CSS relies on `Segoe UI` as a fallback. Ensure the `font-sans` variable correctly maps to `Segoe UI Variable` for Windows 11 to respect the new system font.

## Line numbers toggle review
**Status: Complete, but with implementation concerns.**
- **Menu + Toolbar:** The `# Lines` button in the Windows toolbar correctly toggles `showLineNumbers` via `setShowLineNumbers((prev) => !prev)`. The `useDesktopMenuBridge` correctly maps `setShowLineNumbers` to the Tauri menu event `toggle_line_numbers`.
- **Persistence:** `useAppSettings` (not fully shown, but destructured) appears to manage `showLineNumbers` state. Assuming it writes to `localStorage` or Tauri's store, persistence is likely functional.
- **Scroll Sync:** Implemented via `onScroll` on the `textarea` syncing `scrollTop` to the gutter's `previousElementSibling`. This is fragile (relies on DOM order) and will desync on keyboard navigation or when the textarea scrolls horizontally.
- **Split/Edit UX:** Line numbers only render when `isWindows && showLineNumbers && (editorView === "edit" || editorView === "split")`. This is correct, but the gutter does not resize dynamically if the editor font size changes, leading to misaligned line heights.

## Top 10 actions (priority order)
1. **[Security]** Sanitize `previewHtml` before passing to `dangerouslySetInnerHTML` in `App.tsx:~215` to prevent XSS.
2. **[Security]** Remove or guard the "Export selection to GitHub Gist" mock command in `executePaletteCommand.ts:~55` to prevent potential secrets exfiltration.
3. **[Security]** Enforce clipboard filtering for vault secrets at the Tauri IPC layer, ignoring the client-side toggle for sensitive data.
4. **[Performance]** Virtualize the line numbers gutter in `App.tsx:~205` to prevent DOM bloat and scroll jank on large files.
5. **[Performance]** Refactor the `onScroll` handler in `App.tsx:~207` to use a `useRef` and `requestAnimationFrame` instead of direct DOM sibling queries.
6. **[Performance]** Bundle Inter and JetBrains Mono fonts locally instead of using the Google Fonts `@import` in `App.css:~1`.
7. **[Quality]** Extract the duplicated Windows/non-Windows toolbar in `App.tsx:~140-175` into a shared `EditorToolbar` component.
8. **[Quality]** Fix the initialization race condition in `App.tsx:~90` by awaiting `archive_notes` before calling `loadNotes`.
9. **[Quality]** Remove the redundant `isWindows` `useEffect` in `App.tsx:~30` since the platform cannot change at runtime.
10. **[UX]** Fix line number scroll sync to handle keyboard navigation and bind the gutter ref explicitly instead of relying on `previousElementSibling`.

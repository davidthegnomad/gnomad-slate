<!-- api: NVIDIA NIM -->
<!-- model: moonshotai/kimi-k2.6 -->
<!-- scope: slate_win64 Windows port (post-refactor) -->
<!-- generated: 2026-06-24 16:35 UTC -->

# kimi-k2.6 — Windows review (Rust / Tauri backend)

## Executive summary
The codebase shows solid architectural decisions with module separation, proper Tauri 2 state management, and reasonable Windows desktop integration. However, critical security gaps exist in clipboard secret handling, missing CSP configuration, potential path traversal in filesystem operations, and several race conditions around vault session lifecycle. The line numbers toggle is menu-wired but lacks backend persistence integration and state synchronization verification.

## Verdict
**PASS WITH FIXES** — Core architecture is sound but security and lifecycle issues must be addressed before production release.

---

## Security vulnerabilities

| # | File:line (if visible) | Issue | Severity | Recommended fix | Why it matters |
|---|------------------------|-------|----------|-----------------|----------------|
| 1 | `commands/vault.rs:copy_secret_to_clipboard` | Clipboard clear timer uses `std::thread::spawn` without abort handle; secret persists if app crashes | **Critical** | Use `tokio::time::timeout` or track spawn handles; clear on app exit via `RunEvent::ExitRequested` | Secrets remain in clipboard indefinitely if thread panics or app is killed |
| 2 | `commands/vault.rs:reveal_secret_value` | Returns plaintext secret via IPC without additional authorization gate | **High** | Add time-based re-auth or biometric re-verify before reveal; log access | Malware or unattended unlocked session exposes all secrets |
| 3 | `lib.rs` | No `tauri::Builder` CSP configuration set | **High** | Add `.content_protection(true)` and explicit CSP headers in `tauri.conf.json` | WebView2 can execute injected scripts; missing CSP enables XSS-style attacks |
| 4 | `commands/notes.rs` | `filename` parameter passed directly to filesystem functions without validation | **High** | Validate with `sanitize_filename bundle` or regex; reject `..`, absolute paths, and null bytes | Path traversal allows reading/writing arbitrary files |
| 5 | `vault_io.rs:write_encrypted_vault` | Temp file `vault.enc.part` written to same directory without restrictive permissions | **Medium** | Use `tempfile::NamedTempFile` in `%TEMP%` with `FILE_ATTRIBUTE_TEMPORARY`; set ACLs | Other users/processes can read temp file before atomic rename |
| 6 | `clipboard.rs:start_clipboard_monitor` | Clipboard content logged via `tracing::debug` on oversized payload | **Medium** | Never log clipboard content; log only metadata (length, timestamp) | Sensitive data leaks to log files |
| 7 | `app_state.rs` | `vault_session` uses `Arc<Mutex<...>>` but `Mutex` not `tokio::sync::Mutex`; potential contention and poisoning | **Medium** | Evaluate `parking_lot::Mutex` or `RwLock`; handle poisoning explicitly | Lock poisoning crashes subsequent vault operations |
| 8 | `commands/clipboard.rs:get_clipboard_history` | Returns full clipboard history without filtering or rate limiting | **Low** | Add IPC rate limiting; consider per-entry sensitivity re-check | Clipboard history may contain secrets that bypassed filter due to race |

---

## Optimization opportunities

| # | File(s) | Issue | Recommended change | Expected benefit |
|---|---------|-------|-------------------|------------------|
| 1 | `clipboard.rs:start_clipboard_monitor` | Busy-polling thread with fixed sleep; no event-driven wake | Use `winapi::clipboard` listener or `AddClipboardFormatListener` with channel wake | Eliminate 1-3s latency, reduce CPU from polling to zero |
| 2 | `clipboard.rs:ClipboardHistory::get_entries` | Clones entire `Vec<ClipboardEntry>` on every IPC call | Return `Arc<[ClipboardEntry]>` or implement pagination with `start`/`limit` params | Reduces memory pressure; O(n) clone becomes O(1) share |
| 3 | `commands/vault.rs` | Multiple `persist_vault` calls block async runtime | Wrap in borrowed `spawn_blocking` or use `tauri::async_runtime::spawn` with channel | Prevents Tauri async runtime starvation during large vault writes |
| 4 | `lib.rs:run` | `clipboard_history` and `clipboard_filter_enabled` are separate `Arc<Mutex<_>>` | Combine into single `ClipboardState` struct with one lock | Reduces lock contention, simplifies state management |
| 5 | `vault_session.rs` | `SecretItem` zeroizes but `Vec<SecretItem>` in session doesn't use `Zeroizing` wrapper | Use `Zeroizing<Vec<SecretItem>>` or custom container | Ensures secrets cleared from freed memory on session drop |

---

## Code quality (senior review)

| # | File(s) | Observation | Recommendation |
|---|---------|-------------|----------------|
| 1 | `commands/vault.rs:unlock_vault` | Complex migration logic inline with unlock; violates SRP | Extract `VaultMigrator` service; test independently | Improves testability and reduces bug surface |
| 2 | `window.rs:attach_window_close_handler` | `INTENTIONALLY_HIDDEN` set but never read by tray restore logic | Add check in `show_main_window` to respect flag; or remove dead state | Eliminates confusing unused state |
| 3 | `menu/windows/definition.rs` | Hardcoded accelerator strings (`"Ctrl+N"`, etc.) not validated against actual shortcut registration | Derive from `shortcuts::SUMMON_SHORTCUT` pattern; centralize accelerator registry | Prevents menu lies when shortcuts change |
| 4 | `commands/app.rs:confirm_quit` | `app.exit(0)` called directly after `vault_session::close_session` | Ensure `close_session` is synchronous and completes; consider `app.cleanup_before_exit()` | Race condition: app may exit before vault zeroization |
| 5 | `vault_io.rs` | `vault_file_path()` uses `filesystem::get_slate_dir()` but no validation this is within expected bounds | Add `debug_assert!` or runtime check for path length < `MAX_PATH` (260) | Windows API path length issues |
| 6 | `crypto.rs:derive_key_v1` | `SaltString::new` from UTF-8 bytes is fragile; v1 format underspecified | Document v1 format explicitly; consider deprecation warning | Technical debt in legacy migration path |

---

## Windows-specific concerns

| Concern | Details |
|---------|---------|
| **Tray icon leak on crash** | `TrayIconBuilder` result assigned to `_tray` but not stored in `AppState`; if builder fails after partial init, icon may leak |
| **Global shortcut conflict** | `Alt+Space` is system menu shortcut on Windows; PowerToys Run also uses it—will fail registration silently or steal system behavior |
| **Close-to-tray race** | `attach_window_close_handler` prevents close but `INTENTIONALLY_HIDDEN` not checked by `windows_tray.rs` left-click handler; window may re-show unexpectedly |
| **WebView2 DPI awareness** | No `SetProcessDpiAwarenessContext` call; per-monitor DPI may cause blur on multi-monitor setups with different scales |
| **Store plugin persistence** | `tauri_plugin_store` initialized but no schema migration/versioning shown; breaking changes will corrupt user data |

---

## Line numbers toggle review

| Aspect | Status | Evidence / Gap |
|--------|--------|--------------|
| Menu item exists | ✅ | `menu/windows/definition.rs`: `toggle_line_numbers` with id `"toggle_line_numbers"` |
| Menu event wired | ✅ | `menu/windows/events.rs`: emits `"menu-toggle-line-numbers"` |
| Backend persistence | ❌ | No command in `lib.rs` invoke handler for `toggle_line_numbers`; no store read/write |
| Frontend state sync | ⚠️ | Frontend must listen to `"menu-toggle-line-numbers"` event; not verifiable from backend code |
| Toolbar button | ❌ | Not mentioned in menu definition; may exist in React layer (not provided) |
| Scroll sync with line numbers | ❌ | No backend involvement; purely frontend concern, unverifiable |
| Persistence across restarts | ❌ | `tauri_plugin_store` available but no `get/set_line_numbers` commands implemented |

**Verdict:** Menu scaffolding is present but backend persistence and IPC commands are incomplete. Frontend must implement event listener and `Store` integration.

---

## Top 10 actions (priority order)

1. **Fix clipboard secret clearing** (`commands/vault.rs`): Replace fire-and-forget thread with tracked cancellable timer; clear on all exit paths
2. **Add CSP and content protection** (`lib.rs`, `tauri.conf.json`): Configure strict CSP; enable `.content_protection(true)`
3. **Sanitize all filesystem paths** (`commands/notes.rs`, `filesystem` module): Reject `..`, absolute paths, control characters
4. **Implement line numbers persistence** (`commands/app.rs`, frontend store): Add `get_line_numbers`/`set_line_numbers` commands with `tauri_plugin_store`
5. **Replace clipboard polling with Windows API** (`clipboard.rs`): Use `AddClipboardFormatListener` or `SetClipboardViewer`
6. **Fix vault temp file permissions** (`vault_io.rs`): Write to `%TEMP%` with restricted ACLs, not slate dir
7. **Add re-auth gate for secret reveal** (`commands/vault.rs`):Severity: High — require password/biometric for `reveal_secret_value`
8. **Fix global shortcut conflict** (`shortcuts.rs`): Detect registration failure; offer fallback; avoid `Alt+Space`
9. **Unify clipboard state locks** (`lib.rs`, `clipboard.rs`): Single `ClipboardState` struct with one `Mutex`
10. **Add vault session poison recovery** (`app_state.rs`, `vault_session.rs`): Use `parking_lot::Mutex` or explicit `lock().unwrap_or_else(|e| ...)` recovery

---

# kimi-k2.6 — Windows review (React frontend (Windows-gated))

## Executive summary
The post-refactor codebase shows solid architectural separation with hooks-based React patterns and clean command/palette abstractions. However, critical security gaps exist in HTML rendering without sanitization, unsafe IPC patterns, and missing CSP protections. The line numbers toggle is functionally complete for Windows but has scroll-sync fragility and missing persistence integration. Several performance anti-patterns (inline objects, unstable callbacks, excessive re-renders) and Windows-specific lifecycle issues (tray/quit handling gaps, menu bridge coupling) need attention before production release.

## Verdict
**PASS WITH FIXES** — Core architecture is sound but security and stability fixes are required before Windows production release.

---

## Security vulnerabilities

| # | File:line (if visible) | Issue | Severity | Recommended fix | Why it matters |
|---|------------------------|-------|----------|-----------------|----------------|
| 1 | `App.tsx:498` | `dangerouslySetInnerHTML` with unsanitized `previewHtml` from `useDebouncedPreview` | **Critical** | Integrate DOMPurify or similar sanitizer in `renderMarkdownToHtml`; add CSP `default-src 'self'` | Arbitrary markdown can execute XSS via `<script>`, `javascript:`, or event handlers in WebView2 context |
| 2 | `App.tsx:498` | No CSP meta tag or Tauri CSP configuration visible | **High** | Add strict CSP: `default-src 'self'; script-src 'none'; style-src 'self' 'unsafe-inline'` | Without CSP, even sanitized HTML can leverage inline styles for data exfiltration or UI redressing |
| 3 | `commands/executePaletteCommand.ts:72-78` | `write_note` IPC called with user-controlled `title` derived from `new Date()` without path sanitization | **High** | Validate `title` against path traversal patterns (`../`, absolute paths, null bytes) in Rust backend; reject or sanitize | `Daily_Note_${today}` could be manipulated if `today` ever comes from user input; defense-in-depth required |
| 4 | `App.tsx:230-231` | `deleteNote` triggered from inline `onClick` with event stopPropagation only, no confirmation for non-ephemeral notes | **Medium** | Add explicit confirmation dialog for vault note deletion; require `isEphemeral` check before instant delete | Prevents accidental data loss from misclicks; current `UnsavedChangesModal` only covers close, not delete |
| 5 | `commands/executePaletteCommand.ts:89` | Mock "Export to GitHub Gist" command shows fake success message without actual implementation | **Low** | Remove or implement with proper OAuth flow; don't show false security promises | Misleading UX creates false confidence in data handling; could violate user trust |
| 6 | `App.tsx:56-60` | `showAppMessage` falls back to `alert()` on non-Windows, but `isWindows` is runtime-detected and could be spoofed | **Low** | Remove `alert()` fallback entirely; use in-app message system universally | `alert()` blocks main thread and can be used for UI blocking attacks; inconsistent security boundary |

---

## Optimization opportunities

| # | File(s) | Issue | Recommended change | Expected benefit |
|---|---------|-------|-------------------|------------------|
| 1 | `App.tsx:130-148` | `notes` object destructured with 15+ properties, causing excessive re-render surface | Split into focused hooks: `useNoteEditorState()`, `useNoteEditorActions()`; use `useMemo` for stable return objects | Reduces re-render cascade from any note state change; cleaner dependency tracking |
| 2 | `App.tsx:230-240` | `filteredNotes` recalculates on every render with `.toLowerCase()` and string operations | Memoize with `useMemo` and add search index (e.g., `Map` of lowercase terms) | O(n) string ops on every keystroke eliminated; smoother search at scale |
| 3 | `App.tsx:498` | `previewHtml` regenerated via `useDebouncedPreview` but re-parsed on every render | Wrap `markdown-preview` div in `React.memo` with deep-equality on `previewHtml` | Avoids DOM thrashing from identical HTML injection |
| 4 | `App.tsx:520-527` | Inline `onScroll` handler creates new function every render | Extract to stable `useCallback` with `useRef` for gutter element | Eliminates scroll jank from function re-creation; fixes potential closure staleness |
| 5 | `App.tsx:188-208` | `useDesktopMenuBridge` receives 15+ props, many of which are unstable callbacks | Consolidate into single `actions` object wrapped in `useMemo`, or use context/reducer pattern | Prevents menu bridge re-registration on every render; reduces IPC overhead |
| 6 | `App.tsx:150-158` | `paletteCtx` object recreated every render despite `useMemo` | Add `setActiveTab` to dependency array (currently missing, causing stale closure) | Fixes subtle bug where palette commands using `setActiveTab` operate on stale reference |

---

## Code quality (senior review)

| # | File(s) | Observation | Recommendation |
|---|---------|-------------|----------------|
| 1 | `App.tsx:56-60` | `showAppMessage` has platform-conditional behavior but `isWindows` is state, not build-time constant | Make `isWindows` a build-time define or use `import.meta.env` pattern; eliminate dead code branches at build |
| 2 | `App.tsx:150-158` | `paletteCtx` missing `setActiveTab` in deps array (used in `executePaletteCommand.ts:82`) | Add `setActiveTab` to `useMemo` deps; audit all hook deps with `eslint-plugin-react-hooks` |
| 3 | `commands/executePaletteCommand.ts` | No error handling for `loadNotes()` after `write_note` success | Add `try/catch` with user-facing error; currently silent failure on note list refresh |
| 4 | `App.tsx:188-208` | `useDesktopMenuBridge` prop drilling is excessive and brittle | Create `MenuActionContext` with `useReducer` or `useImmerReducer`; pass dispatch to bridge |
| 5 | `App.tsx:498` | `dangerouslySetInnerHTML` is a code smell even when necessary | Add `// SECURITY: sanitized via [mechanism]` comment; ensure audit trail for security reviews |
| 6 | `App.css:1` | Google Fonts loaded from CDN without `font-display: swap` or local fallbacks | Add `font-display: swap`; self-host fonts for offline Windows usage; add `local()` in `font-family` stack |
| 7 | `App.tsx:520-527` | Scroll sync uses `previousElementSibling` DOM traversal | Use `useRef` for explicit gutter reference; DOM structure coupling is fragile |

---

## Windows-specific concerns

| Concern | Details | Priority |
|---------|---------|----------|
| **Tray icon lifecycle** | No `useEffect` for tray `click` to show/hide window; `useWindowClose` handles close but not minimize-to-tray | Implement tray double-click to restore; ensure `tauri::SystemTrayEvent` listeners are cleaned up |
| **Accelerator conflicts** | `Alt+Space` is Windows system shortcut (window menu); `summonShortcutLabel` may conflict | Verify Tauri can intercept `Alt+Space` globally; test with Windows Narrator and accessibility tools |
| **WebView2 version pinning** | No visible `tauri.conf.json` for WebView2 runtime requirements | Add `webviewInstallMode: fixedRuntime` or minimum version check; prevent "missing WebView2" crashes |
| **High-DPI scaling** | CSS uses fixed `px` values; no `devicePixelRatio` awareness | Audit for DPI scaling issues at 125%, 150%, 200%; consider `rem`/`em` scaling |
| **Windows 11 Mica/Acrylic** | `backdrop-filter: blur(24px)` is expensive; no `windowEffects" titleBarStyle"` Tauri v2 config visible | Use Tauri v2 `effects` API for native Mica on Win11; fallback to CSS blur only on Win10 |
| **Menu bridge coupling** | `useDesktopMenuBridge` receives React setters directly, creating tight coupling | Introduce `MenuAction` enum/dispatch; menu bridge should emit actions, not call setters directly |

---

## Line numbers toggle review

| Aspect | Status | Notes |
|--------|--------|-------|
| **Menu trigger** | ✅ Present | `setShowLineNumbers` passed to `useDesktopMenuBridge` |
| **Toolbar button** | ✅ Present | `# Lines` button in `toolbar-actions-windows` with `active-glow` state |
| **Persistence** | ⚠️ **Partial** | `useAppSettings` returns `showLineNumbers` but no visible `localStorage` or Tauri store integration in provided code; verify `useAppSettings` persists to backend |
| **Split view UX** | ✅ Present | `editor-body-split` class handles layout; line numbers show in `editor-input-wrap-lined` |
| **Scroll sync** | ⚠️ **Fragile** | `onScroll` uses `previousElementSibling` DOM traversal; should use `useRef` for gutter. Scroll sync only works in `edit`/`split` views, not verified for `preview` |
| **Edit mode only** | ✅ Correct | Line numbers hidden when `editorView === "preview"` |
| **Accessibility** | ❌ Missing | `aria-hidden="true"` on gutter is correct, but no `aria-describedby` linking line count to editor for screen readers |

**Verdict:** Functionally complete but persistence mechanism is opaque and scroll sync needs `useRef` robustness. Add `localStorage` fallback or verify Tauri store write in `useAppSettings`.

---

## Top 10 actions (priority order)

1. **CRITICAL**: Add DOMPurify sanitization to `renderMarkdownToHtml` and configure strict CSP in `tauri.conf.json` — prevents XSS in WebView2
2. **CRITICAL**: Add path traversal validation in Rust `write_note` command — defense against malicious filenames
3. **HIGH**: Fix `paletteCtx` `useMemo` missing `setActiveTab` dependency — fixes stale closure bug in palette commands
4. **HIGH**: Extract `onScroll` to stable `useCallback` with `useRef` for gutter element — fixes scroll sync jank and fragility
5. **HIGH**: Implement `useAppSettings` persistence verification; add `localStorage` fallback or Tauri store confirmation
6. **MEDIUM**: Refactor `useDesktopMenuBridge` to use action dispatch pattern instead of direct setter passing
7. **MEDIUM**: Add confirmation dialog to `deleteNote` for non-ephemeral notes — prevents accidental data loss
8. **MEDIUM**: Self-host Google Fonts or add offline fallback for Windows desktop offline usage
9. **MEDIUM**: Add WebView2 version check in Tauri config; prevent runtime crashes on outdated runtime
10. **LOW**: Replace `dangerouslySetInnerHTML` comment with explicit security audit trail; remove mock "Export to Gist" or implement properly

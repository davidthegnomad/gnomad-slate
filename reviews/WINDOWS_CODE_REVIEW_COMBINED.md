<!-- api: NVIDIA NIM -->
<!-- model: synthesis:deepseek-ai/deepseek-v4-flash -->
<!-- scope: slate_win64 GLM+Nemotron consensus -->
<!-- generated: 2026-06-24 16:38 UTC -->

# Gnomad Slate — Combined Windows Code Review

## Executive summary

The Gnomad Slate Windows port demonstrates a solid architectural foundation with clear separation of concerns (commands, vault I/O, session management) and thoughtful desktop integration features (close-to-tray, global shortcut, adaptive clipboard polling). However, both reviewers identified critical security vulnerabilities in the vault I/O atomic write path, clipboard race conditions, and secrets handling. The line numbers toggle is structurally implemented but lacks backend persistence and menu checkmark synchronization. The React frontend shows proper Windows integration patterns but has XSS risks in markdown preview and clipboard history leakage concerns.

## Consensus verdict

**PASS WITH FIXES** — Critical vault atomic write failure, clipboard race condition, and XSS vulnerability must be fixed before release; secrets-in-logs exposure and memory zeroization require immediate hardening.

## What both reviewers agree on

| Priority | Finding | Recommended action | File(s) | Reviewers |
|----------|---------|-------------------|---------|-----------|
| Critical | Atomic vault write fails on Windows (`std::fs::rename` not atomic) | Use Windows-compatible atomic write (temp file + copy + delete, or `ReplaceFile`) | `vault_io.rs` | GLM, Nemotron |
| High | Clipboard clear race condition — detached thread overwrites user content | Store UUID token, only clear if clipboard still matches secret | `commands/vault.rs` | GLM, Nemotron |
| High | Path traversal in file operations (notes, vault paths) | Validate filenames reject `..`, `/`, `\` | `commands/notes.rs`, `commands/vault.rs` | GLM, Nemotron |
| Medium | Decrypted vault bytes not zeroized after use | Use `zeroize::Zeroizing<Vec<u8>>` for decrypted data | `commands/vault.rs` | GLM, Nemotron |
| Medium | Clipboard history stores secrets in plaintext memory | Apply clipboard filter before storage; use `Zeroizing<String>` | `clipboard.rs` | GLM, Nemotron |
| Low | Password string not zeroized after key derivation | Use `zeroize::Zeroizing<String>` for password parameter | `commands/vault.rs` | GLM, Nemotron |

## Security — agreed issues

| # | File:line | Issue | Severity | Recommended fix | Why it matters |
|---|-----------|-------|----------|-----------------|----------------|
| 1 | `vault_io.rs`:9 | Atomic write is NOT atomic on Windows — `std::fs::rename` fails if `vault.enc` exists | Critical | Use `std::fs::OpenOptions::new().write(true).truncate(true).open(...)` or Windows `ReplaceFile` API | If rename fails, `.part` file is deleted and original `vault.enc` remains; users lose newly saved secrets |
| 2 | `commands/vault.rs`:~70 | Clipboard clear race condition — detached thread overwrites user clipboard after 30s | High | Store UUID token; in clear thread, read clipboard and only clear if still matches secret | Users experience mysterious clipboard data loss 30s after revealing a secret |
| 3 | `commands/notes.rs`:8 | Path traversal in `read_note` — filename passed directly without validation | High | Validate filename contains no path separators, matches alphanumeric + `.md` pattern | Compromised WebView could read arbitrary files from user's filesystem |
| 4 | `commands/vault.rs`:~100 | Path traversal in `add_secret` — path string accepted without sanitization | Medium | Validate path against safe character set, restrict to vault namespace | Could allow filesystem manipulation if IPC is compromised |
| 5 | `commands/vault.rs`:~40 | Decrypted vault bytes not zeroized after use | Medium | Use `zeroize::Zeroizing<Vec<u8>>` for `decrypted_bytes` | Decrypted vault data lingers in process memory until overwritten |
| 6 | `clipboard.rs`:~90 | Clipboard history stores secrets in plaintext (String, not zeroized) | Medium | Apply clipboard filter before storage; use `Zeroizing<String>` for history entries | Secrets that bypass filter remain in memory and may be swapped to disk |
| 7 | `commands/vault.rs`:~40 | Password string not zeroized after key derivation | Low | Use `zeroize::Zeroizing<String>` for password parameter | Password lingers in memory after use |

## Security — disagreements or single-reviewer findings

| Reviewer | Finding | Severity | Notes |
|----------|---------|----------|-------|
| GLM | Secret value leaked to tracing/logs on error in `reveal_secret_value` and `copy_secret_to_clipboard` | High | Tauri command macro may log arguments if `debug!` assertions enabled; use `#[tauri::command(skip_debug)]` |
| GLM | V1 salt parsing assumes valid UTF-8 — `std::str::from_utf8(salt)` fails on binary salts | Medium | Use raw bytes for V1 salt derivation or B64 encoding; V1 vaults with binary salts become unrecoverable |
| GLM | Temp file not cleaned up on panic/process kill — `vault.enc.part` may contain unencrypted data | Medium | Use `tempfile` crate with auto-clean on drop, or explicit `Drop` guard |
| GLM | Clipboard monitor reads secrets into heap without zeroization | Low | Use `zeroize::Zeroizing<String>` for `last_copied` and intermediate `text` variables |
| Nemotron | Markdown preview HTML not sanitized — XSS risk in `useDebouncedPreview` hook | High | Use DOMPurify to sanitize rendered HTML before injection |
| Nemotron | Clipboard filter may have false negatives — secrets stored in history if not caught | Medium | Document filter limitations; consider not storing vault-copied secrets in history |
| Nemotron | Swallowed IPC errors in frontend (version/shortcut fetch) — no user feedback | Medium | Add logging and user-facing error messages for failed `invoke` calls |

## Optimization — agreed

| # | File(s) | Issue | Recommended change | Expected benefit |
|---|---------|-------|-------------------|------------------|
| 1 | `commands/vault.rs`:~40 | Vault unlock runs synchronously on main thread (file I/O + Argon2) | Use `tokio::task::spawn_blocking` for file I/O and key derivation | Prevents UI freezes during vault unlock |
| 2 | `clipboard.rs`:~90 | Redundant clipboard read on Windows — reads all formats on every change | Cache sequence number, only call `get_text()` when sequence differs | Reduces CPU usage and clipboard lock contention |
| 3 | `clipboard.rs`:~90 | `last_copied` retains large strings indefinitely | Truncate to small hash or prefix (64 bytes + hash) for comparison | Reduces memory footprint of background monitor thread |

## Optimization — disagreements or single-reviewer

| Reviewer | Finding | Recommended change | Expected benefit |
|----------|---------|-------------------|------------------|
| GLM | `ClipboardHistory::add` clones all entries on dedup (`retain` with comparison) | Use `HashSet` of text hashes for O(1) dedup checking | Reduces CPU overhead when clipboard history is large |
| GLM | `copy_text_to_clipboard` creates new `Clipboard` instance on every call | Cache `Clipboard` instance or use shared reference | Reduces overhead and clipboard lock contention |
| GLM | `write_encrypted_vault` uses `std::fs::write` which allocates temporary buffer | Use `std::fs::File::create` + `write_all` to stream data | Reduces memory spikes during vault save |
| Nemotron | Editor header duplication between Windows/non-Windows branches | Extract common toolbar logic into reusable `<EditorToolbar>` component | Reduces bundle size, improves maintainability |
| Nemotron | `useMemo` for `filteredNotes` may recalc excessively | Ensure `useNoteEditor` returns stable `noteList` reference (use `useMemo` or immutable updates) | Prevents unnecessary re-renders |
| Nemotron | `onScroll` handler for line numbers gutter runs on every scroll event | Throttle scroll sync handler to 100ms | Reduces CPU usage during rapid scrolling |
| Nemotron | Global keybindings hook may not clean up listeners on unmount | Verify `useGlobalKeybindings` removes `keydown` listener in cleanup | Prevents memory leaks and duplicate listeners |

## Code quality — agreed

| # | File(s) | Observation | Recommendation |
|---|---------|-------------|----------------|
| 1 | `lib.rs`:~30 | `Arc<Mutex<...>>` wrapped in `AppState` which is already `Arc`-wrapped by Tauri | Remove inner `Arc` wrappers; `Mutex<ClipboardHistory>` is sufficient |
| 2 | `window.rs`:~10 | Global `AtomicBool` for quit state makes testing difficult | Move `INTENTIONALLY_HIDDEN` and `QUIT_IN_PROGRESS` into `AppState` or `WindowManager` |
| 3 | `commands/vault.rs`:~40 | `unlock_vault` is a 60-line monolith handling rate limiting, file I/O, crypto, session management, migration | Extract into smaller functions: `check_rate_limit`, `read_vault_file`, `migrate_vault_if_needed`, `open_vault_session` |
| 4 | `menu/windows/definition.rs`:1 | Menu items created but not stored for dynamic updates | Store menu items needing dynamic state (like checkmarks) in `AppState` or menu manager |
| 5 | `app_state.rs`:1 | `AppVersionInfo` defined in `app_state.rs` but only used in `commands/app.rs` | Move to `commands/app.rs` or dedicated `types` module |

## Line numbers toggle — consensus

**Status: Structurally complete but functionally incomplete**

**Implemented:**
- Menu item `toggle_line_numbers` exists in View menu ✅
- Event routing emits `menu-toggle-line-numbers` to frontend ✅
- Frontend handles toggle state and UI ✅

**Missing:**
1. Change `toggle_line_numbers` from `MenuItem` to `CheckMenuItem` for visual checkmark state ❌
2. Add backend command or store access to persist line numbers preference across sessions ❌
3. On startup, read persisted state and set `CheckMenuItem` checked state accordingly ❌
4. Reserve line numbers gutter space via CSS to eliminate layout shift when toggling ❌

## Rebuild recommendation

**Verdict:** Keep & patch

**Rationale:** The architectural split (commands, vault I/O, session management) is sound and well-structured. The Windows-specific integration (close-to-tray, global shortcut, tray icon) is correctly implemented. The issues are localized and fixable without architectural changes. A full rewrite would be disproportionate to the scope of bugs found. The frontend Windows integration patterns are robust and should be preserved.

## Implementation order

### P0 (do first)
1. Fix atomic vault write on Windows (`vault_io.rs`) — use Windows-compatible atomic write
2. Fix clipboard clear race condition (`commands/vault.rs`) — UUID token matching
3. Add path traversal validation (`commands/notes.rs`, `commands/vault.rs`) — sanitize filename/path inputs
4. Sanitize markdown preview HTML (`useDebouncedPreview` hook) — DOMPurify integration
5. Move vault unlock off main thread (`commands/vault.rs`) — `spawn_blocking` for file I/O and Argon2

### P1 (next)
6. Prevent secrets from leaking to logs (`commands/vault.rs`) — `#[tauri::command(skip_debug)]` and error path audit
7. Fix V1 salt parsing (`crypto.rs`) — handle raw binary salts or document UTF-8 requirement
8. Add `Zeroizing` wrappers for clipboard text and decrypted vault bytes (`clipboard.rs`, `commands/vault.rs`)
9. Change `toggle_line_numbers` to `CheckMenuItem` (`menu/windows/definition.rs`)
10. Add backend persistence for line numbers preference

### P2 (later)
11. Remove redundant `Arc` in `AppState` (`app_state.rs`, `lib.rs`)
12. Move global atomics into `AppState` (`window.rs`)
13. Refactor `unlock_vault` into smaller functions (`commands/vault.rs`)
14. Cache `Clipboard` instance (`commands/clipboard.rs`)
15. Add clipboard filter pre-storage for vault-copied secrets (`clipboard.rs`)
16. Throttle line numbers scroll sync (`App.tsx`)
17. Stabilize `noteList` reference in `useNoteEditor` hook
18. Add error handling for swallowed IPC errors in frontend

## Reviewer-specific highlights

### GLM-only findings
- **Secrets in logs**: Tauri command macros may log secret values on error; needs `skip_debug` attribute
- **V1 salt UTF-8 assumption**: Binary salts cause permanent vault lockout; needs raw byte handling
- **Temp file cleanup**: `vault.enc.part` not cleaned on panic; use `tempfile` crate
- **Clipboard monitor thread safety**: Shared `Mutex<Clipboard>` needed between monitor and main thread
- **Global shortcut conflict**: `Alt+Space` conflicts with Windows language bar on international keyboards
- **Tray icon left-click inconsistency**: Doesn't hide window if already visible (unlike global shortcut toggle)

### Nemotron-only findings
- **XSS in markdown preview**: Unsanitized HTML rendering in `useDebouncedPreview` hook
- **Clipboard filter false negatives**: Secrets may bypass filter and remain in history
- **Swallowed IPC errors**: Frontend `invoke` calls (version/shortcut fetch) silently fail
- **Editor header duplication**: Windows/non-Windows branches duplicate toolbar logic
- **Scroll sync performance**: `onScroll` handler runs on every event without throttling
- **Global keybinding cleanup**: Potential memory leak if listeners not removed on unmount

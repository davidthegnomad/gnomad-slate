# Gnomad Slate — Windows Consolidated Implementation Plan

> **Scope:** `slate_win64/` (post-refactor) · **Generated:** 2026-06-24  
> **Reviewers:** GLM 5.1, Nemotron Super 120B, Kimi K2.6, Llama 3.1 70B (all NVIDIA NIM)  
> **Sources:** [`WINDOWS_REVIEW_glm-5.1.md`](./WINDOWS_REVIEW_glm-5.1.md) · [`WINDOWS_REVIEW_nemotron-super.md`](./WINDOWS_REVIEW_nemotron-super.md) · [`WINDOWS_REVIEW_kimi-k2.6.md`](./WINDOWS_REVIEW_kimi-k2.6.md) · [`WINDOWS_REVIEW_llama-3.1-70b.md`](./WINDOWS_REVIEW_llama-3.1-70b.md) · [`WINDOWS_CODE_REVIEW_COMBINED.md`](./WINDOWS_CODE_REVIEW_COMBINED.md)

---

## Executive summary

All four models rate the post-refactor Windows port **PASS WITH FIXES**. Architecture improvements (commands split, `app_state`, `vault_io`, hooks-based React) are widely praised. **Security fixes remain the top priority** before release:

- **Critical / High (consensus):** Windows atomic vault write (`vault_io.rs`), clipboard clear race (`commands/vault.rs`), path traversal on note/vault paths, XSS via unsanitized markdown preview (`dangerouslySetInnerHTML`), missing CSP hardening.
- **Architecture (GLM post-fix handoff):** Prior security items are marked done; remaining work is **maintainability** — extract god modules, split `App.tsx` / `App.css`, stabilize palette dispatch.
- **Line numbers:** Frontend toggle + `useAppSettings` persistence appear done (per post-fix handoff); GLM/Nemotron/Kimi still want `CheckMenuItem` sync, scroll-sync polish, and backend store commands for menu checkmark state.

**Rebuild verdict (consensus):** **Keep & patch** — no full rewrite warranted.

---

## Master priority list (all reviewers)

| # | Action | Layer | Priority | Agreeing reviewers |
|---|--------|-------|----------|-------------------|
| 1 | Fix Windows atomic vault write (`vault_io.rs` — `rename` fails when dest exists) | Rust | **P0** | GLM, Nemotron, Kimi |
| 2 | Fix clipboard clear race — only clear if clipboard still matches copied secret | Rust | **P0** | GLM, Nemotron, Kimi |
| 3 | Sanitize markdown preview HTML (DOMPurify in `renderMarkdownToHtml` / `useDebouncedPreview`) | Frontend | **P0** | GLM, Nemotron, Kimi, Llama |
| 4 | Path traversal validation on `filename` / vault `path` | Rust | **P0** | GLM, Nemotron, Kimi |
| 5 | Add CSP + content protection (`tauri.conf.json`, builder) | Rust | **P0** | Kimi, Nemotron |
| 6 | Move vault unlock / Argon2 off main thread (`spawn_blocking`) | Rust | **P0** | GLM, Nemotron, Kimi |
| 7 | Extract `Sidebar.tsx` | Frontend | **P0** | GLM (post-fix) |
| 8 | Extract `EditorWorkstation.tsx` | Frontend | **P0** | GLM (post-fix) |
| 9 | Command map + stable palette `id` | Frontend | **P0** | GLM (post-fix) |
| 10 | Split `App.css` by domain | Frontend | **P0** | GLM (post-fix) |
| 11 | Extract vault migration from `commands/vault.rs` | Rust | **P0** | GLM (post-fix), Kimi |
| 12 | Move secret clipboard I/O to `clipboard` module | Rust | **P0** | GLM (post-fix) |
| 13 | Move window lifecycle flags into `AppState` | Rust | **P0** | GLM, GLM (post-fix), Kimi |
| 14 | `CheckMenuItem` for line numbers + startup sync | Rust + FE | **P1** | GLM, Nemotron, Kimi |
| 15 | Zeroize decrypted vault bytes + clipboard strings | Rust | **P1** | GLM, Nemotron, Kimi |
| 16 | Remove/guard mock Gist export command | Frontend | **P1** | GLM, Kimi |
| 17 | Virtualize / throttle line-number gutter scroll sync | Frontend | **P1** | GLM, Nemotron, Kimi |
| 18 | Replace clipboard polling with Windows listener API | Rust | **P2** | Kimi, Nemotron, Llama |
| 19 | Split `crypto.rs` into submodules | Rust | **P2** | GLM (post-fix) |
| 20 | Typed errors instead of `Result<T, String>` | Rust | **P2** | GLM (post-fix) |

---

## Security — multi-reviewer consensus

| Issue | Severity | Fix | Reviewers |
|-------|----------|-----|-----------|
| `vault_io.rs` atomic write not atomic on Windows | Critical | `ReplaceFile` / truncate-in-place / temp+copy pattern on same volume | GLM, Nemotron |
| Clipboard 30s clear overwrites unrelated user copies | High | Token-match before clear | GLM, Nemotron, Kimi |
| `dangerouslySetInnerHTML` without sanitization | Critical/High | DOMPurify in markdown pipeline | GLM, Nemotron, Kimi, Llama |
| Path traversal in `commands/notes.rs` / `commands/vault.rs` | High | Reject `..`, `/`, `\`, null bytes | GLM, Nemotron, Kimi |
| Missing CSP / content protection | High | `tauri.conf.json` + builder CSP | Kimi, Nemotron |
| Secrets in logs / IPC error paths | High | `skip_debug`, audit error mapping | GLM, Kimi |
| Decrypted vault bytes not zeroized | Medium | `Zeroizing<Vec<u8>>` | GLM, Nemotron, Kimi |
| Clipboard history may retain secrets | Medium | Filter before store; exclude vault copies | GLM, Nemotron, Kimi |
| V1 salt UTF-8 fragility | Medium | Raw-byte or B64 salt handling | GLM, Kimi |
| Mock Gist export risks future exfiltration | High | Remove or block on secrets tab | GLM, Kimi |

---

## Optimization — multi-reviewer consensus

| Issue | Fix | Reviewers |
|-------|-----|-----------|
| Blocking vault unlock on UI thread | `spawn_blocking` for read + Argon2 | GLM, Nemotron, Kimi |
| Clipboard polling CPU use | `AddClipboardFormatListener` / sequence-number gate | GLM, Nemotron, Kimi, Llama |
| `ClipboardHistory::get_entries` clones full vec | `Arc` slice or pagination | GLM, Nemotron, Kimi |
| Line numbers gutter DOM bloat | Virtualize gutter or CSS `counter()` | GLM, Nemotron |
| `filteredNotes` O(n) search each keystroke | Memoize / index lowercase terms | GLM, Nemotron, Kimi |
| Google Fonts `@import` blocks offline boot | Bundle fonts locally | GLM |
| Editor toolbar duplicated Windows/non-Windows | Extract `EditorToolbar` | GLM, Nemotron |

---

## Line numbers toggle — consolidated status

| Aspect | Post-fix handoff | Fresh reviews | Action |
|--------|------------------|---------------|--------|
| Toolbar `# Lines` button | Done | Done | None |
| `useAppSettings` persistence | Done | Assumed done (verify) | Audit `useAppSettings.ts` |
| Menu event `menu-toggle-line-numbers` | Done | Done | None |
| `CheckMenuItem` checkmark in native menu | Not mentioned | Missing | P1: `definition.rs` |
| Backend get/set commands for menu sync | Not required per handoff | Requested by Kimi/GLM | P1 if menu must reflect state without FE round-trip |
| Scroll sync via `previousElementSibling` | Not mentioned | Fragile | P1: `useRef` + `requestAnimationFrame` |
| Layout shift on toggle | Not mentioned | Flagged | P2: reserve gutter space in CSS |

---

## Reviewer-specific highlights

### Kimi-only
- Re-auth gate before `reveal_secret_value`
- Vault temp file ACLs / write to `%TEMP%`
- `Alt+Space` shortcut conflict detection + fallback
- Tray icon not stored in `AppState` (leak on crash)
- `confirm_quit` race before `app.exit(0)`

### Llama-only (lower confidence — generic suggestions)
- Broader error-handling and documentation passes
- Tray/window API consistency notes

### Nemotron-only
- Throttle line-numbers `onScroll` to ~100ms
- Verify `useGlobalKeybindings` cleanup on unmount
- Clipboard filter false-negative documentation

### GLM-only (r2 review)
- Remove redundant `isWindows` `useEffect`
- `archive_notes` before `loadNotes` init ordering
- `JSON.parse` reviver for prototype pollution in palette
- Bundle Inter / JetBrains Mono locally

---

## Implementation phases

### Phase 0 — Security blockers (ship gate)
1. `vault_io.rs` Windows-safe atomic write
2. Clipboard clear token-match
3. DOMPurify on preview HTML
4. Path sanitization on all filesystem IPC
5. CSP in `tauri.conf.json`
6. `spawn_blocking` for vault unlock

### Phase 1 — Architecture & UX (GLM post-fix P0 + review P1)
7. Extract `Sidebar.tsx`, `EditorWorkstation.tsx`
8. Palette command map + stable IDs
9. Split `App.css`
10. Extract vault migration + clipboard I/O from `commands/vault.rs`
11. Window lifecycle flags → `AppState`
12. `CheckMenuItem` line numbers + scroll-sync fix
13. Remove/guard Gist mock

### Phase 2 — Polish
14. `crypto.rs` submodule split
15. Clipboard event-driven listener
16. Typed error enum
17. Virtualized line gutter
18. Local font bundling

---

# GLM 5.1 — Windows Port Suggestions (Post-fix)

> **Scope:** `slate_win64/` · **Verdict:** PASS WITH FIXES (Rust + React)  
> **Full review:** [`WINDOWS_REVIEW_glm-5.1-postfix.md`](./WINDOWS_REVIEW_glm-5.1-postfix.md)  
> **Model:** `z-ai/glm-5.1` via NVIDIA NIM · **Date:** 2026-06-24

Handoff doc for agents testing or continuing refactors. Security/lifecycle fixes from the initial review are in place; remaining work is **maintainability and architecture**.

---

## Already completed (do not re-flag)

| Area | Done |
|------|------|
| Vault crypto | v3 per-vault KDF salt, v1/v2 auto-migrate on unlock |
| Vault I/O | Atomic writes via `vault.enc.part` |
| Clipboard | 30s TTL on secret copy; filter docs |
| Lifecycle | `QUIT_IN_PROGRESS`, close-to-tray, exit lifecycle |
| Frontend | Debounced preview, `filteredNotes` useMemo, secret reveal via ref |
| De-monolith | `CommandPalette`, `ClipboardCapsule`, `useGlobalKeybindings`, `menu/windows/{definition,events}.rs` |
| Line numbers | Toggle (default off), persisted via `useAppSettings` |

---

## Rust / Tauri — P0 (before next feature)

1. **Extract vault migration** — Move `needs_v3_migration`, `was_legacy`, salt rotation from `commands/vault.rs::unlock_vault` → `vault_session.rs` or new `vault_migration.rs`. Command should call something like `vault_session::unlock_and_migrate(...)`.
2. **Extract clipboard I/O from vault** — `copy_secret_to_clipboard` in `commands/vault.rs` uses `arboard` + spawns a thread. Move to `clipboard` module as `copy_and_clear_after(text, Duration)`.
3. **Remove global statics in `window.rs`** — Move `INTENTIONALLY_HIDDEN` and `QUIT_IN_PROGRESS` into `AppState` or a `WindowLifecycle` struct.

### Rust — P1 (clarity & comments)

- Add state-machine docstring to `unlock_vault` (Empty / V1 / V2 / V3 classification + migration + rate-limit timing).
- Split `crypto.rs` → `crypto/mod.rs`, `crypto/v3.rs`, `crypto/legacy.rs`.
- Type-alias mutex wrappers in `app_state.rs` (e.g. `ClipboardHistoryState`).
- Document `decrypt_vault` magic-byte dispatch (implicit V1 fallback).
- Document `ClipboardHistory::add` MRU dedup (`retain` + `insert(0)`).
- Expand `clipboard_filter.rs` doc on benign-text heuristics (DB/broker URLs).
- Rename/document magic numbers: `ENTROPY_THRESHOLD`, `MIN_TOKEN_LEN`.
- Extract `try_poll_clipboard` from `start_clipboard_monitor` loop for testability.

### Rust — P2 (nice to have)

- Typed error enum instead of `Result<T, String>` everywhere.
- `enum MenuEvent` instead of raw strings in `menu/windows/events.rs`.

### Rust — God modules to watch

| File | Issue |
|------|-------|
| `commands/vault.rs` | IPC + migration + rate limit + clipboard I/O |
| `crypto.rs` | v1/v2/v3 branching in one file |
| `window.rs` | Global `AtomicBool` statics |
| `app_state.rs` | Leaks `Arc<Mutex<...>>` to commands |

---

## React / Frontend — P0 (before next feature)

1. **Extract `Sidebar.tsx`** — Search, tabs, notes list, footer settings from `App.tsx`.
2. **Extract `EditorWorkstation.tsx`** — Editor `<section>`, view modes, line numbers, preview split.
3. **Command map** — Replace `switch` in `executePaletteCommand.ts` with `Record<string, (ctx) => Promise<void>>`.
4. **Stable palette IDs** — Add `id`/`key` to `PaletteCommand`; dispatch by id, not display `name`.
5. **Split `App.css`** — `styles/layout.css`, `editor.css`, `sidebar.css`, `vault.css`, `platform.css`.

### Frontend — P1 (clarity & comments)

- Extract `getPreviewText(content)` in `utils/notePreview.ts` (strip `<!-- ephemeral -->` marker).
- Docblock on mount `archive_notes` effect (48h scratch archival contract).
- Docblock on "Prettify Markdown" list normalization (`-item` → `- item`).
- Consider `EditorHeader.tsx` with `<WindowsEditorHeader>` / `<MacEditorHeader>` or more `[data-platform]` CSS.
- Tooltip CSS comment in `App.css` (`.tooltip-below`, delete-button override).

### Frontend — P2 (nice to have)

- Group `useNoteEditor` returns into `{ state, actions }` objects.
- Encapsulate `pendingWindowClose` + `quittingRef` inside `useWindowClose`.
- Standardize IPC error handling (`catch(() => undefined)` vs `console.error`).

### Frontend — God modules to watch

| File | Issue |
|------|-------|
| `App.tsx` | ~634 lines; 10+ hooks; platform-branching JSX |
| `App.css` | Layout + theme + components + platform in one file |
| `executePaletteCommand.ts` | String-matching switch dispatch |

---

## Testing focus for agents

Use this as a smoke/regression checklist while manual or automated testing runs.

### Security & vault

- [ ] Unlock v1/v2 vault → auto-migrates to v3 salt on disk
- [ ] Wrong password → rate limiter engages; no vault JSON leak to frontend
- [ ] Copy secret → clipboard clears after ~30s
- [ ] Path traversal blocked on note/vault file ops

### Lifecycle & window

- [ ] Close window → hides to tray (not quit) unless Exit chosen
- [ ] Alt+F4 / File → Exit → clean quit, no orphaned tray icon
- [ ] Summon shortcut restores hidden window

### Editor & notes

- [ ] Ctrl+S saves; scratchpad draft promotes to Saved when appropriate
- [ ] Ctrl+N new draft; debounced autosave persists
- [ ] Archive stale scratch notes on mount (>48h)
- [ ] Line numbers toggle (Windows) persists across sessions

### Palette & clipboard

- [ ] Ctrl+P opens command palette (not while typing in input/textarea)
- [ ] Ctrl+Shift+V opens clipboard history
- [ ] Esc dismisses overlays
- [ ] Format JSON / Prettify Markdown / theme switches work from palette
- [ ] Paste from clipboard history inserts at cursor

### Menu bridge (Windows)

- [ ] File / View / Window / Help menu items emit correct frontend events
- [ ] F1 opens Help dialog

### Build & tests

```bash
cd slate_win64
npm test          # expect 4/4
npm run build     # tsc + vite
cargo test        # 14 tests (if Rust toolchain available)
```

---

## Top actions (combined, priority order)

| # | Action | Layer | Priority |
|---|--------|-------|----------|
| 1 | Extract `Sidebar.tsx` | Frontend | P0 |
| 2 | Extract `EditorWorkstation.tsx` | Frontend | P0 |
| 3 | Command map + stable palette `id` | Frontend | P0 |
| 4 | Split `App.css` by domain | Frontend | P0 |
| 5 | Extract vault migration from `commands/vault.rs` | Rust | P0 |
| 6 | Move secret clipboard I/O to `clipboard` module | Rust | P0 |
| 7 | Move window lifecycle flags into `AppState` | Rust | P0 |
| 8 | Document `unlock_vault` migration matrix | Rust | P1 |
| 9 | Split `crypto.rs` into submodules | Rust | P1 |
| 10 | Extract `getPreviewText` utility | Frontend | P1 |

---

## Paths (quick reference)

```
slate_win64/
  src/
    App.tsx                          # still large — next extractions
    commands/paletteCommands.ts
    commands/executePaletteCommand.ts
    components/CommandPalette.tsx
    components/ClipboardCapsule.tsx
    hooks/useCommandPalette.ts
    hooks/useClipboardCapsule.ts
    hooks/useGlobalKeybindings.ts
  src-tauri/src/
    commands/vault.rs                  # God module — P0 split
    crypto.rs                          # P1 split
    window.rs                          # P0 statics → AppState
    menu/windows/definition.rs
    menu/windows/events.rs
```

---

*Generated from GLM 5.1 post-fix review. Do not treat P0 items as blockers for functional testing — code is PASS WITH FIXES, not FAIL.*

---

## Source artifacts

| File | Model | Notes |
|------|-------|-------|
| [`WINDOWS_REVIEW_glm-5.1.md`](./WINDOWS_REVIEW_glm-5.1.md) | GLM 5.1 | Full r2 review (Rust + React) |
| [`WINDOWS_REVIEW_nemotron-super.md`](./WINDOWS_REVIEW_nemotron-super.md) | Nemotron Super 120B | Full r2 review |
| [`WINDOWS_REVIEW_kimi-k2.6.md`](./WINDOWS_REVIEW_kimi-k2.6.md) | Kimi K2.6 | Full r2 review |
| [`WINDOWS_REVIEW_llama-3.1-70b.md`](./WINDOWS_REVIEW_llama-3.1-70b.md) | Llama 3.1 70B | Full r2 review |
| [`WINDOWS_CODE_REVIEW_COMBINED.md`](./WINDOWS_CODE_REVIEW_COMBINED.md) | DeepSeek V4 Flash synthesis | GLM + Nemotron agreements/differences |
| [`WINDOWS_GLM_SUGGESTIONS.md`](./WINDOWS_GLM_SUGGESTIONS.md) | GLM 5.1 post-fix | Standalone handoff (included above) |

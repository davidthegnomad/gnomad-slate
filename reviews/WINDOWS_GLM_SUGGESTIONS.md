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

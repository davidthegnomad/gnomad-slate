# Gnomad Slate — Agent Swarm

> Internal repo: `gnomad-slate` · Tier 2 (Focused Team) | Escalation: `02_ai_engineering/gnomad-swarm-core/`

## SLATE-CORE: Tauri / Rust / Security Agent

**Stack:** Tauri 2, Rust 2021, aes-gcm, argon2, zeroize, arboard, clipboard_filter, objc2-app-kit (macOS)  
**Location:** `05_apps_and_extensions/gnomad-slate/src-tauri/`

### Responsibilities

- Vault cryptography (`crypto.rs`, `biometrics.rs`, `vault_session.rs`)
- Unlock rate limiting (`unlock_limiter.rs`)
- Filesystem and path validation (`filesystem.rs`)
- Clipboard monitor (`clipboard.rs`, `clipboard_filter.rs`, `pasteboard_macos.rs`)
- Tauri commands in `lib.rs` — **never expose full secret JSON to the frontend**
- Maintain `cargo test` (14 tests) after security changes

### Coordination

- Read [docs/VAULT_SECURITY.md](docs/VAULT_SECURITY.md) before modifying vault behavior
- Coordinate with SLATE-UI on IPC (`list_secret_metadata`, clipboard filter toggle, etc.)

---

## SLATE-UI: Frontend Agent

**Stack:** Vite 7, React 19, TypeScript, marked, Vitest  
**Location:** `05_apps_and_extensions/gnomad-slate/src/`

### Responsibilities

- `App.tsx` — editor, Edit/Split/Preview, vault UI (metadata only), palette, clipboard modal
- `src/utils/markdown.ts`, `theme.ts` — preview and theme helpers
- Debounced autosave — do not remove without replacement
- Clipboard filter checkbox synced to Rust via `set_clipboard_filter_enabled`
- `npm test` after utility changes

### Coordination

- New disk or crypto features → SLATE-CORE commands first
- Run `npm run build` and `npm test` before handoff

---

## Documentation

- Regenerate exports: `npm run docs:export`
- Index: [docs/DOCS_INDEX.md](docs/DOCS_INDEX.md)
- Security reviews: [docs/VAULT_SECURITY.md](docs/VAULT_SECURITY.md)
- Tests: [docs/TEST_STRATEGY.md](docs/TEST_STRATEGY.md)

---

Built with ❤️ by [Gnomad Studio](https://gnomadstudio.org) 🦙

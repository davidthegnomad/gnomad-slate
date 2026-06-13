# Changelog — Gnomad Slate

All notable changes to this project are documented here.

---

## [2.1.0-beta.1] — June 2026

### Added

- About menu with version info and Check for Updates (Tauri updater)
- Custom app icon and refreshed window sizing
- Cmd+S / Cmd+N shortcuts; platform-aware beta labels

### Release

- GitHub Release `v2.1.0-beta.1` with macOS Apple Silicon artifacts (`.app.tar.gz`, `.zip`)
- Marketing and downloads on [davidcole.cloud/apps/slate](https://davidcole.cloud/apps/slate)

---

## [0.1.0] — June 2026

### Added

- Tauri 2 desktop scratchpad with React 19 + Vite 7 frontend
- Scratchpad (ephemeral) and Vault (persistent) markdown notes with **Edit / Split / Preview** modes (`marked`)
- Vault-X encrypted secrets registry (AES-256-GCM + Argon2id)
- Clipboard history capsule (⌘⇧V) with configurable sensitive-content filtering
- Global shortcut (Option+Space) to show/hide window
- Command palette (⌘P) for editor actions
- Auto-archive of scratch notes older than 48 hours
- Debounced note autosave (400 ms)
- Full documentation suite (MD, HTML, TXT) — see `docs/DOCS_INDEX.md`

### Security

- Argon2id master password verification (legacy SHA-256 auto-migration)
- Vault v2 on-disk format with session-only decrypted secrets in Rust
- `Zeroize` / `ZeroizeOnDrop` on in-memory `SecretItem` and vault session key
- Unlock rate limiting (5 failures → 30s lockout)
- Path traversal protection for note filenames
- Portable app data directory via `dirs` crate (legacy path under `~/.gemini/...` per user home)
- Clipboard filter: prefix blocklist + Shannon entropy (reduced false positives vs rigid mixed-case rule)
- macOS pasteboard `changeCount` check + adaptive poll interval (1s active / 3s idle)

### Testing

- **14** Rust unit tests (`cargo test`) — crypto, clipboard filter, filesystem, unlock limiter
- **4** Vitest tests (`npm test`) — theme resolution, markdown preview sanitization

### Developer experience

- `npm run docs:export` — regenerate HTML/TXT from Markdown
- `npm run test` / `npm run test:watch` — frontend unit tests
- `npm run tauri:dev` / `npm run tauri:build` — desktop scripts

---

Built with ❤️ by [Gnomad Studio](https://gnomadstudio.org) 🦙

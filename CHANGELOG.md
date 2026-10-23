# Changelog — Gnomad Slate

All notable changes to this project are documented here.

---

## [2.1.0-beta.8] — October 2026

### Added

- Issue a new Vault-X recovery key while the vault is unlocked. Quit and hide stay blocked until the one-time key dialog is closed.

### Fixed

- Copying a recovery key or secret skips clipboard history and clears the clipboard after a short timeout if that text is still there
- Creating or resetting a vault asks for the password twice
- The command palette Lock command locks the vault
- Quitting locks the vault on macOS, Linux, and Windows
- Windows no longer rewrites a v2 vault in a way that drops the recovery wrap
- Daily notes append instead of replacing the journal
- A second saved note on the same day keeps the first note
- Switching to Secrets saves the open note first

---

## [2.1.0-beta.7] — October 2026

### Added

- Check for Updates on macOS and Linux downloads and installs the newest release. The menu item and the About dialog both use it.

---

## [2.1.0-beta.6] — October 2026

### Added

- Vault-X one-time recovery key. It is shown once when a vault is created, and it can set a new master password without wiping secrets.

### Fixed

- A crash during password reset can no longer leave `vault.enc` and `vault_config.json` on different keys
- Setup refuses to overwrite a leftover `vault.enc` unless the wipe is confirmed
- Unlock and recover fail when the encrypted vault file is missing, instead of saving an empty vault

---

## [2.1.0-beta.5] — October 2026

### Added

- Loopback agent API at `http://127.0.0.1:47321` for notes, clipboard, vault, settings, and window controls. Bearer token, localhost only.

### Fixed

- Notepad paste inserts at the caret, and copy no longer loses the selection
- Line numbers use the editor font and measured line heights, so wrapped lines stay aligned

---

## [1.0.0-alpha.1] — Windows — June 2026

### Added — Windows port (`slate_win64/`)

- Separate Windows codebase: native menu, system tray, `Alt+Space` summon, WebView2 bootstrap
- Security P0: atomic vault writes, clipboard clear race fix, path validation, DOMPurify, CSP, async unlock
- Frontend extractions: `Sidebar`, `EditorWorkstation`, palette command IDs
- **17** Rust unit tests in `slate_win64/src-tauri`

### Release

- Windows `.msi` / `.exe` via CI (`slate_win64` tree) · alpha channel

---

## [2.1.0-beta.4] — June 2026

### Fixed

- **Linux menu launch crash** — set `GDK_BACKEND=x11` and `WEBKIT_DISABLE_DMABUF_RENDERER=1` at startup when unset, fixing WebKitGTK `Gdk Error 71` on Wayland sessions (app menu / `.desktop` launches)

---

## [2.1.0-beta.3] — June 2026

### Added — Linux desktop (alpha)

- Platform-split menus (`menu/linux.rs`, `menu/macos.rs`), system tray, `Super+Shift+Space` summon shortcut
- Linux UI: filename on own row, line numbers toggle, in-app dialogs (no `alert()`), detailed Help (F1) with support contact
- Packaging: `gnomad-slate` binary, Fedora/Nobara RPM and DEB with `.desktop` entry, libsoup3 deps
- Settings persistence via `tauri-plugin-store`; strict CSP; structured `tracing` logs
- **File → Close** (`Ctrl+W`) and **File → Quit** (`Ctrl+Q`) with vault-aware `confirm_quit` lifecycle

### Fixed

- Unified quit flow (`confirm_quit`) — no zombie process when global shortcut plugin keeps event loop alive
- Linux close path uses runtime `isLinuxDesktop()` so quit works even if `plugin-os` loads after mount
- Version display on Linux matches `package.json` / `Cargo.toml` semver

### Release

- GitHub Release `v2.1.0-beta.3` with Linux `.rpm` and `.deb` (Nobara/Fedora); macOS/Windows matrix builds
- Session log: [docs/LINUX_SESSION_2026-06-23.md](docs/LINUX_SESSION_2026-06-23.md)

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

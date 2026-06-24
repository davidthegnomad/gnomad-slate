# Gnomad Slate (Windows Edition) — Agent Reference

> **Target audience:** LLM agents and developers working on this codebase.
> For human-facing docs (quick start, features, support), see [HUMAN.md](HUMAN.md).

---

## Architecture overview

```
slate_win64/
├── src/                          # React 19 + TypeScript frontend
│   ├── App.tsx                   # Main app shell (~1600 lines)
│   ├── App.css                   # Stylesheet (includes Windows-specific layout)
│   ├── main.tsx                  # React entry point
│   ├── components/
│   │   ├── AboutDialog.tsx       # Version info + updater UI
│   │   ├── HelpDialog.tsx        # Windows help reference (→ content/windowsHelp.ts)
│   │   ├── AppMessageDialog.tsx  # Modal message overlay
│   │   └── UnsavedChangesModal.tsx
│   ├── hooks/
│   │   └── useDesktopMenuBridge.ts  # Wires native menu events → React state
│   ├── utils/
│   │   ├── ipc.ts                # Tauri invoke/listen wrappers + platform window shim
│   │   ├── platform.ts           # isWindowsDesktop() detection
│   │   ├── modKeyLabel.ts        # Ctrl/⌘ label handling
│   │   ├── settings.ts           # @tauri-apps/plugin-store wrapper
│   │   ├── theme.ts              # resolveTheme(preference, prefersDark)
│   │   ├── markdown.ts           # marked renderer
│   │   └── updater.ts            # Update check + install (tauri-plugin-updater)
│   └── content/
│       └── windowsHelp.ts        # In-app help text (data location, shortcuts, etc.)
│
└── src-tauri/                    # Rust backend (Tauri 2)
    ├── Cargo.toml                # Dependencies (winapi for clipboard, zeroize, etc.)
    ├── tauri.conf.json           # Window config, CSP, bundle targets (msi + nsis)
    ├── build.rs                  # tauri_build::build()
    ├── capabilities/
    │   └── default.json          # Permission set (core, opener, global-shortcut, updater, etc.)
    ├── icons/                    # App icons (ico, png, Windows store logos)
    └── src/
        ├── main.rs               # #![windows_subsystem = "windows"] → run()
        ├── lib.rs                # Plugin registration, state init, route mounting
        ├── app_state.rs          # AppState + WindowLifecycle (AtomicBool flags)
        ├── window.rs             # Window show/hide/toggle, global shortcut, close-to-tray
        ├── shortcuts.rs          # Alt+Space constant
        ├── logging.rs            # tracing_subscriber init
        ├── filesystem.rs         # Note CRUD, path validation, %APPDATA% data root
        ├── crypto.rs             # AES-256-GCM v2/v3 encryption + Argon2id KDF
        ├── biometrics.rs         # Argon2id password hash storage + verification
        ├── vault_session.rs      # In-memory decrypted session (Zeroize on drop)
        ├── vault_io.rs           # Encrypted vault file I/O (atomic write)
        ├── vault_migration.rs    # v1/v2 → v3 migration plan on unlock
        ├── unlock_limiter.rs     # Rate-limited unlock (5 attempts / 30s lockout)
        ├── clipboard.rs          # ClipboardHistory + background polling thread
        ├── clipboard_filter.rs   # Heuristic credential detection (entropy + patterns)
        ├── windows_clipboard.rs  # GetClipboardSequenceNumber for efficient polling
        ├── windows_tray.rs       # System tray icon with Show/Hide/Exit menu
        ├── commands/
        │   ├── mod.rs
        │   ├── app.rs            # get_app_version_info, get_summon_shortcut_label, confirm_quit
        │   ├── clipboard.rs      # Clipboard history + filter toggle commands
        │   ├── notes.rs          # Note CRUD commands
        │   └── vault.rs          # Vault lifecycle + secret management commands
        └── menu/
            ├── mod.rs            # Dispatch to windows.rs
            └── windows.rs        # Full native menu (File/Edit/View/Window/Help)
```

---

## Module interaction map

```
┌─────────────────────────────────────────────────────────────┐
│  Frontend (React)                                           │
│  App.tsx ──invoke──▶ Tauri IPC ──▶ commands/*.rs           │
│  App.tsx ◀──listen── Tauri Events ◀── window.rs (emit)     │
│  useDesktopMenuBridge ◀──listen── menu events              │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  lib.rs (entry point)                                       │
│  ├── plugins: opener, os, updater, process, store           │
│  ├── manage: AppState                                       │
│  ├── setup: menu::install, window::register_global_shortcut │
│  │          windows_tray::install, window::attach_close      │
│  └── invoke_handler: all commands/* routed                  │
└─────────────────────────────────────────────────────────────┘
                              │
          ┌───────────────────┼───────────────────┐
          ▼                   ▼                   ▼
┌──────────────┐   ┌──────────────┐   ┌──────────────┐
│ commands/    │   │ clipboard.rs  │   │ window.rs     │
│ app.rs       │   │ + filter      │   │ + shortcuts   │
│ clipboard.rs │   │ + win32 api   │   │ + tray        │
│ notes.rs     │   └──────────────┘   └──────────────┘
│ vault.rs ────┼───▶ vault_io.rs ──▶ filesystem.rs
└──────────────┘         │
                         ▼
              ┌──────────────────┐
              │ vault_session.rs  │
              │ (in-memory,       │
              │  ZeroizeOnDrop)   │
              └──────┬───────────┘
                     │
         ┌───────────┼───────────┐
         ▼           ▼           ▼
   crypto.rs   biometrics.rs   unlock_limiter.rs
```

---

## IPC contract

### Commands (Rust → Frontend, via `invoke`)

| Command | Module | Args | Returns |
|---|---|---|---|
| `get_app_version_info` | `commands::app` | — | `{ version, channel, display_version }` |
| `get_summon_shortcut_label` | `commands::app` | — | `"Alt+Space"` |
| `confirm_quit` | `commands::app` | — | void (exits process) |
| `copy_text_to_clipboard` | `commands::clipboard` | `{ text }` | `Result<()>` |
| `get_clipboard_history` | `commands::clipboard` | — | `Vec<ClipboardEntry>` |
| `get_clipboard_filter_enabled` | `commands::clipboard` | — | `bool` |
| `set_clipboard_filter_enabled` | `commands::clipboard` | `{ enabled }` | void |
| `list_notes` | `commands::notes` | — | `Result<Vec<Note>>` |
| `read_note` | `commands::notes` | `{ filename }` | `Result<String>` |
| `write_note` | `commands::notes` | `{ title, content, is_ephemeral, previous_filename }` | `Result<Note>` |
| `delete_note` | `commands::notes` | `{ filename }` | `Result<()>` |
| `archive_notes` | `commands::notes` | — | `Result<usize>` |
| `is_vault_initialized` | `commands::vault` | — | `bool` |
| `is_vault_unlocked` | `commands::vault` | — | `bool` |
| `initialize_vault` | `commands::vault` | `{ password }` | `Result<()>` |
| `unlock_vault` | `commands::vault` | `{ password }` | `Result<()>` (async, spawn_blocking) |
| `lock_vault` | `commands::vault` | — | void |
| `list_secret_metadata` | `commands::vault` | — | `Result<Vec<SecretMeta>>` |
| `reveal_secret_value` | `commands::vault` | `{ index }` | `Result<String>` |
| `copy_secret_to_clipboard` | `commands::vault` | `{ index }` | `Result<()>` |
| `add_secret` | `commands::vault` | `{ name, path, value }` | `Result<()>` |
| `delete_secret` | `commands::vault` | `{ index }` | `Result<()>` |

### Events (Rust → Frontend, via `emit`)

| Event | Source | Payload | Trigger |
|---|---|---|---|
| `app-quit-requested` | `window::emit_quit_requested` | `()` | File→Exit, tray→Exit, or `ExitRequested` run event |
| `menu-about` | `menu::windows::handle_event` | `()` | Help→About |
| `menu-check-updates` | `menu::windows::handle_event` | `()` | Help→Check for Updates |
| `menu-new-draft` | `menu::windows::handle_event` | `()` | File→New Scratchpad Draft |
| `menu-new-saved-note` | `menu::windows::handle_event` | `()` | File→New Saved Note |
| `menu-save` | `menu::windows::handle_event` | `()` | File→Save |
| `menu-save-permanent` | `menu::windows::handle_event` | `()` | File→Save as Permanent |
| `menu-delete-note` | `menu::windows::handle_event` | `()` | File→Delete |
| `menu-archive-stale` | `menu::windows::handle_event` | `()` | File→Archive Stale |
| `menu-view-edit` | `menu::windows::handle_event` | `()` | View→Edit Mode |
| `menu-view-split` | `menu::windows::handle_event` | `()` | View→Split Mode |
| `menu-view-preview` | `menu::windows::handle_event` | `()` | View→Preview Mode |
| `menu-command-palette` | `menu::windows::handle_event` | `()` | View→Command Palette |
| `menu-clipboard-history` | `menu::windows::handle_event` | `()` | View→Clipboard History |
| `menu-toggle-line-numbers` | `menu::windows::handle_event` | `()` | View→Toggle Line Numbers |
| `menu-theme-light` | `menu::windows::handle_event` | `()` | View→Light Theme |
| `menu-theme-dark` | `menu::windows::handle_event` | `()` | View→Dark Theme |
| `menu-theme-system` | `menu::windows::handle_event` | `()` | View→System Theme |
| `menu-help` | `menu::windows::handle_event` | `()` | Help→Help |

---

## Key design decisions

### 1. Close-to-tray pattern
Windows desktop convention: clicking X hides to system tray, doesn't quit. See [window.rs](src-tauri/src/window.rs) — `attach_window_close_handler` intercepts `CloseRequested`, prevents default, and hides the window. Only explicit File→Exit or tray→Exit triggers `app-quit-requested` → `confirm_quit`.

### 2. Vault format evolution
Three formats handled with transparent migration on unlock. See [crypto.rs](src-tauri/src/crypto.rs):
- **v1** (legacy): Per-blob PHC salt, Argon2id
- **v2**: Fixed application-wide KDF salt, `0x02` header
- **v3** (current): Per-file random 16-byte KDF salt, `0x03` header, AES-256-GCM

`vault_migration::plan_unlock` reads the file, classifies the format, decrypts, and returns an `UnlockPlan` with migration flags. On v1/v2 vaults, the session is opened and the vault is immediately re-saved as v3.

### 3. Clipboard change detection
Uses Win32 `GetClipboardSequenceNumber` (see [windows_clipboard.rs](src-tauri/src/windows_clipboard.rs)) for efficient polling — only reads clipboard text when the sequence number changes. Falls back to simple polling on non-Windows. Adaptive polling: 1s active → 3s idle after 5 unchanged polls.

### 4. Credential filtering
Heuristic approach in [clipboard_filter.rs](src-tauri/src/clipboard_filter.rs):
- Pattern matching against known prefixes (`ghp_`, `sk-proj-`, `AKIA`, `-----BEGIN`, connection strings)
- Shannon entropy threshold (≥4.2 bits/char for tokens 32–256 chars)
- Benign classifiers: email addresses, prose with spaces, file extensions, camelCase identifiers
- Whitelist before blacklist — emails/URLs/identifiers pass through before entropy check

### 5. Security boundaries
- **Secrets never leave Rust as full JSON.** `list_secret_metadata` returns only name + path + index. `reveal_secret_value` returns a single value by index. No command exposes all values.
- **Heap zeroization:** `SecretItem` derives `ZeroizeOnDrop`. Sensitive buffers use `zeroize::Zeroizing<String>`.
- **Rate limiting:** 5 failed unlock attempts → 30-second lockout. Reset after 5 minutes of inactivity.
- **Atomic vault writes:** `vault_io.rs` uses truncate-in-place for existing files (Windows `rename` semantics), `.part` temp file + rename for first writes.

---

## Data flow: Vault unlock

```
1. Frontend calls invoke("unlock_vault", { password })
2. commands::vault::unlock_vault:
   a. Check unlock_limiter::check_allowed()
   b. biometrics::verify_master_password(password) — Argon2id verify
   c. vault_migration::plan_unlock(password) — read file, classify format, decrypt
      → spawn_blocking (CPU-bound Argon2id KDF)
   d. vault_session::open_session_with_json(password, salt, plaintext)
      → derive key, create VaultSession in-memory
   e. If migrate needed: persist_vault() → re-encrypt as v3
3. Frontend refreshes: invoke("list_secret_metadata") → Vec<SecretMeta>
4. Frontend reveals value: invoke("reveal_secret_value", { index }) → String
5. Frontend copies: invoke("copy_secret_to_clipboard", { index })
```

---

## Build pipeline

```powershell
# Development
cd slate_win64
npm install
npm run tauri:dev          # Vite HMR + Tauri window

# Production
npm run tauri:build        # → src-tauri/target/release/bundle/
                            #   ├── msi/gnomad-slate_*.msi
                            #   └── nsis/gnomad-slate_*.exe

# Tests
npm test                   # Vitest (frontend)
cd src-tauri && cargo test # Rust unit tests
```

TypeScript check: `npx tsc --noEmit` (enforced in `npm run build`).

CI/CD: [../.github/workflows/release.yml](../.github/workflows/release.yml) — matrix builds for `x86_64-pc-windows-msvc`.

---

## Dependencies worth noting

| Crate | Purpose |
|---|---|
| `aes-gcm 0.10.3` | AES-256-GCM encryption for vault |
| `argon2 0.5.3` | Argon2id password hashing + KDF |
| `zeroize 1.8` | Heap buffer clearing on drop |
| `arboard 3.6.1` | Cross-platform clipboard access |
| `winapi 0.3` | `GetClipboardSequenceNumber` for efficient polling |
| `tauri-plugin-global-shortcut` | `Alt+Space` system-wide hotkey |
| `tauri-plugin-updater` | In-app update check + install |
| `tauri-plugin-store` | Persistent key-value settings |

---

## Test strategy

- **Frontend:** Vitest with jsdom — theme resolution, markdown rendering. 4 tests across 2 suites.
- **Rust:** `#[cfg(test)]` blocks in `crypto.rs`, `filesystem.rs`, `clipboard_filter.rs`, `unlock_limiter.rs`, `vault_io.rs`.
- **Key invariants tested:**
  - AES-256-GCM v2 roundtrip (`crypto::tests::v2_roundtrip`)
  - Path traversal rejection (`filesystem::tests::rejects_path_traversal_filenames`)
  - Lockout after 5 failures (`unlock_limiter::tests::lockout_after_five_failures`)
  - Credential pattern detection (`clipboard_filter::tests::blocks_github_pat_prefix`)
  - Vault file path construction (`vault_io::tests::vault_file_path_ends_with_vault_enc`)

---

## Modifying this project

1. **Adding a Rust command:** Create or extend a file in `src-tauri/src/commands/`, add `#[tauri::command]`, register in `lib.rs`'s `invoke_handler`. Update the IPC table above.
2. **Adding a frontend feature:** Work in `src/App.tsx` (state + UI) or `src/utils/` (pure logic). Wire to Rust via `invoke` / `listen` from `ipc.ts`.
3. **Menu changes:** Edit `menu/windows.rs`. Menu items emit events consumed by `useDesktopMenuBridge` in `hooks/useDesktopMenuBridge.ts`.
4. **Security changes:** See `docs/VAULT_SECURITY.md` (parent project). Coordinate Rust + frontend — never expose full secret JSON over IPC.
5. **CSS:** `App.css` uses `[data-platform="windows"]` selectors for Windows-specific layout. Editor header uses `editor-header-windows` class.

---

Built with ❤️ by [Gnomad Studio](https://gnomadstudio.org) 🦙

# Gnomad Slate — Windows Edition

**TL;DR:** Gnomad Slate is a native Windows desktop scratchpad with markdown notes, AES-256 encrypted secrets vault, clipboard history, and a system tray icon. Press `Alt+Space` to summon it from anywhere. Closing the window minimizes to tray — use **File → Exit** to fully quit.

---

## What is this?

Gnomad Slate is a lightweight desktop companion for quick notes, clipboard tracking, and secure credential storage. It lives in your system tray and appears instantly with a global hotkey.

- **Scratchpad notes** — write markdown, auto-saved. Ephemeral drafts auto-archive after 48 hours.
- **Vault-X secrets** — AES-256-GCM encrypted vault protected by an Argon2id master password. Secrets never leave Rust memory in full.
- **Clipboard history** — last 15 clipboard entries with smart filtering that blocks API keys and tokens.
- **Command palette** — `Ctrl+P` for quick formatting, JSON prettification, theme switching, and more.

Built with [Tauri 2](https://v2.tauri.app/), React 19, TypeScript, and Rust.

---

## Quick start

### Build from source

**Prerequisites:** [Node.js 22+](https://nodejs.org/), [Rust](https://rustup.rs/), [Microsoft Visual C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)

```powershell
cd slate_win64
npm install
npm run tauri:dev       # dev mode with hot reload
npm run tauri:build     # production .msi / .exe
```

### Install the pre-built release

Download the latest `.msi` or `.exe` installer from [GitHub Releases](https://github.com/davidthegnomad/gnomad-slate/releases).

---

## Features at a glance

| Feature | Shortcut | Description |
|---|---|---|
| Summon/hide window | `Alt+Space` | Global hotkey from anywhere |
| New draft | `Ctrl+N` | Start a scratchpad note |
| Save note | `Ctrl+S` | Save or promote to permanent |
| Command palette | `Ctrl+P` | Quick actions and formatting |
| Clipboard history | `Ctrl+Shift+V` | Browse and paste from history |
| Help | `F1` | In-app reference guide |

### Editor modes
- **Edit** — raw markdown editor
- **Split** — editor + live preview side-by-side
- **Preview** — rendered markdown only

### Vault
- Argon2id master password with rate-limited unlock (5 attempts → 30s lockout)
- AES-256-GCM v3 vault format with per-file random KDF salt
- Legacy v1/v2 vault migration on unlock

### Clipboard
- Last 15 entries tracked
- Automatic credential filtering (detects API keys, tokens, private keys via entropy + pattern matching)
- Toggle filter from the sidebar

---

## Where your data lives

All data is stored locally on your machine:

```
%APPDATA%\gnomad-slate\
  ├── notes\           # Markdown files
  ├── archive\         # Auto-archived ephemeral notes
  ├── vault.enc        # Encrypted secrets (AES-256-GCM)
  └── vault_config.json # Argon2id password hash
```

Back up the `gnomad-slate` folder to preserve your notes and secrets.

---

## Platform notes

- **Alt+Space conflict:** Windows PowerToys Run uses `Alt+Space` by default. Change either app's shortcut in its settings to avoid conflict.
- **Close vs Exit:** Clicking X minimizes to the system tray. Use **File → Exit** or right-click the tray icon → **Exit** to fully quit.
- **WebView2:** The app bundles Microsoft Edge WebView2. If missing, the installer downloads it automatically.

---

## Support

Questions, bugs, or feedback: `david@gnomadstudio.org`

Built with ❤️ by [Gnomad Studio](https://gnomadstudio.org) 🦙

# User Guide — Gnomad Slate

**Version:** 0.1.0

Studio: [gnomadstudio.org](https://gnomadstudio.org)

---

## Introduction

**Gnomad Slate** is a desktop scratchpad for quick markdown notes, clipboard history, and an encrypted **Vault-X** registry for API keys and tokens. Press **Option+Space** (macOS) to summon the window from anywhere.

---

## Installation

### From source (developers)

**Prerequisites:** Node.js LTS, Rust stable, [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/)

```bash
cd 05_apps_and_extensions/gnomad-slate
npm install
npm run tauri dev
```

### Installers

Download from **[GitHub Pages](https://davidthegnomad.github.io/gnomad-slate/)** or [GitHub Releases](https://github.com/davidthegnomad/gnomad-slate/releases):

| Platform | Version | Artifacts |
|----------|---------|-----------|
| macOS | 2.1.0-beta.4 | `.dmg`, `.app.tar.gz` |
| Linux | 2.1.0-beta.4 | `.deb`, `.rpm`, AppImage |
| Windows | 1.0.0-alpha.1 | `.msi`, `.exe` (built from `slate_win64/`) |

See [BUILD_PLATFORMS.md](BUILD_PLATFORMS.md) and [PLATFORM_LAYOUT.md](PLATFORM_LAYOUT.md).

---

## Main interface

```
┌─────────────────────────────────────────────────────────────┐
│  Sidebar — search, tabs, notes, settings, themes            │
├─────────────────────────────────────────────────────────────┤
│  Toolbar: Edit | Split | Preview                            │
│  Editor / preview  OR  Vault-X (Secrets tab)               │
└─────────────────────────────────────────────────────────────┘
```

### Sidebar tabs

| Tab | Purpose |
|-----|---------|
| **Scratchpad** | Ephemeral `temp_*.md` notes — auto-archived after 48 hours |
| **Vaults** | Persistent `vault_*.md` notes |
| **Secrets** | Vault-X encrypted credentials |

---

## Writing notes

1. Type in the editor — title and body **autosave after ~400 ms** (debounced).
2. Use **Edit**, **Split**, or **Preview** in the toolbar:
   - **Edit** — raw markdown textarea only  
   - **Split** — editor and rendered preview side by side  
   - **Preview** — rendered markdown only  
3. **Scratchpad Draft** toggles ephemeral vs saved vault note.
4. **New Draft** creates a fresh ephemeral buffer.
5. Delete notes with **✕** on the list item.

**Search** filters the current tab by title or content.

Preview uses [marked](https://marked.js.org/) with basic HTML sanitization (scripts stripped). Do not paste untrusted markdown expecting full XSS hardening.

---

## Keyboard shortcuts

| Shortcut | Action |
|----------|--------|
| **Option+Space** | Show/hide app (global) |
| **⌘P** | Command palette |
| **⌘⇧V** | Clipboard history capsule |
| **Esc** | Close palette or clipboard modal |

Slate **hides when you focus another app**. Bring it back with **Option+Space** or by **clicking the Dock icon**.

Command palette actions include JSON format, markdown cleanup, daily note helpers, and theme switching.

---

## Clipboard capsule

1. Press **⌘⇧V**.
2. Click an entry to paste into the editor.
3. By default, likely credentials are **filtered out** (API key prefixes, private keys, high-entropy tokens).

### Clipboard filter setting

In the sidebar footer, toggle **Filter sensitive clipboard items**:

- **On** (default) — uses prefix blocklist + entropy heuristics; allows emails, prose, and typical camelCase identifiers.
- **Off** — stores all clipboard text (except empty / oversized entries).

Setting is saved in `localStorage` and synced to the Rust monitor.

---

## Vault-X (encrypted secrets)

### First-time setup

1. Open the **Secrets** tab.
2. Enter a master password (**minimum 8 characters**).
3. Click **Initialize Vault**.

### Unlock / lock

- **Unlock Vault** — enter master password; password is cleared from the UI after success.
- **Lock Vault** — drops the Rust session and zeroizes in-memory secrets.

### Managing secrets

| Action | How |
|--------|-----|
| Add | **+ Add Secret Key** → name, optional project path, value |
| Copy | Click masked cell — Rust copies to OS clipboard |
| Show | **Show** / **Hide** — value fetched on demand only |
| Delete | **Delete** on the row |

Read [VAULT_SECURITY.md](VAULT_SECURITY.md) for encryption and password details.

### Failed unlock attempts

After **5 wrong passwords**, unlocking is blocked for **30 seconds**.

---

## Themes & settings

| Setting | Location |
|---------|----------|
| **Light / Dark / System** | Sidebar footer (`gnomad-theme`) |
| **Clipboard filter** | Sidebar checkbox (`gnomad-clipboard-filter`) |

---

## Data location

| Platform | Path |
|----------|------|
| **macOS** | `~/Library/Application Support/gnomad-slate/` |
| **Linux** | `~/.local/share/gnomad-slate/` (typical) |
| **Windows** | `%APPDATA%\gnomad-slate\` |

Legacy data may exist at `~/.gemini/antigravity/scratch/slate_data` and is migrated on first launch.

Back up the app data folder to preserve notes and encrypted secrets.

---

## Related documents

- [VAULT_SECURITY.md](VAULT_SECURITY.md) — passwords and encryption  
- [TROUBLESHOOTING.md](TROUBLESHOOTING.md) — common issues  
- [PRIVACY.md](PRIVACY.md) — what stays local  
- [TEST_STRATEGY.md](TEST_STRATEGY.md) — automated tests  

---

Built with ❤️ by [Gnomad Studio](https://gnomadstudio.org) 🦙

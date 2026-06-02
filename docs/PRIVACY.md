# Privacy — Gnomad Slate

**Version:** 0.1.0  
**Last updated:** June 2026

---

## Summary

Gnomad Slate is **local-first**. Notes, clipboard history, encrypted vault data, and UI preferences stay on your device. There is no Gnomad-operated sync server or analytics pipeline in v0.1.

---

## What we collect

| Data | Collected by app? | Sent to Gnomad? |
|------|-------------------|-----------------|
| Markdown notes | Local files | No |
| Vault secrets | Encrypted `vault.enc` | No |
| Master password | Argon2 hash only | No |
| Clipboard history | In-memory (max 15); optional filter | No |
| Theme preference | `localStorage` (`gnomad-theme`) | No |
| Clipboard filter toggle | `localStorage` + Rust monitor | No |
| Usage analytics | Not implemented | No |

---

## Where data lives

| Data | Location |
|------|----------|
| Notes | `<app-data>/gnomad-slate/notes/` |
| Encrypted secrets | `<app-data>/gnomad-slate/vault.enc` |
| Password hash | `<app-data>/gnomad-slate/vault_config.json` |
| Legacy migration source | `~/.gemini/antigravity/scratch/slate_data` (if present) |

**macOS example:** `~/Library/Application Support/gnomad-slate/`

---

## Clipboard filter

When **Filter sensitive clipboard items** is enabled (default), Slate tries to exclude likely API keys from in-app history. This does not prevent other applications from reading the system clipboard. You can disable filtering in the sidebar.

---

## Third-party services

| Service | When | Data shared |
|---------|------|-------------|
| **Google Fonts** | UI load (CSS import) | Font requests if online |
| **GitHub** | Clone / releases | Standard Git hosting |

No third party receives vault plaintext or your master password.

---

## Your responsibilities

- Back up the app data folder.
- Use a strong vault master password — see [VAULT_SECURITY.md](VAULT_SECURITY.md).
- Lock the vault on shared machines.
- Treat markdown preview as rendering your own content only.

---

Built with ❤️ by [Gnomad Studio](https://gnomadstudio.org) 🦙

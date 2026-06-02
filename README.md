# Gnomad Slate

[![Version](https://img.shields.io/badge/version-0.1.0-06b6d4)](CHANGELOG.md)
[![Stack](https://img.shields.io/badge/stack-Tauri%202%20%2B%20React%2019-8b5cf6)](docs/TECH_STACK.md)
[![Security](https://img.shields.io/badge/vault-AES--256--GCM%20%2B%20Argon2id-333)](docs/VAULT_SECURITY.md)
[![Tests](https://img.shields.io/badge/tests-14%20Rust%20%2B%204%20Vitest-333)](docs/TEST_STRATEGY.md)

**Gnomad Slate** is a desktop scratchpad and encrypted secrets vault — quick markdown capture with live preview, clipboard history, and **Vault-X** for API keys and tokens. Summon with **Option+Space** from anywhere.

Built with ❤️ by [Gnomad Studio](https://gnomadstudio.org) 🦙

**Repository:** [github.com/davidthegnomad/gnomad-slate](https://github.com/davidthegnomad/gnomad-slate)

**Status & roadmap:** [`docs/STATUS.md`](docs/STATUS.md) · [`docs/ROADMAP.md`](docs/ROADMAP.md)

---

## Why this exists

You need a place to drop thoughts, snippets, and credentials without opening a browser or leaving secrets in plain notes. Slate stays ready in the background: scratchpad notes with markdown preview, persistent vault notes, filtered clipboard history, and a **Rust-backed** encrypted secret store the UI never holds in full.

---

## Capabilities (v0.1.0)

| Area | What you get |
|------|----------------|
| **Scratchpad** | Ephemeral markdown notes; auto-archive after 48h |
| **Vault notes** | Persistent `vault_*.md` buffers |
| **Markdown** | Edit, Split, and Preview modes (`marked`) |
| **Vault-X** | AES-256-GCM secrets; Argon2id master password |
| **Security** | Secrets in Rust only; zeroized heap; rate-limited unlock |
| **Clipboard** | History capsule (⌘⇧V); entropy + prefix filter; **toggle in sidebar** |
| **Capture** | Global shortcut, command palette, hide on blur |
| **Themes** | Dark, light, system |

---

## Quick start

### Developers

**Prerequisites:** Node.js LTS, Rust stable, [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/)

```bash
cd 05_apps_and_extensions/gnomad-slate
npm install
npm run tauri dev      # Desktop + HMR (port 1420)
npm test               # Vitest (4 tests)
cd src-tauri && cargo test   # Rust (14 tests)
```

**Production build:** `npm run tauri build`

### Users

See [`docs/USER_GUIDE.md`](docs/USER_GUIDE.md) · [HTML](docs/USER_GUIDE.html) · [TXT](docs/USER_GUIDE.txt)

**Vault security (passwords & encryption):** [`docs/VAULT_SECURITY.md`](docs/VAULT_SECURITY.md) · [HTML](docs/VAULT_SECURITY.html)

---

## Architecture (summary)

```
┌─────────────────────────────────────────────────────────┐
│  React UI — notes, markdown preview, vault metadata    │
└───────────────────────────┬─────────────────────────────┘
                            │ Tauri invoke
┌───────────────────────────▼─────────────────────────────┐
│  Rust — crypto, vault session, notes FS, clipboard     │
└─────────────────────────────────────────────────────────┘
```

Deep dive: [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) · [`docs/TECH_STACK.md`](docs/TECH_STACK.md)

---

## Documentation index

All docs ship as **Markdown**, **HTML**, and **TXT**. Full index: [`docs/DOCS_INDEX.md`](docs/DOCS_INDEX.md). Regenerate: `npm run docs:export`.

| Document | MD | HTML |
|----------|----|------|
| **Project status** | [docs/STATUS.md](docs/STATUS.md) | [docs/STATUS.html](docs/STATUS.html) |
| **Roadmap** | [docs/ROADMAP.md](docs/ROADMAP.md) | [docs/ROADMAP.html](docs/ROADMAP.html) |
| **All docs** | [docs/DOCS_INDEX.md](docs/DOCS_INDEX.md) | [docs/DOCS_INDEX.html](docs/DOCS_INDEX.html) |
| **Project site** | — | [docs/index.html](docs/index.html) |
| User Guide | [docs/USER_GUIDE.md](docs/USER_GUIDE.md) | [docs/USER_GUIDE.html](docs/USER_GUIDE.html) |
| **Vault & Password Security** | [docs/VAULT_SECURITY.md](docs/VAULT_SECURITY.md) | [docs/VAULT_SECURITY.html](docs/VAULT_SECURITY.html) |
| Architecture | [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | [docs/ARCHITECTURE.html](docs/ARCHITECTURE.html) |
| Test Strategy | [docs/TEST_STRATEGY.md](docs/TEST_STRATEGY.md) | [docs/TEST_STRATEGY.html](docs/TEST_STRATEGY.html) |
| Build Platforms | [docs/BUILD_PLATFORMS.md](docs/BUILD_PLATFORMS.md) | [docs/BUILD_PLATFORMS.html](docs/BUILD_PLATFORMS.html) |
| Changelog | [CHANGELOG.md](CHANGELOG.md) | [CHANGELOG.html](CHANGELOG.html) |

---

## License

Private project — see repository settings. Contact [Gnomad Studio](https://gnomadstudio.org) for licensing.

---

## Acknowledgments

Built with [Tauri](https://v2.tauri.app/), [React](https://react.dev/), [Vite](https://vite.dev/), and [marked](https://marked.js.org/).

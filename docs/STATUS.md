# Project Status — Gnomad Slate

**macOS / Linux:** 2.1.0-beta.4 · **Windows:** 1.0.0-alpha.1  
**Phase:** Beta (mac/linux) · Alpha (Windows) — installers on GitHub Releases + Pages  
**Last updated:** 24 June 2026  
**Roadmap:** [ROADMAP.md](ROADMAP.md) · **Platform trees:** [PLATFORM_LAYOUT.md](PLATFORM_LAYOUT.md)

---

## At a glance

| Field | Value |
|-------|--------|
| **Product** | Desktop scratchpad + encrypted secrets vault (Vault-X) |
| **Stack** | Tauri 2 · React 19 · Vite 7 · Rust (Argon2id, AES-256-GCM) |
| **Repo** | [github.com/davidthegnomad/gnomad-slate](https://github.com/davidthegnomad/gnomad-slate) |
| **Default branch** | `main` |
| **Docs site** | [davidthegnomad.github.io/gnomad-slate](https://davidthegnomad.github.io/gnomad-slate/) |
| **CI** | PR tests + tag releases + GitHub Pages deploy |
| **Tests (root)** | 14 Rust · 5 Vitest — passing |
| **Tests (Windows)** | 17 Rust · 5 Vitest — passing |

---

## Platform status

| Platform | Code tree | Version | Status |
|----------|-----------|---------|--------|
| **macOS** | repository root | 2.1.0-beta.4 | Beta — `.dmg` / `.app.tar.gz` |
| **Linux** | repository root | 2.1.0-beta.4 | Beta — `.deb` / `.rpm` / AppImage |
| **Windows** | `slate_win64/` | 1.0.0-alpha.1 | Alpha — `.msi` / `.exe` |

macOS and Linux share one codebase. Windows is a **separate port** with its own menu, tray, and security hardening pass (see `reviews/WINDOWS_CONSOLIDATED_PLAN.md`).

---

## What works today

- **Scratchpad & vault notes** — Markdown buffers; scratchpad auto-archives after 48h
- **Vault-X** — Argon2id + AES-256-GCM; secrets stay in Rust; rate-limited unlock
- **Clipboard** — History capsule; entropy + prefix filter; sidebar toggle
- **Global summon** — Platform-specific shortcut; command palette
- **Windows port** — PR1–4 security fixes, Sidebar/EditorWorkstation split, DOMPurify preview

---

## Health check (24 June 2026)

| Check | Status |
|-------|--------|
| `cargo test` (root `src-tauri`) | ✓ 14 passed |
| `cargo test` (`slate_win64/src-tauri`) | ✓ 17 passed |
| `npm test` (both trees) | ✓ 5 passed |
| GitHub Actions release matrix | ✓ mac / linux / windows |
| GitHub Pages deploy | ✓ `docs/` on push to `main` |

---

## Next actions

1. Cut tagged release `v2.1.0-beta.4` (CI builds all platforms; Windows ships as `1.0.0-alpha.1`)
2. Smoke-test installers on clean VMs
3. Wire Windows menu `CheckMenuItem` sync for line numbers on startup
4. macOS notarization + Windows code signing (v1.0 GA)

Detail: **[ROADMAP.md](ROADMAP.md)**

---

Built with ❤️ by [Gnomad Studio](https://gnomadstudio.org) 🦙

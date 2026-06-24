# Platform layout — Gnomad Slate

**Last updated:** June 2026

Gnomad Slate ships as **two separate codebases** in one repository. Do not mix platform-specific Rust or menu code between trees.

---

## Repository map

| Tree | Platforms | Version channel | Build from |
|------|-----------|-----------------|------------|
| **Repository root** (`/`) | macOS, Linux | `2.1.0-beta.4` (beta) | `npm run tauri:build` |
| **`slate_win64/`** | Windows only | `1.0.0-alpha.1` (alpha) | `cd slate_win64 && npm run tauri:build` |

### Root (`macOS` + `Linux`)

- **Frontend:** `src/`
- **Rust backend:** `src-tauri/src/`
- **Platform menus:** `src-tauri/src/menu/macos.rs`, `menu/linux.rs`
- **macOS clipboard:** `pasteboard_macos.rs`
- **Linux session / Wayland fixes:** startup env in `lib.rs`

### Windows (`slate_win64/`)

- **Frontend:** `slate_win64/src/`
- **Rust backend:** `slate_win64/src-tauri/src/`
- **Windows menu / tray / clipboard:** `menu/windows/`, `windows_tray.rs`, `windows_clipboard.rs`
- **Security hardening (PR1–4):** vault I/O, clipboard race fix, DOMPurify, `spawn_blocking` unlock

---

## Version policy

| OS | Semver | Label |
|----|--------|-------|
| macOS | `2.1.0-beta.4` | Beta |
| Linux | `2.1.0-beta.4` | Beta (alpha-quality desktop) |
| Windows | `1.0.0-alpha.1` | Alpha |

Windows is on a separate alpha line while the Win64 port stabilizes. macOS and Linux share the root beta line.

---

## Downloads

Installers are published to [GitHub Releases](https://github.com/davidthegnomad/gnomad-slate/releases). The [documentation site](https://davidthegnomad.github.io/gnomad-slate/) links to the current artifacts.

| Platform | Typical artifacts |
|----------|-------------------|
| **macOS** | `.dmg`, `.app.tar.gz` (Apple Silicon + Intel) |
| **Linux** | `.deb`, `.rpm`, AppImage |
| **Windows** | `.exe` (NSIS) from `slate_win64` builds — MSI ships at `1.0.0` stable (WiX rejects alpha semver) |

---

## Merging changes across trees

When fixing shared behavior (vault crypto, markdown preview, palette):

1. Land the change in the tree you are testing on.
2. Port manually to the other tree if the feature applies to both.
3. Run tests in **both** trees before release:
   - Root: `npm test` + `cd src-tauri && cargo test` (14 Rust)
   - Windows: `cd slate_win64 && npm test` + `cd src-tauri && cargo test` (17 Rust)

---

## CI

- **PR / `main`:** `.github/workflows/build-test.yml` — root mac/linux tree
- **Tags `v*`:** `.github/workflows/release.yml` — matrix: macOS (arm64 + x64), Linux, Windows (`slate_win64`)
- **Docs site:** `.github/workflows/pages.yml` — deploys `docs/` to GitHub Pages

---

Built with ❤️ by [Gnomad Studio](https://gnomadstudio.org) 🦙

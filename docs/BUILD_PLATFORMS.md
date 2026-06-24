# Building Gnomad Slate for macOS, Linux, and Windows

**Last updated:** June 2026

---

## Platform trees (read first)

macOS and Linux build from the **repository root**. Windows builds from **`slate_win64/`** only.

See [PLATFORM_LAYOUT.md](PLATFORM_LAYOUT.md) for the full map and merge policy.

| Tree | OS | Version |
|------|-----|---------|
| `/` (root) | macOS, Linux | `2.1.0-beta.4` |
| `slate_win64/` | Windows | `1.0.0-alpha.1` |

---

## Prerequisites

- Node.js LTS (22 recommended)
- Rust stable (`rustup`)
- [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/)
- **Windows:** Visual Studio Build Tools (MSVC) + WebView2

```bash
cd gnomad-slate          # macOS / Linux
npm install
npm run build
```

```powershell
cd slate_win64           # Windows only
npm install
npm run build
```

---

## Development

### macOS / Linux (root)

```bash
npm run tauri:dev        # Wayland default on Linux
npm run tauri:dev:x11    # Linux WebKit/GPU fallback (X11)
npm test
cd src-tauri && cargo test   # 14 Rust tests
```

### Windows (`slate_win64`)

```powershell
npm run tauri:dev
npm test
cd src-tauri; cargo test     # 17 Rust tests
```

---

## Production build

```bash
# macOS / Linux
npm run tauri:build

# Windows (separate tree)
cd slate_win64 && npm run tauri:build
```

| Platform | Tree | Artifacts |
|----------|------|-----------|
| **macOS** | root | `.app`, `.dmg`, `.app.tar.gz` |
| **Linux** | root | `.deb`, `.rpm`, AppImage |
| **Windows** | `slate_win64` | `.exe` (NSIS) |

---

## Downloads

Pre-built installers: [GitHub Releases](https://github.com/davidthegnomad/gnomad-slate/releases) · [GitHub Pages landing](https://davidthegnomad.github.io/gnomad-slate/)

CI publishes all platforms on `v*` tags via `.github/workflows/release.yml`.

---

## Platform notes

### Summon shortcut

| OS | Shortcut |
|----|----------|
| macOS | `Option+Space` |
| Windows | `Alt+Space` |
| Linux | `Super+Shift+Space` |

### Linux dependencies

```bash
bash scripts/install-linux-deps.sh
```

Requires **webkit2gtk 4.1** and **libsoup3** dev headers (Tauri 2). See [LINUX_DESKTOP.md](LINUX_DESKTOP.md).

### macOS

- Hide-on-blur when losing focus (launcher behavior)
- Pasteboard `changeCount` reduces idle clipboard polling

### Windows

- Separate codebase under `slate_win64/` (alpha `1.0.0-alpha.1`)
- WebView2 Evergreen bootstrapper in `slate_win64/src-tauri/tauri.conf.json`
- Window stays visible when clicking away (no hide-on-blur)

---

## CI

- **PR/push:** `.github/workflows/build-test.yml` (root tree)
- **Tags `v*`:** `.github/workflows/release.yml` (macOS arm64 + x86_64, Linux, Windows from `slate_win64`)
- **Docs site:** `.github/workflows/pages.yml` → GitHub Pages (`docs/`)

Releases publish as **pre-releases** until marked stable.

See [RELEASE_RUNBOOK.md](RELEASE_RUNBOOK.md) and [CROSS_PLATFORM_CODE_REVIEW.md](CROSS_PLATFORM_CODE_REVIEW.md).

---

Built with ❤️ by [Gnomad Studio](https://gnomadstudio.org) 🦙

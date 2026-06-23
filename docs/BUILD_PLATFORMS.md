# Building Gnomad Slate for macOS, Linux, and Windows

**Version:** 0.1.0

---

## Prerequisites

- Node.js LTS (22 recommended)
- Rust stable (`rustup`)
- [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/)

```bash
cd 05_apps_and_extensions/gnomad-slate
npm install
npm run build
```

---

## Development

```bash
# Desktop (Vite on :1420 + Tauri) — Wayland default on Linux
npm run tauri:dev

# Linux WebKit/GPU fallback (X11)
npm run tauri:dev:x11

# Frontend only
npm run dev

# Tests
npm test
cd src-tauri && cargo test
```

---

## Production build

```bash
npm run tauri:build
# CI (signing via env when configured)
npm run tauri:build:ci
```

| Platform | Artifacts |
|----------|-----------|
| **macOS** | `.app`, `.dmg` |
| **Windows** | `.msi`, `.exe` |
| **Linux** | `.deb`, `.rpm`, AppImage |

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
- WebView2 Evergreen bootstrapper configured in `tauri.conf.json`
- Window stays visible when clicking away (no hide-on-blur)

---

## CI

- **PR/push:** `.github/workflows/build-test.yml`
- **Tags `v*`:** `.github/workflows/release.yml` (macOS arm64 + x86_64, Linux, Windows)

Releases publish as **pre-releases** until marked stable.

See [RELEASE_RUNBOOK.md](RELEASE_RUNBOOK.md) and [CROSS_PLATFORM_CODE_REVIEW.md](CROSS_PLATFORM_CODE_REVIEW.md).

---

Built with ❤️ by [Gnomad Studio](https://gnomadstudio.org) 🦙

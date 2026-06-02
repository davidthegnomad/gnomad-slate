# Building Gnomad Slate for macOS, Linux, and Windows

**Version:** 0.1.0

---

## Prerequisites

- Node.js LTS
- Rust stable (`rustup`)
- [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/) for your OS

```bash
cd 05_apps_and_extensions/gnomad-slate
npm install
npm run build
```

---

## Development

```bash
# Desktop (Vite on :1420 + Tauri shell)
npm run tauri dev

# Frontend only
npm run dev

# Tests
npm test                    # Vitest (4 tests)
cd src-tauri && cargo test  # Rust (14 tests)

# Regenerate docs HTML/TXT
npm run docs:export
```

---

## Production build

```bash
npm run tauri build
```

Artifacts under `src-tauri/target/release/bundle/`:

| Platform | Output |
|----------|--------|
| **macOS** | `.app`, `.dmg` |
| **Windows** | `.msi`, `.exe` |
| **Linux** | `.deb`, `.rpm`, AppImage |

Build on the **target OS** for reliable results. macOS builds link `objc2-app-kit` for pasteboard `changeCount`.

---

## Repository

**GitHub:** [github.com/davidthegnomad/gnomad-slate](https://github.com/davidthegnomad/gnomad-slate) (private)

---

Built with ❤️ by [Gnomad Studio](https://gnomadstudio.org) 🦙

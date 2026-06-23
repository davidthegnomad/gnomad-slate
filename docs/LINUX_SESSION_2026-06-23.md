# Gnomad Slate — Linux Desktop Session Log

**Date:** 23 June 2026  
**Branch:** `feat/linux-desktop` → merged to `main` as **v2.1.0-beta.3**  
**PR:** [#1 — feat(linux): desktop integration for Nobara/Fedora (alpha)](https://github.com/davidthegnomad/gnomad-slate/pull/1)  
**Target machine:** Nobara 43 (Fedora family), Wayland/X11 hybrid  
**Release channel:** Linux **alpha** (macOS remains **beta**)

---

## Executive summary

Today we brought Gnomad Slate from a macOS-focused beta to a **first-class Linux desktop build** for Nobara/Fedora, without forking the repo. Work spanned Rust shell integration (tray, menus, shortcuts, quit lifecycle), Linux-only frontend UX, packaging (RPM/DEB, `gnomad-slate` binary), multi-model code review, local dogfooding, and a production quit/close fix verified on the user's machine.

**Shipped outcome:** `v2.1.0-beta.3` with Linux `.rpm` / `.deb` on GitHub Releases; installable via `dnf install` on Fedora/Nobara.

---

## Timeline

| Phase | What happened |
|-------|----------------|
| **Morning** | GLM P0/P1/P2 reviews; implemented tray, `Super+Shift+Space`, platform-split menus, CSP, plugin-store, atomic vault writes |
| **Midday** | Professional workflow: feature branch, 4 structured commits, PR #1, CI green (`build-test.yml`) |
| **Afternoon** | User UX requests (Linux-only): filename on own line, line numbers, help content, tooltip placement, version sync |
| **Local install** | Built RPM from repo (no Linux assets on GitHub yet); installed `2.1.0-beta.2` → later `beta.3` with fixes |
| **Evening** | Kimi K2.6 + Qwen 3.5 (NIM) reviews flagged quit lifecycle; implemented `confirm_quit` unified flow |
| **Final** | User reported app would not close; root-caused `isLinux` mount race + missing File → Close; fixed and verified |

---

## Architecture decisions

### Single repo, platform split (no Linux fork)

| Layer | macOS | Linux |
|-------|-------|-------|
| Menus | `src-tauri/src/menu/macos.rs` | `src-tauri/src/menu/linux.rs` |
| Tray | macOS native patterns | `src-tauri/src/linux_tray.rs` |
| Hide-on-blur | Yes (launcher behavior) | **No** — window stays until user hides/quits |
| Summon shortcut | `Option+Space` | `Super+Shift+Space` |
| Version label | `v2.1 beta` style | `v{semver}` from `CARGO_PKG_VERSION` |
| Release channel | beta | alpha |

Frontend gates Linux UI via `src/utils/platform.ts` → `isLinuxDesktop()` (`@tauri-apps/plugin-os`).

### Quit lifecycle (critical fix)

**Problem:** Multiple quit paths (`request_quit`, tray, window X, GTK predefined Quit) raced with frontend save/cleanup. On Linux, `win.destroy()` without `app.exit(0)` left a **zombie process** because the global shortcut plugin keeps the Tokio/event loop alive.

**Solution — unified flow:**

```
Tray Quit / File → Quit / app-quit-requested
  → frontend: save if needed, vault cleanup
  → IPC confirm_quit
  → vault_session::close_session
  → app.exit(0)
  → std::process::exit(0)   # Linux hard kill for shortcut plugin

File → Close / window X / Super+Shift+Space hide
  → window-close-requested
  → hide window (Linux) — process stays for tray summon
```

**Second bug (user report):** `isLinux` was captured once in `useState(() => isLinuxDesktop())`. If `plugin-os` was not ready at mount, `closeApp()` called `win.destroy()` instead of `confirm_quit` on quit paths. **Fix:** call `isLinuxDesktop()` at invocation time; hydrate `isLinux` in `useEffect`.

---

## Commits on `feat/linux-desktop` (6 + release)

| Commit | Summary |
|--------|---------|
| `e27f5e7` | Rust shell: tray, `menu/{linux,macos,fallback}.rs`, shortcuts, quit hooks, logging |
| `94d2cfc` | Frontend: Linux-gated toolbar, help dialog, line numbers, `AppMessageDialog`, close handlers |
| `f3bf64b` | Packaging: `gnomad-slate` binary, RPM/DEB desktop template, libsoup3 deps, CSP, plugin-store |
| `f091f2c` | Docs/CI: `LINUX_DESKTOP.md`, `BUILD_PLATFORMS.md`, review artifacts, CI libsoup3 |
| `48bd04f` | Replace `request_quit` with backend `confirm_quit`; Kimi/Qwen review artifacts |
| `8c47cd4` | File → Close (`Ctrl+W`), File → Quit (`Ctrl+Q`); runtime `isLinuxDesktop()`; Linux `process::exit` |
| *(release)* | `2.1.0-beta.3`: version bump, CHANGELOG, session log, release CI signing fix |

---

## Rust / Tauri changes (detail)

### New / split modules

- `src-tauri/src/menu/mod.rs` — dispatches to `linux` / `macos` / `fallback`
- `src-tauri/src/menu/linux.rs` — File, Edit, View, Window, Help; Ctrl accelerators; no GTK predefined Quit
- `src-tauri/src/linux_tray.rs` — Show / Hide / Quit; left-click show (Wayland-friendly)
- `src-tauri/src/shortcuts.rs` — `SUMMON_SHORTCUT` constant (`Super+Shift+Space` on Linux)
- `src-tauri/src/logging.rs` — `tracing` + `EnvFilter`

### `lib.rs` commands & behavior

- **`confirm_quit`** — idempotent via `QUIT_IN_PROGRESS`; locks vault; `app.exit(0)` + `std::process::exit(0)` on Linux
- **`emit_quit_requested`** — tray/menu → `app-quit-requested` event to frontend
- **`get_app_version_info`** — channel `alpha` on Linux, `beta` on macOS; display version matches package semver on Linux
- **`prepare_for_quit`** — vault lock helper (frontend calls before final quit)
- Hide-on-blur gated with `#[cfg(target_os = "macos")]`

### Linux File menu (final)

| Item | Accelerator | Action |
|------|-------------|--------|
| Close | `Ctrl+W` | `window-close-requested` → hide |
| Quit Gnomad Slate | `Ctrl+Q` | `emit_quit_requested` → full exit |

Removed `PredefinedMenuItem::quit` (bypassed our lifecycle).

### Packaging (`tauri.conf.json`)

- `mainBinaryName`: `gnomad-slate`
- Linux RPM/DEB depends: `webkit2gtk4.1`, `libsoup3`, `gtk3`, `libappindicator-gtk3`
- Desktop template: `bundle/linux/gnomad-slate.desktop.hbs` (`StartupWMClass=gnomad-slate`)
- Strict CSP (no inline scripts except `'unsafe-inline'` for styles)

### Filesystem

- Atomic vault/note writes (temp + rename)
- Data dir: `~/.local/share/gnomad-slate/` (`notes/`, `vault.enc`, `archive/`)
- Settings: `tauri-plugin-store` → app config dir `settings.json`

---

## Frontend changes (Linux-gated)

| Feature | Files | Notes |
|---------|-------|-------|
| Filename below toolbar | `App.tsx`, `App.css` | `.linux-toolbar` layout; Edit/Split/Preview on row 1, filename row 2 |
| Line numbers toggle | `App.tsx`, View menu | Persisted via plugin-store |
| Help dialog | `src/content/linuxHelp.ts`, F1 | Sections: getting started, shortcuts, vault, clipboard, support **david@gnomadstudio.org** |
| In-app dialogs | `AppMessageDialog.tsx` | Replaces `alert()` on Linux (WebKitGTK focus) |
| Tooltips below buttons | `App.css` | `tooltip-position: below` for mode buttons |
| Version sync | About dialog | `get_app_version_info` IPC — UI matches `package.json` / `Cargo.toml` |
| Close / quit | `App.tsx`, `ipc.ts` | `onCloseRequested`, `window-close-requested`, `app-quit-requested` listeners |

### Dev script

```bash
npm run tauri:dev:x11   # GDK_BACKEND=x11 WEBKIT_DISABLE_DMABUF_RENDERER=1 tauri dev
```

---

## AI code reviews (consensus)

| Reviewer | Model | Verdict | Key finding |
|----------|-------|---------|-------------|
| GLM 5.1 | P0 | PASS w/ fixes | Quit race, `alert()` on Linux |
| GLM 5.1 | P2 | PASS | Tray, shortcuts, CSP |
| Kimi K2.6 | NIM | PASS w/ fixes | Unify quit via `confirm_quit` |
| Qwen 3.5 | NIM | PASS w/ fixes | Same quit lifecycle |
| Nemotron | — | PASS w/ notes | Platform gating |
| OpenRouter Qwen Coder | — | **Blocked** | 429 rate limit |

Artifacts: `reviews/LINUX_REVIEW_*.md`, `reviews/LINUX_P0_REVIEW_glm-5.1.md`, `reviews/LINUX_P2_REVIEW_glm-5.1.md`

---

## Local build & install (Nobara)

### Dependencies

```bash
bash scripts/install-linux-deps.sh   # dnf: webkit2gtk4.1-devel, libsoup3-devel, etc.
npm install
npm run tauri:build                  # produces .rpm + .deb (AppImage may fail without linuxdeploy locally)
```

### Install RPM

```bash
sudo dnf install "./src-tauri/target/release/bundle/rpm/Gnomad Slate-2.1.0-beta.3-1.x86_64.rpm"
```

Binary: `/usr/bin/gnomad-slate`  
Desktop entry: **Gnomad Slate** in application menu

### Launch workaround (Wayland / WebKit)

From agent shell, pure Wayland launch failed (`Gdk Error 71`). User desktop works with:

```bash
GDK_BACKEND=x11 WEBKIT_DISABLE_DMABUF_RENDERER=1 gnomad-slate
```

Optional: wrap in `~/.local/bin/gnomad-slate` or patch `.desktop` `Exec=` line.

### Verify quit

```bash
pgrep -a gnomad-slate          # after File → Close: still running (expected)
# File → Quit or tray Quit:
pgrep gnomad-slate || echo OK  # should print OK
```

---

## CI / release

| Workflow | Trigger | Status |
|----------|---------|--------|
| `build-test.yml` | PR / push | ✅ Green on `feat/linux-desktop` |
| `release.yml` | tag `v*` | Fixed: `createUpdaterArtifacts: false` until `TAURI_SIGNING_PRIVATE_KEY` secret configured (beta.2 Linux job failed on missing signing key after successful RPM/DEB build) |

**v2.1.0-beta.2** release had macOS assets only (CI Linux failed at updater signing). **v2.1.0-beta.3** publishes Linux RPM/DEB (+ AppImage on Ubuntu runner).

---

## Known limitations

1. **AppImage** — local build may fail without `linuxdeploy`; CI builds it on Ubuntu.
2. **Auto-updater** — Linux alpha; updater artifacts disabled in CI until signing secret is added.
3. **Wayland** — global shortcut registration is compositor-dependent; tray is fallback.
4. **Hide vs quit** — Close / X hides; only Quit exits (by design for summon launcher).

---

## Test matrix (run before release)

```bash
npm test                    # Vitest (7 tests)
cd src-tauri && cargo test  # 15 Rust tests
npm run build
npm run tauri:build         # on Linux host
```

Manual: summon, tray, save-on-close prompt, vault lock on quit, Help F1, line numbers, theme persist.

---

## References

- [LINUX_DESKTOP.md](LINUX_DESKTOP.md) — user-facing Linux guide
- [BUILD_PLATFORMS.md](BUILD_PLATFORMS.md) — cross-platform build
- [VAULT_SECURITY.md](VAULT_SECURITY.md) — vault crypto (unchanged semantics)
- [CHANGELOG.md](../CHANGELOG.md) — v2.1.0-beta.3 entry

---

Built with ❤️ by [Gnomad Studio](https://gnomadstudio.org) 🦙

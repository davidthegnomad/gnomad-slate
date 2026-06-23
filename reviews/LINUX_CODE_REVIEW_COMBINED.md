<!-- synthesized from LINUX_REVIEW_nemotron-ultra.md + LINUX_REVIEW_glm-5.2.md -->
<!-- note: merged locally; future runs use NVIDIA NIM via reviews/run-linux-multi-review.py -->
<!-- generated: 2026-06-23 -->

# Gnomad Slate — Combined Linux Code Review

**Sources:** Nemotron 3 Ultra + GLM 5.2 (OpenRouter runs)  
**Scope:** Linux desktop only (`menu/linux.rs`, Linux-gated UI, Tauri lifecycle, RPM packaging)  
**Target:** Nobara / Fedora, Wayland-first

---

## Executive summary

- **Both reviewers agree:** the core vault/crypto and note logic are sound; problems are in **desktop integration** (Wayland focus, global shortcuts, hide-on-blur, quit/close lifecycle).
- **Hide-on-blur + `Alt+Space`** are the biggest UX risks on Linux DEs — unreliable on Wayland and conflicts with window managers.
- **macOS code leaks into Linux** (`maybe_show_on_app_activate` / `macos_app`) and must be gated or removed.
- **`alert()` in the React UI** blocks WebKitGTK and breaks focus on Wayland — replace with in-app modals.
- **Packaging is incomplete:** `bundle.targets: "all"` lacks Linux `.desktop`, hicolor icons, MIME types, and `libsoup3` deps.
- **Rebuild verdict split:** Nemotron recommends rewriting the Tauri *glue layer* (window/tray/shortcuts); GLM says **keep & patch**. Consensus: **partial rewrite of OS integration**, keep crypto/vault/filesystem crates.

---

## Recommended changes (master list)

| Priority | Change (what to do) | Why (rationale) | File(s) | Reviewers |
|----------|---------------------|-----------------|---------|-----------|
| P0 | Gate `maybe_show_on_app_activate` and `macos_app` with `#[cfg(target_os = "macos")]` | macOS-only API runs on Linux `MainEventsCleared` loop | `src-tauri/src/lib.rs` | Both |
| P0 | Replace hide-on-blur thread sleep with Wayland-safe approach (or remove on Wayland) | Focus events are async/unreliable; causes random hide/show | `src-tauri/src/lib.rs` | Both |
| P0 | Change default global shortcut from `Alt+Space` to `Super+Shift+Space` (configurable) | Conflicts with KDE/GNOME/tiling WMs; registration may fail | `src-tauri/src/lib.rs`, `menu/linux.rs` | Both |
| P0 | Add system tray / status icon with Show + Quit | When hidden, user has no recovery if shortcut fails | `src-tauri/` (new tray plugin) | Nemotron |
| P0 | Fix close/quit lifecycle: `CloseRequested` → frontend prompt → graceful exit; Quit must not bypass vault lock | `destroy()` + `exit(0)` skips cleanup; close modal races window destroy | `src-tauri/src/lib.rs`, `src/App.tsx`, `src/utils/ipc.ts` | Both |
| P0 | Replace all `alert()` with React modal or `@tauri-apps/plugin-dialog` | GTK alert dialogs block WebKit and steal focus on Wayland | `src/App.tsx` | GLM |
| P0 | Remove `win.setSize()` on mount; set size in `tauri.conf.json` | Causes geometry flicker on Wayland | `src/App.tsx`, `tauri.conf.json` | GLM |
| P1 | Add `libsoup3-devel` / `libsoup-3.0-dev` to install script | WebKit2GTK 4.1 requires libsoup3; fresh Fedora build fails | `scripts/install-linux-deps.sh` | GLM |
| P1 | Complete Linux bundle config: `.desktop`, hicolor icons, MIME, RPM metadata | Current RPM installs as `tauri-app` with minimal desktop integration | `tauri.conf.json`, icons | Nemotron |
| P1 | Use `appWindow.onCloseRequested` + `event.preventDefault()` pattern (Tauri 2) | Custom `window-close-requested` emit may not block close | `src/App.tsx`, `src-tauri/src/lib.rs` | Nemotron |
| P1 | Strip browser-extension code (`chrome.runtime`, `navigator.clipboard`) from Tauri path | Dead code; fails silently; misses PRIMARY selection on Linux | `src/App.tsx`, `src/utils/ipc.ts` | Both |
| P1 | Gate `win.setTheme()` to macOS only | No-op / warnings on Linux GTK | `src/App.tsx` | Nemotron |
| P1 | Register menu accelerators as `Ctrl+*` on Linux, not `CmdOrCtrl` labels | Menu shows meaningless "Cmd" on Linux | `src-tauri/src/menu/linux.rs` | Nemotron |
| P1 | Atomic vault writes (`vault.enc.tmp` → rename) | Power loss mid-write corrupts encrypted vault | `src-tauri/src/lib.rs` | Nemotron |
| P1 | Verify XDG paths (`dirs::data_local_dir()`) for notes/vault | Linux users expect `~/.local/share/gnomad-slate` | `src-tauri/src/filesystem.rs` | Nemotron |
| P2 | Add strict CSP instead of `null` | Reduces XSS risk in WebKit view | `tauri.conf.json` | Both |
| P2 | Replace `eprintln!` with `tracing` → journalctl | Easier debugging on systemd distros | `src-tauri/src/lib.rs` | GLM |
| P2 | Migrate settings from `localStorage` to `@tauri-apps/plugin-store` | WebKit storage can be cleared by system tools | `src/App.tsx` | GLM |
| P2 | Use `modShortcut()` everywhere (palette shows `⌘` on Linux today) | Inconsistent shortcut labels | `src/App.tsx` | GLM |
| P2 | Debounce sidebar search; consolidate duplicate save paths | Perf/maintainability | `src/App.tsx` | Nemotron |
| P2 | Rename binary from `tauri-app` to `gnomad-slate` in Cargo/package | RPM ships wrong binary name for users | `src-tauri/Cargo.toml`, `tauri.conf.json` | (implied by packaging) |

---

## Consensus themes

1. **Wayland changes the rules** — focus stealing, hide-on-blur, and client-side global shortcuts do not behave like macOS. Linux needs tray icon + compositor-friendly show/hide.
2. **Shortcut choice matters** — `Alt+Space` is a non-starter on most Linux DEs; make it configurable with a safe default.
3. **Native desktop polish** — `.desktop` file, correct `StartupWMClass`, icon theme sizes, and RPM/DEB metadata are required for “real app” feel on Nobara.
4. **Frontend must stop acting like a browser extension** — `alert()`, `navigator.clipboard`, and Chrome bridge code are wrong for Tauri on Linux.
5. **Close = save + lock vault + exit process** — Linux keeps process alive after window destroy (global shortcut plugin); must call `process.exit` only after cleanup.

---

## Disagreements

| Topic | Nemotron Ultra | GLM 5.2 |
|-------|----------------|---------|
| **Rebuild scope** | Rewrite Tauri glue layer (window/tray/shortcuts); keep crypto/vault crates | Keep & patch; no full rewrite |
| **Global shortcuts in frontend** | Must move `⌘P`, `⌘S`, etc. to Rust `GlobalShortcut` | Notes Wayland limits; less prescriptive on moving all shortcuts |
| **Hide-on-blur** | Replace with layer-shell or GTK timeout | Remove entirely on Wayland; rely on shortcut/tray |

**Recommended path:** Partial rewrite — new `linux_shell.rs` (or module) owning window visibility, tray, shortcuts, and close lifecycle; keep existing vault/notes/crypto.

---

## Rebuild recommendation

**Verdict: Partial Linux-first rewrite of OS integration (not a full app rewrite)**

**Rationale:** Business logic (Vault-X, notes FS, crypto, clipboard filter) is solid and test-covered. What’s broken is the **launcher pattern** on Linux: hide-on-blur, shortcut conflicts, no tray, wrong quit semantics, and weak packaging. A focused 2–3 sprint refactor of `lib.rs` window events + new tray plugin + packaging config delivers more than rewriting React.

---

## Implementation order

### Week 1 (P0)
1. Fix macOS leak in `lib.rs` (`#[cfg]` gate)
2. Remove or Wayland-gate hide-on-blur
3. Change global shortcut default + conflict detection
4. Fix close/quit flow (modal → save → vault lock → `exit`)
5. Replace `alert()` with in-app dialogs
6. Remove mount-time `setSize()`

### Week 2–3 (P1)
1. Add system tray (AppIndicator3 / `tauri-plugin-tray-icon`)
2. Complete `tauri.conf.json` Linux bundle + `.desktop` + icons
3. Update `install-linux-deps.sh` (libsoup3)
4. Strip extension clipboard bridge; use Rust clipboard only
5. Fix menu accelerators and binary naming (`gnomad-slate`)

### Later (P2)
1. CSP, tracing/journalctl, plugin-store for settings
2. Atomic vault writes, search debounce, command palette shortcut cleanup

---

## Individual reports

- [`LINUX_REVIEW_nemotron-ultra.md`](LINUX_REVIEW_nemotron-ultra.md)
- [`LINUX_REVIEW_glm-5.2.md`](LINUX_REVIEW_glm-5.2.md)

## Next review runs

Use NVIDIA NIM (not OpenRouter):

```bash
cd reviews
python3 run-linux-multi-review.py --synthesis-only   # merge existing reports
python3 run-linux-multi-review.py --resume           # full re-review via NIM
```

Default synthesis model: `nvidia/nemotron-3-ultra-550b-a55b` @ `integrate.api.nvidia.com`

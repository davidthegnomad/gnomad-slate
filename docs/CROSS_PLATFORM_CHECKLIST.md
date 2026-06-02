# Cross-Platform Checklist — Gnomad Slate

**Version:** 0.1.0

---

## All platforms

- [ ] `npm test` — 4 Vitest tests pass
- [ ] `cd src-tauri && cargo test` — 14 Rust tests pass
- [ ] `npm run build` succeeds
- [ ] Edit / Split / Preview modes work
- [ ] Clipboard filter toggle syncs to backend
- [ ] Vault unlock / lock / add / copy / delete

---

## macOS (primary dev)

- [ ] `npm run tauri dev` — hot reload
- [ ] `npm run tauri build` — `.app` + `.dmg`
- [ ] Option+Space shortcut
- [ ] Pasteboard `changeCount` skips idle reads (lower CPU when unchanged)
- [ ] App data: `~/Library/Application Support/gnomad-slate/`
- [ ] Legacy migration from `~/.gemini/.../slate_data` if present

---

## Windows

- [ ] Build on Windows VM or runner
- [ ] Global shortcut (Alt+Space fallback)
- [ ] App data: `%APPDATA%\gnomad-slate\`
- [ ] WebView2 installed
- [ ] Adaptive clipboard polling (no changeCount — 1s/3s backoff)

---

## Linux

- [ ] webkit2gtk dependencies
- [ ] `npm run tauri build` — deb/AppImage
- [ ] Clipboard via arboard
- [ ] App data: `~/.local/share/gnomad-slate/` (typical)

---

See [BUILD_PLATFORMS.md](BUILD_PLATFORMS.md).

---

Built with ❤️ by [Gnomad Studio](https://gnomadstudio.org) 🦙

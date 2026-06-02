# Troubleshooting — Gnomad Slate

**Version:** 0.1.0

---

## App won't start

| Symptom | Fix |
|---------|-----|
| Rust compile errors | `rustup update stable`; install [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) |
| Port 1420 in use | Stop other Vite processes or change port in `vite.config.ts` + `tauri.conf.json` |
| macOS objc2 build errors | Ensure Xcode CLT installed; `cargo clean` and rebuild |

---

## Global shortcut doesn't work

- Confirm **Option+Space** isn't taken by another app.
- Check terminal for `failed to register global shortcut` on startup.
- On Linux, shortcut may differ — see `src-tauri/src/lib.rs`.

---

## Vault issues

| Symptom | Fix |
|---------|-----|
| `Invalid password` | Verify caps lock; wait out 30s lockout after 5 failures |
| `Too many failed attempts` | Wait for countdown |
| Secrets empty after unlock | Initialize vault; check `vault.enc` in app data dir |
| Legacy vault won't open | Update to latest build; unlock once to trigger v2 migration |

**Data directory (macOS):** `~/Library/Application Support/gnomad-slate/`

---

## Notes not saving

- Title cannot be empty.
- Wait ~400 ms after typing for debounced save before quitting.
- Switching notes flushes pending save — use that before force-quit.

---

## Markdown preview

| Symptom | Fix |
|---------|-----|
| Preview blank | Ensure note has content; try **Split** mode |
| Raw HTML visible | Use **Preview** or **Split**, not **Edit** |
| Scripts in preview | Stripped by design; don't rely on preview for untrusted HTML |

---

## Clipboard history

| Symptom | Fix |
|---------|-----|
| History empty | macOS Privacy → allow clipboard access |
| Expected item missing | May be filtered — disable **Filter sensitive clipboard items** in sidebar |
| Sensitive item appeared | Filter off or heuristic miss — use Vault-X for secrets |

---

## Tests & docs

```bash
npm test
cd src-tauri && cargo test
npm run docs:export
```

---

Built with ❤️ by [Gnomad Studio](https://gnomadstudio.org) 🦙

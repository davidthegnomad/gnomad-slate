# QA Checklist — Gnomad Slate

**Version:** 0.1.0

---

## Automated (run before manual QA)

```bash
cd src-tauri && cargo test    # 14 tests
npm test                      # 4 tests
npm run build
```

---

## Smoke (every build)

- [ ] App launches (`npm run tauri dev`)
- [ ] Create scratch note — debounced autosave (~400 ms)
- [ ] Switch notes — pending save flushes
- [ ] **Edit / Split / Preview** modes render markdown
- [ ] Promote to vault note
- [ ] Option+Space show/hide
- [ ] ⌘P palette; Esc closes
- [ ] ⌘⇧V clipboard modal

---

## Clipboard filter

- [ ] Default: copy `ghp_…` token — **not** in history
- [ ] Copy email / camelCase class name — **appears** in history
- [ ] Disable **Filter sensitive clipboard items** — token appears
- [ ] Re-enable filter — new sensitive copies blocked

---

## Vault-X

- [ ] Initialize vault (password ≥ 8 chars)
- [ ] Add secret — list shows name only
- [ ] Copy — clipboard works; no full list in React state
- [ ] Show / Hide — value clears on Hide
- [ ] Lock vault — secrets cleared from UI
- [ ] Unlock — secrets restored
- [ ] 5 wrong passwords — lockout message
- [ ] [VAULT_SECURITY.md](VAULT_SECURITY.md) matches behavior

---

## Regression

- [ ] Path traversal blocked (`temp_../../../etc/passwd` via devtools invoke if tested)
- [ ] Legacy data path under `$HOME` only (not hardcoded username)

---

Built with ❤️ by [Gnomad Studio](https://gnomadstudio.org) 🦙

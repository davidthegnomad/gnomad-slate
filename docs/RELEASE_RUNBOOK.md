# Release Runbook — Gnomad Slate

**Version:** 0.1.0

---

## Pre-release

1. Update `version` in `package.json` and `src-tauri/tauri.conf.json`
2. Update [CHANGELOG.md](../CHANGELOG.md)
3. Run automated tests:
   ```bash
   npm test
   cd src-tauri && cargo test
   npm run build
   ```
4. Manual QA: [QA_CHECKLIST.md](QA_CHECKLIST.md)
5. Regenerate docs: `npm run docs:export`
6. Commit Markdown + exported HTML/TXT

---

## Tag & build (manual v0.1)

```bash
git tag v0.1.0
git push origin v0.1.0
npm run tauri build
```

Upload artifacts from `src-tauri/target/release/bundle/` to GitHub Releases.

---

## CI (planned)

Mirror [gnomad-preview](https://github.com/davidthegnomad/gnomad-preview) workflows:

- PR: `cargo test`, `npm test`, `npm run build`
- Tag: `tauri build` matrix (macOS, Windows, Linux)

---

## Post-release

- [ ] Verify installer on clean VM
- [ ] Spot-check vault migration, clipboard filter, markdown preview
- [ ] Update [HUMAN.md](../HUMAN.md) if signing/notarization added

---

Built with ❤️ by [Gnomad Studio](https://gnomadstudio.org) 🦙

# Release Runbook — Gnomad Slate

**Last updated:** June 2026

---

## Platform trees

| Tree | OS | Version field |
|------|-----|---------------|
| Root | macOS, Linux | `package.json` + `src-tauri/tauri.conf.json` → `2.1.0-beta.4` |
| `slate_win64/` | Windows | `slate_win64/package.json` + `slate_win64/src-tauri/tauri.conf.json` → `1.0.0-alpha.1` |

See [PLATFORM_LAYOUT.md](PLATFORM_LAYOUT.md).

---

## Pre-release

1. Bump versions in the correct tree(s)
2. Update [CHANGELOG.md](../CHANGELOG.md)
3. Run tests:
   ```bash
   npm test && cd src-tauri && cargo test
   cd slate_win64 && npm test && cd src-tauri && cargo test
   npm run build
   ```
4. Manual QA: [QA_CHECKLIST.md](QA_CHECKLIST.md)
5. Regenerate docs: `npm run docs:export`
6. Commit Markdown + exported HTML/TXT + `docs/index.html`

---

## Tag & CI release

```bash
git tag v2.1.0-beta.4
git push origin main
git push origin v2.1.0-beta.4
```

`.github/workflows/release.yml` builds:

- macOS arm64 + x64 (root)
- Linux (root)
- Windows (`slate_win64/` → `1.0.0-alpha.1` artifacts)

`.github/workflows/pages.yml` deploys `docs/` (download links on GitHub Pages).

---

## Post-release

- [ ] Verify installers on clean VM per platform
- [ ] Confirm [docs/index.html](index.html) download URLs match release asset names
- [ ] Spot-check vault, clipboard filter, markdown preview

---

Built with ❤️ by [Gnomad Studio](https://gnomadstudio.org) 🦙

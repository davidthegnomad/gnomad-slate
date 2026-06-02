# HUMAN.md — Actions only you can take

**Project:** Gnomad Slate · **Version:** 0.1.0  
**Last updated:** June 2026

Tasks that require a human owner — signing keys, release tags, legal sign-off.

---

## Priority order

| # | Task | Blocker for | Est. time |
|---|------|-------------|-----------|
| 1 | [First release tag](#1-first-release-tag) | Downloadable installers | ~5 min + build |
| 2 | [Run test suites](#2-run-test-suites) | Release confidence | ~2 min |
| 3 | [Review vault security doc](#3-review-vault-security-doc) | External trust | ~30 min |
| 3 | [Apple notarization](#3-apple-notarization) | Enterprise macOS | ~1 hr + $99/yr |
| 4 | [GitHub Actions release](#4-github-actions-release) | Automated builds | ~1 hr |

---

## 1. First release tag

```bash
cd 05_apps_and_extensions/gnomad-slate
npm run build
npm test
cd src-tauri && cargo test
npm run docs:export
npm run tauri build
git tag v0.1.0
git push origin v0.1.0
```

Upload `src-tauri/target/release/bundle/` artifacts to GitHub Releases.

**Docs:** [docs/RELEASE_RUNBOOK.md](docs/RELEASE_RUNBOOK.md)

---

## 2. Run test suites

```bash
npm test
cd src-tauri && cargo test
```

Expected: **14** Rust tests, **4** Vitest tests — all passing.

---

## 3. Review vault security doc

Before sharing the app or repo with reviewers, read and sign off:

- [docs/VAULT_SECURITY.md](docs/VAULT_SECURITY.md) (or [HTML](docs/VAULT_SECURITY.html))

Confirm messaging matches your risk tolerance for local-only credential storage.

---

## 4. Apple notarization

For distribution outside your own machine, follow the Gnomad Desktop Assistant pattern once Apple Developer secrets are in GitHub.

---

## 5. GitHub Actions release

Copy and adapt `gnomad-preview/.github/workflows/release.yml` when ready for matrix builds.

---

Built with ❤️ by [Gnomad Studio](https://gnomadstudio.org) 🦙

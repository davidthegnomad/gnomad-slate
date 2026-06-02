# Project Status — Gnomad Slate

**Version:** 0.1.0  
**Phase:** v0.1.0 shipped — development-ready, pre-distribution  
**Last updated:** 2 June 2026  
**Roadmap:** [ROADMAP.md](ROADMAP.md)

---

## At a glance

| Field | Value |
|-------|--------|
| **Product** | Desktop scratchpad + encrypted secrets vault (Vault-X) |
| **Stack** | Tauri 2 · React 19 · Vite 7 · Rust (Argon2id, AES-256-GCM) |
| **Repo** | [github.com/davidthegnomad/gnomad-slate](https://github.com/davidthegnomad/gnomad-slate) (private) |
| **Default branch** | `main` |
| **Latest commit** | `00548ae` — *Harden vault security, add UX polish, and ship full documentation suite* |
| **GitHub release** | None yet (local builds only) |
| **CI** | Not configured (planned v0.2) |
| **Tests** | 14 Rust (`cargo test`) · 4 Vitest (`npm test`) — passing |

---

## Current phase

**v0.1.0 is feature-complete for internal use.** Security hardening, documentation, and core UX are done. The app is suitable for daily dogfooding on macOS. Distribution (signed installers, CI, tagged releases) is the main gap before calling v0.2 “released.”

### What works today

- **Scratchpad & vault notes** — Markdown buffers; scratchpad auto-archives after 48h; vault notes persist as `vault_*.md`
- **Markdown** — Edit / Split / Preview via `marked`
- **Vault-X** — Argon2id master password; AES-256-GCM v2; legacy v1 auto-migration on unlock
- **Secret handling** — Values live in Rust `VaultSession` only; UI sees metadata; reveal/copy via IPC; heap zeroized on lock
- **Unlock protection** — Rate limit (5 failures → 30s lockout)
- **Clipboard** — History (⌘⇧V); Shannon entropy + prefix blocklist; sidebar toggle synced to Rust
- **macOS clipboard polling** — `NSPasteboard.changeCount` with adaptive 1s/3s interval
- **Global summon** — Option+Space; command palette; hide on blur
- **Data paths** — `dirs::data_dir()/gnomad-slate` with migration from legacy Antigravity scratch path
- **Themes** — Dark, light, system

### Not done yet (tracked on roadmap)

| Gap | Target phase |
|-----|----------------|
| GitHub Actions (test + build on PR) | v0.2 |
| Tagged release + installers | v0.2 |
| Note rename by filename (orphan fix) | v0.2 |
| Bundled Inter fonts (offline) | v0.2 |
| Auto-lock on hide, Keychain/Touch ID | v0.3 |
| macOS notarization, Windows signing | v1.0 |
| Playwright / expanded E2E | v1.0 |

---

## Health check (2 June 2026)

| Check | Status |
|-------|--------|
| `cargo test` (src-tauri) | ✓ 14 passed |
| `npm test` | ✓ 4 passed |
| `main` pushed to origin | ✓ |
| Docs suite (MD + export) | ✓ |
| Production build verified this session | — run `npm run tauri build` before release |

---

## Recent milestones

| Date | Milestone |
|------|-----------|
| Jun 2026 | Standalone repo + private GitHub remote |
| Jun 2026 | Security pass: Argon2id, vault session, path safety, rate limit, zeroization |
| Jun 2026 | Clipboard filter + macOS changeCount polling |
| Jun 2026 | Full `docs/` portfolio + `npm run docs:export` |
| 2 Jun 2026 | Commit `00548ae` on `main` |

---

## Risks & watch items

- **No CI** — Regressions rely on manual `cargo test` / `npm test` until v0.2 workflows land.
- **macOS-first clipboard** — Windows/Linux use timer polling only; event-driven pasteboard is v0.3.
- **Markdown preview** — Uses `marked` without DOMPurify; acceptable for local trusted notes; harden before v1.0 GA.
- **Fonts** — Inter loaded from CDN in dev; bundle locally for offline v0.2.

---

## Next actions (priority order)

1. Add `.github/workflows` — PR gate: Rust tests, Vitest, `npm run build`
2. Fix note updates when filename changes (avoid orphan files)
3. Cut first tagged release (`v0.1.0` or `v0.2.0`) with release runbook
4. Bundle Inter fonts in the Tauri asset pipeline

Detail and priorities: **[ROADMAP.md](ROADMAP.md)**

---

## For agents & contributors

- **Session memory:** [.agents/LOGS/SESSION_STATE.md](../.agents/LOGS/SESSION_STATE.md) (hot state; update at checkpoint/end)
- **Build/test SOP:** [BUILD.md](BUILD.md) · [TEST_STRATEGY.md](TEST_STRATEGY.md) · [QA_CHECKLIST.md](QA_CHECKLIST.md)
- **Security:** [VAULT_SECURITY.md](VAULT_SECURITY.md) · [SECURITY_MODEL.md](SECURITY_MODEL.md)

---

Built with ❤️ by [Gnomad Studio](https://gnomadstudio.org) 🦙

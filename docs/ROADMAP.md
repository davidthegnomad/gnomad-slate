# Product Roadmap — Gnomad Slate

**Version:** 0.1.0  
**Last updated:** 2 June 2026  
**Current status:** [STATUS.md](STATUS.md)  
**Horizon:** v1.0 GA (indicative)

---

## Vision

Gnomad Slate is the **default quick-capture surface** for Gnomad Studio — markdown scratchpad, persistent vault notes, filtered clipboard history, and **Vault-X** for API keys and tokens, summoned from anywhere without leaving secrets in the UI heap.

---

## Released — v0.1.0 ✓

| Theme | Delivered |
|-------|-----------|
| **Notes** | Scratchpad (48h archive) + persistent vault markdown files |
| **Markdown** | Edit / Split / Preview (`marked`) |
| **Vault-X** | Argon2id + AES-256-GCM v2; legacy v1 migration |
| **Secrets** | Rust-only session; metadata IPC; zeroize on lock |
| **Unlock** | Rate limiting (5 failures → 30s) |
| **Clipboard** | History capsule; entropy + prefix filter; UI toggle |
| **macOS** | `changeCount` pasteboard polling; adaptive interval |
| **Paths** | `dirs::data_dir()` + legacy migration |
| **UX** | Option+Space; themes; debounced autosave |
| **Quality** | 14 Rust + 4 Vitest tests |
| **Docs** | Full portfolio + `docs:export` (MD/HTML/TXT) |

---

## v0.2 — Release & CI

| Item | Priority | Status | Notes |
|------|----------|--------|-------|
| GitHub Actions PR gate | P0 | Planned | `cargo test`, `npm test`, `npm run build` |
| Release workflow | P0 | Planned | Tag → macOS `.dmg` / artifacts per [RELEASE_RUNBOOK.md](RELEASE_RUNBOOK.md) |
| First GitHub Release | P0 | Planned | No releases yet; see [STATUS.md](STATUS.md) |
| Note update by `filename` | P1 | Planned | Fix rename/orphan edge case |
| Bundle Inter fonts locally | P1 | Planned | Offline UI; remove CDN dependency |
| `npm run tauri build` in CI | P1 | Planned | Smoke artifact on `main` (optional) |

---

## v0.3 — Security UX

| Item | Priority | Status | Notes |
|------|----------|--------|-------|
| Auto-lock vault on window hide | P1 | Planned | Configurable timeout |
| macOS Keychain / Touch ID unlock | P1 | Planned | Optional alongside master password |
| Vault password strength meter | P2 | Planned | On create/change password |
| Encrypted backup export (`.slate`) | P2 | Planned | Portable encrypted bundle |
| Event-driven clipboard (Win/Linux) | P2 | Planned | Parity with macOS `changeCount` |

---

## v1.0 — Production

| Item | Priority | Status | Notes |
|------|----------|--------|-------|
| macOS notarization | P0 | Planned | Gatekeeper-ready distribution |
| Signed Windows builds | P1 | Planned | Authenticode / SmartScreen |
| Expanded E2E (Playwright) | P1 | Planned | Unlock → note → vault → clipboard flows |
| WCAG accessibility pass | P1 | Planned | See [ACCESSIBILITY.md](ACCESSIBILITY.md) |
| Markdown preview sandbox | P1 | Planned | DOMPurify or stricter HTML policy |
| Auto-updater (Tauri) | P2 | Planned | Post-signing pipeline |

---

## Non-goals (v1.0)

- Cloud sync or multi-device vault replication
- Team/shared vaults or RBAC
- Full note-taking app (wikilinks, plugins, mobile)
- Built-in AI chat or code execution
- Browser extension (desktop-only focus)

---

## How to propose features

1. Add a line to [.agents/LOGS/SESSION_STATE.md](../.agents/LOGS/SESSION_STATE.md) or open a GitHub issue
2. Update this roadmap with phase and priority (P0–P3)
3. Reflect delivery in [STATUS.md](STATUS.md) and [CHANGELOG.md](../CHANGELOG.md)
4. Record significant architecture choices under `docs/adr/` when introduced

---

Built with ❤️ by [Gnomad Studio](https://gnomadstudio.org) 🦙

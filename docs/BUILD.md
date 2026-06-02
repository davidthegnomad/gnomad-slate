# Build Narrative — Gnomad Slate

**Version:** 0.1.0  
**Last updated:** June 2026

---

## Vision

An **always-available desktop scratchpad** with encrypted credential storage — fast capture, markdown preview, minimal chrome, security-sensitive logic in Rust.

---

## Phase 1 — Core scratchpad ✓

| Deliverable | Status |
|-------------|--------|
| Tauri 2 + React + Vite scaffold | ✓ |
| Markdown notes (scratch + vault prefixes) | ✓ |
| Sidebar tabs and search | ✓ |
| Debounced autosave (400 ms) | ✓ |
| 48h ephemeral archive | ✓ |

---

## Phase 2 — Power features ✓

| Deliverable | Status |
|-------------|--------|
| Clipboard history capsule | ✓ |
| Command palette | ✓ |
| Global shortcut (Option+Space) | ✓ |
| Hide on blur | ✓ |
| Light / dark / system themes | ✓ |

---

## Phase 3 — Vault-X security ✓

| Deliverable | Status |
|-------------|--------|
| AES-256-GCM vault file (v2) | ✓ |
| Argon2id master password + KDF | ✓ |
| Rust-only decrypted secrets | ✓ |
| `ZeroizeOnDrop` on in-memory secrets | ✓ |
| Unlock rate limiting | ✓ |
| Path traversal protection | ✓ |
| Portable data directory + legacy migration | ✓ |
| Legacy v1 vault migration | ✓ |

---

## Phase 4 — Documentation ✓

| Deliverable | Status |
|-------------|--------|
| Full `docs/` suite (MD/HTML/TXT) | ✓ |
| [VAULT_SECURITY.md](VAULT_SECURITY.md) | ✓ |
| `npm run docs:export` | ✓ |

---

## Phase 5 — Quality & UX polish ✓

| Deliverable | Status |
|-------------|--------|
| Clipboard filter v2 (entropy + allowlist, UI toggle) | ✓ |
| macOS pasteboard `changeCount` + adaptive polling | ✓ |
| Markdown Edit / Split / Preview (`marked`) | ✓ |
| Rust unit tests (14) | ✓ |
| Vitest frontend tests (4) | ✓ |

---

## Phase 6 — Next (planned)

| Deliverable | Status |
|-------------|--------|
| GitHub Actions CI (`cargo test` + `npm test`) | Planned |
| Note save by filename (no orphan on rename) | Planned |
| Touch ID / Keychain unlock | Planned |
| macOS notarization | Planned |
| Auto-lock vault on blur (optional setting) | Under consideration |
| Windows/Linux clipboard change notifications | Planned |

See [ROADMAP.md](ROADMAP.md).

---

Built with ❤️ by [Gnomad Studio](https://gnomadstudio.org) 🦙

# Security Model — Gnomad Slate

**Version:** 0.1.0  
**Last updated:** June 2026

For **vault passwords and encrypted secrets**, see the dedicated guide: [VAULT_SECURITY.md](VAULT_SECURITY.md).

---

## Trust boundaries

```
┌─────────────────────────────────────────┐
│  Trusted: Tauri shell, Rust backend    │
│  Trusted: Bundled React UI              │
├─────────────────────────────────────────┤
│  Sensitive: vault.enc (encrypted)       │
│  Sensitive: vault session (when unlocked, zeroized on lock) │
├─────────────────────────────────────────┤
│  User plaintext: notes/*.md             │
│  User preview: markdown → HTML (sanitized scripts) │
│  Shared: OS clipboard (other apps)      │
└─────────────────────────────────────────┘
```

---

## Vault-X (summary)

| Control | Detail |
|---------|--------|
| Password storage | Argon2id PHC hash only |
| Secrets at rest | AES-256-GCM (`vault.enc` v2) |
| Secrets in memory | Rust `VaultSession`; `ZeroizeOnDrop` on lock |
| Secrets in UI | Metadata only; brief reveal on Show |
| Brute force | 5 failures → 30s lockout |

Full detail: [VAULT_SECURITY.md](VAULT_SECURITY.md).

---

## Notes filesystem

| Rule | Implementation |
|------|----------------|
| Data directory | `dirs::data_dir()/gnomad-slate` |
| Legacy migration | `~/.gemini/antigravity/scratch/slate_data` under user `$HOME` |
| Path traversal | Reject `..`, slashes; canonical path check |
| Encryption | Notes are **not** encrypted — use Vault-X for secrets |

---

## Clipboard history

| Control | Implementation |
|---------|----------------|
| Filter module | `clipboard_filter.rs` — prefix blocklist + Shannon entropy (≥4.2 bits/char) |
| Benign allowlist | Emails, prose, http(s) URLs, camelCase identifiers ≤55 chars |
| User toggle | `set_clipboard_filter_enabled` — disable filtering entirely |
| Monitor efficiency | macOS `changeCount`; adaptive 1s / 3s poll |
| Size limit | Skip entries > 20 KB |
| Storage | In-memory only (max 15 items) |

**Limitation:** Heuristics can miss secrets or occasionally block unusual tokens. Vault-X is the correct store for credentials. Disable the filter in sidebar settings if you need full clipboard history for debugging.

---

## Markdown preview

| Control | Detail |
|---------|--------|
| Renderer | `marked` (GFM) in `src/utils/markdown.ts` |
| Sanitization | Strip `<script>` tags and inline `on*` handlers |
| Trust | User-authored note content — not a sandboxed iframe |

Do not treat preview as safe for untrusted pasted HTML.

---

## Global shortcut & window behavior

- **Option+Space** toggles visibility (parse errors logged, no panic).
- **Blur** hides window — does not auto-lock vault.

**Recommendation:** Lock Vault-X manually when stepping away.

---

## Network exposure

| Component | Network |
|-----------|---------|
| App core | None — no telemetry or sync in v0.1 |
| Google Fonts (CSS `@import`) | Optional — UI fonts only |

---

## Threat summary

| Threat | Mitigation | Residual risk |
|--------|------------|---------------|
| Stolen `vault.enc` | AES-GCM + strong password | Offline brute force if password weak |
| Secrets in RAM after lock | `ZeroizeOnDrop` on `SecretItem` | Swap / crash dumps while unlocked |
| Secrets in WebView | Rust-only store; minimal IPC | Brief DOM exposure on Show |
| Path traversal on notes | Basename + canonicalize | Low |
| Clipboard sniffing | Optional filter + user toggle | Other apps read clipboard |
| Markdown XSS | Script stripping | Not full HTML sanitizer |
| Malware on device | None (OS-level) | Full compromise |

---

## Pre-release checklist

- [ ] `cargo test` and `npm test` pass
- [ ] No secrets in repo or sample notes
- [ ] Tauri capabilities minimal
- [ ] Review [VAULT_SECURITY.md](VAULT_SECURITY.md) after vault changes
- [ ] Verify clipboard filter toggle syncs to Rust monitor

---

Built with ❤️ by [Gnomad Studio](https://gnomadstudio.org) 🦙

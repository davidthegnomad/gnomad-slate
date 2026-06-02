# Demo Script — Gnomad Slate

**Version:** 0.1.0 · ~6 minutes

---

## Setup

- `npm run tauri dev`
- Empty vault or demo password ready

---

## 1. Quick capture (1 min)

1. **Option+Space** — window appears.
2. Type markdown (`# Hello`, bullet list) — show **Split** preview updating.
3. Toggle **Promoted to Vault** — badge becomes **Saved**.

## 2. Clipboard (1 min)

1. Copy a normal sentence — **⌘⇧V** — appears in history.
2. Copy a string starting with `ghp_` — filtered (with default filter on).
3. Uncheck **Filter sensitive clipboard items** — copy again — now visible.

## 3. Command palette (30 sec)

1. **⌘P** → **Format JSON** or theme switch.

## 4. Vault-X (2 min)

1. **Secrets** → **Initialize Vault**.
2. Add `OPENAI_API_KEY` — metadata only in list.
3. **Copy** and **Show** — explain Rust-side security.
4. **Lock Vault**.
5. Point to [VAULT_SECURITY.html](VAULT_SECURITY.html).

## 5. Engineering (30 sec)

- `cargo test` — 14 tests  
- `npm test` — 4 tests  
- `docs/index.html` — full doc set  

---

Built with ❤️ by [Gnomad Studio](https://gnomadstudio.org) 🦙

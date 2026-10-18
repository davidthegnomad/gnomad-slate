# Gnomad Slate agent API

Loopback JSON API inside the running Slate process. It calls the same note, clipboard, vault, and settings functions as the UI. It does not reimplement Argon2id or AES-GCM.

Caret position, the undo stack, the command palette, editor view, and menu actions that only emit webview events (about, undo, redo, help) are not in this API.

## Base URL

`http://127.0.0.1:47321`

The process binds `127.0.0.1` only. If port 47321 is already taken, Slate keeps running and logs `agent api port 47321 is already in use on 127.0.0.1`. The token is never written to the log.

## Find the port and token

Both files live in the Slate data directory (the same folder as `notes/` and `vault.enc`):

| Platform | Directory |
|----------|-----------|
| macOS | `~/Library/Application Support/gnomad-slate/` |
| Linux | `~/.local/share/gnomad-slate/` |

| File | Mode | Contents |
|------|------|----------|
| `agent-api.json` | `0644` | `{ "port": 47321 }` only |
| `agent-api.token` | `0600` | bearer token, created on first launch and reused after that |

Read the port from `agent-api.json`. Read the token from `agent-api.token`. Do not expect the token in the port file or in logs.

## Auth

Every route, including capabilities, requires:

```http
Authorization: Bearer <token>
```

Missing or wrong credentials return HTTP 401:

```json
{ "ok": false, "error": "unauthorized" }
```

Command failures use the same shape with HTTP 400. Success is `{ "ok": true, "data": ... }`.

## Routes

`GET /v1/capabilities` lists every action as `{ "name", "method", "path" }` and includes no secrets.

| Method | Path | UI command |
|--------|------|------------|
| GET | `/v1/version` | `get_app_version_info` |
| GET | `/v1/shortcut` | `get_summon_shortcut_label` |
| POST | `/v1/window/show` | show main window |
| POST | `/v1/window/hide` | hide main window |
| POST | `/v1/window/toggle` | summon shortcut toggle |
| POST | `/v1/window/minimize` | minimize main window |
| POST | `/v1/app/quit` | `confirm_quit` (lock vault, then exit) |
| GET | `/v1/notes` | `list_notes` |
| GET | `/v1/notes/{filename}` | `read_note` |
| PUT | `/v1/notes` | `write_note` |
| DELETE | `/v1/notes/{filename}` | `delete_note` |
| POST | `/v1/notes/archive` | `archive_notes` (stale scratchpads) |
| GET | `/v1/clipboard` | `read_text_from_clipboard` |
| PUT | `/v1/clipboard` | `copy_text_to_clipboard` |
| GET | `/v1/clipboard/history` | `get_clipboard_history` |
| GET | `/v1/clipboard/filter` | `get_clipboard_filter_enabled` |
| PUT | `/v1/clipboard/filter` | `set_clipboard_filter_enabled` |
| GET | `/v1/vault` | `is_vault_initialized`, `is_vault_unlocked` |
| POST | `/v1/vault/init` | `initialize_vault` |
| POST | `/v1/vault/unlock` | `unlock_vault` |
| POST | `/v1/vault/lock` | `lock_vault` |
| GET | `/v1/vault/secrets` | `list_secret_metadata` |
| POST | `/v1/vault/secrets` | `add_secret` |
| POST | `/v1/vault/secrets/{index}/reveal` | `reveal_secret_value` |
| POST | `/v1/vault/secrets/{index}/copy` | `copy_secret_to_clipboard` |
| DELETE | `/v1/vault/secrets/{index}` | `delete_secret` |
| GET | `/v1/settings` | theme, line numbers, clipboard filter |
| PUT | `/v1/settings` | same store the UI writes |

`prepare_for_quit` only locks the session. Use `POST /v1/vault/lock` for that. Quit is the separate exit path.

### Bodies

```json
PUT /v1/notes
{ "title": "scratch", "content": "hello", "is_ephemeral": true, "previous_filename": null }

PUT /v1/clipboard
{ "text": "hello" }

PUT /v1/clipboard/filter
{ "enabled": true }

POST /v1/vault/init
POST /v1/vault/unlock
{ "password": "..." }

POST /v1/vault/secrets
{ "name": "...", "path": "...", "value": "..." }

PUT /v1/settings
{ "theme": "dark", "line_numbers": true, "clipboard_filter": true }
```

`theme` is `light`, `dark`, or `system`. Omit any settings field you are not changing.

Notes use the existing filename rules: scratchpads are `temp_<title>.md`, saved notes are `vault_<title>.md`. Archive moves scratchpads older than 48 hours.

`GET /v1/clipboard/history` returns the in-memory history. It does not read or replace the pasteboard. `GET /v1/clipboard` reads the pasteboard and does not append history.

Vault unlock and init use the existing Argon2id and AES-GCM commands. Reveal, add, copy, and delete run only while that in-process session is unlocked. Lock clears the session the same way the UI does. Passwords and secret values are not logged.

Settings are the UI store (`settings.json` via the Tauri store plugin): `gnomad-theme`, `gnomad-line-numbers`, `gnomad-clipboard-filter`. Clipboard filter also updates the live in-process flag. Theme and line numbers are saved for the next launch; on Linux, theme also emits the existing menu theme events.

## Example

This reads the token from the file. It does not embed a real token.

```bash
DIR="$HOME/Library/Application Support/gnomad-slate"
PORT=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["port"])' "$DIR/agent-api.json")
TOKEN=$(cat "$DIR/agent-api.token")
curl -sS -H "Authorization: Bearer $TOKEN" "http://127.0.0.1:${PORT}/v1/capabilities"
```

On Linux, set `DIR` to `$HOME/.local/share/gnomad-slate`.

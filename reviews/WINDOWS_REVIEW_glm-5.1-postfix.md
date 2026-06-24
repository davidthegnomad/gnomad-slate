<!-- api: NVIDIA NIM -->
<!-- model: z-ai/glm-5.1 -->
<!-- scope: slate_win64 post-fix modularity review -->

# GLM 5.1 — Post-fix review (Rust / Tauri backend)

## Executive summary
The Rust backend has successfully integrated the security and lifecycle fixes (v3 salts, QUIT_IN_PROGRESS, clipboard TTL). However, the architecture suffers from significant monolithic coupling and poor separation of concerns. The `commands/vault.rs` module acts as a God module, mixing IPC routing, business logic, format migration, and direct OS/clipboard interaction. The `crypto.rs` module bundles three distinct encryption formats into a single file with complex branching logic. Global mutable statics in `window.rs` block testability, and the `AppState` struct leaks implementation details by exposing `Arc<Mutex<...>>` wrappers directly to commands. Refactoring these God modules and isolating side-effects will drastically improve maintainability and enable unit testing.

## Verdict
PASS WITH FIXES

## Monolithic structure findings
| # | File | Problem | Recommended split | Target module(s) |
|---|------|---------|-------------------|------------------|
| 1 | `commands/vault.rs` | God module: mixes IPC, vault migration logic, rate limiting orchestration, and direct clipboard I/O. | Extract vault format migration logic. Extract clipboard I/O. | `vault_migration.rs`, `clipboard/mod.rs` (or `commands/clipboard.rs`) |
| 2 | `crypto.rs` | Bundles v1, v2, and v3 formats with complex branching in `decrypt_vault`. | Split legacy formats (v1, v2) into their own module. | `crypto/legacy.rs`, `crypto/v3.rs`, `crypto/mod.rs` |
| 3 | `app_state.rs` | Leaks concurrency implementation (`Arc<Mutex<...>>`) to command handlers, making them hard to test without Tauri state. | Encapsulate state access behind domain-specific methods or type aliases. | `app_state.rs` (internal refactoring) |
| 4 | `window.rs` | Uses global `AtomicBool` statics (`INTENTIONALLY_HIDDEN`, `QUIT_IN_PROGRESS`), preventing parallel test execution and making logic hard to test. | Move window lifecycle state into `AppState` or a dedicated `WindowManager` managed by Tauri. | `window.rs` (state refactoring) |

## Complicated code needing explanation
| # | File:area | Why it's complex | Comment/doc to add |
|---|-----------|------------------|-------------------|
| 1 | `commands/vault.rs:unlock_vault` | Complex branching matrix for v1/v2/v3 migration and salt rotation. The control flow for `needs_v3_migration` vs `was_legacy` is non-obvious and interleaves rate-limiting side-effects. | Add doc comment explaining the state machine: "On unlock, vaults are classified as Empty, V1, V2, or V3. V1/V2 are decrypted, upgraded to V3 salt in-memory, and immediately persisted. Rate limiting is checked before IO and recorded after password verification." |
| 2 | `crypto.rs:decrypt_vault` | Dynamic dispatch based on magic byte with implicit fallback to v1. | Add comment: "Dispatches based on format byte. Note: Any byte other than 0x02 or 0x03 is assumed to be V1 (legacy PHC format). Fails safely if decryption garbles data." |
| 3 | `clipboard.rs:ClipboardHistory::add` | Deduplication logic moves existing items to the front, but the `retain` + `insert(0)` pattern is subtle and easy to break if changed to `insert` without checking duplicates. | Add comment: "Deduplication: removes any existing copy of this text, then inserts at the head to maintain MRU order." |
| 4 | `clipboard_filter.rs:looks_like_benign_text` | Heuristic rules for allowing text are complex and rely on implicit assumptions (e.g., URLs with `://` are safe *unless* they are database URLs). | Expand doc: "Heuristic to reduce false positives. Allows multi-word text, emails, and general URLs. Explicitly blocks DB/Message broker URLs (mysql://, amqp://) which often embed credentials in the authority section." |

## Refactor plan (priority order)
### P0 — split before next feature
1. **Extract Vault Migration Logic**: Move the `needs_v3_migration`, `was_legacy`, and salt rotation logic from `commands/vault.rs::unlock_vault` into `vault_session.rs` or a new `vault_migration.rs` module. The command should just call `vault_session::unlock_and_migrate(...)`.
2. **Extract Clipboard I/O from Vault Commands**: `copy_secret_to_clipboard` in `commands/vault.rs` directly instantiates `arboard::Clipboard` and spawns a thread. Move this to `clipboard/mod.rs` as `clipboard::copy_and_clear_after(text, Duration)`.
3. **Remove Global Statics from `window.rs`**: Move `INTENTIONALLY_HIDDEN` and `QUIT_IN_PROGRESS` into `AppState` (or a new `WindowLifecycle` struct inside `AppState`) so they can be injected/tested.

### P1 — clarity & comments
1. **Document `unlock_vault` matrix**: Add the suggested state-machine docstring to `unlock_vault`.
2. **Split `crypto.rs`**: Move `decrypt_v1`, `derive_key_v1`, and `decrypt_v2` into `crypto/legacy.rs`. Keep `crypto/mod.rs` as the public API (`encrypt_vault`, `decrypt_vault`). This isolates the complex branching.
3. **Typealias Mutexes in AppState**: In `app_state.rs`, replace `Arc<Mutex<ClipboardHistory>>` with `type ClipboardHistoryState = Arc<Mutex<ClipboardHistory>>;` to reduce noise and make it easier to swap concurrency implementations later.

### P2 — nice to have
1. **Standardize Error Types**: Replace `Result<T, String>` across commands and modules with a typed `Error` enum (e.g., `SlateError::Vault(VaultError), SlateError::Clipboard(ClipboardError)`) to improve `match` ergonomics and avoid stringly-typed errors.
2. **Extract Menu Event Strings**: The `menu/windows/events.rs` uses raw string matching (`"window_minimize"`, etc.). Consider mapping these to an `enum MenuEvent` to get compile-time guarantees on event routing.

## Specific file-level recommendations
- **`commands/vault.rs`**: Too many responsibilities. Must be split (see P0).
- **`crypto.rs`**: Needs module split for legacy formats (see P1).
- **`window.rs`**: Global statics must be moved to managed state (see P0).
- **`app_state.rs`**: Exposes too much internal locking mechanism. Encapsulate.
- **`clipboard.rs`**: The `start_clipboard_monitor` thread loop is dense; extracting the "read & filter" step into a `try_poll_clipboard` function would make it easier to test the polling logic without spawning a thread.
- **`clipboard_filter.rs`**: Good tests, but `ENTROPY_THRESHOLD` (4.2) and `MIN_TOKEN_LEN` (32) are magic numbers. Consider `const ENTROPY_THRESHOLD_BITS: f64 = 4.2;` with a doc comment explaining the calibration.

## Top 10 actions
1. **Extract vault migration logic** from `commands/vault.rs::unlock_vault` into `vault_session` or a new `vault_migration` module.
2. **Move clipboard side-effects** out of `commands/vault.rs::copy_secret_to_clipboard` into the `clipboard` module.
3. **Move `QUIT_IN_PROGRESS` and `INTENTIONALLY_HIDDEN`** out of global statics in `window.rs` into `AppState`.
4. **Add state-machine documentation** to the `unlock_vault` function explaining the v1/v2/v3 migration matrix.
5. **Split `crypto.rs`** into `mod.rs`, `v3.rs`, and `legacy.rs` to isolate format branching.
6. **Encapsulate `AppState` fields** behind methods or type aliases to hide `Arc<Mutex<...>>` noise from command handlers.
7. **Rename/document magic numbers** in `clipboard_filter.rs` (`ENTROPY_THRESHOLD`, `MIN_TOKEN_LEN`).
8. **Extract `try_poll_clipboard`** function from the `start_clipboard_monitor` loop in `clipboard.rs` for testability.
9. **Add doc comments** to `clipboard.rs::ClipboardHistory::add` explaining the MRU deduplication logic.
10. **Document the `decrypt_vault` dispatch** in `crypto.rs` regarding the implicit V1 fallback.

---

# GLM 5.1 — Post-fix review (React frontend)

## Executive summary
The post-fix React frontend successfully addresses prior security and lifecycle concerns (v3 salt, clipboard TTL, QUIT_IN_PROGRESS). However, the architecture has drifted into a monolithic state. `App.tsx` acts as a God Component, orchestrating 10 hooks, managing 12 local state variables, and rendering massive inline platform-branching JSX. The command palette execution logic relies on fragile string-matching switches, and the CSS file mixes layout, theming, components, and platform overrides into an unmaintainable 500+ line blob. Testability is severely blocked by tight coupling and lack of dependency injection.

## Verdict
PASS WITH FIXES

## Monolithic structure findings
| # | File | Problem | Recommended split | Target module(s) |
|---|------|---------|-------------------|------------------|
| 1 | `App.tsx` | God Component: 12 states, 10 hooks, massive conditional render tree with platform branching | Extract editor UI into `EditorWorkstation.tsx`, sidebar into `Sidebar.tsx`, and platform header logic into `EditorHeader.tsx` | `components/EditorWorkstation.tsx`, `components/Sidebar.tsx`, `components/EditorHeader.tsx` |
| 2 | `App.css` | God Stylesheet: Mixes layout, theming, components, vault, and platform-specific overrides | Split by domain: layout, editor, sidebar, vault, overlays, platform | `styles/layout.css`, `styles/editor.css`, `styles/vault.css`, `styles/platform.css` |
| 3 | `executePaletteCommand.ts` | Switch-based string dispatch is fragile and scales poorly; mixes I/O, state, and business logic | Replace switch with Command pattern mapping string keys to injectable executor functions | `commands/builtinCommands.ts` |
| 4 | `App.tsx` | Inline ephemeral marker stripping logic in JSX | Extract to utility function | `utils/notePreview.ts` |
| 5 | `App.tsx` | Platform detection and `data-platform` attribute side effect | Move to custom hook or app initializer | `hooks/usePlatform.ts` |

## Complicated code needing explanation
| # | File:area | Why it's complex | Comment/doc to add |
|---|-----------|------------------|-------------------|
| 1 | `App.tsx`:L70-76 | `archive_notes` IPC call in `useEffect` silently re-fetches notes based on a numeric cast; side effect is hidden | `// On mount, archive stale scratch notes (>48h old). If any were archived, invalidate and reload the note list.` |
| 2 | `App.tsx`:L193 | Inline regex/replace `note.content.replace("<!-- ephemeral -->\n", "").substring(0, 40)` is business logic hiding in markup | Extract to `getPreviewText(content)` and document why the marker is stripped |
| 3 | `executePaletteCommand.ts`:L49-56 | "Prettify Markdown" logic assumes specific list syntax without explaining why | `// Normalizes hyphen-based lists lacking a space after the hyphen (e.g., "-item" -> "- item")` |
| 4 | `App.tsx`:L229-282 | Deeply nested ternary and platform-branching for `editor-header` makes control flow opaque | Add block comment: `// Windows layout stacks toolbar above title/filename; Mac layout places them side-by-side.` |
| 5 | `App.css`:L324-340 | Tooltip positioning logic with `calc` and `transform` interacts with `tipBelow` prop non-obviously | `/* Tooltip defaults above element; .tooltip-below inverts anchor to bottom. Delete button overrides to right-align. */` |

## Refactor plan (priority order)
### P0 — split before next feature
1. **Extract `EditorWorkstation.tsx`**: Move the `<section>` containing the editor body, view modes, and line numbers out of `App.tsx`. Pass `editorContent`, `editorView`, `isWindows`, etc., as props.
2. **Extract `Sidebar.tsx`**: Move the `<aside>` tree, including search, tabs, notes list, and footer settings, into its own component.
3. **Refactor Command Execution**: Replace the `switch` block in `executePaletteCommand.ts` with a `Record<string, (ctx) => Promise<void>>` map. This allows tree-shaking, easier testing, and removes the fragile string-matching.
4. **Split `App.css`**: Break the monolithic CSS into imported modules (`@import './editor.css'`, etc.) to ease merge conflicts and navigation.

### P1 — clarity & comments
1. **Extract `getPreviewText`**: Move the inline `note.content.replace(...)` to a pure function in `utils/notePreview.ts` and unit test it.
2. **Document `archive_notes` side effect**: Add explicit comments on the mount `useEffect` explaining the 48-hour archival contract.
3. **Simplify `isWindows` branching**: In `EditorHeader`, consider a `<WindowsEditorHeader>` and `<MacEditorHeader>` composition instead of ternary nesting, or use CSS `[data-platform]` more aggressively to avoid JS branching.

### P2 — nice to have
1. **Consolidate refs**: `pendingWindowClose` and `quittingRef` in `App.tsx` might be better encapsulated inside `useWindowClose` hook rather than leaking into the parent scope.
2. **Standardize IPC error handling**: The `catch(() => undefined)` pattern in `get_app_version_info` is okay, but `archive_notes` uses `console.error`. Standardize IPC error boundaries.

## Specific file-level recommendations
* `src/App.tsx`: Too large (300+ lines of JSX). Must extract `EditorWorkstation`, `Sidebar`, and `EditorHeader`.
* `src/App.css`: Too broad. Split by component domain.
* `src/commands/executePaletteCommand.ts`: Replace `switch` with a command map dictionary.
* `src/commands/paletteCommands.ts`: Add an `id` or `key` property to `PaletteCommand` to avoid matching by display `name` string.
* `src/components/CommandPalette.tsx`: Truncated in slice, but ensure it uses the new `id`/`key` from `paletteCommands` instead of `cmd.name`.
* `src/hooks/useNoteEditor.ts`: (Not in slice, but inferred from destructuring) Exports too many primitives (15+). Consider returning a grouped object (e.g., `editorState`, `editorActions`) to reduce `App.tsx` destructuring surface area.

## Top 10 actions
1. **Extract `EditorWorkstation.tsx`** from `App.tsx` (P0)
2. **Extract `Sidebar.tsx`** from `App.tsx` (P0)
3. **Replace `switch` with Command Map** in `executePaletteCommand.ts` (P0)
4. **Add stable `id` field** to `PaletteCommand` type and use for dispatch (P0)
5. **Split `App.css`** into layout, editor, sidebar, and vault domain files (P0)
6. **Extract `getPreviewText`** utility function from inline JSX (P1)
7. **Add docblock** to `archive_notes` `useEffect` explaining 48h contract (P1)
8. **Add docblock** to "Prettify Markdown" explaining list normalization rule (P1)
9. **Group `useNoteEditor` return values** into `state`/`actions` objects to clean up `App.tsx` (P2)
10. **Encapsulate window close refs** (`pendingWindowClose`, `quittingRef`) inside `useWindowClose` hook instead of lifting them to `App.tsx` (P2)

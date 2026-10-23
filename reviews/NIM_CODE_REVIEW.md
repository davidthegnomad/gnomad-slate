Status 2026-10-23: Findings 1 and 2 ship in v2.1.0-beta.6. Finding 12 ships in v2.1.0-beta.6 / beta.7.
Findings 3, 4, 5, 6, 7, 8, 9, 10, and 11 ship in v2.1.0-beta.8.
The original write-up below is kept as written.
This status is only a banner; it does not replace the findings.

# Gnomad Slate code review

The recovery wrap matches the crypto it claims: 160-bit key, random wrap salt, AES-256-GCM, and a wrong key returns before either vault file is replaced.
`docs/VAULT_SECURITY.md` still describes that v2 design and does not mention recovery at all.
Two file bugs can still destroy access: a crash during password reset can leave ciphertext and config that no longer match, and Initialize overwrites a leftover `vault.enc` when the config file is gone.
The one-time key is easy to lose on quit, and Copy usually drops it into clipboard history.
Scratchpad saves can also replace another note. Do not trust this build with secrets until the reset commit and the missing-file behavior are fixed.

## 1. Recovery reset can lock the vault permanently

- **Severity:** blocker
- **Area:** vault
- **What is wrong:** `commit_recovered_vault` in `src-tauri/src/lib.rs` replaces `vault.enc` first and writes `vault_config.json` second. The in-memory rotate in `crypto.rs` is safe: a wrong recovery key never produces a new packet. A crash, kill, or failed restore after the ciphertext rename leaves the new file key on disk and the old password hash plus the old wrap in config. The old password then fails decryption, the new password fails the stored hash, and the recovery key unwraps the previous file key, which cannot open the new file. The error path tries to put the old bytes back and ignores that write if it fails. Nothing fsyncs either file.
- **What to change:** Write the new ciphertext to a side file and fsync it, record the new hash and wrap as pending next to the old config, and point `vault.enc` at the new bytes only after that pending config is durable. On launch, finish or delete an incomplete pair. Add a test that a failed config write leaves the original `vault.enc` bytes in place.

## 2. A missing half of the vault is treated as empty

- **Severity:** blocker
- **Area:** vault
- **What is wrong:** `is_vault_initialized` is only “`vault_config.json` exists.” `initialize_vault` then encrypts `[]` and renames it over `vault.enc`. Deleting the config and clicking Initialize destroys ciphertext that the old password could still have opened. `unlock_vault` and `recover_vault` do the opposite when the config exists and `vault.enc` is missing or empty: they substitute `[]` and, on recover, commit that empty vault under the new password. `docs/VAULT_SECURITY.md` says a successful unlock decrypts `vault.enc`. This is the path a reset hits if one of the two files is still there.
- **What to change:** If `vault.enc` exists and the config does not, refuse Initialize until the user explicitly confirms a wipe. If the config exists and `vault.enc` does not, fail unlock and recover with a distinct error and do not write a new vault.

## 3. The recovery key is committed before the user can keep it

- **Severity:** should-fix
- **Area:** vault
- **What is wrong:** `initialize_vault` saves the wrap and the empty vault, then returns the recovery key. `handleInitVault` in `src/App.tsx` marks the vault unlocked and shows the one-time dialog after that. Quit, close, and `confirm_quit` do not look at `recoveryKeyReveal`. Closing the app from that dialog drops the only copy; the checkbox is not checked, and there is no command to mint a new wrap while the master password still unlocks the session. A crash after `save_vault_config` and before the UI paints has the same result.
- **What to change:** Block quit and hide while the reveal dialog is open, and add an unlocked “issue a new recovery key” command that rewraps the current session key and shows the new key once.

## 4. Copy stores the recovery key in clipboard history

- **Severity:** should-fix
- **Area:** vault
- **What is wrong:** The reveal dialog copies through `copy_text_to_clipboard`. The clipboard monitor in `clipboard.rs` reads that text back on its next poll. `clipboard_filter.rs` only treats high-entropy tokens as secret at 4.2 bits/char and 32–256 characters, and this key is lowercase groups plus hyphens, so it misses the prefix list and the mixed-case allowlist. A 2,000-draw check of the same 8×4 format put about three quarters of the keys under 4.2, so they are stored in history with the filter on. History is shown in the clipboard capsule. Linux also leaves the OS clipboard set; `slate_win64` already clears a copied secret after 30 seconds if it is still there.
- **What to change:** Classify this grouped key as sensitive in `is_sensitive`, skip history for that copy, and clear the OS clipboard after a short timeout using the Windows “only if unchanged” behavior. Use that path for `copy_secret_to_clipboard` too.

## 5. Create and reset accept a single password field

- **Severity:** should-fix
- **Area:** ux
- **What is wrong:** Initialize and “Set new password” each send one field. Rust only checks length ≥ 8. A mistyped new password on recover still returns success, clears the form, and unlocks the session. After Lock, that password is whatever was typed once. The recovery key can reset it again only if the user still has the key.
- **What to change:** Require a second matching field in `App.tsx` before `initialize_vault` or `recover_vault`, and keep the Rust length check.

## 6. “Lock / Unlock Secret Vault” does not lock

- **Severity:** should-fix
- **Area:** ux
- **What is wrong:** The palette entry in `App.tsx` only runs `setActiveTab("secrets")`. The description says it toggles the vault lock. The session stays open, revealed values stay in React state, and `lock_vault` is not called. Someone using the palette before stepping away leaves secrets in process memory. The real control is the Lock Vault button inside the secrets panel.
- **What to change:** Make that command call `handleLockVault` when the session is open, and call nothing that looks like unlock until the lock screen password succeeds.

## 7. macOS quit never locks the vault

- **Severity:** should-fix
- **Area:** vault
- **What is wrong:** `RunEvent::ExitRequested` in `lib.rs` always calls `prevent_exit`. `closeApp` in `App.tsx` calls `confirm_quit` only on Linux. On macOS it calls `destroy()`. `confirm_quit` is what drops `VaultSession`. Linux then `process::exit`s because `app.exit` is not enough; macOS has no equivalent, so a closed window can leave an unlocked session in a process that the exit handler will not finish.
- **What to change:** From every platform, save, then call `confirm_quit`, and let that confirmed exit proceed instead of trapping it in `prevent_exit`.

## 8. Windows unlock rewrites v2 and strands the recovery wrap

- **Severity:** should-fix
- **Area:** windows
- **What is wrong:** The Linux/macOS tree encrypts v2 with the fixed salt in `crypto.rs` and stores the wrap in config fields Windows does not know. `slate_win64` decrypts v2, then `vault_migration.rs` persists a v3 file with a new per-file salt on unlock. Linux `decrypt_vault` sends a `0x03` packet down the v1 path, and `decrypt_vault_with_key` rejects anything that is not v2, so the recovery key still unwraps the pre-migration key. Windows `VaultConfig` is only `password_hash`; a legacy hash upgrade rewrites the file and drops the wrap fields. Same data directory, two apps, vault becomes unreadable on Linux.
- **What to change:** Do not let `slate_win64` rotate a v2 vault to v3 until the same commit rewraps the new file key and preserves the recovery fields. Keep that build off this data directory until Linux can read v3.

## 9. Daily note commands replace the journal

- **Severity:** should-fix
- **Area:** data
- **What is wrong:** “Create Daily Note” and “Append to Daily Note” in `App.tsx` both call `write_note` with only the new fragment. `filesystem.rs` `write_note` uses `File::create`, which truncates. The title is always `Daily_Note_<date>`, so the second use that day replaces the file. Append does not read the existing note, and it sends the whole editor body rather than the selection the palette describes. The success text says the note was appended.
- **What to change:** Read today’s note when it exists, append the new block, and write that combined body. Write via a temp file and rename.

## 10. New saved notes on the same day overwrite each other

- **Severity:** should-fix
- **Area:** data
- **What is wrong:** `createNewSavedNote` always titles the note `Note_<YYYY-MM-DD>` and passes `previousFilename: null`. `write_note` turns that into one filename and truncates it. A second saved note the same day wipes the first to empty. The sanitizer has the same effect for distinct titles that collapse to one stem (`My Note` and `My_Note`, or two scratch drafts in the same second). When the target path is some other note, that note is destroyed and the previous file can be deleted afterward.
- **What to change:** If the destination filename exists and is not `previous_filename`, pick a unique name or refuse. Do not truncate an unrelated note.

## 11. Switching to Secrets can drop unsaved note text

- **Severity:** should-fix
- **Area:** data
- **What is wrong:** `saveNoteNow` returns immediately when `activeTabRef` is `secrets`, before it writes. The tab buttons call `setActiveTab` with no flush. A debounce that fires after that switch clears itself and then hits the early return, so the keystrokes never reach disk. `flushPendingSave` only runs when a timer is still pending, and `selectNote` uses that flush. Choosing another note afterward loads the new note over the unsaved text. Quit from the Secrets tab calls the same early return, including the Linux close path.
- **What to change:** Flush the open note before leaving the editor, and remove the Secrets guard from the save used by quit. Keep a dirty flag so a swallowed timer cannot look like a clean buffer.

## 12. The new commit path has no failure test

- **Severity:** later
- **Area:** tests
- **What is wrong:** `crypto.rs` and `biometrics.rs` test key format, wrap roundtrip, wrong-key refusal, and config JSON. They never call `commit_recovered_vault`, never simulate a config write failure, and never assert that a missing `vault.enc` is an error. The lockout tests in `unlock_limiter.rs` only cover the in-memory counter. That is why findings 1 and 2 are invisible to `cargo test`.
- **What to change:** Add tests for “config write fails → original ciphertext restored” and “missing `vault.enc` → error, files unchanged,” and run them in `src-tauri` with the existing crypto tests.

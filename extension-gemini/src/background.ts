import { Note, SecretItem, SecretMeta, ClipboardEntry, getLocalNotes, writeLocalNote, deleteLocalNote, archiveLocalNotes, getLocalClipboardHistory, saveLocalClipboardHistory } from "./storage-extension";
import { deriveKey, encryptData, decryptData, hashPasswordForVerification, verifyPassword } from "./crypto-extension";
import { runSync, pullSync, getGoogleAuthToken, logoutGoogleDrive } from "./sync-engine";

// --- Clipboard Filter Logic (Ported from clipboard_filter.rs) ---

const SENSITIVE_PREFIXES = [
  "ghp_", "sk-proj-", "PRIVATE KEY", "Bearer ", "mongodb://", "postgres://", "mysql://",
  "amqp://", "redis://"
];

function calculateShannonEntropy(str: string): number {
  if (!str) return 0;
  const freqs: Record<string, number> = {};
  for (const char of str) {
    freqs[char] = (freqs[char] || 0) + 1;
  }
  let entropy = 0;
  const len = str.length;
  for (const char in freqs) {
    const p = freqs[char] / len;
    entropy -= p * Math.log2(p);
  }
  return entropy;
}

function isSensitiveClipboard(text: string): boolean {
  const trimmed = text.trim();
  
  // Rule 1: Prefix blocklist
  for (const prefix of SENSITIVE_PREFIXES) {
    if (trimmed.includes(prefix)) return true;
  }

  // Rule 2: High entropy tokens (32-256 chars, entropy >= 4.2)
  if (trimmed.length >= 32 && trimmed.length <= 256) {
    const entropy = calculateShannonEntropy(trimmed);
    if (entropy >= 4.2) {
      // Exclude simple allowlist items (like common emails, long URL routes without tokens, etc.)
      const isEmail = /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(trimmed);
      const isSimpleUrl = /^https?:\/\/[^\s/]+(?:\/[^\s/]*)*$/.test(trimmed) && !trimmed.includes("?");
      if (!isEmail && !isSimpleUrl) {
        return true;
      }
    }
  }

  return false;
}

// --- Vault Lockout Rate Limiter ---

interface RateLimiter {
  failedAttempts: number;
  lockoutUntil: number;
}

let rateLimiter: RateLimiter = { failedAttempts: 0, lockoutUntil: 0 };

function checkLockout(): void {
  if (rateLimiter.lockoutUntil > Date.now()) {
    const remainingSecs = Math.ceil((rateLimiter.lockoutUntil - Date.now()) / 1000);
    throw new Error(`Too many failed attempts. Vault locked for ${remainingSecs} seconds.`);
  }
}

function recordUnlockFailure(): void {
  rateLimiter.failedAttempts += 1;
  if (rateLimiter.failedAttempts >= 5) {
    rateLimiter.lockoutUntil = Date.now() + 30 * 1000; // 30 second lockout
  }
}

function recordUnlockSuccess(): void {
  rateLimiter.failedAttempts = 0;
  rateLimiter.lockoutUntil = 0;
}

// --- Helper to Retrieve Active CryptoKey from Session Storage ---

async function getSessionKey(): Promise<CryptoKey | null> {
  const result = await chrome.storage.session.get("vaultKey");
  if (!result.vaultKey) return null;
  
  try {
    return await crypto.subtle.importKey(
      "jwk",
      result.vaultKey,
      { name: "AES-GCM" },
      true,
      ["encrypt", "decrypt"]
    );
  } catch (e) {
    console.error("Failed to restore session key from JWK", e);
    return null;
  }
}

async function saveSessionKey(key: CryptoKey): Promise<void> {
  const jwk = await crypto.subtle.exportKey("jwk", key);
  await chrome.storage.session.set({ vaultKey: jwk });
}

// --- IPC Command Router ---

chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (message.type !== "tauri-command") {
    // Check for other types like OAuth handlers
    if (message.type === "check-gdrive-auth") {
      getGoogleAuthToken(false)
        .then(() => sendResponse({ authorized: true }))
        .catch(() => sendResponse({ authorized: false }));
      return true;
    }
    if (message.type === "gdrive-login") {
      getGoogleAuthToken(true)
        .then(() => sendResponse({ success: true }))
        .catch((err) => sendResponse({ success: false, error: err.message }));
      return true;
    }
    if (message.type === "gdrive-logout") {
      logoutGoogleDrive()
        .then(() => sendResponse({ success: true }))
        .catch((err) => sendResponse({ success: false, error: err.message }));
      return true;
    }
    if (message.type === "manual-sync") {
      (async () => {
        try {
          const key = await getSessionKey();
          await pullSync(key);
          await runSync(key);
          sendResponse({ success: true });
        } catch (e: any) {
          sendResponse({ success: false, error: e.message });
        }
      })();
      return true;
    }
    return false;
  }

  // Handle tauri emulation commands
  const { cmd, args = {} } = message;

  (async () => {
    try {
      const key = await getSessionKey();

      switch (cmd) {
        case "get_app_version_info": {
          sendResponse({
            data: {
              version: "2.1.0",
              channel: "stable",
              display_version: "v2.1 Extension",
            },
          });
          break;
        }

        case "list_notes": {
          // Sync pull on check
          await pullSync(key).catch(() => null);
          const notes = await getLocalNotes();
          sendResponse({ data: notes });
          break;
        }

        case "read_note": {
          const result = await chrome.storage.local.get("notes");
          const note = (result.notes || {})[args.filename] as Note | undefined;
          sendResponse({ data: note ? note.content : "" });
          break;
        }

        case "write_note": {
          const note = await writeLocalNote(args.title, args.content, args.isEphemeral);
          await runSync(key).catch((e) => console.warn("Sync failed on write", e));
          sendResponse({ data: note });
          break;
        }

        case "delete_note": {
          await deleteLocalNote(args.filename);
          await runSync(key).catch((e) => console.warn("Sync failed on delete", e));
          sendResponse({ data: null });
          break;
        }

        case "archive_notes": {
          const count = await archiveLocalNotes();
          sendResponse({ data: count });
          break;
        }

        case "is_vault_initialized": {
          const result = await chrome.storage.local.get("vaultConfig");
          sendResponse({ data: !!result.vaultConfig });
          break;
        }

        case "is_vault_unlocked": {
          sendResponse({ data: !!key });
          break;
        }

        case "initialize_vault": {
          const { hash, salt } = await hashPasswordForVerification(args.password);
          await chrome.storage.local.set({
            vaultConfig: { password_hash: hash, salt },
            vaultEncrypted: null,
            vault: [],
          });
          const newKey = await deriveKey(args.password);
          await saveSessionKey(newKey);
          recordUnlockSuccess();
          sendResponse({ data: null });
          break;
        }

        case "unlock_vault": {
          checkLockout();
          const result = await chrome.storage.local.get(["vaultConfig", "vaultEncrypted"]);
          if (!result.vaultConfig) {
            throw new Error("Vault is not initialized");
          }

          const verified = await verifyPassword(
            args.password,
            result.vaultConfig.password_hash,
            result.vaultConfig.salt
          );

          if (!verified) {
            recordUnlockFailure();
            throw new Error("Invalid password");
          }

          const derived = await deriveKey(args.password);
          await saveSessionKey(derived);
          recordUnlockSuccess();

          // Load secrets into local memory
          if (result.vaultEncrypted) {
            try {
              const decryptedStr = await decryptData(result.vaultEncrypted, derived);
              const secrets = JSON.parse(decryptedStr);
              await chrome.storage.local.set({ vault: secrets });
            } catch (e) {
              console.error("Vault decryption failed", e);
              throw new Error("Vault decryption failed (corrupted data)");
            }
          } else {
            await chrome.storage.local.set({ vault: [] });
          }

          // Pull remote changes
          await pullSync(derived).catch(() => null);
          sendResponse({ data: null });
          break;
        }

        case "lock_vault": {
          await chrome.storage.session.remove("vaultKey");
          await chrome.storage.local.remove("vault"); // drop decrypted secrets from storage
          sendResponse({ data: null });
          break;
        }

        case "list_secret_metadata": {
          if (!key) throw new Error("Vault is locked");
          const result = await chrome.storage.local.get("vault");
          const secrets = (result.vault || []) as SecretItem[];
          const meta: SecretMeta[] = secrets.map((s, idx) => ({
            index: idx,
            name: s.name,
            path: s.path,
          }));
          sendResponse({ data: meta });
          break;
        }

        case "reveal_secret_value": {
          if (!key) throw new Error("Vault is locked");
          const result = await chrome.storage.local.get("vault");
          const secret = (result.vault || [])[args.index] as SecretItem | undefined;
          sendResponse({ data: secret ? secret.value : "" });
          break;
        }

        case "copy_secret_to_clipboard": {
          if (!key) throw new Error("Vault is locked");
          const result = await chrome.storage.local.get("vault");
          const secret = (result.vault || [])[args.index] as SecretItem | undefined;
          if (secret) {
            // Write to clipboard natively in service worker
            await navigator.clipboard.writeText(secret.value);
          }
          sendResponse({ data: null });
          break;
        }

        case "add_secret": {
          if (!key) throw new Error("Vault is locked");
          const result = await chrome.storage.local.get("vault");
          const secrets = (result.vault || []) as SecretItem[];
          secrets.push({ name: args.name, path: args.path, value: args.value });
          
          const encrypted = await encryptData(JSON.stringify(secrets), key);
          await chrome.storage.local.set({ vault: secrets, vaultEncrypted: encrypted });
          await runSync(key).catch((e) => console.warn("Sync failed on add secret", e));
          sendResponse({ data: null });
          break;
        }

        case "delete_secret": {
          if (!key) throw new Error("Vault is locked");
          const result = await chrome.storage.local.get("vault");
          const secrets = (result.vault || []) as SecretItem[];
          if (args.index >= 0 && args.index < secrets.length) {
            secrets.splice(args.index, 1);
            const encrypted = await encryptData(JSON.stringify(secrets), key);
            await chrome.storage.local.set({ vault: secrets, vaultEncrypted: encrypted });
            await runSync(key).catch((e) => console.warn("Sync failed on delete secret", e));
          }
          sendResponse({ data: null });
          break;
        }

        case "get_clipboard_history": {
          const history = await getLocalClipboardHistory();
          sendResponse({ data: history });
          break;
        }

        case "get_clipboard_filter_enabled": {
          const result = await chrome.storage.local.get("clipboardFilterEnabled");
          sendResponse({ data: result.clipboardFilterEnabled !== false });
          break;
        }

        case "set_clipboard_filter_enabled": {
          await chrome.storage.local.set({ clipboardFilterEnabled: args.enabled });
          sendResponse({ data: null });
          break;
        }

        default:
          throw new Error(`Unknown command: ${cmd}`);
      }
    } catch (error: any) {
      sendResponse({ error: error.message || String(error) });
    }
  })();

  return true; // Keep the response channel open for async operations
});

// --- Clipboard Capsule Trigger (Activated when popup is opened) ---

chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (message.type === "record-clipboard") {
    const { text } = message;
    (async () => {
      const settings = await chrome.storage.local.get("clipboardFilterEnabled");
      const filterEnabled = settings.clipboardFilterEnabled !== false;

      if (filterEnabled && isSensitiveClipboard(text)) {
        sendResponse({ ignored: true, reason: "sensitive" });
        return;
      }

      const history = await getLocalClipboardHistory();
      
      // Deduplicate: remove identical text if it exists and put it on top
      const filtered = history.filter((entry) => entry.text !== text);
      filtered.unshift({ text, timestamp: Date.now() });

      // Max 15 entries
      if (filtered.length > 15) {
        filtered.pop();
      }

      await saveLocalClipboardHistory(filtered);
      
      // Notify active panels to reload history
      chrome.runtime.sendMessage({ type: "tauri-event", event: "clipboard-updated", payload: filtered }).catch(() => null);
      sendResponse({ success: true });
    })();
    return true;
  }
});

// --- Open Side Panel on Command ---

chrome.commands.onCommand.addListener(async (command) => {
  if (command === "open_side_panel") {
    const [window] = await chrome.tabs.query({ active: true, currentWindow: true });
    if (window && chrome.sidePanel) {
      await chrome.sidePanel.open({ windowId: window.windowId });
    }
  }
});

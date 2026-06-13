(function() {
  "use strict";
  async function getLocalNotes() {
    const result = await chrome.storage.local.get("notes");
    const notesMap = result.notes || {};
    return Object.values(notesMap);
  }
  async function writeLocalNote(title, content, isEphemeral, filename) {
    const result = await chrome.storage.local.get("notes");
    const notesMap = result.notes || {};
    const fileId = filename || (isEphemeral ? `temp_${Date.now()}.md` : `vault_${Date.now()}.md`);
    const note = {
      filename: fileId,
      title,
      content,
      is_ephemeral: isEphemeral,
      last_modified: Date.now()
    };
    notesMap[fileId] = note;
    await chrome.storage.local.set({ notes: notesMap });
    return note;
  }
  async function deleteLocalNote(filename) {
    const result = await chrome.storage.local.get("notes");
    const notesMap = result.notes || {};
    delete notesMap[filename];
    await chrome.storage.local.set({ notes: notesMap });
  }
  async function archiveLocalNotes() {
    const result = await chrome.storage.local.get("notes");
    const notesMap = result.notes || {};
    let count = 0;
    const fortyEightHoursAgo = Date.now() - 48 * 60 * 60 * 1e3;
    for (const [filename, note] of Object.entries(notesMap)) {
      if (note.is_ephemeral && note.last_modified < fortyEightHoursAgo) {
        const archiveResult = await chrome.storage.local.get("archive");
        const archiveMap = archiveResult.archive || {};
        archiveMap[filename] = note;
        delete notesMap[filename];
        await chrome.storage.local.set({ archive: archiveMap });
        count++;
      }
    }
    if (count > 0) {
      await chrome.storage.local.set({ notes: notesMap });
    }
    return count;
  }
  async function getLocalClipboardHistory() {
    const result = await chrome.storage.session.get("clipboardHistory");
    return result.clipboardHistory || [];
  }
  async function saveLocalClipboardHistory(history) {
    await chrome.storage.session.set({ clipboardHistory: history });
  }
  const VAULT_SALT = "com.gnomadstudio.gnomad-slate.vault.v2";
  const ITERATIONS = 1e5;
  const KEY_LEN = 256;
  function stringToBuffer(str) {
    return new TextEncoder().encode(str);
  }
  function bufferToHex(buffer) {
    return Array.from(new Uint8Array(buffer)).map((b) => b.toString(16).padStart(2, "0")).join("");
  }
  function hexToBuffer(hex) {
    const bytes = new Uint8Array(hex.length / 2);
    for (let i = 0; i < bytes.length; i++) {
      bytes[i] = parseInt(hex.substring(i * 2, i * 2 + 2), 16);
    }
    return bytes;
  }
  async function deriveKey(password) {
    const baseKey = await crypto.subtle.importKey(
      "raw",
      stringToBuffer(password),
      { name: "PBKDF2" },
      false,
      ["deriveKey", "deriveBits"]
    );
    return crypto.subtle.deriveKey(
      {
        name: "PBKDF2",
        salt: stringToBuffer(VAULT_SALT),
        iterations: ITERATIONS,
        hash: "SHA-256"
      },
      baseKey,
      { name: "AES-GCM", length: KEY_LEN },
      true,
      // exportable for session storage
      ["encrypt", "decrypt"]
    );
  }
  async function encryptData(plaintext, key) {
    const iv = crypto.getRandomValues(new Uint8Array(12));
    const encoded = stringToBuffer(plaintext);
    const ciphertextBuffer = await crypto.subtle.encrypt(
      {
        name: "AES-GCM",
        iv
      },
      key,
      encoded
    );
    const ciphertext = new Uint8Array(ciphertextBuffer);
    const packet = new Uint8Array(1 + iv.length + ciphertext.length);
    packet[0] = 2;
    packet.set(iv, 1);
    packet.set(ciphertext, 1 + iv.length);
    return bufferToHex(packet.buffer);
  }
  async function decryptData(packetHex, key) {
    const packet = hexToBuffer(packetHex);
    if (packet.length < 14 || packet[0] !== 2) {
      throw new Error("Invalid cipher packet version");
    }
    const iv = packet.slice(1, 13);
    const ciphertext = packet.slice(13);
    const decryptedBuffer = await crypto.subtle.decrypt(
      {
        name: "AES-GCM",
        iv
      },
      key,
      ciphertext
    );
    return new TextDecoder().decode(decryptedBuffer);
  }
  async function hashPasswordForVerification(password, saltHex) {
    const salt = saltHex ? hexToBuffer(saltHex) : crypto.getRandomValues(new Uint8Array(16));
    const baseKey = await crypto.subtle.importKey(
      "raw",
      stringToBuffer(password),
      { name: "PBKDF2" },
      false,
      ["deriveBits"]
    );
    const derivedBits = await crypto.subtle.deriveBits(
      {
        name: "PBKDF2",
        salt,
        iterations: ITERATIONS,
        hash: "SHA-256"
      },
      baseKey,
      256
      // bits
    );
    return {
      hash: bufferToHex(derivedBits),
      salt: bufferToHex(salt.buffer)
    };
  }
  async function verifyPassword(password, storedHash, saltHex) {
    const { hash } = await hashPasswordForVerification(password, saltHex);
    if (hash.length !== storedHash.length) return false;
    let diff = 0;
    for (let i = 0; i < hash.length; i++) {
      diff |= hash.charCodeAt(i) ^ storedHash.charCodeAt(i);
    }
    return diff === 0;
  }
  const GD_BACKUP_FILENAME = "gnomad_slate_backup.enc";
  async function getGoogleAuthToken(interactive = true) {
    return new Promise((resolve, reject) => {
      if (typeof chrome === "undefined" || !chrome.identity) {
        return reject(new Error("chrome.identity API is not available"));
      }
      chrome.identity.getAuthToken({ interactive }, (token) => {
        if (chrome.runtime.lastError) {
          reject(new Error(chrome.runtime.lastError.message));
        } else if (!token) {
          reject(new Error("Failed to retrieve Google OAuth token"));
        } else {
          resolve(token);
        }
      });
    });
  }
  async function logoutGoogleDrive() {
    const token = await getGoogleAuthToken(false).catch(() => null);
    if (token) {
      await new Promise((resolve) => {
        fetch(`https://accounts.google.com/o/oauth2/revoke?token=${token}`, { method: "POST" }).finally(() => {
          chrome.identity.removeCachedAuthToken({ token }, () => resolve());
        });
      });
    }
  }
  async function findBackupFile(token) {
    const response = await fetch(
      "https://www.googleapis.com/drive/v3/files?spaces=appDataFolder&q=name='" + GD_BACKUP_FILENAME + "'",
      {
        headers: { Authorization: `Bearer ${token}` }
      }
    );
    if (!response.ok) {
      throw new Error(`Failed to list Google Drive files: ${response.statusText}`);
    }
    const data = await response.json();
    if (data.files && data.files.length > 0) {
      return data.files[0].id;
    }
    return null;
  }
  async function uploadToGoogleDrive(payload, token) {
    const fileId = await findBackupFile(token);
    if (fileId) {
      const response = await fetch(
        `https://www.googleapis.com/upload/drive/v3/files/${fileId}?uploadType=media`,
        {
          method: "PATCH",
          headers: {
            Authorization: `Bearer ${token}`,
            "Content-Type": "application/octet-stream"
          },
          body: payload
        }
      );
      if (!response.ok) {
        throw new Error(`Failed to update backup file: ${response.statusText}`);
      }
    } else {
      const metadata = {
        name: GD_BACKUP_FILENAME,
        parents: ["appDataFolder"]
      };
      const boundary = "314159265358979323846";
      const delimiter = `\r
--${boundary}\r
`;
      const closeDelimiter = `\r
--${boundary}--`;
      const multipartBody = delimiter + "Content-Type: application/json; charset=UTF-8\r\n\r\n" + JSON.stringify(metadata) + delimiter + "Content-Type: application/octet-stream\r\n\r\n" + payload + closeDelimiter;
      const response = await fetch(
        "https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart",
        {
          method: "POST",
          headers: {
            Authorization: `Bearer ${token}`,
            "Content-Type": `multipart/related; boundary=${boundary}`
          },
          body: multipartBody
        }
      );
      if (!response.ok) {
        throw new Error(`Failed to create backup file: ${response.statusText}`);
      }
    }
  }
  async function downloadFromGoogleDrive(token) {
    const fileId = await findBackupFile(token);
    if (!fileId) return null;
    const response = await fetch(
      `https://www.googleapis.com/drive/v3/files/${fileId}?alt=media`,
      {
        headers: { Authorization: `Bearer ${token}` }
      }
    );
    if (!response.ok) {
      throw new Error(`Failed to download backup file: ${response.statusText}`);
    }
    return response.text();
  }
  async function mergeSyncPayload(remote) {
    const localResult = await chrome.storage.local.get(["notes", "vault", "vaultConfig"]);
    const localNotes = localResult.notes || {};
    localResult.vault || [];
    const localVaultConfig = localResult.vaultConfig || null;
    let changed = false;
    const mergedNotes = { ...localNotes };
    for (const [filename, remoteNote] of Object.entries(remote.notes)) {
      const localNote = localNotes[filename];
      if (!localNote || remoteNote.last_modified > localNote.last_modified) {
        mergedNotes[filename] = remoteNote;
        changed = true;
      }
    }
    let mergedVaultConfig = localVaultConfig;
    if (remote.vaultConfig && (!localVaultConfig || remote.lastUpdated > (localResult.lastUpdated || 0))) {
      mergedVaultConfig = remote.vaultConfig;
      changed = true;
    }
    if (remote.vaultEncrypted && (!localResult.vault || remote.lastUpdated > (localResult.lastUpdated || 0))) {
      await chrome.storage.local.set({ vaultEncrypted: remote.vaultEncrypted });
      changed = true;
    }
    if (changed) {
      await chrome.storage.local.set({
        notes: mergedNotes,
        vaultConfig: mergedVaultConfig,
        lastUpdated: Date.now()
      });
    }
    return changed;
  }
  async function runSync(key) {
    const settings = await chrome.storage.local.get(["syncEngine", "encryptNotes", "notes", "vaultEncrypted", "vaultConfig"]);
    const engine = settings.syncEngine || "none";
    const encrypt = settings.encryptNotes !== false;
    if (engine === "none") return;
    const notes = settings.notes || {};
    const payloadNotes = {};
    if (encrypt && key) {
      for (const [filename, note] of Object.entries(notes)) {
        const encryptedContent = await encryptData(note.content, key);
        payloadNotes[filename] = {
          ...note,
          content: encryptedContent
        };
      }
    } else {
      Object.assign(payloadNotes, notes);
    }
    const payload = {
      notes: payloadNotes,
      vaultEncrypted: settings.vaultEncrypted || null,
      vaultConfig: settings.vaultConfig || null,
      lastUpdated: Date.now()
    };
    const payloadStr = JSON.stringify(payload);
    if (engine === "chrome") {
      const payloadBytes = payloadStr.length * 2;
      if (payloadBytes > 95 * 1024) {
        throw new Error("Chrome Sync quota exceeded. Please switch to Google Drive Sync.");
      }
      await chrome.storage.sync.set({ backup: payloadStr });
    } else if (engine === "gdrive") {
      let finalPayload = payloadStr;
      if (key) {
        finalPayload = await encryptData(payloadStr, key);
      }
      const token = await getGoogleAuthToken(true);
      await uploadToGoogleDrive(finalPayload, token);
    }
  }
  async function pullSync(key) {
    const settings = await chrome.storage.local.get(["syncEngine", "encryptNotes"]);
    const engine = settings.syncEngine || "none";
    const decrypt = settings.encryptNotes !== false;
    if (engine === "none") return false;
    let remoteStr = null;
    if (engine === "chrome") {
      const result = await chrome.storage.sync.get("backup");
      remoteStr = result.backup || null;
    } else if (engine === "gdrive") {
      const token = await getGoogleAuthToken(false).catch(() => null);
      if (!token) return false;
      remoteStr = await downloadFromGoogleDrive(token);
      if (remoteStr && key) {
        try {
          remoteStr = await decryptData(remoteStr, key);
        } catch (e) {
          console.error("Failed to decrypt Google Drive backup payload", e);
          return false;
        }
      }
    }
    if (!remoteStr) return false;
    const remotePayload = JSON.parse(remoteStr);
    if (decrypt && key) {
      for (const [filename, note] of Object.entries(remotePayload.notes)) {
        try {
          note.content = await decryptData(note.content, key);
        } catch (e) {
          console.warn(`Failed to decrypt synced note: ${filename}`, e);
        }
      }
    }
    return await mergeSyncPayload(remotePayload);
  }
  const SENSITIVE_PREFIXES = [
    "ghp_",
    "sk-proj-",
    "PRIVATE KEY",
    "Bearer ",
    "mongodb://",
    "postgres://",
    "mysql://",
    "amqp://",
    "redis://"
  ];
  function calculateShannonEntropy(str) {
    if (!str) return 0;
    const freqs = {};
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
  function isSensitiveClipboard(text) {
    const trimmed = text.trim();
    for (const prefix of SENSITIVE_PREFIXES) {
      if (trimmed.includes(prefix)) return true;
    }
    if (trimmed.length >= 32 && trimmed.length <= 256) {
      const entropy = calculateShannonEntropy(trimmed);
      if (entropy >= 4.2) {
        const isEmail = /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(trimmed);
        const isSimpleUrl = /^https?:\/\/[^\s/]+(?:\/[^\s/]*)*$/.test(trimmed) && !trimmed.includes("?");
        if (!isEmail && !isSimpleUrl) {
          return true;
        }
      }
    }
    return false;
  }
  let rateLimiter = { failedAttempts: 0, lockoutUntil: 0 };
  function checkLockout() {
    if (rateLimiter.lockoutUntil > Date.now()) {
      const remainingSecs = Math.ceil((rateLimiter.lockoutUntil - Date.now()) / 1e3);
      throw new Error(`Too many failed attempts. Vault locked for ${remainingSecs} seconds.`);
    }
  }
  function recordUnlockFailure() {
    rateLimiter.failedAttempts += 1;
    if (rateLimiter.failedAttempts >= 5) {
      rateLimiter.lockoutUntil = Date.now() + 30 * 1e3;
    }
  }
  function recordUnlockSuccess() {
    rateLimiter.failedAttempts = 0;
    rateLimiter.lockoutUntil = 0;
  }
  async function getSessionKey() {
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
  async function saveSessionKey(key) {
    const jwk = await crypto.subtle.exportKey("jwk", key);
    await chrome.storage.session.set({ vaultKey: jwk });
  }
  chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
    if (message.type !== "tauri-command") {
      if (message.type === "check-gdrive-auth") {
        getGoogleAuthToken(false).then(() => sendResponse({ authorized: true })).catch(() => sendResponse({ authorized: false }));
        return true;
      }
      if (message.type === "gdrive-login") {
        getGoogleAuthToken(true).then(() => sendResponse({ success: true })).catch((err) => sendResponse({ success: false, error: err.message }));
        return true;
      }
      if (message.type === "gdrive-logout") {
        logoutGoogleDrive().then(() => sendResponse({ success: true })).catch((err) => sendResponse({ success: false, error: err.message }));
        return true;
      }
      if (message.type === "manual-sync") {
        (async () => {
          try {
            const key = await getSessionKey();
            await pullSync(key);
            await runSync(key);
            sendResponse({ success: true });
          } catch (e) {
            sendResponse({ success: false, error: e.message });
          }
        })();
        return true;
      }
      return false;
    }
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
                display_version: "v2.1 Extension"
              }
            });
            break;
          }
          case "list_notes": {
            await pullSync(key).catch(() => null);
            const notes = await getLocalNotes();
            sendResponse({ data: notes });
            break;
          }
          case "read_note": {
            const result = await chrome.storage.local.get("notes");
            const note = (result.notes || {})[args.filename];
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
              vault: []
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
            await pullSync(derived).catch(() => null);
            sendResponse({ data: null });
            break;
          }
          case "lock_vault": {
            await chrome.storage.session.remove("vaultKey");
            await chrome.storage.local.remove("vault");
            sendResponse({ data: null });
            break;
          }
          case "list_secret_metadata": {
            if (!key) throw new Error("Vault is locked");
            const result = await chrome.storage.local.get("vault");
            const secrets = result.vault || [];
            const meta = secrets.map((s, idx) => ({
              index: idx,
              name: s.name,
              path: s.path
            }));
            sendResponse({ data: meta });
            break;
          }
          case "reveal_secret_value": {
            if (!key) throw new Error("Vault is locked");
            const result = await chrome.storage.local.get("vault");
            const secret = (result.vault || [])[args.index];
            sendResponse({ data: secret ? secret.value : "" });
            break;
          }
          case "copy_secret_to_clipboard": {
            if (!key) throw new Error("Vault is locked");
            const result = await chrome.storage.local.get("vault");
            const secret = (result.vault || [])[args.index];
            if (secret) {
              await navigator.clipboard.writeText(secret.value);
            }
            sendResponse({ data: null });
            break;
          }
          case "add_secret": {
            if (!key) throw new Error("Vault is locked");
            const result = await chrome.storage.local.get("vault");
            const secrets = result.vault || [];
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
            const secrets = result.vault || [];
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
      } catch (error) {
        sendResponse({ error: error.message || String(error) });
      }
    })();
    return true;
  });
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
        const filtered = history.filter((entry) => entry.text !== text);
        filtered.unshift({ text, timestamp: Date.now() });
        if (filtered.length > 15) {
          filtered.pop();
        }
        await saveLocalClipboardHistory(filtered);
        chrome.runtime.sendMessage({ type: "tauri-event", event: "clipboard-updated", payload: filtered }).catch(() => null);
        sendResponse({ success: true });
      })();
      return true;
    }
  });
  chrome.commands.onCommand.addListener(async (command) => {
    if (command === "open_side_panel") {
      const [window] = await chrome.tabs.query({ active: true, currentWindow: true });
      if (window && chrome.sidePanel) {
        await chrome.sidePanel.open({ windowId: window.windowId });
      }
    }
  });
})();

import { Note } from "./storage-extension";
import { encryptData, decryptData } from "./crypto-extension";

const GD_BACKUP_FILENAME = "gnomad_slate_backup.enc";

export interface SyncPayload {
  notes: Record<string, Note>;
  vaultEncrypted: string | null;
  vaultConfig: any | null;
  lastUpdated: number;
}

/** Check if Google Drive OAuth token can be obtained */
export async function getGoogleAuthToken(interactive = true): Promise<string> {
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

/** Revoke current Google Drive OAuth token */
export async function logoutGoogleDrive(): Promise<void> {
  const token = await getGoogleAuthToken(false).catch(() => null);
  if (token) {
    await new Promise<void>((resolve) => {
      fetch(`https://accounts.google.com/o/oauth2/revoke?token=${token}`, { method: "POST" })
        .finally(() => {
          chrome.identity.removeCachedAuthToken({ token }, () => resolve());
        });
    });
  }
}

/** Fetch backup file ID from Google Drive AppData folder */
async function findBackupFile(token: string): Promise<string | null> {
  const response = await fetch(
    "https://www.googleapis.com/drive/v3/files?spaces=appDataFolder&q=name='" + GD_BACKUP_FILENAME + "'",
    {
      headers: { Authorization: `Bearer ${token}` },
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

/** Write encrypted sync payload to Google Drive AppData folder */
export async function uploadToGoogleDrive(payload: string, token: string): Promise<void> {
  const fileId = await findBackupFile(token);

  if (fileId) {
    // Update existing file
    const response = await fetch(
      `https://www.googleapis.com/upload/drive/v3/files/${fileId}?uploadType=media`,
      {
        method: "PATCH",
        headers: {
          Authorization: `Bearer ${token}`,
          "Content-Type": "application/octet-stream",
        },
        body: payload,
      }
    );
    if (!response.ok) {
      throw new Error(`Failed to update backup file: ${response.statusText}`);
    }
  } else {
    // Create new file
    const metadata = {
      name: GD_BACKUP_FILENAME,
      parents: ["appDataFolder"],
    };

    const boundary = "314159265358979323846";
    const delimiter = `\r\n--${boundary}\r\n`;
    const closeDelimiter = `\r\n--${boundary}--`;

    const multipartBody =
      delimiter +
      "Content-Type: application/json; charset=UTF-8\r\n\r\n" +
      JSON.stringify(metadata) +
      delimiter +
      "Content-Type: application/octet-stream\r\n\r\n" +
      payload +
      closeDelimiter;

    const response = await fetch(
      "https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart",
      {
        method: "POST",
        headers: {
          Authorization: `Bearer ${token}`,
          "Content-Type": `multipart/related; boundary=${boundary}`,
        },
        body: multipartBody,
      }
    );
    if (!response.ok) {
      throw new Error(`Failed to create backup file: ${response.statusText}`);
    }
  }
}

/** Download backup payload from Google Drive AppData folder */
export async function downloadFromGoogleDrive(token: string): Promise<string | null> {
  const fileId = await findBackupFile(token);
  if (!fileId) return null;

  const response = await fetch(
    `https://www.googleapis.com/drive/v3/files/${fileId}?alt=media`,
    {
      headers: { Authorization: `Bearer ${token}` },
    }
  );
  if (!response.ok) {
    throw new Error(`Failed to download backup file: ${response.statusText}`);
  }
  return response.text();
}

/** Merge remote payload into local storage using Last-Write-Wins logic */
export async function mergeSyncPayload(remote: SyncPayload): Promise<boolean> {
  const localResult = await chrome.storage.local.get(["notes", "vault", "vaultConfig"]);
  const localNotes = localResult.notes || {};
  const localVault = localResult.vault || [];
  const localVaultConfig = localResult.vaultConfig || null;

  let changed = false;

  // Merge Notes
  const mergedNotes = { ...localNotes };
  for (const [filename, remoteNote] of Object.entries(remote.notes)) {
    const localNote = localNotes[filename];
    if (!localNote || remoteNote.last_modified > localNote.last_modified) {
      mergedNotes[filename] = remoteNote;
      changed = true;
    }
  }

  // Merge Vault Config (prefer newer or remote if local is empty)
  let mergedVaultConfig = localVaultConfig;
  if (remote.vaultConfig && (!localVaultConfig || remote.lastUpdated > (localResult.lastUpdated || 0))) {
    mergedVaultConfig = remote.vaultConfig;
    changed = true;
  }

  // Merge Vault secrets (if encrypted values match or are newer)
  let mergedVault = localVault;
  if (remote.vaultEncrypted && (!localResult.vault || remote.lastUpdated > (localResult.lastUpdated || 0))) {
    // Save raw remote encrypted vault to local
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

/** Synchronize data via selected engine */
export async function runSync(key: CryptoKey | null): Promise<void> {
  const settings = await chrome.storage.local.get(["syncEngine", "encryptNotes", "notes", "vaultEncrypted", "vaultConfig"]);
  const engine = settings.syncEngine || "none";
  const encrypt = settings.encryptNotes !== false;

  if (engine === "none") return;

  // Assemble sync payload
  const notes = settings.notes || {};
  const payloadNotes: Record<string, Note> = {};

  if (encrypt && key) {
    // Encrypt note contents before syncing
    for (const [filename, note] of Object.entries(notes) as [string, Note][]) {
      const encryptedContent = await encryptData(note.content, key);
      payloadNotes[filename] = {
        ...note,
        content: encryptedContent,
      };
    }
  } else {
    // Plaintext sync
    Object.assign(payloadNotes, notes);
  }

  const payload: SyncPayload = {
    notes: payloadNotes,
    vaultEncrypted: settings.vaultEncrypted || null,
    vaultConfig: settings.vaultConfig || null,
    lastUpdated: Date.now(),
  };

  const payloadStr = JSON.stringify(payload);

  if (engine === "chrome") {
    // Verify limits (100 KB total size constraint)
    const payloadBytes = payloadStr.length * 2;
    if (payloadBytes > 95 * 1024) {
      throw new Error("Chrome Sync quota exceeded. Please switch to Google Drive Sync.");
    }
    await chrome.storage.sync.set({ backup: payloadStr });
  } else if (engine === "gdrive") {
    // Encrypt the entire backup string for Google Drive (Double-envelope encryption)
    let finalPayload = payloadStr;
    if (key) {
      finalPayload = await encryptData(payloadStr, key);
    }
    
    const token = await getGoogleAuthToken(true);
    await uploadToGoogleDrive(finalPayload, token);
  }
}

/** Pull and merge updates from selected sync engine */
export async function pullSync(key: CryptoKey | null): Promise<boolean> {
  const settings = await chrome.storage.local.get(["syncEngine", "encryptNotes"]);
  const engine = settings.syncEngine || "none";
  const decrypt = settings.encryptNotes !== false;

  if (engine === "none") return false;

  let remoteStr: string | null = null;

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

  const remotePayload = JSON.parse(remoteStr) as SyncPayload;

  // Decrypt notes if needed
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

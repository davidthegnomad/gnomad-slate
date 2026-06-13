export interface Note {
  filename: string;
  title: string;
  content: string;
  is_ephemeral: boolean;
  last_modified: number;
}

export interface SecretItem {
  name: string;
  path: string;
  value: string;
}

export interface SecretMeta {
  index: number;
  name: string;
  path: string;
}

export interface ClipboardEntry {
  text: string;
  timestamp: number;
}

/** Get list of notes from chrome.storage.local */
export async function getLocalNotes(): Promise<Note[]> {
  const result = await chrome.storage.local.get("notes");
  const notesMap = result.notes || {};
  return Object.values(notesMap) as Note[];
}

/** Write/Update note in chrome.storage.local */
export async function writeLocalNote(title: string, content: string, isEphemeral: boolean, filename?: string): Promise<Note> {
  const result = await chrome.storage.local.get("notes");
  const notesMap = result.notes || {};

  const fileId = filename || (isEphemeral ? `temp_${Date.now()}.md` : `vault_${Date.now()}.md`);
  const note: Note = {
    filename: fileId,
    title,
    content,
    is_ephemeral: isEphemeral,
    last_modified: Date.now(),
  };

  notesMap[fileId] = note;
  await chrome.storage.local.set({ notes: notesMap });
  return note;
}

/** Delete a note from chrome.storage.local */
export async function deleteLocalNote(filename: string): Promise<void> {
  const result = await chrome.storage.local.get("notes");
  const notesMap = result.notes || {};
  delete notesMap[filename];
  await chrome.storage.local.set({ notes: notesMap });
}

/** Archive notes older than 48 hours */
export async function archiveLocalNotes(): Promise<number> {
  const result = await chrome.storage.local.get("notes");
  const notesMap = result.notes || {};
  let count = 0;
  const fortyEightHoursAgo = Date.now() - 48 * 60 * 60 * 1000;

  for (const [filename, note] of Object.entries(notesMap) as [string, Note][]) {
    if (note.is_ephemeral && note.last_modified < fortyEightHoursAgo) {
      // Move to archive partition
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

/** Get Clipboard History from session storage */
export async function getLocalClipboardHistory(): Promise<ClipboardEntry[]> {
  const result = await chrome.storage.session.get("clipboardHistory");
  return result.clipboardHistory || [];
}

/** Save Clipboard History to session storage */
export async function saveLocalClipboardHistory(history: ClipboardEntry[]): Promise<void> {
  await chrome.storage.session.set({ clipboardHistory: history });
}

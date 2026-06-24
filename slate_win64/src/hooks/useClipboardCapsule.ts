import { useCallback, useState, type RefObject } from "react";
import { invoke } from "../utils/ipc";
import type { ClipboardEntry } from "../types/app";

type UseClipboardCapsuleOptions = {
  editorRef: RefObject<HTMLTextAreaElement | null>;
  setEditorContent: (value: string | ((prev: string) => string)) => void;
  saveActiveNote: (content?: string, title?: string, forceVault?: boolean) => void;
};

/**
 * Clipboard history capsule state and paste-into-editor logic.
 * Desktop path uses Rust IPC; extension bridge is isolated in `fetchClipboardHistory`.
 */
export function useClipboardCapsule({
  editorRef,
  setEditorContent,
  saveActiveNote,
}: UseClipboardCapsuleOptions) {
  const [open, setOpen] = useState(false);
  const [history, setHistory] = useState<ClipboardEntry[]>([]);
  const [selectedIdx, setSelectedIdx] = useState(0);

  const fetchClipboardHistory = useCallback(async () => {
    try {
      // Browser extension: capture active clipboard text before fetching history.
      if (
        typeof window !== "undefined" &&
        !(window as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__ &&
        typeof navigator !== "undefined" &&
        navigator.clipboard?.readText
      ) {
        try {
          const text = await navigator.clipboard.readText();
          if (text?.trim()) {
            const chromeApi = (globalThis as { chrome?: typeof chrome }).chrome;
            const runtime = chromeApi?.runtime;
            if (runtime?.sendMessage) {
              await new Promise<void>((resolve) => {
                runtime.sendMessage!({ type: "record-clipboard", text }, () => resolve());
              });
            }
          }
        } catch {
          // Permission not granted yet — ignore.
        }
      }

      const historyEntries = await invoke<ClipboardEntry[]>("get_clipboard_history");
      setHistory(historyEntries);
    } catch (e) {
      console.error(e);
    }
  }, []);

  const openCapsule = useCallback(() => {
    setOpen(true);
    setSelectedIdx(0);
    void fetchClipboardHistory();
  }, [fetchClipboardHistory]);

  const toggleCapsule = useCallback(() => {
    setOpen((prev) => !prev);
    setSelectedIdx(0);
    void fetchClipboardHistory();
  }, [fetchClipboardHistory]);

  const closeCapsule = useCallback(() => setOpen(false), []);

  const pasteFromHistory = useCallback(
    (text: string) => {
      if (editorRef.current) {
        const textarea = editorRef.current;
        const start = textarea.selectionStart;
        const end = textarea.selectionEnd;
        const val = textarea.value;
        const updatedVal = val.substring(0, start) + text + val.substring(end);
        setEditorContent(updatedVal);
        saveActiveNote(updatedVal);
      } else {
        setEditorContent((prev) => prev + "\n" + text);
      }
      closeCapsule();
    },
    [editorRef, setEditorContent, saveActiveNote, closeCapsule]
  );

  return {
    open,
    setOpen,
    history,
    selectedIdx,
    setSelectedIdx,
    fetchClipboardHistory,
    openCapsule,
    toggleCapsule,
    closeCapsule,
    pasteFromHistory,
  };
}

export type ClipboardCapsuleApi = ReturnType<typeof useClipboardCapsule>;

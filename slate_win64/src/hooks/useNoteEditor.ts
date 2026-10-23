import { useCallback, useEffect, useMemo, useRef, useState, type MutableRefObject } from "react";
import { invoke, quitDesktopApp } from "../utils/ipc";
import { NOTE_SAVE_DEBOUNCE_MS } from "../constants/timing";
import type { EditorTab, Note } from "../types/app";

type UseNoteEditorOptions = {
  activeTabRef: MutableRefObject<EditorTab>;
  setActiveTab: (tab: EditorTab) => void;
  quittingRef: MutableRefObject<boolean>;
};

/**
 * Note list, editor state, debounced autosave, and CRUD.
 * Refs mirror React state so debounced saves never use stale closures.
 */
export function useNoteEditor({ activeTabRef, setActiveTab, quittingRef }: UseNoteEditorOptions) {
  const [notes, setNotes] = useState<Note[]>([]);
  const [selectedNote, setSelectedNote] = useState<Note | null>(null);
  const [editorTitle, setEditorTitle] = useState("");
  const [editorContent, setEditorContent] = useState("");
  const [isEphemeral, setIsEphemeral] = useState(true);
  const [saveFeedback, setSaveFeedback] = useState<string | null>(null);

  const editorRef = useRef<HTMLTextAreaElement>(null);
  const saveDebounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const noteDirtyRef = useRef(false);
  const editorTitleRef = useRef(editorTitle);
  const editorContentRef = useRef(editorContent);
  const isEphemeralRef = useRef(isEphemeral);
  const selectedNoteRef = useRef<Note | null>(null);
  const saveFeedbackTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    selectedNoteRef.current = selectedNote;
  }, [selectedNote]);

  useEffect(() => {
    editorTitleRef.current = editorTitle;
  }, [editorTitle]);

  useEffect(() => {
    editorContentRef.current = editorContent;
  }, [editorContent]);

  useEffect(() => {
    isEphemeralRef.current = isEphemeral;
  }, [isEphemeral]);

  useEffect(() => {
    return () => {
      if (saveDebounceRef.current) {
        clearTimeout(saveDebounceRef.current);
      }
      if (saveFeedbackTimerRef.current) {
        clearTimeout(saveFeedbackTimerRef.current);
      }
    };
  }, []);

  const showSaveFeedback = useCallback((message: string) => {
    setSaveFeedback(message);
    if (saveFeedbackTimerRef.current) {
      clearTimeout(saveFeedbackTimerRef.current);
    }
    saveFeedbackTimerRef.current = setTimeout(() => {
      setSaveFeedback(null);
      saveFeedbackTimerRef.current = null;
    }, 1500);
  }, []);

  const saveNoteNow = useCallback(
    async (options?: { promoteToSaved?: boolean; forQuit?: boolean }) => {
      if (activeTabRef.current === "secrets" && !options?.forQuit) {
        return;
      }

      if (saveDebounceRef.current) {
        clearTimeout(saveDebounceRef.current);
        saveDebounceRef.current = null;
      }

      if (options?.forQuit && !noteDirtyRef.current) {
        return;
      }

      const title = editorTitleRef.current.trim() || "Untitled note";
      const content = editorContentRef.current;
      const promote = options?.promoteToSaved ?? false;
      const ephemeralState = promote ? false : isEphemeralRef.current;

      if (!editorTitleRef.current.trim()) {
        setEditorTitle(title);
        editorTitleRef.current = title;
      }

      try {
        const previousFilename = selectedNote?.filename ?? null;
        const saved = await invoke<Note>("write_note", {
          title,
          content,
          isEphemeral: ephemeralState,
          previousFilename,
        });

        setNotes((prev) => {
          const filtered = prev.filter((n) => n.filename !== saved.filename);
          if (previousFilename && previousFilename !== saved.filename) {
            return [saved, ...filtered.filter((n) => n.filename !== previousFilename)];
          }
          return [saved, ...filtered];
        });

        setSelectedNote(saved);
        setIsEphemeral(saved.is_ephemeral);
        isEphemeralRef.current = saved.is_ephemeral;
        noteDirtyRef.current = false;
        showSaveFeedback(promote || !saved.is_ephemeral ? "Saved" : "Draft saved");
      } catch (e) {
        console.error("Save failed:", e);
        showSaveFeedback("Save failed");
      }
    },
    [activeTabRef, showSaveFeedback, selectedNote]
  );

  const saveActiveNote = useCallback(
    async (overrideContent?: string, overrideTitle?: string, forceVault?: boolean) => {
      if (overrideContent !== undefined) {
        editorContentRef.current = overrideContent;
        setEditorContent(overrideContent);
      }
      if (overrideTitle !== undefined) {
        editorTitleRef.current = overrideTitle;
        setEditorTitle(overrideTitle);
      }
      if (forceVault !== undefined) {
        isEphemeralRef.current = !forceVault;
        setIsEphemeral(!forceVault);
      }

      if (!editorTitleRef.current.trim()) {
        return;
      }

      await saveNoteNow({ promoteToSaved: forceVault === true });
    },
    [saveNoteNow]
  );

  const flushPendingSave = useCallback(async () => {
    const pending = saveDebounceRef.current !== null || noteDirtyRef.current;
    if (saveDebounceRef.current) {
      clearTimeout(saveDebounceRef.current);
      saveDebounceRef.current = null;
    }
    if (pending) {
      await saveActiveNote();
    }
  }, [saveActiveNote]);

  const scheduleSaveNote = useCallback(
    (overrideContent?: string, overrideTitle?: string, forceVault?: boolean) => {
      noteDirtyRef.current = true;
      if (saveDebounceRef.current) {
        clearTimeout(saveDebounceRef.current);
      }
      saveDebounceRef.current = setTimeout(() => {
        saveDebounceRef.current = null;
        void saveActiveNote(overrideContent, overrideTitle, forceVault);
      }, NOTE_SAVE_DEBOUNCE_MS);
    },
    [saveActiveNote]
  );

  const selectNote = useCallback(
    async (note: Note) => {
      await flushPendingSave();
      setSelectedNote(note);
      setEditorTitle(note.title);
      let content = note.content;
      if (content.startsWith("<!-- ephemeral -->\n")) {
        content = content.replace("<!-- ephemeral -->\n", "");
      }
      setEditorContent(content);
      setIsEphemeral(note.is_ephemeral);
      noteDirtyRef.current = false;
      if (saveDebounceRef.current) {
        clearTimeout(saveDebounceRef.current);
        saveDebounceRef.current = null;
      }
    },
    [flushPendingSave]
  );

  const loadNotes = useCallback(async () => {
    try {
      const allNotes = await invoke<Note[]>("list_notes");
      setNotes(allNotes);
      if (allNotes.length > 0 && !selectedNoteRef.current) {
        await selectNote(allNotes[0]);
      }
    } catch (e) {
      console.error("Failed to load notes:", e);
    }
  }, [selectNote]);

  const createNewDraft = useCallback(async () => {
    await saveNoteNow();
    const draftTitle = `Note_${new Date().toLocaleTimeString().replace(/\s/g, "_")}`;
    setEditorTitle(draftTitle);
    setEditorContent("");
    setIsEphemeral(true);
    setActiveTab("scratch");
    editorTitleRef.current = draftTitle;
    editorContentRef.current = "";
    isEphemeralRef.current = true;
    activeTabRef.current = "scratch";

    try {
      const saved = await invoke<Note>("write_note", {
        title: draftTitle,
        content: "",
        isEphemeral: true,
        previousFilename: null,
      });
      setNotes((prev) => [saved, ...prev.filter((n) => n.filename !== saved.filename)]);
      setSelectedNote(saved);
    } catch (e) {
      console.error("Failed to create draft:", e);
    }
  }, [saveNoteNow, setActiveTab, activeTabRef]);

  const createNewSavedNote = useCallback(async () => {
    await saveNoteNow();
    const title = `Note_${new Date().toISOString().slice(0, 10)}`;
    setEditorTitle(title);
    setEditorContent("");
    setIsEphemeral(false);
    setActiveTab("vault");
    editorTitleRef.current = title;
    editorContentRef.current = "";
    isEphemeralRef.current = false;
    activeTabRef.current = "vault";

    try {
      const saved = await invoke<Note>("write_note", {
        title,
        content: "",
        isEphemeral: false,
        previousFilename: null,
      });
      setNotes((prev) => [saved, ...prev.filter((n) => n.filename !== saved.filename)]);
      setSelectedNote(saved);
    } catch (e) {
      console.error("Failed to create saved note:", e);
    }
  }, [saveNoteNow, setActiveTab, activeTabRef]);

  const deleteCurrentNote = useCallback(async () => {
    if (!selectedNote) {
      return;
    }
    try {
      await invoke("delete_note", { filename: selectedNote.filename });
      setNotes((prev) => prev.filter((n) => n.filename !== selectedNote.filename));
      setSelectedNote(null);
      setEditorTitle("");
      setEditorContent("");
    } catch (e) {
      console.error(e);
    }
  }, [selectedNote]);

  const deleteNote = useCallback(async (filename: string, e: React.MouseEvent) => {
    e.stopPropagation();
    try {
      await invoke("delete_note", { filename });
      setNotes((prev) => prev.filter((n) => n.filename !== filename));
      if (selectedNote?.filename === filename) {
        setSelectedNote(null);
        setEditorTitle("");
        setEditorContent("");
      }
    } catch (err) {
      console.error(err);
    }
  }, [selectedNote]);

  const handleEditorShortcut = useCallback(
    (e: React.KeyboardEvent) => {
      const isCmdOrCtrl = e.metaKey || e.ctrlKey;
      if (!isCmdOrCtrl) {
        return;
      }
      if (!e.shiftKey && (e.key === "s" || e.key === "S")) {
        e.preventDefault();
        void saveNoteNow({ promoteToSaved: isEphemeralRef.current });
        return;
      }
      if (!e.shiftKey && (e.key === "n" || e.key === "N")) {
        e.preventDefault();
        void createNewDraft();
      }
    },
    [saveNoteNow, createNewDraft]
  );

  const closeApp = useCallback(async () => {
    if (quittingRef.current) {
      return;
    }
    await saveNoteNow({ forQuit: true });
    quittingRef.current = true;
    try {
      await quitDesktopApp();
    } catch (error) {
      quittingRef.current = false;
      console.error("Quit failed:", error);
    }
  }, [saveNoteNow, quittingRef]);

  const lineNumberRows = useMemo(() => {
    const lineCount = Math.max(1, editorContent.split("\n").length);
    return Array.from({ length: lineCount }, (_, index) => index + 1);
  }, [editorContent]);

  return {
    notes,
    selectedNote,
    editorTitle,
    setEditorTitle,
    editorContent,
    setEditorContent,
    isEphemeral,
    setIsEphemeral,
    saveFeedback,
    editorRef,
    saveDebounceRef,
    loadNotes,
    selectNote,
    saveNoteNow,
    saveActiveNote,
    flushPendingSave,
    scheduleSaveNote,
    createNewDraft,
    createNewSavedNote,
    deleteCurrentNote,
    deleteNote,
    handleEditorShortcut,
    closeApp,
    lineNumberRows,
    isEphemeralRef,
  };
}

export type NoteEditorApi = ReturnType<typeof useNoteEditor>;

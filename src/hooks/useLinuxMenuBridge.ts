import { useEffect, useRef } from "react";
import { invoke, listen } from "../utils/ipc";
import type { ThemePreference } from "../utils/theme";

type MenuBridgeOptions = {
  createNewDraft: () => Promise<void>;
  createNewSavedNote: () => Promise<void>;
  saveNoteNow: (options?: { promoteToSaved?: boolean }) => Promise<void>;
  deleteCurrentNote: () => Promise<void>;
  loadNotes: () => Promise<void>;
  loadClipboardHistory: () => Promise<void>;
  setEditorView: (view: "edit" | "preview" | "split") => void;
  setShowPalette: (open: boolean) => void;
  setPaletteQuery: (query: string) => void;
  setSelectedPaletteIdx: (idx: number) => void;
  setShowClipboard: (open: boolean) => void;
  setSelectedClipIdx: (idx: number) => void;
  setShowLineNumbers: (value: boolean | ((prev: boolean) => boolean)) => void;
  setTheme: (theme: ThemePreference) => void;
  setShowHelp: (open: boolean) => void;
  setAboutAutoCheck: (check: boolean) => void;
  setShowAbout: (open: boolean) => void;
};

/** Wire native Linux menu bar events to React state (no-op on other platforms). */
export function useLinuxMenuBridge(options: MenuBridgeOptions) {
  const optionsRef = useRef(options);
  optionsRef.current = options;

  useEffect(() => {
    const handlers: Array<Promise<() => void>> = [
      listen("menu-about", () => {
        optionsRef.current.setAboutAutoCheck(false);
        optionsRef.current.setShowAbout(true);
      }),
      listen("menu-check-updates", () => {
        optionsRef.current.setAboutAutoCheck(true);
        optionsRef.current.setShowAbout(true);
      }),
      listen("menu-new-draft", () => {
        void optionsRef.current.createNewDraft();
      }),
      listen("menu-new-saved-note", () => {
        void optionsRef.current.createNewSavedNote();
      }),
      listen("menu-save", () => {
        void optionsRef.current.saveNoteNow();
      }),
      listen("menu-save-permanent", () => {
        void optionsRef.current.saveNoteNow({ promoteToSaved: true });
      }),
      listen("menu-delete-note", () => {
        void optionsRef.current.deleteCurrentNote();
      }),
      listen("menu-archive-stale", () => {
        void invoke<number>("archive_notes").then((count) => {
          if (count > 0) {
            void optionsRef.current.loadNotes();
          }
        });
      }),
      listen("menu-view-edit", () => optionsRef.current.setEditorView("edit")),
      listen("menu-view-split", () => optionsRef.current.setEditorView("split")),
      listen("menu-view-preview", () => optionsRef.current.setEditorView("preview")),
      listen("menu-command-palette", () => {
        optionsRef.current.setPaletteQuery("");
        optionsRef.current.setSelectedPaletteIdx(0);
        optionsRef.current.setShowPalette(true);
        optionsRef.current.setShowClipboard(false);
      }),
      listen("menu-clipboard-history", () => {
        void optionsRef.current.loadClipboardHistory();
        optionsRef.current.setSelectedClipIdx(0);
        optionsRef.current.setShowClipboard(true);
        optionsRef.current.setShowPalette(false);
      }),
      listen("menu-toggle-line-numbers", () => {
        optionsRef.current.setShowLineNumbers((prev) => !prev);
      }),
      listen("menu-theme-light", () => optionsRef.current.setTheme("light")),
      listen("menu-theme-dark", () => optionsRef.current.setTheme("dark")),
      listen("menu-theme-system", () => optionsRef.current.setTheme("system")),
      listen("menu-help", () => optionsRef.current.setShowHelp(true)),
    ];

    return () => {
      void Promise.all(handlers).then((unsubs) => {
        unsubs.forEach((unsub) => unsub());
      });
    };
  }, []);
}

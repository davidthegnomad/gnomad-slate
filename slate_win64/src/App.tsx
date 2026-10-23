import { useState, useEffect, useRef, useCallback, useMemo } from "react";
import { invoke } from "./utils/ipc";
import { AboutDialog } from "./components/AboutDialog";
import { AppMessageDialog } from "./components/AppMessageDialog";
import { ClipboardCapsule } from "./components/ClipboardCapsule";
import { CommandPalette } from "./components/CommandPalette";
import { HelpDialog } from "./components/HelpDialog";
import { Sidebar } from "./components/Sidebar";
import { EditorWorkstation } from "./components/EditorWorkstation";
import { VaultPanel } from "./components/VaultPanel";
import UnsavedChangesModal from "./components/UnsavedChangesModal";
import { useDesktopMenuBridge } from "./hooks/useDesktopMenuBridge";
import { useAppSettings } from "./hooks/useAppSettings";
import { useClipboardCapsule } from "./hooks/useClipboardCapsule";
import { useCommandPalette } from "./hooks/useCommandPalette";
import { useDebouncedPreview } from "./hooks/useDebouncedPreview";
import { useGlobalKeybindings } from "./hooks/useGlobalKeybindings";
import { useNoteEditor } from "./hooks/useNoteEditor";
import { useVault } from "./hooks/useVault";
import { useWindowClose } from "./hooks/useWindowClose";
import { SEARCH_DEBOUNCE_MS } from "./constants/timing";
import { isWindowsDesktop } from "./utils/platform";
import type { AppVersionInfo, EditorTab, EditorViewMode } from "./types/app";
import "./App.css";

function App() {
  const [searchQuery, setSearchQuery] = useState("");
  const [debouncedSearchQuery, setDebouncedSearchQuery] = useState("");
  const [activeTab, setActiveTab] = useState<EditorTab>("scratch");
  const [editorView, setEditorView] = useState<EditorViewMode>("edit");
  const activeTabRef = useRef(activeTab);

  const [isWindows] = useState(() => isWindowsDesktop());
  const [closePromptOpen, setClosePromptOpen] = useState(false);
  const [appMessage, setAppMessage] = useState<string | null>(null);
  const [showAbout, setShowAbout] = useState(false);
  const [showHelp, setShowHelp] = useState(false);
  const [summonShortcutLabel, setSummonShortcutLabel] = useState("Alt+Space");
  const [aboutAutoCheck, setAboutAutoCheck] = useState(false);
  const [appVersionLabel, setAppVersionLabel] = useState("v2.1 beta");
  const pendingWindowClose = useRef(false);
  const quittingRef = useRef(false);

  useEffect(() => {
    activeTabRef.current = activeTab;
  }, [activeTab]);

  const showAppMessage = useCallback(
    (message: string) => {
      if (isWindows) {
        setAppMessage(message);
      } else {
        alert(message);
      }
    },
    [isWindows]
  );

  const {
    theme,
    setTheme,
    clipboardFilterEnabled,
    setClipboardFilterEnabled,
    showLineNumbers,
    setShowLineNumbers,
  } = useAppSettings(isWindows);

  const vault = useVault(showAppMessage);
  const { checkVaultStatus } = vault;

  const notes = useNoteEditor({ activeTabRef, setActiveTab, quittingRef });
  const {
    editorTitle,
    setEditorTitle,
    editorContent,
    setEditorContent,
    isEphemeral,
    setIsEphemeral,
    selectedNote,
    saveFeedback,
    editorRef,
    saveDebounceRef,
    loadNotes,
    selectNote,
    saveNoteNow,
    saveActiveNote,
    createNewDraft,
    createNewSavedNote,
    deleteCurrentNote,
    deleteNote,
    handleEditorShortcut,
    closeApp,
    flushPendingSave,
    lineNumberRows,
    notes: noteList,
    isEphemeralRef,
    scheduleSaveNote,
  } = notes;

  const previewHtml = useDebouncedPreview(editorContent, editorView);

  const paletteCtx = useMemo(
    () => ({
      editorContent,
      setEditorContent,
      saveActiveNote,
      setTheme,
      setActiveTab,
      loadNotes,
      showAppMessage,
      appendSource: () => {
        const el = editorRef.current;
        if (!el) return editorContent;
        if (el.selectionStart !== el.selectionEnd) {
          return el.value.slice(el.selectionStart, el.selectionEnd);
        }
        return "";
      },
      isVaultUnlocked: vault.isVaultUnlocked,
      lockVault: vault.handleLockVault,
      flushEditor: async () => {
        if (activeTabRef.current !== "secrets") {
          await saveNoteNow();
        }
      },
    }),
    [editorContent, setEditorContent, saveActiveNote, setTheme, setActiveTab, loadNotes, showAppMessage, editorRef, vault.isVaultUnlocked, vault.handleLockVault, saveNoteNow]
  );

  const palette = useCommandPalette(paletteCtx);
  const clipboard = useClipboardCapsule({ editorRef, setEditorContent, saveActiveNote });

  const dismissOverlays = useCallback(() => {
    palette.closePalette();
    clipboard.closeCapsule();
  }, [palette, clipboard]);

  const handleTogglePalette = useCallback(() => {
    clipboard.closeCapsule();
    palette.togglePalette();
  }, [clipboard, palette]);

  const handleToggleClipboard = useCallback(() => {
    palette.closePalette();
    clipboard.toggleCapsule();
  }, [clipboard, palette]);

  const handleSaveNote = useCallback(() => {
    void saveNoteNow({ promoteToSaved: isEphemeralRef.current });
  }, [saveNoteNow, isEphemeralRef]);

  const handleNewDraft = useCallback(() => {
    void createNewDraft();
  }, [createNewDraft]);

  useGlobalKeybindings({
    isWindows,
    activeTabRef,
    isEphemeralRef,
    onShowHelp: () => setShowHelp(true),
    onDismissOverlays: dismissOverlays,
    onSaveNote: handleSaveNote,
    onNewDraft: handleNewDraft,
    onTogglePalette: handleTogglePalette,
    onToggleClipboard: handleToggleClipboard,
  });

  useEffect(() => {
    const timer = setTimeout(() => setDebouncedSearchQuery(searchQuery), SEARCH_DEBOUNCE_MS);
    return () => clearTimeout(timer);
  }, [searchQuery]);

  useEffect(() => {
    if (isWindows) {
      document.documentElement.setAttribute("data-platform", "windows");
    }
  }, [isWindows]);

  useEffect(() => {
    void invoke<AppVersionInfo>("get_app_version_info")
      .then((info) => setAppVersionLabel(info.display_version))
      .catch(() => undefined);
    void invoke<string>("get_summon_shortcut_label")
      .then((label) => setSummonShortcutLabel(label))
      .catch(() => undefined);
  }, []);

  useEffect(() => {
    void (async () => {
      const count = await invoke<number>("archive_notes").catch(() => 0);
      if (count > 0) {
        console.log(`Auto-archived ${count} stale temporary notes.`);
      }
      await loadNotes();
      await checkVaultStatus();
    })();
  }, [loadNotes, checkVaultStatus]);

  useWindowClose({
    saveDebounceRef,
    pendingWindowClose,
    quittingRef,
    setClosePromptOpen,
    saveNoteNow,
    closeApp,
  });

  useDesktopMenuBridge({
    createNewDraft,
    createNewSavedNote,
    saveNoteNow,
    deleteCurrentNote,
    loadNotes,
    loadClipboardHistory: clipboard.fetchClipboardHistory,
    setEditorView,
    setShowPalette: palette.setOpen,
    setPaletteQuery: palette.setQuery,
    setSelectedPaletteIdx: palette.setSelectedIdx,
    setShowClipboard: clipboard.setOpen,
    setSelectedClipIdx: clipboard.setSelectedIdx,
    setShowLineNumbers,
    setTheme,
    setShowHelp,
    setAboutAutoCheck,
    setShowAbout,
  });

  // --- Render Notes Filter list ---
  const filteredNotes = useMemo(() => {
    const query = debouncedSearchQuery.toLowerCase();
    return noteList.filter((note) => {
      const matchesSearch =
        note.title.toLowerCase().includes(query) ||
        note.content.toLowerCase().includes(query);

      if (activeTab === "scratch") return matchesSearch && note.is_ephemeral;
      if (activeTab === "vault") return matchesSearch && !note.is_ephemeral;
      return matchesSearch;
    });
  }, [noteList, debouncedSearchQuery, activeTab]);

  return (
    <div className="app-container">
      <Sidebar
        appVersionLabel={appVersionLabel}
        searchQuery={searchQuery}
        onSearchChange={setSearchQuery}
        activeTab={activeTab}
        onTabChange={(tab) => {
          void (async () => {
            if (tab === "secrets" && activeTabRef.current !== "secrets") {
              await saveNoteNow();
            }
            setActiveTab(tab);
          })();
        }}
        filteredNotes={filteredNotes}
        selectedNoteFilename={selectedNote?.filename ?? null}
        onSelectNote={selectNote}
        onDeleteNote={deleteNote}
        summonShortcutLabel={summonShortcutLabel}
        clipboardFilterEnabled={clipboardFilterEnabled}
        onClipboardFilterChange={setClipboardFilterEnabled}
        theme={theme}
        onThemeChange={setTheme}
      />

      <section style={{ height: "100%", overflow: "hidden" }}>
        {activeTab === "secrets" ? (
          <VaultPanel vault={vault} />
        ) : (
          <EditorWorkstation
            isWindows={isWindows}
            editorView={editorView}
            onEditorViewChange={setEditorView}
            showLineNumbers={showLineNumbers}
            onToggleLineNumbers={() => setShowLineNumbers((prev) => !prev)}
            saveFeedback={saveFeedback}
            isEphemeral={isEphemeral}
            onToggleEphemeral={() => {
              const nextVal = !isEphemeral;
              setIsEphemeral(nextVal);
              saveActiveNote(editorContent, editorTitle, !nextVal);
            }}
            onNewDraft={() => void createNewDraft()}
            editorTitle={editorTitle}
            onEditorTitleChange={setEditorTitle}
            onEditorShortcut={handleEditorShortcut}
            selectedFilename={selectedNote?.filename ?? null}
            editorContent={editorContent}
            onEditorContentChange={setEditorContent}
            editorRef={editorRef}
            lineNumberRows={lineNumberRows}
            previewHtml={previewHtml}
            scheduleSaveNote={scheduleSaveNote}
          />
        )}
      </section>

      <ClipboardCapsule
        open={clipboard.open}
        history={clipboard.history}
        selectedIdx={clipboard.selectedIdx}
        closeCapsule={clipboard.closeCapsule}
        pasteFromHistory={clipboard.pasteFromHistory}
      />

      <CommandPalette
        open={palette.open}
        query={palette.query}
        setQuery={palette.setQuery}
        selectedIdx={palette.selectedIdx}
        closePalette={palette.closePalette}
        runCommand={palette.runCommand}
        filteredCommands={palette.filteredCommands}
      />

      <AboutDialog
        open={showAbout}
        autoCheckUpdates={aboutAutoCheck}
        onClose={() => {
          setShowAbout(false);
          setAboutAutoCheck(false);
        }}
      />
      {isWindows && (
        <HelpDialog open={showHelp} onClose={() => setShowHelp(false)} />
      )}
      <AppMessageDialog
        open={appMessage !== null}
        message={appMessage ?? ""}
        onClose={() => setAppMessage(null)}
      />
      {closePromptOpen && (
        <UnsavedChangesModal
          noteTitle={editorTitle}
          onSave={() => {
            void (async () => {
              await flushPendingSave();
              setClosePromptOpen(false);
              await closeApp();
            })();
          }}
          onDiscard={() => {
            void (async () => {
              setClosePromptOpen(false);
              await closeApp();
            })();
          }}
          onCancel={() => {
            pendingWindowClose.current = false;
            setClosePromptOpen(false);
          }}
        />
      )}
    </div>
  );
}

export default App;

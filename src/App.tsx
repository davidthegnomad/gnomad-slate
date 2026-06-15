import { useState, useEffect, useRef, useCallback, useMemo, type ButtonHTMLAttributes } from "react";
import { invoke, listen, getPlatformWindow, LogicalSize } from "./utils/ipc";
import { AboutDialog } from "./components/AboutDialog";
import UnsavedChangesModal from "./components/UnsavedChangesModal";
import { renderMarkdownToHtml } from "./utils/markdown";
import { modKeyLabel, modShortcut } from "./utils/modKeyLabel";
import { resolveTheme, type ThemePreference } from "./utils/theme";
import "./App.css";

interface Note {
  filename: string;
  title: string;
  content: string;
  is_ephemeral: boolean;
  last_modified: number;
}

interface ClipboardEntry {
  text: string;
  timestamp: number;
}

interface SecretMeta {
  index: number;
  name: string;
  path: string;
}

interface AppVersionInfo {
  version: string;
  channel: string;
  display_version: string;
}

const NOTE_SAVE_DEBOUNCE_MS = 400;

function TipButton({
  tip,
  tipBelow = false,
  className,
  children,
  ...rest
}: ButtonHTMLAttributes<HTMLButtonElement> & { tip: string; tipBelow?: boolean }) {
  return (
    <button
      {...rest}
      className={`has-tooltip${tipBelow ? " tooltip-below" : ""}${className ? ` ${className}` : ""}`}
      data-tooltip={tip}
    >
      {children}
    </button>
  );
}

function invokeErrorMessage(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  return String(error);
}

function App() {
  // --- Notes State ---
  const [notes, setNotes] = useState<Note[]>([]);
  const [selectedNote, setSelectedNote] = useState<Note | null>(null);
  const [editorTitle, setEditorTitle] = useState("");
  const [editorContent, setEditorContent] = useState("");
  const [isEphemeral, setIsEphemeral] = useState(true);
  const [searchQuery, setSearchQuery] = useState("");
  const [activeTab, setActiveTab] = useState<"all" | "scratch" | "vault" | "secrets">("scratch");
  const [editorView, setEditorView] = useState<"edit" | "preview" | "split">("edit");

  // --- Clipboard Capsule State ---
  const [showClipboard, setShowClipboard] = useState(false);
  const [clipboardHistory, setClipboardHistory] = useState<ClipboardEntry[]>([]);
  const [selectedClipIdx, setSelectedClipIdx] = useState(0);

  // --- Command Palette State ---
  const [showPalette, setShowPalette] = useState(false);
  const [paletteQuery, setPaletteQuery] = useState("");
  const [selectedPaletteIdx, setSelectedPaletteIdx] = useState(0);

  // --- Secure Vault (Vault-X) State ---
  const [isVaultInitialized, setIsVaultInitialized] = useState(false);
  const [isVaultUnlocked, setIsVaultUnlocked] = useState(false);
  const [vaultPassword, setVaultPassword] = useState("");
  const [secretMetadata, setSecretMetadata] = useState<SecretMeta[]>([]);
  const [revealedIndex, setRevealedIndex] = useState<number | null>(null);
  const [revealedValue, setRevealedValue] = useState("");
  
  // Add Secret Form
  const [showAddSecret, setShowAddSecret] = useState(false);
  const [newSecretName, setNewSecretName] = useState("");
  const [newSecretPath, setNewSecretPath] = useState("");
  const [newSecretValue, setNewSecretValue] = useState("");

  const [showAbout, setShowAbout] = useState(false);
  const [aboutAutoCheck, setAboutAutoCheck] = useState(false);
  const [appVersionLabel, setAppVersionLabel] = useState("v2.1 beta");
  const [closePromptOpen, setClosePromptOpen] = useState(false);
  const pendingWindowClose = useRef(false);

  const editorRef = useRef<HTMLTextAreaElement>(null);
  const saveDebounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const editorTitleRef = useRef(editorTitle);
  const editorContentRef = useRef(editorContent);
  const isEphemeralRef = useRef(isEphemeral);
  const activeTabRef = useRef(activeTab);

  const [saveFeedback, setSaveFeedback] = useState<string | null>(null);
  const saveFeedbackTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

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
    activeTabRef.current = activeTab;
  }, [activeTab]);

  // --- Theme State ---
  const [theme, setTheme] = useState<ThemePreference>(() => {
    return (localStorage.getItem("gnomad-theme") as ThemePreference) || "dark";
  });

  const [clipboardFilterEnabled, setClipboardFilterEnabled] = useState(() => {
    return localStorage.getItem("gnomad-clipboard-filter") !== "false";
  });

  const previewHtml = useMemo(
    () => renderMarkdownToHtml(editorContent),
    [editorContent]
  );

  useEffect(() => {
    localStorage.setItem("gnomad-theme", theme);

    const applyTheme = () => {
      const active = resolveTheme(
        theme,
        window.matchMedia("(prefers-color-scheme: dark)").matches
      );
      if (active === "light") {
        document.documentElement.setAttribute("data-theme", "light");
      } else {
        document.documentElement.removeAttribute("data-theme");
      }
      const win = getPlatformWindow();
      void win.setTheme(active);
      void win.setBackgroundColor(active === "light" ? "#f3f4f6" : "#0d0d10");
    };

    applyTheme();

    if (theme === "system") {
      const media = window.matchMedia("(prefers-color-scheme: dark)");
      const listener = () => applyTheme();
      media.addEventListener("change", listener);
      return () => media.removeEventListener("change", listener);
    }
  }, [theme]);

  useEffect(() => {
    localStorage.setItem("gnomad-clipboard-filter", String(clipboardFilterEnabled));
    void invoke("set_clipboard_filter_enabled", { enabled: clipboardFilterEnabled });
  }, [clipboardFilterEnabled]);

  useEffect(() => {
    void invoke<boolean>("get_clipboard_filter_enabled")
      .then((enabled) => setClipboardFilterEnabled(enabled))
      .catch(() => undefined);
  }, []);

  useEffect(() => {
    void invoke<AppVersionInfo>("get_app_version_info")
      .then((info) => setAppVersionLabel(info.display_version))
      .catch(() => undefined);
  }, []);

  useEffect(() => {
    const unsubs: Array<Promise<() => void>> = [
      listen("menu-about", () => {
        setAboutAutoCheck(false);
        setShowAbout(true);
      }),
      listen("menu-check-updates", () => {
        setAboutAutoCheck(true);
        setShowAbout(true);
      }),
    ];

    return () => {
      void Promise.all(unsubs).then((handlers) => {
        handlers.forEach((unlisten) => unlisten());
      });
    };
  }, []);

  useEffect(() => {
    void (async () => {
      const win = getPlatformWindow();
      const size = await win.innerSize();
      if (size.width < 960) {
        await win.setSize(new LogicalSize(1120, 640));
      }
    })();
  }, []);

  useEffect(() => {
    return () => {
      if (saveFeedbackTimerRef.current) {
        clearTimeout(saveFeedbackTimerRef.current);
      }
    };
  }, []);

  // --- Palette Commands ---
  const paletteCommands = [
    { name: "Format JSON", desc: "Prettifies and validates raw JSON in the editor", shortcut: `${modKeyLabel()}+F` },
    { name: "Prettify Markdown", desc: "Cleans list structure and header spacing", shortcut: `${modKeyLabel()}+L` },
    { name: "Copy as Raw HTML", desc: "Compiles Markdown to sanitized HTML and copies", shortcut: `${modKeyLabel()}+Shift+H` },
    { name: "Create Daily Note", desc: "Creates or appends to daily journal note", shortcut: "⌘D" },
    { name: "Append to Daily Note", desc: "Appends selected text directly to today's log", shortcut: "⌘⌥D" },
    { name: "Export selection to GitHub Gist", desc: "Publishes selected text as an online gist", shortcut: "⌘G" },
    { name: "Lock / Unlock Secret Vault", desc: "Toggles Vault-X secure credentials lock", shortcut: "⌘⌥L" },
    { name: "Switch to Light Theme", desc: "Sets theme to a crisp white minimalist style", shortcut: "" },
    { name: "Switch to Dark Theme", desc: "Sets theme to deep obsidian neon style", shortcut: "" },
    { name: "Switch to System Theme", desc: "Matches system preferences dynamically", shortcut: "" },
  ];

  // --- Load Initial Notes & Check Vault ---
  useEffect(() => {
    loadNotes();
    checkVaultStatus();
    
    // Auto-run ephemeral sweep on launch
    invoke("archive_notes").then((count) => {
      if (count && (count as number) > 0) {
        console.log(`Auto-archived ${count} stale temporary notes.`);
        loadNotes();
      }
    });
  }, []);

  const loadNotes = async () => {
    try {
      const allNotes = await invoke<Note[]>("list_notes");
      setNotes(allNotes);
      
      // Select first note if none is selected
      if (allNotes.length > 0 && !selectedNote) {
        selectNote(allNotes[0]);
      }
    } catch (e) {
      console.error("Failed to load notes:", e);
    }
  };

  const checkVaultStatus = async () => {
    try {
      const initialized = await invoke<boolean>("is_vault_initialized");
      const unlocked = await invoke<boolean>("is_vault_unlocked");
      setIsVaultInitialized(initialized);
      setIsVaultUnlocked(unlocked);
      if (unlocked) {
        await loadSecretMetadata();
      }
    } catch (e) {
      console.error(e);
    }
  };

  const loadSecretMetadata = async () => {
    try {
      const meta = await invoke<SecretMeta[]>("list_secret_metadata");
      setSecretMetadata(meta);
    } catch (e) {
      console.error("Failed to load secret metadata:", e);
      setSecretMetadata([]);
    }
  };

  const clearRevealedSecret = () => {
    setRevealedIndex(null);
    setRevealedValue("");
  };

  const selectNote = async (note: Note) => {
    await flushPendingSave();
    setSelectedNote(note);
    setEditorTitle(note.title);
    // Strip ephemeral comment if visual presentation
    let content = note.content;
    if (content.startsWith("<!-- ephemeral -->\n")) {
      content = content.replace("<!-- ephemeral -->\n", "");
    }
    setEditorContent(content);
    setIsEphemeral(note.is_ephemeral);
  };

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
    async (options?: { promoteToSaved?: boolean }) => {
      if (activeTabRef.current === "secrets") {
        return;
      }

      if (saveDebounceRef.current) {
        clearTimeout(saveDebounceRef.current);
        saveDebounceRef.current = null;
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
        showSaveFeedback(promote || !saved.is_ephemeral ? "Saved" : "Draft saved");
      } catch (e) {
        console.error("Save failed:", e);
        showSaveFeedback("Save failed");
      }
    },
    [showSaveFeedback, selectedNote]
  );

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
      setNotes((prev) => {
        const filtered = prev.filter((n) => n.filename !== saved.filename);
        return [saved, ...filtered];
      });
      setSelectedNote(saved);
    } catch (e) {
      console.error("Failed to create draft:", e);
    }
  }, [saveNoteNow]);

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

  const saveActiveNote = useCallback(
    async (overrideContent?: string, overrideTitle?: string, forceVault?: boolean) => {
      const titleToSave = overrideTitle !== undefined ? overrideTitle : editorTitle;
      const contentToSave = overrideContent !== undefined ? overrideContent : editorContent;
      const ephemeralState = forceVault !== undefined ? !forceVault : isEphemeral;

      if (!titleToSave.trim()) return;

      try {
        const previousFilename = selectedNote?.filename ?? null;
        const saved = await invoke<Note>("write_note", {
          title: titleToSave,
          content: contentToSave,
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
      } catch (e) {
        console.error("Autosave failed:", e);
      }
    },
    [editorTitle, editorContent, isEphemeral, selectedNote]
  );

  const flushPendingSave = useCallback(async () => {
    if (saveDebounceRef.current) {
      clearTimeout(saveDebounceRef.current);
      saveDebounceRef.current = null;
      await saveActiveNote();
    }
  }, [saveActiveNote]);

  const scheduleSaveNote = useCallback(
    (overrideContent?: string, overrideTitle?: string, forceVault?: boolean) => {
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

  useEffect(() => {
    return () => {
      if (saveDebounceRef.current) {
        clearTimeout(saveDebounceRef.current);
      }
    };
  }, []);

  // --- Delete Note Function ---
  const deleteNote = async (filename: string, e: React.MouseEvent) => {
    e.stopPropagation();
    try {
      await invoke("delete_note", { filename });
      setNotes((prev) => prev.filter((n) => n.filename !== filename));
      if (selectedNote?.filename === filename) {
        setSelectedNote(null);
        setEditorTitle("");
        setEditorContent("");
      }
    } catch (e) {
      console.error(e);
    }
  };

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void listen("window-close-requested", () => {
      if (saveDebounceRef.current) {
        pendingWindowClose.current = true;
        setClosePromptOpen(true);
        return;
      }
      void (async () => {
        await saveNoteNow();
        const win = await getPlatformWindow();
        await win.destroy();
      })();
    }).then((fn) => {
      unlisten = fn;
    });
    return () => {
      unlisten?.();
    };
  }, [saveNoteNow]);

  // --- Keydown Event Handling for Universal Hotkeys ---
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const isCmdOrCtrl = e.metaKey || e.ctrlKey;

      // Esc: Dismiss overlays
      if (e.key === "Escape") {
        setShowClipboard(false);
        setShowPalette(false);
        return;
      }

      // ⌘S: Save current note (promotes scratchpad drafts to Saved)
      if (isCmdOrCtrl && !e.shiftKey && (e.key === "s" || e.key === "S")) {
        e.preventDefault();
        e.stopPropagation();
        if (activeTabRef.current !== "secrets") {
          void saveNoteNow({ promoteToSaved: isEphemeralRef.current });
        }
        return;
      }

      // ⌘N: New scratchpad draft
      if (isCmdOrCtrl && !e.shiftKey && (e.key === "n" || e.key === "N")) {
        e.preventDefault();
        e.stopPropagation();
        if (activeTabRef.current !== "secrets") {
          void createNewDraft();
        }
        return;
      }

      // ⌘P: Open Command Palette
      if (isCmdOrCtrl && e.key.toLowerCase() === "p") {
        e.preventDefault();
        setPaletteQuery("");
        setSelectedPaletteIdx(0);
        setShowPalette((prev) => !prev);
        setShowClipboard(false);
      }

      // ⌘⇧V: Open Clipboard History Capsule
      if (isCmdOrCtrl && e.shiftKey && e.key.toLowerCase() === "v") {
        e.preventDefault();
        loadClipboardHistory();
        setSelectedClipIdx(0);
        setShowClipboard((prev) => !prev);
        setShowPalette(false);
      }
    };

    document.addEventListener("keydown", handleKeyDown, true);
    return () => document.removeEventListener("keydown", handleKeyDown, true);
  }, [saveNoteNow, createNewDraft]);

  // --- Clipboard History Fetch ---
  const loadClipboardHistory = async () => {
    try {
      // In the browser extension, capture the active clipboard text first
      if (typeof window !== "undefined" && !(window as any).__TAURI_INTERNALS__ && typeof navigator !== "undefined" && navigator.clipboard?.readText) {
        try {
          const text = await navigator.clipboard.readText();
          if (text && text.trim()) {
            await new Promise<void>((resolve) => {
              chrome.runtime.sendMessage({ type: "record-clipboard", text }, () => resolve());
            });
          }
        } catch (clipErr) {
          // silent ignore if permission not yet granted
        }
      }

      const history = await invoke<ClipboardEntry[]>("get_clipboard_history");
      setClipboardHistory(history);
    } catch (e) {
      console.error(e);
    }
  };

  const handlePasteFromClipboard = (text: string) => {
    // Inject text at editor cursor
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
    setShowClipboard(false);
  };

  // --- Command Palette Action Executions ---
  const executeCommand = async (commandName: string) => {
    setShowPalette(false);

    switch (commandName) {
      case "Switch to Light Theme":
        setTheme("light");
        break;
      case "Switch to Dark Theme":
        setTheme("dark");
        break;
      case "Switch to System Theme":
        setTheme("system");
        break;
      case "Format JSON":
        try {
          const parsed = JSON.parse(editorContent);
          const formatted = JSON.stringify(parsed, null, 2);
          setEditorContent(formatted);
          saveActiveNote(formatted);
        } catch (err) {
          alert("Error: Active note content does not contain valid JSON.");
        }
        break;

      case "Prettify Markdown": {
        // Basic markdown formatting cleanups
        let prettified = editorContent
          .split("\n")
          .map((line) => {
            if (line.startsWith("-") && !line.startsWith("- ")) {
              return "- " + line.substring(1);
            }
            return line;
          })
          .join("\n");
        setEditorContent(prettified);
        saveActiveNote(prettified);
        break;
      }

      case "Copy as Raw HTML": {
        const html = renderMarkdownToHtml(editorContent);
        try {
          await invoke("copy_text_to_clipboard", { text: html });
          alert("Markdown compiled to HTML and copied to clipboard.");
        } catch {
          alert("Could not copy HTML to clipboard.");
        }
        break;
      }

      case "Create Daily Note":
      case "Append to Daily Note": {
        const today = new Date().toISOString().split("T")[0];
        const title = `Daily_Note_${today}`;
        const newContent = commandName === "Create Daily Note" 
          ? `## Log: ${today}\n\n*   Active drafting logs...` 
          : `\n\n*   [Logged at ${new Date().toLocaleTimeString()}]: ${editorContent}`;

        try {
          await invoke("write_note", {
            title,
            content: newContent,
            isEphemeral: false,
            previousFilename: null,
          });
          loadNotes();
          alert(`Appended to ${title}!`);
        } catch (e) {
          console.error(e);
        }
        break;
      }

      case "Export selection to GitHub Gist":
        alert("Mock action: selection uploaded securely to your GitHub private Gists!");
        break;

      case "Lock / Unlock Secret Vault":
        setActiveTab("secrets");
        break;
    }
  };

  // --- Secure Vault-X Database Operations ---
  const handleInitVault = async () => {
    if (!vaultPassword) return;
    try {
      await invoke("initialize_vault", { password: vaultPassword });
      setIsVaultInitialized(true);
      setIsVaultUnlocked(true);
      setVaultPassword("");
      clearRevealedSecret();
      await loadSecretMetadata();
    } catch (e: unknown) {
      alert("Failed to initialize: " + invokeErrorMessage(e));
    }
  };

  const handleUnlockVault = async () => {
    if (!vaultPassword) return;
    try {
      await invoke("unlock_vault", { password: vaultPassword });
      setIsVaultUnlocked(true);
      setVaultPassword("");
      clearRevealedSecret();
      await loadSecretMetadata();
    } catch (e: unknown) {
      alert(invokeErrorMessage(e));
    }
  };

  const handleLockVault = async () => {
    await invoke("lock_vault");
    setIsVaultUnlocked(false);
    setVaultPassword("");
    setSecretMetadata([]);
    clearRevealedSecret();
  };

  const handleAddSecret = async () => {
    if (!newSecretName || !newSecretValue) return;
    try {
      await invoke("add_secret", {
        name: newSecretName,
        path: newSecretPath,
        value: newSecretValue,
      });
      setNewSecretName("");
      setNewSecretPath("");
      setNewSecretValue("");
      setShowAddSecret(false);
      clearRevealedSecret();
      await loadSecretMetadata();
    } catch (e: unknown) {
      alert("Failed to add secret: " + invokeErrorMessage(e));
    }
  };

  const handleDeleteSecret = async (index: number) => {
    try {
      await invoke("delete_secret", { index });
      clearRevealedSecret();
      await loadSecretMetadata();
    } catch (e: unknown) {
      alert("Failed to delete secret: " + invokeErrorMessage(e));
    }
  };

  const handleCopySecret = async (index: number, name: string) => {
    try {
      await invoke("copy_secret_to_clipboard", { index });
      alert(`${name} copied to clipboard!`);
    } catch (e: unknown) {
      alert("Copy failed: " + invokeErrorMessage(e));
    }
  };

  const handleToggleReveal = async (index: number) => {
    if (revealedIndex === index) {
      clearRevealedSecret();
      return;
    }
    try {
      const value = await invoke<string>("reveal_secret_value", { index });
      setRevealedIndex(index);
      setRevealedValue(value);
    } catch (e: unknown) {
      alert("Could not reveal secret: " + invokeErrorMessage(e));
    }
  };

  // --- Render Notes Filter list ---
  const filteredNotes = notes.filter((note) => {
    // Search filter
    const matchesSearch = note.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
      note.content.toLowerCase().includes(searchQuery.toLowerCase());
    
    // Tab filter
    if (activeTab === "scratch") return matchesSearch && note.is_ephemeral;
    if (activeTab === "vault") return matchesSearch && !note.is_ephemeral;
    return matchesSearch;
  });

  return (
    <div className="app-container">
      {/* 1. Left Sidebar */}
      <aside className="sidebar">
        <div className="sidebar-header">
          <h1 className="logo-text">
            <span style={{ fontSize: "18px" }}>✦</span> Gnomad Slate
          </h1>
          <span style={{ fontSize: "10px", color: "var(--text-muted)", fontStyle: "italic" }}>
            {appVersionLabel}
          </span>
        </div>

        <div className="search-box">
          <input
            type="text"
            className="search-input"
            placeholder="Search notes... (⌘P)"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
          />
        </div>

        <div className="tabs-row">
          <TipButton
            className={`tab-btn ${activeTab === "scratch" ? "active" : ""}`}
            tip="Quick notes that auto-archive after 48 hours"
            onClick={() => setActiveTab("scratch")}
          >
            Scratchpad
          </TipButton>
          <TipButton
            className={`tab-btn ${activeTab === "vault" ? "active" : ""}`}
            tip="Saved notes that stay until you delete them"
            onClick={() => setActiveTab("vault")}
          >
            Saved
          </TipButton>
          <TipButton
            className={`tab-btn ${activeTab === "secrets" ? "active" : ""}`}
            tip="Password-protected store for API keys and tokens"
            onClick={() => setActiveTab("secrets")}
          >
            Secrets
          </TipButton>
        </div>

        <div className="notes-list">
          {activeTab !== "secrets" && filteredNotes.map((note) => (
            <div
              key={note.filename}
              className={`note-item ${selectedNote?.filename === note.filename ? "selected" : ""}`}
              onClick={() => selectNote(note)}
            >
              <div className="note-item-header">
                <span className="note-title">{note.title}</span>
                <span className={`note-badge ${note.is_ephemeral ? "ephemeral" : "vault"}`}>
                  {note.is_ephemeral ? "Scratch" : "Saved"}
                </span>
              </div>
              <p className="note-preview">
                {note.content.replace("<!-- ephemeral -->\n", "").substring(0, 40) || "Empty note"}
              </p>
              <button
                className="delete-btn has-tooltip"
                data-tooltip="Delete this note permanently"
                onClick={(e) => deleteNote(note.filename, e)}
              >
                ✕
              </button>
            </div>
          ))}
          {activeTab !== "secrets" && filteredNotes.length === 0 && (
            <p style={{ padding: "16px", color: "var(--text-muted)", fontSize: "12px", textAlign: "center" }}>
              No notes found. Click New Draft to start writing.
            </p>
          )}
        </div>

        {/* Dynamic Sidebar footer command shortcut tips */}
        <div style={{ padding: "12px 16px", borderTop: "1px solid var(--border-color)", fontSize: "11px", color: "var(--text-muted)" }}>
          <div style={{ display: "flex", justifyContent: "space-between", marginBottom: "4px" }}>
            <span>{modShortcut("P")}</span> <span>Palette</span>
          </div>
          <div style={{ display: "flex", justifyContent: "space-between" }}>
            <span>{modShortcut("V", { shift: true })}</span> <span>Clipboard</span>
          </div>
          <div style={{ display: "flex", justifyContent: "space-between", marginBottom: "8px", fontSize: "10px" }}>
            <span>Alt+Space</span> <span>Summon</span>
          </div>

          <label className="settings-toggle-row has-tooltip" data-tooltip="Hide likely passwords and API keys from clipboard history">
            <input
              type="checkbox"
              checked={clipboardFilterEnabled}
              onChange={(e) => setClipboardFilterEnabled(e.target.checked)}
            />
            <span>Filter sensitive clipboard items</span>
          </label>

          <div className="theme-row">
            <TipButton
              className={`theme-toggle-btn ${theme === "light" ? "active" : ""}`}
              tip="Use a light background and dark text"
              onClick={() => setTheme("light")}
            >
              ☀️ Light
            </TipButton>
            <TipButton
              className={`theme-toggle-btn ${theme === "dark" ? "active" : ""}`}
              tip="Use a dark background — default for Slate"
              onClick={() => setTheme("dark")}
            >
              🌙 Dark
            </TipButton>
            <TipButton
              className={`theme-toggle-btn ${theme === "system" ? "active" : ""}`}
              tip="Match your Mac's light or dark mode"
              onClick={() => setTheme("system")}
            >
              🖥️ System
            </TipButton>
          </div>
        </div>
      </aside>

      {/* 2. Main Workstation Area */}
      <section style={{ height: "100%", overflow: "hidden" }}>
        {activeTab === "secrets" ? (
          <div className="vault-panel">
            {!isVaultUnlocked ? (
              <div className="lock-screen">
                <div className="lock-icon">🔒</div>
                <h2 style={{ marginBottom: "8px", fontSize: "16px", fontWeight: "600" }}>Vault-X Encrypted Secrets</h2>
                <p style={{ color: "var(--text-muted)", fontSize: "12px", marginBottom: "20px", textAlign: "center", maxWidth: "250px" }}>
                  {isVaultInitialized 
                    ? "Enter your master key credentials to unlock safe data."
                    : "Initialize a new secure AES-256 vault by setting a master key password."
                  }
                </p>
                <div className="vault-input-group">
                  <input
                    type="password"
                    className="vault-password-input"
                    placeholder="Enter Master Password"
                    value={vaultPassword}
                    onChange={(e) => setVaultPassword(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === "Enter") {
                        isVaultInitialized ? handleUnlockVault() : handleInitVault();
                      }
                    }}
                  />
                  <button
                    className="action-btn accent has-tooltip"
                    style={{ padding: "10px", justifyContent: "center" }}
                    data-tooltip={
                      isVaultInitialized
                        ? "Unlock your encrypted secrets with your master password"
                        : "Create a new encrypted vault with a master password"
                    }
                    onClick={() =>
                      isVaultInitialized ? handleUnlockVault() : handleInitVault()
                    }
                  >
                    {isVaultInitialized ? "Unlock Vault" : "Initialize Vault"}
                  </button>
                </div>
              </div>
            ) : (
              <div style={{ display: "flex", flexDirection: "column", height: "100%" }}>
                <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "16px" }}>
                  <div>
                    <h2 style={{ fontSize: "18px", fontWeight: "600" }}>Vault-X Credentials Registry</h2>
                    <p style={{ color: "var(--text-muted)", fontSize: "12px" }}>Highly secure offline keys, tokens, and SSH configurations</p>
                  </div>
                  <div style={{ display: "flex", gap: "8px" }}>
                    <TipButton
                      className="action-btn"
                      tip="Add an API key, token, or password to the encrypted vault"
                      onClick={() => setShowAddSecret(true)}
                    >
                      + Add Secret Key
                    </TipButton>
                    <TipButton
                      className="action-btn"
                      tip="Lock the vault and clear secrets from memory"
                      onClick={() => void handleLockVault()}
                    >
                      Lock Vault
                    </TipButton>
                  </div>
                </div>

                {showAddSecret && (
                  <div style={{ background: "rgba(255,255,255,0.03)", padding: "16px", borderRadius: "8px", border: "1px solid var(--border-color)", marginBottom: "16px" }}>
                    <h3 style={{ fontSize: "13px", fontWeight: "600", marginBottom: "12px" }}>Register Secure Credential</h3>
                    <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr 1fr", gap: "12px", marginBottom: "12px" }}>
                      <input
                        type="text"
                        placeholder="Key Name (e.g. OPENAI_KEY)"
                        className="search-input"
                        value={newSecretName}
                        onChange={(e) => setNewSecretName(e.target.value)}
                      />
                      <input
                        type="text"
                        placeholder="Project Mapping Path (Optional)"
                        className="search-input"
                        value={newSecretPath}
                        onChange={(e) => setNewSecretPath(e.target.value)}
                      />
                      <input
                        type="password"
                        placeholder="Secret Value / API Token"
                        className="search-input"
                        value={newSecretValue}
                        onChange={(e) => setNewSecretValue(e.target.value)}
                      />
                    </div>
                    <div style={{ display: "flex", gap: "8px", justifyContent: "flex-end" }}>
                      <TipButton className="action-btn" tip="Close the add-secret form without saving" onClick={() => setShowAddSecret(false)}>Cancel</TipButton>
                      <TipButton className="action-btn accent" tip="Encrypt and store this credential in the vault" onClick={() => void handleAddSecret()}>Save Securely</TipButton>
                    </div>
                  </div>
                )}

                <div className="vault-table-container">
                  <table className="vault-table">
                    <thead>
                      <tr>
                        <th>Secret Name</th>
                        <th>Project Mapping Path</th>
                        <th>Token / Key Value</th>
                        <th style={{ width: "120px" }}>Actions</th>
                      </tr>
                    </thead>
                    <tbody>
                      {secretMetadata.map((secret) => (
                        <tr key={`${secret.name}-${secret.index}`}>
                          <td style={{ fontWeight: "600" }}>{secret.name}</td>
                          <td style={{ fontStyle: "italic", color: "var(--text-muted)", fontSize: "11px" }}>
                            {secret.path || "Global / Unmapped"}
                          </td>
                          <td
                            className="key-mask has-tooltip"
                            data-tooltip="Click to copy this secret to your clipboard"
                            onClick={() => void handleCopySecret(secret.index, secret.name)}
                          >
                            {revealedIndex === secret.index
                              ? revealedValue
                              : "•••••••••••••••• (Click to Copy)"}
                          </td>
                          <td>
                            <div style={{ display: "flex", gap: "8px" }}>
                              <TipButton
                                className="tab-btn"
                                style={{ padding: "2px 6px" }}
                                tip={revealedIndex === secret.index ? "Hide the secret value on screen" : "Temporarily show the secret value on screen"}
                                onClick={() => void handleToggleReveal(secret.index)}
                              >
                                {revealedIndex === secret.index ? "Hide" : "Show"}
                              </TipButton>
                              <TipButton
                                className="tab-btn"
                                style={{ padding: "2px 6px", color: "#ef4444" }}
                                tip="Remove this secret from the vault permanently"
                                onClick={() => void handleDeleteSecret(secret.index)}
                              >
                                Delete
                              </TipButton>
                            </div>
                          </td>
                        </tr>
                      ))}
                      {secretMetadata.length === 0 && (
                        <tr>
                          <td colSpan={4} style={{ textAlign: "center", color: "var(--text-muted)", padding: "24px" }}>
                            No secrets registered yet. Click "+ Add Secret Key" to secure your first credential!
                          </td>
                        </tr>
                      )}
                    </tbody>
                  </table>
                </div>
              </div>
            )}
          </div>
        ) : (
          <div className="editor-container">
            <div className="editor-header">
              <div className="editor-title-container">
                <span style={{ fontSize: "14px", color: isEphemeral ? "var(--accent-cyan)" : "var(--accent-color)" }}>●</span>
                <input
                  type="text"
                  className="editor-title-input"
                  placeholder="Untitled note"
                  value={editorTitle}
                  onKeyDown={handleEditorShortcut}
                  onChange={(e) => {
                    setEditorTitle(e.target.value);
                    scheduleSaveNote(editorContent, e.target.value);
                  }}
                />
              </div>

              <div className="toolbar-actions">
                {saveFeedback && (
                  <span className="save-feedback" aria-live="polite">
                    {saveFeedback}
                  </span>
                )}
                <div className="view-mode-row">
                  <TipButton
                    className={`tab-btn ${editorView === "edit" ? "active" : ""}`}
                    tip="Edit raw markdown in the text area"
                    onClick={() => setEditorView("edit")}
                  >
                    Edit
                  </TipButton>
                  <TipButton
                    className={`tab-btn ${editorView === "split" ? "active" : ""}`}
                    tip="See your markdown and preview side by side"
                    onClick={() => setEditorView("split")}
                  >
                    Split
                  </TipButton>
                  <TipButton
                    className={`tab-btn ${editorView === "preview" ? "active" : ""}`}
                    tip="View rendered markdown only"
                    onClick={() => setEditorView("preview")}
                  >
                    Preview
                  </TipButton>
                </div>
                <TipButton
                  className={`action-btn ${isEphemeral ? "active-glow" : ""}`}
                  tipBelow
                  tip={
                    isEphemeral
                      ? "Scratchpad draft — auto-archives after 48 hours. Click or press ⌘S to save permanently."
                      : "Saved permanently. Click to mark as a scratchpad draft again."
                  }
                  onClick={() => {
                    const nextVal = !isEphemeral;
                    setIsEphemeral(nextVal);
                    saveActiveNote(editorContent, editorTitle, !nextVal);
                  }}
                >
                  {isEphemeral ? "✧ Scratchpad Draft" : "🔒 Saved Note"}
                </TipButton>
                <TipButton
                  className="action-btn"
                  tipBelow
                  tip="Start a fresh scratchpad note (⌘N)"
                  onClick={() => void createNewDraft()}
                >
                  New Draft
                </TipButton>
              </div>
            </div>

            <div
              className={`editor-body ${editorView === "split" ? "editor-body-split" : ""}`}
            >
              {(editorView === "edit" || editorView === "split") && (
                <textarea
                  ref={editorRef}
                  className="text-textarea"
                  placeholder="Start typing in Markdown… ⌘S save · ⌘N new draft · ⌘P palette"
                  value={editorContent}
                  onKeyDown={handleEditorShortcut}
                  onChange={(e) => {
                    setEditorContent(e.target.value);
                    scheduleSaveNote(e.target.value, editorTitle);
                  }}
                />
              )}
              {(editorView === "preview" || editorView === "split") && (
                <div
                  className="markdown-preview"
                  dangerouslySetInnerHTML={{ __html: previewHtml }}
                />
              )}
            </div>
          </div>
        )}
      </section>

      {/* 3. Extended Clipboard history Capsule Dialog */}
      {showClipboard && (
        <div className="overlay-backdrop" onClick={() => setShowClipboard(false)}>
          <div className="modal-box" onClick={(e) => e.stopPropagation()}>
            <div className="modal-header">
              <h3 className="modal-title">📋 Clipboard History Capsule</h3>
              <span className="palette-shortcut">Esc to close</span>
            </div>
            <div className="modal-body">
              {clipboardHistory.map((item, idx) => (
                <div
                  key={idx}
                  className={`clipboard-item ${idx === selectedClipIdx ? "selected" : ""}`}
                  onClick={() => handlePasteFromClipboard(item.text)}
                >
                  {item.text}
                </div>
              ))}
              {clipboardHistory.length === 0 && (
                <p style={{ color: "var(--text-muted)", textAlign: "center", padding: "20px" }}>
                  Clipboard history is empty. Copy text anywhere in your desktop!
                </p>
              )}
            </div>
          </div>
        </div>
      )}

      {/* 4. Visual Command Palette Dialog */}
      {showPalette && (
        <div className="overlay-backdrop" onClick={() => setShowPalette(false)}>
          <div className="modal-box" onClick={(e) => e.stopPropagation()} style={{ width: "500px" }}>
            <input
              type="text"
              autoFocus
              className="palette-search"
              placeholder="Type a notepad action command..."
              value={paletteQuery}
              onChange={(e) => setPaletteQuery(e.target.value)}
            />
            <div style={{ padding: "8px 0", maxHeight: "280px", overflowY: "auto" }}>
              {paletteCommands
                .filter(cmd => cmd.name.toLowerCase().includes(paletteQuery.toLowerCase()) || 
                              cmd.desc.toLowerCase().includes(paletteQuery.toLowerCase()))
                .map((cmd, idx) => (
                  <div
                    key={idx}
                    className={`palette-item ${idx === selectedPaletteIdx ? "selected" : ""}`}
                    onClick={() => executeCommand(cmd.name)}
                  >
                    <div>
                      <div style={{ fontSize: "13px", fontWeight: "500" }}>{cmd.name}</div>
                      <div style={{ fontSize: "11px", color: "var(--text-muted)" }}>{cmd.desc}</div>
                    </div>
                    <span className="palette-shortcut">{cmd.shortcut}</span>
                  </div>
                ))}
            </div>
          </div>
        </div>
      )}

      <AboutDialog
        open={showAbout}
        autoCheckUpdates={aboutAutoCheck}
        onClose={() => {
          setShowAbout(false);
          setAboutAutoCheck(false);
        }}
      />
      {closePromptOpen && (
        <UnsavedChangesModal
          noteTitle={editorTitle}
          onSave={() => {
            void (async () => {
              await flushPendingSave();
              setClosePromptOpen(false);
              const win = await getPlatformWindow();
              await win.destroy();
            })();
          }}
          onDiscard={() => {
            void (async () => {
              setClosePromptOpen(false);
              const win = await getPlatformWindow();
              await win.destroy();
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

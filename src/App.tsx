import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
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

interface SecretItem {
  name: string;
  path: string;
  value: string;
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
  const [vaultSecrets, setVaultSecrets] = useState<SecretItem[]>([]);
  const [revealSecrets, setRevealSecrets] = useState<{ [key: string]: boolean }>({});
  
  // Add Secret Form
  const [showAddSecret, setShowAddSecret] = useState(false);
  const [newSecretName, setNewSecretName] = useState("");
  const [newSecretPath, setNewSecretPath] = useState("");
  const [newSecretValue, setNewSecretValue] = useState("");

  const editorRef = useRef<HTMLTextAreaElement>(null);

  // --- Theme State ---
  const [theme, setTheme] = useState<"light" | "dark" | "system">(() => {
    return (localStorage.getItem("gnomad-theme") as "light" | "dark" | "system") || "dark";
  });

  useEffect(() => {
    localStorage.setItem("gnomad-theme", theme);
    
    const updateTheme = () => {
      let activeTheme = theme;
      if (theme === "system") {
        activeTheme = window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
      }
      
      if (activeTheme === "light") {
        document.documentElement.setAttribute("data-theme", "light");
      } else {
        document.documentElement.removeAttribute("data-theme");
      }
    };

    updateTheme();

    if (theme === "system") {
      const media = window.matchMedia("(prefers-color-scheme: dark)");
      const listener = () => updateTheme();
      media.addEventListener("change", listener);
      return () => media.removeEventListener("change", listener);
    }
  }, [theme]);

  // --- Palette Commands ---
  const paletteCommands = [
    { name: "Format JSON", desc: "Prettifies and validates raw JSON in the editor", shortcut: "⌘F" },
    { name: "Prettify Markdown", desc: "Cleans list structure and header spacing", shortcut: "⌘L" },
    { name: "Copy as Raw HTML", desc: "Compiles Markdown selection to raw HTML & copies", shortcut: "⌘⇧H" },
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
      setIsVaultInitialized(initialized);
    } catch (e) {
      console.error(e);
    }
  };

  const selectNote = (note: Note) => {
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

  // --- Autosave Note Function ---
  const saveActiveNote = async (overrideContent?: string, overrideTitle?: string, forceVault?: boolean) => {
    const titleToSave = overrideTitle !== undefined ? overrideTitle : editorTitle;
    const contentToSave = overrideContent !== undefined ? overrideContent : editorContent;
    const ephemeralState = forceVault !== undefined ? !forceVault : isEphemeral;

    if (!titleToSave.trim()) return;

    try {
      const saved = await invoke<Note>("write_note", {
        title: titleToSave,
        content: contentToSave,
        isEphemeral: ephemeralState,
      });

      // Update notes list in place
      setNotes((prev) => {
        const filtered = prev.filter((n) => n.filename !== saved.filename);
        return [saved, ...filtered];
      });

      // Update selected note references
      setSelectedNote(saved);
      setIsEphemeral(saved.is_ephemeral);
    } catch (e) {
      console.error("Autosave failed:", e);
    }
  };

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

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [editorContent, editorTitle, isEphemeral]);

  // --- Clipboard History Fetch ---
  const loadClipboardHistory = async () => {
    try {
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
        // Standard markdown headers to HTML tags translation mock
        let html = editorContent
          .replace(/^# (.*$)/gim, "<h1>$1</h1>")
          .replace(/^## (.*$)/gim, "<h2>$2</h2>")
          .replace(/^\- (.*$)/gim, "<li>$1</li>")
          .replace(/\*\*(.*)\*\*/gim, "<strong>$1</strong>")
          .replace(/\*(.*)\*/gim, "<em>$1</em>");
        
        navigator.clipboard.writeText(html);
        alert("Renders compiled & HTML copied to Clipboard!");
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
      setVaultSecrets([]);
    } catch (e: any) {
      alert("Failed to initialize: " + e);
    }
  };

  const handleUnlockVault = async () => {
    if (!vaultPassword) return;
    try {
      const secretsJson = await invoke<string>("unlock_vault", { password: vaultPassword });
      const secrets = JSON.parse(secretsJson);
      setVaultSecrets(secrets);
      setIsVaultUnlocked(true);
    } catch (e) {
      alert("Lock Screen: Authentication failed.");
    }
  };

  const handleSaveSecrets = async (updatedSecrets: SecretItem[]) => {
    try {
      await invoke("save_vault", {
        decryptedJson: JSON.stringify(updatedSecrets),
        password: vaultPassword,
      });
      setVaultSecrets(updatedSecrets);
    } catch (e) {
      alert("Failed to save credentials safely: " + e);
    }
  };

  const handleAddSecret = () => {
    if (!newSecretName || !newSecretValue) return;
    const newItem: SecretItem = {
      name: newSecretName,
      path: newSecretPath,
      value: newSecretValue,
    };
    const updated = [...vaultSecrets, newItem];
    handleSaveSecrets(updated);
    
    // Reset Form
    setNewSecretName("");
    setNewSecretPath("");
    setNewSecretValue("");
    setShowAddSecret(false);
  };

  const handleDeleteSecret = (idx: number) => {
    const updated = vaultSecrets.filter((_, i) => i !== idx);
    handleSaveSecrets(updated);
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
          <span style={{ fontSize: "10px", color: "var(--text-muted)", fontStyle: "italic" }}>v2.0</span>
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
          <button
            className={`tab-btn ${activeTab === "scratch" ? "active" : ""}`}
            onClick={() => setActiveTab("scratch")}
          >
            Scratchpad
          </button>
          <button
            className={`tab-btn ${activeTab === "vault" ? "active" : ""}`}
            onClick={() => setActiveTab("vault")}
          >
            Vaults
          </button>
          <button
            className={`tab-btn ${activeTab === "secrets" ? "active" : ""}`}
            onClick={() => setActiveTab("secrets")}
          >
            Secrets
          </button>
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
                className="delete-btn"
                onClick={(e) => deleteNote(note.filename, e)}
              >
                ✕
              </button>
            </div>
          ))}
          {activeTab !== "secrets" && filteredNotes.length === 0 && (
            <p style={{ padding: "16px", color: "var(--text-muted)", fontSize: "12px", textAlign: "center" }}>
              No notes found. Create a new buffer!
            </p>
          )}
        </div>

        {/* Dynamic Sidebar footer command shortcut tips */}
        <div style={{ padding: "12px 16px", borderTop: "1px solid var(--border-color)", fontSize: "11px", color: "var(--text-muted)" }}>
          <div style={{ display: "flex", justifyContent: "space-between", marginBottom: "4px" }}>
            <span>Cmd+P</span> <span>Palette</span>
          </div>
          <div style={{ display: "flex", justifyContent: "space-between" }}>
            <span>Cmd+Shift+V</span> <span>Clipboard</span>
          </div>

          <div className="theme-row">
            <button 
              className={`theme-toggle-btn ${theme === "light" ? "active" : ""}`}
              onClick={() => setTheme("light")}
            >
              ☀️ Light
            </button>
            <button 
              className={`theme-toggle-btn ${theme === "dark" ? "active" : ""}`}
              onClick={() => setTheme("dark")}
            >
              🌙 Dark
            </button>
            <button 
              className={`theme-toggle-btn ${theme === "system" ? "active" : ""}`}
              onClick={() => setTheme("system")}
            >
              🖥️ System
            </button>
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
                    className="action-btn accent"
                    style={{ padding: "10px", justifyContent: "center" }}
                    onClick={isVaultInitialized ? handleUnlockVault() : handleInitVault}
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
                    <button className="action-btn" onClick={() => setShowAddSecret(true)}>
                      + Add Secret Key
                    </button>
                    <button className="action-btn" onClick={() => { setIsVaultUnlocked(false); setVaultPassword(""); }}>
                      Lock Vault
                    </button>
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
                      <button className="action-btn" onClick={() => setShowAddSecret(false)}>Cancel</button>
                      <button className="action-btn accent" onClick={handleAddSecret}>Save Securely</button>
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
                      {vaultSecrets.map((secret, idx) => (
                        <tr key={idx}>
                          <td style={{ fontWeight: "600" }}>{secret.name}</td>
                          <td style={{ fontStyle: "italic", color: "var(--text-muted)", fontSize: "11px" }}>
                            {secret.path || "Global / Unmapped"}
                          </td>
                          <td 
                            className="key-mask"
                            onClick={() => {
                              navigator.clipboard.writeText(secret.value);
                              alert(`${secret.name} copied to clipboard!`);
                            }}
                          >
                            {revealSecrets[secret.name] ? secret.value : "•••••••••••••••• (Click to Copy)"}
                          </td>
                          <td>
                            <div style={{ display: "flex", gap: "8px" }}>
                              <button 
                                className="tab-btn" 
                                style={{ padding: "2px 6px" }}
                                onClick={() => setRevealSecrets(prev => ({ ...prev, [secret.name]: !prev[secret.name] }))}
                              >
                                {revealSecrets[secret.name] ? "Hide" : "Show"}
                              </button>
                              <button 
                                className="tab-btn" 
                                style={{ padding: "2px 6px", color: "#ef4444" }}
                                onClick={() => handleDeleteSecret(idx)}
                              >
                                Delete
                              </button>
                            </div>
                          </td>
                        </tr>
                      ))}
                      {vaultSecrets.length === 0 && (
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
                  placeholder="Untitled Buffer"
                  value={editorTitle}
                  onChange={(e) => {
                    setEditorTitle(e.target.value);
                    saveActiveNote(editorContent, e.target.value);
                  }}
                />
              </div>

              <div className="toolbar-actions">
                <button
                  className={`action-btn ${isEphemeral ? "active-glow" : ""}`}
                  onClick={() => {
                    const nextVal = !isEphemeral;
                    setIsEphemeral(nextVal);
                    saveActiveNote(editorContent, editorTitle, !nextVal);
                  }}
                >
                  {isEphemeral ? "✧ Scratchpad Draft" : "🔒 Promoted to Vault"}
                </button>
                <button
                  className="action-btn"
                  onClick={() => {
                    // Create a new blank slate note buffer
                    const dummyTitle = `Note_${new Date().toLocaleTimeString().replace(/\s/g, "_")}`;
                    setEditorTitle(dummyTitle);
                    setEditorContent("");
                    setIsEphemeral(true);
                    saveActiveNote("", dummyTitle, false);
                  }}
                >
                  New Draft
                </button>
              </div>
            </div>

            <div className="editor-body">
              <textarea
                ref={editorRef}
                className="text-textarea"
                placeholder="Start typing your thoughts in raw Markdown... press ⌘P for actions, ⌘Shift+V for clipboard history capsule."
                value={editorContent}
                onChange={(e) => {
                  setEditorContent(e.target.value);
                  saveActiveNote(e.target.value, editorTitle);
                }}
              />
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
    </div>
  );
}

export default App;

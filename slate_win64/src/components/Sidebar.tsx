import { TipButton } from "./TipButton";
import { modShortcut } from "../utils/modKeyLabel";
import { getPreviewText } from "../utils/notePreview";
import type { EditorTab, Note } from "../types/app";
import type { ThemePreference } from "../utils/theme";

type SidebarProps = {
  appVersionLabel: string;
  searchQuery: string;
  onSearchChange: (value: string) => void;
  activeTab: EditorTab;
  onTabChange: (tab: EditorTab) => void;
  filteredNotes: Note[];
  selectedNoteFilename: string | null;
  onSelectNote: (note: Note) => void;
  onDeleteNote: (filename: string, event: React.MouseEvent) => void;
  summonShortcutLabel: string;
  clipboardFilterEnabled: boolean;
  onClipboardFilterChange: (enabled: boolean) => void;
  theme: ThemePreference;
  onThemeChange: (theme: ThemePreference) => void;
};

/** Left sidebar: search, tabs, notes list, settings footer. */
export function Sidebar({
  appVersionLabel,
  searchQuery,
  onSearchChange,
  activeTab,
  onTabChange,
  filteredNotes,
  selectedNoteFilename,
  onSelectNote,
  onDeleteNote,
  summonShortcutLabel,
  clipboardFilterEnabled,
  onClipboardFilterChange,
  theme,
  onThemeChange,
}: SidebarProps) {
  return (
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
          placeholder={`Search notes... (${modShortcut("P")})`}
          value={searchQuery}
          onChange={(e) => onSearchChange(e.target.value)}
        />
      </div>

      <div className="tabs-row">
        <TipButton
          className={`tab-btn ${activeTab === "scratch" ? "active" : ""}`}
          tip="Quick notes that auto-archive after 48 hours"
          onClick={() => onTabChange("scratch")}
        >
          Scratchpad
        </TipButton>
        <TipButton
          className={`tab-btn ${activeTab === "vault" ? "active" : ""}`}
          tip="Saved notes that stay until you delete them"
          onClick={() => onTabChange("vault")}
        >
          Saved
        </TipButton>
        <TipButton
          className={`tab-btn ${activeTab === "secrets" ? "active" : ""}`}
          tip="Password-protected store for API keys and tokens"
          onClick={() => onTabChange("secrets")}
        >
          Secrets
        </TipButton>
      </div>

      <div className="notes-list">
        {activeTab !== "secrets" &&
          filteredNotes.map((note) => (
            <div
              key={note.filename}
              className={`note-item ${selectedNoteFilename === note.filename ? "selected" : ""}`}
              onClick={() => onSelectNote(note)}
            >
              <div className="note-item-header">
                <span className="note-title">{note.title}</span>
                <span className={`note-badge ${note.is_ephemeral ? "ephemeral" : "vault"}`}>
                  {note.is_ephemeral ? "Scratch" : "Saved"}
                </span>
              </div>
              <p className="note-preview">
                {getPreviewText(note.content, note.is_ephemeral)}
              </p>
              <button
                className="delete-btn has-tooltip"
                data-tooltip="Delete this note permanently"
                onClick={(e) => onDeleteNote(note.filename, e)}
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

      <div
        style={{
          padding: "12px 16px",
          borderTop: "1px solid var(--border-color)",
          fontSize: "11px",
          color: "var(--text-muted)",
        }}
      >
        <div style={{ display: "flex", justifyContent: "space-between", marginBottom: "4px" }}>
          <span>{modShortcut("P")}</span> <span>Palette</span>
        </div>
        <div style={{ display: "flex", justifyContent: "space-between" }}>
          <span>{modShortcut("V", { shift: true })}</span> <span>Clipboard</span>
        </div>
        <div style={{ display: "flex", justifyContent: "space-between", marginBottom: "8px", fontSize: "10px" }}>
          <span>{summonShortcutLabel}</span> <span>Summon</span>
        </div>

        <label
          className="settings-toggle-row has-tooltip"
          data-tooltip="Hide likely passwords and API keys from clipboard history"
        >
          <input
            type="checkbox"
            checked={clipboardFilterEnabled}
            onChange={(e) => onClipboardFilterChange(e.target.checked)}
          />
          <span>Filter sensitive clipboard items</span>
        </label>

        <div className="theme-row">
          <TipButton
            className={`theme-toggle-btn ${theme === "light" ? "active" : ""}`}
            tip="Use a light background and dark text"
            onClick={() => onThemeChange("light")}
          >
            ☀️ Light
          </TipButton>
          <TipButton
            className={`theme-toggle-btn ${theme === "dark" ? "active" : ""}`}
            tip="Use a dark background — default for Slate"
            onClick={() => onThemeChange("dark")}
          >
            🌙 Dark
          </TipButton>
          <TipButton
            className={`theme-toggle-btn ${theme === "system" ? "active" : ""}`}
            tip="Match your system's light or dark mode"
            onClick={() => onThemeChange("system")}
          >
            🖥️ System
          </TipButton>
        </div>
      </div>
    </aside>
  );
}

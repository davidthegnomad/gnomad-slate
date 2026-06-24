import { useCallback, useRef, type KeyboardEvent, type RefObject } from "react";
import { TipButton } from "./TipButton";
import { modShortcut } from "../utils/modKeyLabel";
import type { EditorViewMode } from "../types/app";

type EditorWorkstationProps = {
  isWindows: boolean;
  editorView: EditorViewMode;
  onEditorViewChange: (view: EditorViewMode) => void;
  showLineNumbers: boolean;
  onToggleLineNumbers: () => void;
  saveFeedback: string | null;
  isEphemeral: boolean;
  onToggleEphemeral: () => void;
  onNewDraft: () => void;
  editorTitle: string;
  onEditorTitleChange: (title: string) => void;
  onEditorShortcut: (event: KeyboardEvent) => void;
  selectedFilename: string | null;
  editorContent: string;
  onEditorContentChange: (content: string) => void;
  editorRef: RefObject<HTMLTextAreaElement | null>;
  lineNumberRows: number[];
  previewHtml: string;
  scheduleSaveNote: (content: string, title: string) => void;
};

/** Main editor section: toolbar, title, markdown textarea, preview, line numbers. */
export function EditorWorkstation({
  isWindows,
  editorView,
  onEditorViewChange,
  showLineNumbers,
  onToggleLineNumbers,
  saveFeedback,
  isEphemeral,
  onToggleEphemeral,
  onNewDraft,
  editorTitle,
  onEditorTitleChange,
  onEditorShortcut,
  selectedFilename,
  editorContent,
  onEditorContentChange,
  editorRef,
  lineNumberRows,
  previewHtml,
  scheduleSaveNote,
}: EditorWorkstationProps) {
  const gutterRef = useRef<HTMLDivElement>(null);

  const syncGutterScroll = useCallback((scrollTop: number) => {
    const gutter = gutterRef.current;
    if (gutter) {
      gutter.scrollTop = scrollTop;
    }
  }, []);

  const titleInput = (
    <div className="editor-title-container">
      <span
        style={{
          fontSize: "14px",
          color: isEphemeral ? "var(--accent-cyan)" : "var(--accent-color)",
        }}
      >
        ●
      </span>
      <input
        type="text"
        className="editor-title-input"
        placeholder="Untitled note"
        value={editorTitle}
        onKeyDown={onEditorShortcut}
        onChange={(e) => {
          onEditorTitleChange(e.target.value);
          scheduleSaveNote(editorContent, e.target.value);
        }}
      />
    </div>
  );

  const viewModeButtons = (
    <div className="view-mode-row">
      <TipButton
        className={`tab-btn ${editorView === "edit" ? "active" : ""}`}
        tipBelow={isWindows}
        tip="Edit raw markdown in the text area"
        onClick={() => onEditorViewChange("edit")}
      >
        Edit
      </TipButton>
      <TipButton
        className={`tab-btn ${editorView === "split" ? "active" : ""}`}
        tipBelow={isWindows}
        tip="See your markdown and preview side by side"
        onClick={() => onEditorViewChange("split")}
      >
        Split
      </TipButton>
      <TipButton
        className={`tab-btn ${editorView === "preview" ? "active" : ""}`}
        tipBelow={isWindows}
        tip="View rendered markdown only"
        onClick={() => onEditorViewChange("preview")}
      >
        Preview
      </TipButton>
    </div>
  );

  const actionButtons = (
    <>
      {isWindows && (
        <TipButton
          className={`action-btn ${showLineNumbers ? "active-glow" : ""}`}
          tipBelow
          tip="Show or hide line numbers in the editor"
          onClick={onToggleLineNumbers}
        >
          # Lines
        </TipButton>
      )}
      <TipButton
        className={`action-btn ${isEphemeral ? "active-glow" : ""}`}
        tipBelow={isWindows}
        tip={
          isEphemeral
            ? `Scratchpad draft — auto-archives after 48 hours. Click or press ${modShortcut("S")} to save permanently.`
            : "Saved permanently. Click to mark as a scratchpad draft again."
        }
        onClick={onToggleEphemeral}
      >
        {isEphemeral ? "✧ Scratchpad Draft" : "🔒 Saved Note"}
      </TipButton>
      <TipButton
        className="action-btn"
        tipBelow={isWindows}
        tip={`Start a fresh scratchpad note (${modShortcut("N")})`}
        onClick={onNewDraft}
      >
        New Draft
      </TipButton>
    </>
  );

  return (
    <div className="editor-container">
      <div className={`editor-header${isWindows ? " editor-header-windows" : ""}`}>
        {isWindows ? (
          <>
            <div className="toolbar-actions toolbar-actions-windows">
              {saveFeedback && (
                <span className="save-feedback" aria-live="polite">
                  {saveFeedback}
                </span>
              )}
              {viewModeButtons}
              {actionButtons}
            </div>
            <div className="editor-meta-row">
              {titleInput}
              <span className="editor-filename" title="Note file on disk">
                {selectedFilename ?? "unsaved.md"}
              </span>
            </div>
          </>
        ) : (
          <>
            {titleInput}
            <div className="toolbar-actions">
              {saveFeedback && (
                <span className="save-feedback" aria-live="polite">
                  {saveFeedback}
                </span>
              )}
              {viewModeButtons}
              {actionButtons}
            </div>
          </>
        )}
      </div>

      <div className={`editor-body ${editorView === "split" ? "editor-body-split" : ""}`}>
        {(editorView === "edit" || editorView === "split") && (
          <div
            className={`editor-input-wrap${isWindows && showLineNumbers ? " editor-input-wrap-lined" : ""}`}
          >
            {isWindows && showLineNumbers && (
              <div ref={gutterRef} className="line-numbers-gutter" aria-hidden="true">
                {lineNumberRows.map((line) => (
                  <div key={line} className="line-number">
                    {line}
                  </div>
                ))}
              </div>
            )}
            <textarea
              ref={editorRef}
              className="text-textarea"
              placeholder={`Start typing in Markdown… ${modShortcut("S")} save · ${modShortcut("N")} new draft · ${modShortcut("P")} palette`}
              value={editorContent}
              onKeyDown={onEditorShortcut}
              onChange={(e) => {
                onEditorContentChange(e.target.value);
                scheduleSaveNote(e.target.value, editorTitle);
              }}
              onScroll={
                isWindows && showLineNumbers
                  ? (e) => {
                      requestAnimationFrame(() => {
                        syncGutterScroll(e.currentTarget.scrollTop);
                      });
                    }
                  : undefined
              }
            />
          </div>
        )}
        {(editorView === "preview" || editorView === "split") && (
          <div className="markdown-preview" dangerouslySetInnerHTML={{ __html: previewHtml }} />
        )}
      </div>
    </div>
  );
}

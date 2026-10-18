/** Pure notepad edit helpers. Clipboard history is a separate shortcut (Cmd/Ctrl+Shift+V). */

export interface EditorSnapshot {
  value: string;
  caret: number;
}

export interface UndoState {
  undo: EditorSnapshot[];
  redo: EditorSnapshot[];
}

export interface ShortcutEvent {
  key: string;
  metaKey: boolean;
  ctrlKey: boolean;
  altKey: boolean;
  shiftKey: boolean;
}

export type EditorEditAction = "copy" | "cut" | "paste" | "selectAll" | "undo" | "redo";

const UNDO_LIMIT = 200;

function clampIndex(value: string, index: number): number {
  if (!Number.isFinite(index)) return 0;
  return Math.max(0, Math.min(value.length, Math.trunc(index)));
}

export function emptyUndo(): UndoState {
  return { undo: [], redo: [] };
}

export function logicalLines(text: string): string[] {
  if (text === "") return [""];
  return text.split("\n");
}

/** Gutter is sized in `ch`. Two columns until line 100, then grows (10 and 100 must not clip). */
export function gutterDigits(lineCount: number): number {
  const count = Math.max(1, Math.floor(lineCount) || 1);
  return Math.max(2, String(count).length);
}

export function caretLineIndex(text: string, caret: number): number {
  const end = clampIndex(text, caret);
  let line = 0;
  for (let i = 0; i < end; i += 1) {
    if (text.charCodeAt(i) === 10) line += 1;
  }
  return line;
}

/** Tops of each logical line. Wrapped lines pass a taller height so the next number stays aligned. */
export function lineStartOffsets(lineHeights: number[]): number[] {
  const offsets: number[] = [];
  let y = 0;
  for (const height of lineHeights) {
    offsets.push(y);
    y += height;
  }
  return offsets;
}

export function sameHeights(a: number[], b: number[]): boolean {
  if (a.length !== b.length) return false;
  for (let i = 0; i < a.length; i += 1) {
    if (Math.abs(a[i] - b[i]) > 0.5) return false;
  }
  return true;
}

export function insertText(
  value: string,
  start: number,
  end: number,
  text: string
): { value: string; caret: number } {
  const from = clampIndex(value, Math.min(start, end));
  const to = clampIndex(value, Math.max(start, end));
  const next = value.slice(0, from) + text + value.slice(to);
  return { value: next, caret: from + text.length };
}

/** Copy reads the selection. The note string is unchanged. */
export function selectedText(value: string, start: number, end: number): string {
  const from = clampIndex(value, Math.min(start, end));
  const to = clampIndex(value, Math.max(start, end));
  return value.slice(from, to);
}

export function isClipboardHistoryShortcut(event: ShortcutEvent): boolean {
  return (
    (event.metaKey || event.ctrlKey) &&
    event.shiftKey &&
    !event.altKey &&
    event.key.toLowerCase() === "v"
  );
}

/** Plain Cmd/Ctrl+V is paste. Shift+V is history and must not land here. */
export function editorEditAction(event: ShortcutEvent): EditorEditAction | null {
  if (!(event.metaKey || event.ctrlKey) || event.altKey) return null;
  const key = event.key.toLowerCase();
  if (event.shiftKey) {
    if (key === "z") return "redo";
    return null;
  }
  switch (key) {
    case "c":
      return "copy";
    case "x":
      return "cut";
    case "v":
      return "paste";
    case "a":
      return "selectAll";
    case "z":
      return "undo";
    case "y":
      return "redo";
    default:
      return null;
  }
}

/** Shortcuts the command palette already advertises. Gist export is a mock and is not mapped. */
export function paletteCommandForShortcut(event: ShortcutEvent): string | null {
  if (!(event.metaKey || event.ctrlKey) || isClipboardHistoryShortcut(event)) return null;
  const key = event.key.toLowerCase();
  if (event.altKey && event.shiftKey) return null;
  if (event.altKey) {
    if (key === "d") return "Append to Daily Note";
    if (key === "l") return "Lock / Unlock Secret Vault";
    return null;
  }
  if (event.shiftKey) {
    if (key === "h") return "Copy as Raw HTML";
    return null;
  }
  switch (key) {
    case "f":
      return "Format JSON";
    case "l":
      return "Prettify Markdown";
    case "d":
      return "Create Daily Note";
    default:
      return null;
  }
}

export function recordUndo(state: UndoState, before: EditorSnapshot, afterValue: string): UndoState {
  if (before.value === afterValue) return state;
  const undo = [...state.undo, before];
  if (undo.length > UNDO_LIMIT) undo.splice(0, undo.length - UNDO_LIMIT);
  return { undo, redo: [] };
}

export function undoContent(
  state: UndoState,
  current: EditorSnapshot
): { state: UndoState; snapshot: EditorSnapshot } | null {
  const snapshot = state.undo[state.undo.length - 1];
  if (!snapshot) return null;
  return {
    state: { undo: state.undo.slice(0, -1), redo: [...state.redo, current] },
    snapshot,
  };
}

export function redoContent(
  state: UndoState,
  current: EditorSnapshot
): { state: UndoState; snapshot: EditorSnapshot } | null {
  const snapshot = state.redo[state.redo.length - 1];
  if (!snapshot) return null;
  return {
    state: { undo: [...state.undo, current], redo: state.redo.slice(0, -1) },
    snapshot,
  };
}

/** Match the textarea's content box so wrapped logical lines share the gutter's row heights. */
export function measureEditorLineHeights(
  textarea: HTMLTextAreaElement,
  measure: HTMLElement
): number[] {
  const style = window.getComputedStyle(textarea);
  measure.style.width = `${textarea.clientWidth}px`;
  measure.style.font = style.font;
  measure.style.letterSpacing = style.letterSpacing;
  measure.style.tabSize = style.tabSize;
  measure.style.padding = style.padding;
  measure.style.lineHeight = style.lineHeight;
  measure.style.whiteSpace = "pre-wrap";
  measure.style.overflowWrap = "break-word";
  measure.style.wordBreak = "normal";
  measure.style.boxSizing = "border-box";
  const rows = measure.querySelectorAll<HTMLElement>("[data-line]");
  return Array.from(rows, (row) => row.getBoundingClientRect().height || row.offsetHeight);
}

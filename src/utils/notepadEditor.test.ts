import { describe, expect, it } from "vitest";
import {
  caretLineIndex,
  editorEditAction,
  emptyUndo,
  gutterDigits,
  insertText,
  isClipboardHistoryShortcut,
  lineStartOffsets,
  logicalLines,
  paletteCommandForShortcut,
  recordUndo,
  redoContent,
  selectedText,
  undoContent,
} from "./notepadEditor";

const mod = { metaKey: true, ctrlKey: false, altKey: false, shiftKey: false };

describe("notepad paste insertion", () => {
  it("inserts at the caret", () => {
    const result = insertText("alpha beta", 6, 6, "XX");
    expect(result).toEqual({ value: "alpha XXbeta", caret: 8 });
  });

  it("replaces the selection", () => {
    const result = insertText("alpha beta", 6, 10, "gamma");
    expect(result).toEqual({ value: "alpha gamma", caret: 11 });
  });

  it("inserts at the start and the end", () => {
    expect(insertText("abc", 0, 0, "Z")).toEqual({ value: "Zabc", caret: 1 });
    expect(insertText("abc", 3, 3, "Z")).toEqual({ value: "abcZ", caret: 4 });
  });

  it("does not replace the note with clipboard history", () => {
    const note = "keep this sentence";
    const history = "OLD CLIP";
    const result = insertText(note, 5, 5, history);
    expect(result.value).toBe("keep OLD CLIPthis sentence");
    expect(result.caret).toBe(5 + history.length);
    expect(result.value).not.toBe(history);
  });
});

describe("notepad copy", () => {
  it("reads the selection and leaves the note unchanged", () => {
    const value = "hello world";
    expect(selectedText(value, 0, 5)).toBe("hello");
    expect(value).toBe("hello world");
    expect(selectedText(value, 6, 4)).toBe("o ");
  });
});

describe("clipboard shortcuts", () => {
  it("treats plain Cmd/Ctrl+V as paste and Shift+V as history", () => {
    expect(editorEditAction({ ...mod, key: "v" })).toBe("paste");
    expect(editorEditAction({ ...mod, key: "V" })).toBe("paste");
    expect(editorEditAction({ ...mod, ctrlKey: true, metaKey: false, key: "v" })).toBe("paste");
    expect(editorEditAction({ ...mod, shiftKey: true, key: "v" })).toBeNull();
    expect(isClipboardHistoryShortcut({ ...mod, shiftKey: true, key: "v" })).toBe(true);
    expect(isClipboardHistoryShortcut({ ...mod, key: "v" })).toBe(false);
  });

  it("maps select all, undo, and redo", () => {
    expect(editorEditAction({ ...mod, key: "a" })).toBe("selectAll");
    expect(editorEditAction({ ...mod, key: "z" })).toBe("undo");
    expect(editorEditAction({ ...mod, shiftKey: true, key: "z" })).toBe("redo");
    expect(editorEditAction({ ...mod, ctrlKey: true, metaKey: false, key: "y" })).toBe("redo");
  });
});

describe("line numbers", () => {
  it("counts logical lines, including an empty last line", () => {
    expect(logicalLines("")).toEqual([""]);
    expect(logicalLines("a").length).toBe(1);
    expect(logicalLines("a\n").length).toBe(2);
    expect(logicalLines("a\nb\nc").length).toBe(3);
  });

  it("grows the gutter past 9 and past 99", () => {
    expect(gutterDigits(1)).toBe(2);
    expect(gutterDigits(9)).toBe(2);
    expect(gutterDigits(10)).toBe(2);
    expect(gutterDigits(99)).toBe(2);
    expect(gutterDigits(100)).toBe(3);
    expect(gutterDigits(1000)).toBe(4);
  });

  it("keeps the next number aligned when a logical line wraps", () => {
    const heights = [20, 40, 20];
    const offsets = lineStartOffsets(heights);
    expect(offsets).toEqual([0, 20, 60]);
    expect(offsets[2]).not.toBe(2 * 20);
  });

  it("tracks the caret line", () => {
    expect(caretLineIndex("ab\ncd\nef", 0)).toBe(0);
    expect(caretLineIndex("ab\ncd\nef", 3)).toBe(1);
    expect(caretLineIndex("ab\ncd\nef", 6)).toBe(2);
  });
});

describe("undo and palette shortcuts", () => {
  it("undo and redo restore prior text", () => {
    let state = emptyUndo();
    state = recordUndo(state, { value: "a", caret: 1 }, "ab");
    state = recordUndo(state, { value: "ab", caret: 2 }, "abc");
    const first = undoContent(state, { value: "abc", caret: 3 });
    expect(first?.snapshot.value).toBe("ab");
    const second = undoContent(first!.state, { value: "ab", caret: 2 });
    expect(second?.snapshot.value).toBe("a");
    const redone = redoContent(second!.state, { value: "a", caret: 1 });
    expect(redone?.snapshot.value).toBe("ab");
  });

  it("maps advertised palette shortcuts and not the gist mock", () => {
    expect(paletteCommandForShortcut({ ...mod, key: "f" })).toBe("Format JSON");
    expect(paletteCommandForShortcut({ ...mod, key: "l" })).toBe("Prettify Markdown");
    expect(paletteCommandForShortcut({ ...mod, shiftKey: true, key: "h" })).toBe("Copy as Raw HTML");
    expect(paletteCommandForShortcut({ ...mod, key: "d" })).toBe("Create Daily Note");
    expect(paletteCommandForShortcut({ ...mod, altKey: true, key: "d" })).toBe("Append to Daily Note");
    expect(paletteCommandForShortcut({ ...mod, key: "g" })).toBeNull();
    expect(paletteCommandForShortcut({ ...mod, shiftKey: true, key: "v" })).toBeNull();
  });
});

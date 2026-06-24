import { useEffect, type MutableRefObject } from "react";
import type { EditorTab } from "../types/app";

type GlobalKeybindingOptions = {
  isWindows: boolean;
  activeTabRef: MutableRefObject<EditorTab>;
  isEphemeralRef: MutableRefObject<boolean>;
  onShowHelp: () => void;
  onDismissOverlays: () => void;
  onSaveNote: () => void;
  onNewDraft: () => void;
  onTogglePalette: () => void;
  onToggleClipboard: () => void;
};

/**
 * Document-level shortcuts (F1 help, Ctrl+S save, Ctrl+N draft, Ctrl+P palette, Ctrl+Shift+V clipboard).
 * Palette toggle is suppressed while focus is inside an editable field.
 */
export function useGlobalKeybindings({
  isWindows,
  activeTabRef,
  isEphemeralRef,
  onShowHelp,
  onDismissOverlays,
  onSaveNote,
  onNewDraft,
  onTogglePalette,
  onToggleClipboard,
}: GlobalKeybindingOptions) {
  useEffect(() => {
    const isEditableTarget = (target: EventTarget | null): boolean => {
      if (!(target instanceof HTMLElement)) return false;
      const tag = target.tagName;
      return tag === "INPUT" || tag === "TEXTAREA" || target.isContentEditable;
    };

    const handleKeyDown = (e: KeyboardEvent) => {
      const isCmdOrCtrl = e.metaKey || e.ctrlKey;

      if (isWindows && e.key === "F1") {
        e.preventDefault();
        onShowHelp();
        return;
      }

      if (e.key === "Escape") {
        onDismissOverlays();
        return;
      }

      if (isCmdOrCtrl && !e.shiftKey && (e.key === "s" || e.key === "S")) {
        e.preventDefault();
        e.stopPropagation();
        if (activeTabRef.current !== "secrets") {
          onSaveNote();
        }
        return;
      }

      if (isCmdOrCtrl && !e.shiftKey && (e.key === "n" || e.key === "N")) {
        e.preventDefault();
        e.stopPropagation();
        if (activeTabRef.current !== "secrets") {
          onNewDraft();
        }
        return;
      }

      if (isCmdOrCtrl && e.key.toLowerCase() === "p" && !isEditableTarget(e.target)) {
        e.preventDefault();
        onTogglePalette();
        return;
      }

      if (isCmdOrCtrl && e.shiftKey && e.key.toLowerCase() === "v") {
        e.preventDefault();
        onToggleClipboard();
      }
    };

    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [
    isWindows,
    activeTabRef,
    isEphemeralRef,
    onShowHelp,
    onDismissOverlays,
    onSaveNote,
    onNewDraft,
    onTogglePalette,
    onToggleClipboard,
  ]);
}

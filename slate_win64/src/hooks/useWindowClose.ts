import { useEffect, type MutableRefObject } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen, isTauri } from "../utils/ipc";
import { isWindowsDesktop } from "../utils/platform";

type SaveDebounceRef = MutableRefObject<ReturnType<typeof setTimeout> | null>;

type WindowCloseOptions = {
  saveDebounceRef: SaveDebounceRef;
  pendingWindowClose: MutableRefObject<boolean>;
  quittingRef: MutableRefObject<boolean>;
  setClosePromptOpen: (open: boolean) => void;
  saveNoteNow: () => Promise<void>;
  closeApp: () => Promise<void>;
};

/**
 * Consolidates three quit/close paths into one policy:
 * 1. Tray/menu emits `window-close-requested` or `app-quit-requested`
 * 2. WebView2 `onCloseRequested` (Windows)
 *
 * If a debounced save is pending, we show UnsavedChangesModal instead of
 * quitting immediately (prevents data loss + stuck preventDefault loops).
 */
export function useWindowClose({
  saveDebounceRef,
  pendingWindowClose,
  quittingRef,
  setClosePromptOpen,
  saveNoteNow,
  closeApp,
}: WindowCloseOptions) {
  const promptOrQuit = () => {
    if (saveDebounceRef.current) {
      pendingWindowClose.current = true;
      setClosePromptOpen(true);
      return;
    }
    void (async () => {
      await saveNoteNow();
      await closeApp();
    })();
  };

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void listen("window-close-requested", promptOrQuit).then((fn) => {
      unlisten = fn;
    });
    return () => {
      unlisten?.();
    };
  }, [saveNoteNow, closeApp, saveDebounceRef, pendingWindowClose, setClosePromptOpen]);

  useEffect(() => {
    if (!isWindowsDesktop() || !isTauri()) {
      return;
    }
    let unlisten: (() => void) | undefined;
    void getCurrentWindow()
      .onCloseRequested(async (event) => {
        event.preventDefault();
        promptOrQuit();
      })
      .then((fn) => {
        unlisten = fn;
      });
    return () => {
      unlisten?.();
    };
  }, [saveNoteNow, closeApp, saveDebounceRef, pendingWindowClose, setClosePromptOpen]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void listen("app-quit-requested", () => {
      if (quittingRef.current) {
        return;
      }
      if (saveDebounceRef.current) {
        pendingWindowClose.current = true;
        setClosePromptOpen(true);
        return;
      }
      void closeApp();
    }).then((fn) => {
      unlisten = fn;
    });
    return () => {
      unlisten?.();
    };
  }, [closeApp, saveDebounceRef, pendingWindowClose, quittingRef, setClosePromptOpen]);
}

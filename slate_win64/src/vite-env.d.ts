/// <reference types="vite/client" />

/** Minimal Chrome extension types for shared desktop/extension ipc shim. */
interface ChromeRuntime {
  sendMessage?: (
    message: unknown,
    callback?: (response: unknown) => void,
  ) => void;
  onMessage?: {
    addListener: (callback: (message: unknown) => void) => void;
    removeListener: (callback: (message: unknown) => void) => void;
  };
  lastError?: { message?: string };
}

interface ChromeApi {
  runtime?: ChromeRuntime;
}

declare const chrome: ChromeApi | undefined;

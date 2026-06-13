import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { listen as tauriListen } from "@tauri-apps/api/event";
import { getCurrentWindow as getTauriWindow, LogicalSize as TauriLogicalSize } from "@tauri-apps/api/window";
import { openUrl as tauriOpenUrl } from "@tauri-apps/plugin-opener";

/** Determine if running in the Tauri desktop environment */
export const isTauri = (): boolean => {
  return typeof window !== "undefined" && (window as any).__TAURI_INTERNALS__ !== undefined;
};

/** Platform-agnostic openUrl helper */
export async function openUrl(url: string): Promise<void> {
  if (isTauri()) {
    return tauriOpenUrl(url);
  } else {
    window.open(url, "_blank", "noopener,noreferrer");
  }
}

/** Platform-agnostic invoke wrapper */
export async function invoke<T>(cmd: string, args?: Record<string, any>): Promise<T> {
  if (isTauri()) {
    return tauriInvoke<T>(cmd, args);
  } else {
    // Chrome Extension messaging fallback
    if (typeof chrome !== "undefined" && chrome.runtime?.sendMessage) {
      return new Promise<T>((resolve, reject) => {
        chrome.runtime.sendMessage({ type: "tauri-command", cmd, args }, (response) => {
          if (chrome.runtime.lastError) {
            reject(new Error(chrome.runtime.lastError.message));
          } else if (response && response.error) {
            reject(response.error);
          } else {
            resolve(response?.data as T);
          }
        });
      });
    }
    return Promise.reject(new Error("No platform IPC handler available"));
  }
}

/** Platform-agnostic event listener wrapper */
export function listen<T>(event: string, handler: (event: { payload: T }) => void): Promise<() => void> {
  if (isTauri()) {
    return tauriListen<T>(event, handler);
  } else {
    // Browser extension event fallback via chrome.runtime.onMessage
    if (typeof chrome !== "undefined" && chrome.runtime?.onMessage) {
      const listener = (message: any) => {
        if (message && message.type === "tauri-event" && message.event === event) {
          handler({ payload: message.payload });
        }
      };
      chrome.runtime.onMessage.addListener(listener);
      return Promise.resolve(() => {
        chrome.runtime.onMessage.removeListener(listener);
      });
    }
    return Promise.resolve(() => {});
  }
}

/** Platform-agnostic window helper */
export const getPlatformWindow = () => {
  if (isTauri()) {
    const win = getTauriWindow();
    return {
      innerSize: () => win.innerSize(),
      setSize: (size: any) => win.setSize(size),
      setTheme: (theme: "light" | "dark") => win.setTheme(theme),
      setBackgroundColor: (color: string) => win.setBackgroundColor(color),
    };
  } else {
    // Mock window methods for extension environment
    return {
      innerSize: () => Promise.resolve({ width: window.innerWidth, height: window.innerHeight }),
      setSize: async (_size: any) => {
        // Extensions cannot resize their own popups/sidepanels directly, so we do nothing
      },
      setTheme: async (_theme: "light" | "dark") => {
        // Themes are handled purely via document data-themes in CSS, so do nothing
      },
      setBackgroundColor: async (_color: string) => {
        // Handled via CSS, so do nothing
      },
    };
  }
};

/** Mock LogicalSize that maps to Tauri's LogicalSize if available */
export class LogicalSize {
  width: number;
  height: number;

  constructor(width: number, height: number) {
    this.width = width;
    this.height = height;
    if (isTauri()) {
      return new TauriLogicalSize(width, height) as any;
    }
  }
}

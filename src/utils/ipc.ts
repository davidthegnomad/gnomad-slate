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

/** Chrome extension API when running outside Tauri (optional). */
function extensionChrome(): typeof chrome | undefined {
  return (globalThis as { chrome?: typeof chrome }).chrome;
}

/** Platform-agnostic invoke wrapper */
export async function invoke<T>(cmd: string, args?: Record<string, any>): Promise<T> {
  if (isTauri()) {
    return tauriInvoke<T>(cmd, args);
  } else {
    const chromeApi = extensionChrome();
    const runtime = chromeApi?.runtime;
    if (runtime?.sendMessage) {
      return new Promise<T>((resolve, reject) => {
        runtime.sendMessage!({ type: "tauri-command", cmd, args }, (response: unknown) => {
          if (runtime.lastError) {
            reject(new Error(runtime.lastError.message));
          } else if (
            response &&
            typeof response === "object" &&
            "error" in response &&
            (response as { error?: unknown }).error
          ) {
            reject((response as { error: unknown }).error);
          } else {
            resolve((response as { data?: T } | undefined)?.data as T);
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
    const chromeApi = extensionChrome();
    const onMessage = chromeApi?.runtime?.onMessage;
    if (onMessage) {
      const listener = (message: unknown) => {
        if (
          message &&
          typeof message === "object" &&
          (message as { type?: string; event?: string }).type === "tauri-event" &&
          (message as { event?: string }).event === event
        ) {
          handler({ payload: (message as { payload: T }).payload });
        }
      };
      onMessage.addListener(listener);
      return Promise.resolve(() => {
        onMessage.removeListener(listener);
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
      destroy: () => win.destroy(),
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
      destroy: async () => {
        window.close();
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

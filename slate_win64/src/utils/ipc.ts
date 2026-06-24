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

/** Platform-agnostic invoke wrapper (Tauri desktop only). */
export async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (isTauri()) {
    return tauriInvoke<T>(cmd, args);
  }
  return Promise.reject(new Error("Desktop IPC is only available in the Tauri app"));
}

/** Platform-agnostic event listener wrapper (Tauri desktop only). */
export function listen<T>(event: string, handler: (event: { payload: T }) => void): Promise<() => void> {
  if (isTauri()) {
    return tauriListen<T>(event, handler);
  }
  return Promise.resolve(() => {});
}

/** After frontend save/cleanup: backend locks vault and exits. */
export async function quitDesktopApp(): Promise<void> {
  if (!isTauri()) {
    window.close();
    return;
  }
  await tauriInvoke("confirm_quit");
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
      close: () => win.close(),
    };
  } else {
    // Mock window methods for extension environment
    return {
      innerSize: () => Promise.resolve({ width: window.innerWidth, height: window.innerHeight }),
      setSize: async (_size: any) => {},
      setTheme: async (_theme: "light" | "dark") => {},
      setBackgroundColor: async (_color: string) => {},
      destroy: async () => {
        window.close();
      },
      close: async () => {
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

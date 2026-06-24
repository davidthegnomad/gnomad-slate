import { type as osType } from "@tauri-apps/plugin-os";
import { isTauri } from "./ipc";

/** True when running the Tauri desktop build on Windows. */
export function isWindowsDesktop(): boolean {
  if (!isTauri()) return false;
  try {
    return osType() === "windows";
  } catch {
    return false;
  }
}

/** True when running the Tauri desktop build on Linux. */
export function isLinuxDesktop(): boolean {
  if (!isTauri()) return false;
  try {
    return osType() === "linux";
  } catch {
    return false;
  }
}

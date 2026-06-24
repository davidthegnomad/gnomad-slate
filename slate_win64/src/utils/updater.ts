import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

export type UpdateCheckResult =
  | { status: "latest"; message: string }
  | { status: "available"; message: string; update: Update }
  | { status: "error"; message: string };

export async function checkForUpdates(
  installIfAvailable = false
): Promise<UpdateCheckResult> {
  try {
    const update = await check();

    if (!update) {
      return {
        status: "latest",
        message: "You're on the latest version.",
      };
    }

    if (!installIfAvailable) {
      return {
        status: "available",
        message: `Version ${update.version} is available.`,
        update,
      };
    }

    await update.downloadAndInstall();
    await relaunch();

    return {
      status: "available",
      message: `Updated to ${update.version}. Restarting…`,
      update,
    };
  } catch (error) {
    const message =
      error instanceof Error ? error.message : "Could not check for updates.";

    return {
      status: "error",
      message,
    };
  }
}

export async function installAvailableUpdate(
  update: Update
): Promise<{ ok: true } | { ok: false; message: string }> {
  try {
    await update.downloadAndInstall();
    await relaunch();
    return { ok: true };
  } catch (error) {
    return {
      ok: false,
      message:
        error instanceof Error ? error.message : "Update installation failed.",
    };
  }
}

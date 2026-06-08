import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import type { Update } from "@tauri-apps/plugin-updater";
import {
  checkForUpdates,
  installAvailableUpdate,
  type UpdateCheckResult,
} from "../utils/updater";

interface AppVersionInfo {
  version: string;
  channel: string;
  display_version: string;
}

interface AboutDialogProps {
  open: boolean;
  autoCheckUpdates?: boolean;
  onClose: () => void;
}

export function AboutDialog({
  open,
  autoCheckUpdates = false,
  onClose,
}: AboutDialogProps) {
  const [versionInfo, setVersionInfo] = useState<AppVersionInfo | null>(null);
  const [updateStatus, setUpdateStatus] = useState<string | null>(null);
  const [pendingUpdate, setPendingUpdate] = useState<Update | null>(null);
  const [checking, setChecking] = useState(false);
  const [installing, setInstalling] = useState(false);

  const runUpdateCheck = useCallback(async (install = false) => {
    setChecking(true);
    setUpdateStatus(null);
    setPendingUpdate(null);

    const result: UpdateCheckResult = await checkForUpdates(install);

    if (result.status === "available" && "update" in result) {
      setPendingUpdate(result.update);
      setUpdateStatus(result.message);
    } else {
      setUpdateStatus(result.message);
    }

    setChecking(false);
  }, []);

  useEffect(() => {
    if (!open) {
      return;
    }

    void invoke<AppVersionInfo>("get_app_version_info").then(setVersionInfo);

    if (autoCheckUpdates) {
      void runUpdateCheck(false);
    }
  }, [open, autoCheckUpdates, runUpdateCheck]);

  if (!open) {
    return null;
  }

  const channelNote =
    versionInfo?.channel === "alpha"
      ? "Linux and Windows builds are alpha — updates ship after platform QA."
      : "macOS beta channel — check for updates anytime.";

  return (
    <div className="overlay-backdrop" onClick={onClose}>
      <div
        className="modal-box about-dialog"
        onClick={(e) => e.stopPropagation()}
        role="dialog"
        aria-labelledby="about-title"
        aria-modal="true"
      >
        <div className="modal-header">
          <h3 className="modal-title" id="about-title">
            About Gnomad Slate
          </h3>
          <button
            type="button"
            className="about-close-btn"
            aria-label="Close About dialog"
            onClick={onClose}
          >
            ✕
          </button>
        </div>
        <div className="modal-body about-body">
          <div className="about-logo-row">
            <span className="about-logo-mark" aria-hidden="true">
              ✦
            </span>
            <div>
              <h2 className="about-app-name">Gnomad Slate</h2>
              <p className="about-version">
                {versionInfo?.display_version ?? "v2.1 beta"}
              </p>
              {versionInfo && (
                <p className="about-version-detail">{versionInfo.version}</p>
              )}
            </div>
          </div>

          <p className="about-channel-note">{channelNote}</p>

          <div className="about-actions">
            <button
              type="button"
              className="action-btn accent"
              disabled={checking || installing}
              onClick={() =>
                pendingUpdate
                  ? void (async () => {
                      setInstalling(true);
                      const result = await installAvailableUpdate(pendingUpdate);
                      if (!result.ok) {
                        setUpdateStatus(result.message);
                        setInstalling(false);
                      }
                    })()
                  : void runUpdateCheck(false)
              }
            >
              {checking
                ? "Checking…"
                : installing
                  ? "Installing…"
                  : pendingUpdate
                    ? "Install Update"
                    : "Check for Updates"}
            </button>
          </div>

          {updateStatus && <p className="about-update-status">{updateStatus}</p>}

          <p className="about-credit">
            Built with ❤️ by{" "}
            <button
              type="button"
              className="about-link"
              onClick={() => void openUrl("https://gnomadstudio.org")}
            >
              Gnomad Studio 🦙
            </button>
          </p>
        </div>
      </div>
    </div>
  );
}

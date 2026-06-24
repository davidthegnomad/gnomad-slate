import { useEffect, useRef, useState } from "react";
import { invoke, getPlatformWindow } from "../utils/ipc";
import { readSetting, writeSetting } from "../utils/settings";
import { resolveTheme, type ThemePreference } from "../utils/theme";

/**
 * Persists user preferences via Tauri plugin-store and syncs theme to the
 * native window + WebView background (avoids flash on theme switch).
 */
export function useAppSettings(isWindows: boolean) {
  const [theme, setTheme] = useState<ThemePreference>("dark");
  const [clipboardFilterEnabled, setClipboardFilterEnabled] = useState(true);
  const [showLineNumbers, setShowLineNumbers] = useState(false);
  const settingsReady = useRef(false);

  // Load persisted settings once on mount.
  useEffect(() => {
    void (async () => {
      const storedTheme = await readSetting("gnomad-theme");
      if (storedTheme === "light" || storedTheme === "dark" || storedTheme === "system") {
        setTheme(storedTheme);
      }
      const clipFilter = await readSetting("gnomad-clipboard-filter");
      const clipEnabled = clipFilter !== "false";
      setClipboardFilterEnabled(clipEnabled);
      const lineNumbers = await readSetting("gnomad-line-numbers");
      if (lineNumbers === "true") {
        setShowLineNumbers(true);
      } else if (lineNumbers === "false") {
        setShowLineNumbers(false);
      }
      settingsReady.current = true;
      void invoke("set_clipboard_filter_enabled", { enabled: clipEnabled });
    })();
  }, []);

  // Apply theme to DOM + native window; re-subscribe when "system" tracks OS changes.
  useEffect(() => {
    if (settingsReady.current) {
      void writeSetting("gnomad-theme", theme);
    }

    const applyTheme = () => {
      const active = resolveTheme(
        theme,
        window.matchMedia("(prefers-color-scheme: dark)").matches
      );
      if (active === "light") {
        document.documentElement.setAttribute("data-theme", "light");
      } else {
        document.documentElement.removeAttribute("data-theme");
      }
      const win = getPlatformWindow();
      void win.setTheme(active);
      void win.setBackgroundColor(active === "light" ? "#f3f4f6" : "#0d0d10");
    };

    applyTheme();

    if (theme === "system") {
      const media = window.matchMedia("(prefers-color-scheme: dark)");
      const listener = () => applyTheme();
      media.addEventListener("change", listener);
      return () => media.removeEventListener("change", listener);
    }
  }, [theme]);

  useEffect(() => {
    if (!settingsReady.current) {
      return;
    }
    void writeSetting("gnomad-clipboard-filter", String(clipboardFilterEnabled));
    void invoke("set_clipboard_filter_enabled", { enabled: clipboardFilterEnabled });
  }, [clipboardFilterEnabled]);

  useEffect(() => {
    if (!isWindows || !settingsReady.current) {
      return;
    }
    void writeSetting("gnomad-line-numbers", String(showLineNumbers));
  }, [isWindows, showLineNumbers]);

  return {
    theme,
    setTheme,
    clipboardFilterEnabled,
    setClipboardFilterEnabled,
    showLineNumbers,
    setShowLineNumbers,
    settingsReady,
  };
}

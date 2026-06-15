export function isMacOS(): boolean {
  if (typeof navigator === "undefined") return false;
  const platform = navigator.platform ?? "";
  const ua = navigator.userAgent ?? "";
  return /Mac|iPhone|iPod|iPad/i.test(platform) || /Mac OS X/i.test(ua);
}

export function modKeyLabel(): string {
  return isMacOS() ? "⌘" : "Ctrl";
}

export function formatShortcut(shortcut: string): string {
  if (isMacOS()) {
    return shortcut.replace(/\bCtrl\b/g, "⌘").replace(/\bCmd\b/g, "⌘");
  }
  return shortcut.replace(/\bCmd\b/g, "Ctrl");
}

export function modShortcut(key: string, extras?: { shift?: boolean; alt?: boolean }): string {
  const parts: string[] = [modKeyLabel()];
  if (extras?.shift) parts.push("Shift");
  if (extras?.alt) parts.push(isMacOS() ? "⌥" : "Alt");
  parts.push(key);
  return parts.join("+");
}

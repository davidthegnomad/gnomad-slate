import { modKeyLabel, modShortcut } from "../utils/modKeyLabel";

export type PaletteCommandId =
  | "format-json"
  | "prettify-markdown"
  | "copy-html"
  | "daily-note"
  | "append-daily"
  | "lock-vault"
  | "theme-light"
  | "theme-dark"
  | "theme-system";

export type PaletteCommand = {
  id: PaletteCommandId;
  name: string;
  desc: string;
  shortcut: string;
};

/** Static command palette entries — dispatch by stable `id`, not display `name`. */
export const PALETTE_COMMANDS: PaletteCommand[] = [
  { id: "format-json", name: "Format JSON", desc: "Prettifies and validates raw JSON in the editor", shortcut: `${modKeyLabel()}+F` },
  { id: "prettify-markdown", name: "Prettify Markdown", desc: "Cleans list structure and header spacing", shortcut: `${modKeyLabel()}+L` },
  { id: "copy-html", name: "Copy as Raw HTML", desc: "Compiles Markdown to sanitized HTML and copies", shortcut: modShortcut("H", { shift: true }) },
  { id: "daily-note", name: "Create Daily Note", desc: "Creates or appends to daily journal note", shortcut: modShortcut("D") },
  { id: "append-daily", name: "Append to Daily Note", desc: "Appends selected text directly to today's log", shortcut: modShortcut("D", { alt: true }) },
  { id: "lock-vault", name: "Lock / Unlock Secret Vault", desc: "Toggles Vault-X secure credentials lock", shortcut: modShortcut("L", { alt: true }) },
  { id: "theme-light", name: "Switch to Light Theme", desc: "Sets theme to a crisp white minimalist style", shortcut: "" },
  { id: "theme-dark", name: "Switch to Dark Theme", desc: "Sets theme to deep obsidian neon style", shortcut: "" },
  { id: "theme-system", name: "Switch to System Theme", desc: "Matches system preferences dynamically", shortcut: "" },
];

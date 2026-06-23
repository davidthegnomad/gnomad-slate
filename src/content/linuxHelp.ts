/** Detailed help copy shown in the Linux Help dialog. */
export const LINUX_HELP_SECTIONS = [
  {
    title: "Getting started",
    body: `Gnomad Slate is a desktop scratchpad for quick markdown notes, clipboard history, and Vault-X encrypted secrets.

Press Super+Shift+Space from anywhere to summon or hide the window. If the window is hidden, use the system tray icon or Window menu to show it again.`,
  },
  {
    title: "Notes",
    body: `Scratchpad notes are ephemeral and auto-archive after 48 hours. Saved notes live in Vaults until you delete them.

Ctrl+S saves the current note. If you're on a scratchpad draft, Ctrl+S promotes it to a saved note. Ctrl+N starts a new scratchpad draft.

Use Edit, Split, or Preview in the toolbar to change how you work with markdown.`,
  },
  {
    title: "Command palette & clipboard",
    body: `Ctrl+P opens the command palette for JSON formatting, markdown cleanup, theme switching, and more.

Ctrl+Shift+V opens the clipboard history capsule. Toggle "Filter sensitive clipboard items" in the sidebar to block likely API keys and tokens.`,
  },
  {
    title: "Vault-X secrets",
    body: `Open the Secrets tab to create or unlock an AES-256-GCM vault protected by an Argon2id master password.

Secrets never leave Rust memory in full — the UI only sees metadata until you copy or reveal a value. Lock the vault when you're done.`,
  },
  {
    title: "Data location",
    body: `Notes and vault data are stored locally at ~/.local/share/gnomad-slate/

Back up that folder to preserve your notes and encrypted secrets.`,
  },
  {
    title: "Support",
    body: `Questions, bugs, or feedback: david@gnomadstudio.org

Built with ❤️ by Gnomad Studio — https://gnomadstudio.org`,
  },
] as const;

export const LINUX_SUPPORT_EMAIL = "david@gnomadstudio.org";

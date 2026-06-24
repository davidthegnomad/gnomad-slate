import { invoke } from "../utils/ipc";
import { renderMarkdownToHtml } from "../utils/markdown";
import type { PaletteCommandId } from "./paletteCommands";
import type { ThemePreference } from "../utils/theme";

type ExecutePaletteContext = {
  editorContent: string;
  setEditorContent: (value: string) => void;
  saveActiveNote: (content?: string, title?: string, forceVault?: boolean) => void;
  setTheme: (theme: ThemePreference) => void;
  setActiveTab: (tab: "all" | "scratch" | "vault" | "secrets") => void;
  loadNotes: () => Promise<void>;
  showAppMessage: (message: string) => void;
};

const JSON_REVIVER = (_key: string, value: unknown) => {
  if (_key === "__proto__" || _key === "constructor") {
    return undefined;
  }
  return value;
};

const PALETTE_HANDLERS: Record<
  PaletteCommandId,
  (ctx: ExecutePaletteContext) => void | Promise<void>
> = {
  "theme-light": (ctx) => {
    ctx.setTheme("light");
  },
  "theme-dark": (ctx) => {
    ctx.setTheme("dark");
  },
  "theme-system": (ctx) => {
    ctx.setTheme("system");
  },
  "format-json": (ctx) => {
    try {
      const parsed = JSON.parse(ctx.editorContent, JSON_REVIVER);
      const formatted = JSON.stringify(parsed, null, 2);
      ctx.setEditorContent(formatted);
      ctx.saveActiveNote(formatted);
    } catch {
      ctx.showAppMessage("Error: Active note content does not contain valid JSON.");
    }
  },
  "prettify-markdown": (ctx) => {
    const prettified = ctx.editorContent
      .split("\n")
      .map((line) => {
        if (line.startsWith("-") && !line.startsWith("- ")) {
          return "- " + line.substring(1);
        }
        return line;
      })
      .join("\n");
    ctx.setEditorContent(prettified);
    ctx.saveActiveNote(prettified);
  },
  "copy-html": async (ctx) => {
    const html = renderMarkdownToHtml(ctx.editorContent);
    try {
      await invoke("copy_text_to_clipboard", { text: html });
      ctx.showAppMessage("Markdown compiled to HTML and copied to clipboard.");
    } catch {
      ctx.showAppMessage("Could not copy HTML to clipboard.");
    }
  },
  "daily-note": async (ctx) => {
    const today = new Date().toISOString().split("T")[0];
    const title = `Daily_Note_${today}`;
    const newContent = `## Log: ${today}\n\n*   Active drafting logs...`;
    try {
      await invoke("write_note", {
        title,
        content: newContent,
        isEphemeral: false,
        previousFilename: null,
      });
      await ctx.loadNotes();
      ctx.showAppMessage(`Created ${title}!`);
    } catch (e) {
      console.error(e);
      ctx.showAppMessage("Could not create daily note.");
    }
  },
  "append-daily": async (ctx) => {
    const today = new Date().toISOString().split("T")[0];
    const title = `Daily_Note_${today}`;
    const newContent = `\n\n*   [Logged at ${new Date().toLocaleTimeString()}]: ${ctx.editorContent}`;
    try {
      await invoke("write_note", {
        title,
        content: newContent,
        isEphemeral: false,
        previousFilename: null,
      });
      await ctx.loadNotes();
      ctx.showAppMessage(`Appended to ${title}!`);
    } catch (e) {
      console.error(e);
      ctx.showAppMessage("Could not append to daily note.");
    }
  },
  "lock-vault": (ctx) => {
    ctx.setActiveTab("secrets");
  },
};

/** Runs a command palette action by stable id. */
export async function executePaletteCommand(
  commandId: PaletteCommandId,
  ctx: ExecutePaletteContext
): Promise<void> {
  const handler = PALETTE_HANDLERS[commandId];
  if (handler) {
    await handler(ctx);
  }
}

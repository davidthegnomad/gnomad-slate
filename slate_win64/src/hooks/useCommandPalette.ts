import { useCallback, useMemo, useState } from "react";
import { executePaletteCommand } from "../commands/executePaletteCommand";
import { PALETTE_COMMANDS, type PaletteCommandId } from "../commands/paletteCommands";
import type { EditorTab } from "../types/app";
import type { ThemePreference } from "../utils/theme";

type CommandPaletteContext = {
  editorContent: string;
  setEditorContent: (value: string) => void;
  saveActiveNote: (content?: string, title?: string, forceVault?: boolean) => void;
  setTheme: (theme: ThemePreference) => void;
  setActiveTab: (tab: EditorTab) => void;
  loadNotes: () => Promise<void>;
  showAppMessage: (message: string) => void;
  appendSource: () => string;
  isVaultUnlocked: boolean;
  lockVault: () => Promise<void>;
  flushEditor: () => Promise<void>;
};

/** Command palette open state, filtering, and execution. */
export function useCommandPalette(ctx: CommandPaletteContext) {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [selectedIdx, setSelectedIdx] = useState(0);

  const filteredCommands = useMemo(() => {
    const q = query.toLowerCase();
    return PALETTE_COMMANDS.filter(
      (cmd) => cmd.name.toLowerCase().includes(q) || cmd.desc.toLowerCase().includes(q)
    );
  }, [query]);

  const openPalette = useCallback(() => {
    setQuery("");
    setSelectedIdx(0);
    setOpen(true);
  }, []);

  const togglePalette = useCallback(() => {
    setOpen((prev) => {
      if (prev) {
        return false;
      }
      setQuery("");
      setSelectedIdx(0);
      return true;
    });
  }, []);

  const closePalette = useCallback(() => setOpen(false), []);

  const runCommand = useCallback(
    async (commandId: PaletteCommandId) => {
      closePalette();
      await executePaletteCommand(commandId, ctx);
    },
    [closePalette, ctx]
  );

  return {
    open,
    setOpen,
    query,
    setQuery,
    selectedIdx,
    setSelectedIdx,
    filteredCommands,
    openPalette,
    togglePalette,
    closePalette,
    runCommand,
  };
}

export type CommandPaletteApi = ReturnType<typeof useCommandPalette>;

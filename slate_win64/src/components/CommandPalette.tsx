import { PALETTE_COMMANDS } from "../commands/paletteCommands";
import type { CommandPaletteApi } from "../hooks/useCommandPalette";

type CommandPaletteProps = Pick<
  CommandPaletteApi,
  "open" | "query" | "setQuery" | "selectedIdx" | "closePalette" | "runCommand" | "filteredCommands"
>;

/** Modal command palette — search and run editor / app actions. */
export function CommandPalette({
  open,
  query,
  setQuery,
  selectedIdx,
  closePalette,
  runCommand,
  filteredCommands,
}: CommandPaletteProps) {
  if (!open) {
    return null;
  }

  return (
    <div className="overlay-backdrop" onClick={closePalette}>
      <div className="modal-box" onClick={(e) => e.stopPropagation()} style={{ width: "500px" }}>
        <input
          type="text"
          autoFocus
          className="palette-search"
          placeholder="Type a notepad action command..."
          value={query}
          onChange={(e) => setQuery(e.target.value)}
        />
        <div style={{ padding: "8px 0", maxHeight: "280px", overflowY: "auto" }}>
          {(filteredCommands.length > 0 ? filteredCommands : PALETTE_COMMANDS).map((cmd, idx) => (
            <div
              key={cmd.id}
              className={`palette-item ${idx === selectedIdx ? "selected" : ""}`}
              onClick={() => void runCommand(cmd.id)}
            >
              <div>
                <div style={{ fontSize: "13px", fontWeight: "500" }}>{cmd.name}</div>
                <div style={{ fontSize: "11px", color: "var(--text-muted)" }}>{cmd.desc}</div>
              </div>
              <span className="palette-shortcut">{cmd.shortcut}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

import type { ClipboardCapsuleApi } from "../hooks/useClipboardCapsule";

type ClipboardCapsuleProps = Pick<
  ClipboardCapsuleApi,
  "open" | "history" | "selectedIdx" | "closeCapsule" | "pasteFromHistory"
>;

/** Modal overlay listing recent clipboard entries for paste-into-editor. */
export function ClipboardCapsule({
  open,
  history,
  selectedIdx,
  closeCapsule,
  pasteFromHistory,
}: ClipboardCapsuleProps) {
  if (!open) {
    return null;
  }

  return (
    <div className="overlay-backdrop" onClick={closeCapsule}>
      <div className="modal-box" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h3 className="modal-title">📋 Clipboard History Capsule</h3>
          <span className="palette-shortcut">Esc to close</span>
        </div>
        <div className="modal-body">
          {history.map((item, idx) => (
            <div
              key={`${item.timestamp}-${idx}`}
              className={`clipboard-item ${idx === selectedIdx ? "selected" : ""}`}
              onClick={() => pasteFromHistory(item.text)}
            >
              {item.text}
            </div>
          ))}
          {history.length === 0 && (
            <p style={{ color: "var(--text-muted)", textAlign: "center", padding: "20px" }}>
              Clipboard history is empty. Copy text anywhere in your desktop!
            </p>
          )}
        </div>
      </div>
    </div>
  );
}

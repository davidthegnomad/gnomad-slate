interface UnsavedChangesModalProps {
  noteTitle: string;
  onSave: () => void;
  onDiscard: () => void;
  onCancel: () => void;
}

export default function UnsavedChangesModal({
  noteTitle,
  onSave,
  onDiscard,
  onCancel,
}: UnsavedChangesModalProps) {
  return (
    <div className="overlay-backdrop" onClick={onCancel}>
      <div className="modal-box" onClick={(e) => e.stopPropagation()} style={{ width: "420px" }}>
        <div className="modal-header">
          <h3 className="modal-title">Unsaved changes</h3>
        </div>
        <div className="modal-body" style={{ padding: "16px 20px" }}>
          <p style={{ fontSize: "13px", color: "var(--text-muted)", marginBottom: "16px" }}>
            Save changes to <strong style={{ color: "var(--text-main)" }}>{noteTitle || "Untitled"}</strong> before closing?
          </p>
          <div style={{ display: "flex", gap: "8px", justifyContent: "flex-end" }}>
            <button className="action-btn" onClick={onCancel}>
              Cancel
            </button>
            <button className="action-btn" onClick={onDiscard} style={{ color: "#ef4444" }}>
              Discard
            </button>
            <button className="action-btn accent" onClick={onSave}>
              Save
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

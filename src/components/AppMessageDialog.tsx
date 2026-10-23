interface AppMessageDialogProps {
  open: boolean;
  title?: string;
  message: string;
  onClose: () => void;
  confirmLabel?: string;
  onConfirm?: () => void;
}

/** In-app message dialog — avoids blocking GTK alert() on Linux WebKit. */
export function AppMessageDialog({
  open,
  title = "Gnomad Slate",
  message,
  onClose,
  confirmLabel,
  onConfirm,
}: AppMessageDialogProps) {
  if (!open) {
    return null;
  }

  return (
    <div className="overlay-backdrop app-message-layer" onClick={onClose}>
      <div
        className="modal-box app-message-dialog"
        onClick={(e) => e.stopPropagation()}
        role="alertdialog"
        aria-labelledby="app-message-title"
        aria-modal="true"
      >
        <div className="modal-header">
          <h3 className="modal-title" id="app-message-title">
            {title}
          </h3>
        </div>
        <div className="modal-body" style={{ padding: "16px 20px" }}>
          <p style={{ fontSize: "13px", lineHeight: 1.5, marginBottom: "16px" }}>{message}</p>
          <div style={{ display: "flex", gap: "8px", justifyContent: "flex-end" }}>
            {onConfirm ? (
              <>
                <button type="button" className="action-btn" onClick={onClose}>
                  Cancel
                </button>
                <button
                  type="button"
                  className="action-btn"
                  style={{ color: "#ef4444" }}
                  onClick={onConfirm}
                >
                  {confirmLabel ?? "Continue"}
                </button>
              </>
            ) : (
              <button type="button" className="action-btn accent" onClick={onClose}>
                OK
              </button>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

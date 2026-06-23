import { LINUX_HELP_SECTIONS, LINUX_SUPPORT_EMAIL } from "../content/linuxHelp";
import { openUrl } from "../utils/ipc";

interface HelpDialogProps {
  open: boolean;
  onClose: () => void;
}

export function HelpDialog({ open, onClose }: HelpDialogProps) {
  if (!open) {
    return null;
  }

  return (
    <div className="overlay-backdrop" onClick={onClose}>
      <div
        className="modal-box help-dialog"
        onClick={(e) => e.stopPropagation()}
        role="dialog"
        aria-labelledby="help-title"
        aria-modal="true"
      >
        <div className="modal-header">
          <h3 className="modal-title" id="help-title">
            Gnomad Slate — Help
          </h3>
          <button
            type="button"
            className="about-close-btn"
            aria-label="Close Help dialog"
            onClick={onClose}
          >
            ✕
          </button>
        </div>
        <div className="modal-body help-body">
          <p className="help-intro">
            Quick reference for the Linux desktop build. Need a human?{" "}
            <button
              type="button"
              className="about-link"
              onClick={() => void openUrl(`mailto:${LINUX_SUPPORT_EMAIL}`)}
            >
              {LINUX_SUPPORT_EMAIL}
            </button>
          </p>
          {LINUX_HELP_SECTIONS.map((section) => (
            <section key={section.title} className="help-section">
              <h4 className="help-section-title">{section.title}</h4>
              <p className="help-section-body">{section.body}</p>
            </section>
          ))}
        </div>
      </div>
    </div>
  );
}

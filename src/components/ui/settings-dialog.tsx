import { useId, useRef, type ReactNode } from "react";
import { X } from "./icons";
import { DialogPortal } from "./dialog-portal";
import { useDialogKeyboard } from "./use-dialog-keyboard";
import "./settings-dialog.css";

interface SettingsDialogProps {
  title: string;
  description?: string;
  closeLabel?: string;
  onClose: () => void;
  childDialogActive?: boolean;
  children: ReactNode;
}

export function SettingsDialog({
  title,
  description,
  closeLabel = title,
  onClose,
  childDialogActive = false,
  children,
}: SettingsDialogProps) {
  const titleId = useId();
  const closeRef = useRef<HTMLButtonElement>(null);
  const dialogRef = useRef<HTMLElement>(null);
  useDialogKeyboard({
    rootRef: dialogRef,
    initialFocusRef: closeRef,
    onEscape: onClose,
    enabled: !childDialogActive,
  });

  return (
    <DialogPortal>
      <div className="sd-overlay">
        <button
          type="button"
          className="sd-backdrop-dismiss"
          tabIndex={-1}
          aria-label={closeLabel}
          onClick={onClose}
        />
        <section ref={dialogRef} className="sd-dialog relief" role="dialog" aria-modal="true" aria-labelledby={titleId}>
          <header className="sd-head">
            <div>
              <h2 id={titleId}>{title}</h2>
              {description && <p>{description}</p>}
            </div>
            <button
              ref={closeRef}
              type="button"
              className="icon-btn icon-btn-secondary"
              aria-label={closeLabel}
              onClick={onClose}
            >
              <X size="var(--icon-sm)" />
            </button>
          </header>
          {children}
        </section>
      </div>
    </DialogPortal>
  );
}

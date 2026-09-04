import { useCallback, useId, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { ShieldWarning } from "@/components/ui/icons";
import { DialogPortal } from "@/components/ui/dialog-portal";
import { useDialogKeyboard } from "@/components/ui/use-dialog-keyboard";
import type { ExtensionRecord } from "@/types/extensions";
import "./extension-activation-dialog.css";

interface ExtensionActivationDialogProps {
  extension: ExtensionRecord;
  busy: boolean;
  errorKey: string | null;
  onCancel: () => void;
  onConfirm: () => void;
}

export function ExtensionActivationDialog({
  extension,
  busy,
  errorKey,
  onCancel,
  onConfirm,
}: ExtensionActivationDialogProps) {
  const { t } = useTranslation();
  const titleId = useId();
  const descriptionId = useId();
  const cancelRef = useRef<HTMLButtonElement>(null);
  const dialogRef = useRef<HTMLDivElement>(null);
  const advanced = extension.manifest.ui?.mode === "advanced";
  const [advancedConfirmed, setAdvancedConfirmed] = useState(false);
  const handleEscape = useCallback(() => { if (!busy) onCancel(); }, [busy, onCancel]);
  useDialogKeyboard({
    rootRef: dialogRef,
    initialFocusRef: cancelRef,
    onEscape: handleEscape,
  });
  return (
    <DialogPortal>
      <div
        className="wk-dialog-overlay"
        role="presentation"
        onClick={(event) => {
          if (event.target === event.currentTarget && !busy) onCancel();
        }}
      >
        <div
          ref={dialogRef}
          className="wk-dialog extc-dialog"
          role="dialog"
          aria-modal="true"
          aria-labelledby={titleId}
          aria-describedby={descriptionId}
        >
          <h3 id={titleId}>
            {t("extensions.activation.title", { name: extension.manifest.name })}
          </h3>
          <div className="extc-warning">
            <ShieldWarning size="var(--icon-xl)" weight="fill" />
            <p id={descriptionId}>{t("extensions.activation.description")}</p>
          </div>
          {advanced && (
            <div className="extc-advanced">
              <p>{t("extensions.activation.advancedDescription")}</p>
              <label>
                <input
                  type="checkbox"
                  checked={advancedConfirmed}
                  disabled={busy}
                  onChange={(event) => setAdvancedConfirmed(event.target.checked)}
                />
                <span>{t("extensions.activation.advancedConfirmation")}</span>
              </label>
            </div>
          )}
          {errorKey && (
            <p className="extp-message extp-message-error" role="alert">
              {t(errorKey)}
            </p>
          )}
          <div className="wk-dialog-footer">
            <button
              ref={cancelRef}
              type="button"
              className="btn btn-sm btn-secondary"
              disabled={busy}
              onClick={onCancel}
            >
              {t("extensions.actions.cancel")}
            </button>
            <button
              type="button"
              className="btn btn-sm btn-primary"
              disabled={busy || (advanced && !advancedConfirmed)}
              onClick={onConfirm}
            >
              {t("extensions.activation.confirm")}
            </button>
          </div>
        </div>
      </div>
    </DialogPortal>
  );
}

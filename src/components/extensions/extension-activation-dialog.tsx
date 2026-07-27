import { useTranslation } from "react-i18next";
import { ShieldWarning } from "@/components/ui/icons";
import type { ExtensionRecord } from "@/types/extensions";
import "./extension-activation-dialog.css";

interface ExtensionActivationDialogProps {
  extension: ExtensionRecord;
  busy: boolean;
  onCancel: () => void;
  onConfirm: () => void;
}

export function ExtensionActivationDialog({
  extension,
  busy,
  onCancel,
  onConfirm,
}: ExtensionActivationDialogProps) {
  const { t } = useTranslation();
  return (
    <div
      className="wk-dialog-overlay"
      role="button"
      tabIndex={-1}
      aria-label={t("extensions.actions.cancel")}
      onClick={(event) => {
        if (event.target === event.currentTarget && !busy) onCancel();
      }}
      onKeyDown={(event) => {
        if (event.key === "Escape" && !busy) onCancel();
      }}
    >
      <div
        className="wk-dialog extc-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="extc-title"
      >
        <h3 id="extc-title">
          {t("extensions.activation.title", { name: extension.manifest.name })}
        </h3>
        <div className="extc-warning">
          <ShieldWarning size="var(--icon-xl)" weight="fill" />
          <p>{t("extensions.activation.description")}</p>
        </div>
        <div className="wk-dialog-footer">
          <button
            type="button"
            className="wk-btn-secondary"
            disabled={busy}
            onClick={onCancel}
          >
            {t("extensions.actions.cancel")}
          </button>
          <button
            type="button"
            className="wk-btn-primary"
            disabled={busy}
            onClick={onConfirm}
          >
            {t("extensions.activation.confirm")}
          </button>
        </div>
      </div>
    </div>
  );
}

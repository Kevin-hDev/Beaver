import { useCallback, useId, useRef } from "react";
import { useTranslation } from "react-i18next";
import { ShieldWarning } from "@/components/ui/icons";
import { DialogPortal } from "@/components/ui/dialog-portal";
import { useDialogKeyboard } from "@/components/ui/use-dialog-keyboard";
import type { ExtensionUiStartupState } from "@/types/extensions";
import "./extension-ui-recovery-dialog.css";

interface ExtensionUiRecoveryDialogProps {
  state: ExtensionUiStartupState;
  busy: boolean;
  error?: boolean;
  onSafe: () => void;
  onOpen: (extensionId: string) => void;
  onRetry: () => void;
  onDiscard: () => void;
}

export function ExtensionUiRecoveryDialog(props: ExtensionUiRecoveryDialogProps) {
  const { busy, onSafe } = props;
  const { t } = useTranslation();
  const titleId = useId();
  const descriptionId = useId();
  const safeRef = useRef<HTMLButtonElement>(null);
  const dialogRef = useRef<HTMLDivElement>(null);
  const invalid = props.state.mode.kind === "safe"
    && props.state.mode.reason === "invalidMarker";
  const interrupted = props.state.mode.kind === "pendingInterruptedUi"
    ? props.state.mode
    : null;

  const handleEscape = useCallback(() => { if (!busy) onSafe(); }, [busy, onSafe]);
  useDialogKeyboard({
    rootRef: dialogRef,
    initialFocusRef: safeRef,
    onEscape: handleEscape,
  });

  return (
    <DialogPortal>
      <div
        className="wk-dialog-overlay extur-overlay"
        role="presentation"
        onClick={(event) => {
          if (event.target === event.currentTarget && !props.busy) props.onSafe();
        }}
      >
        <div
          ref={dialogRef}
          className="wk-dialog extur-dialog"
          role="dialog"
          aria-modal="true"
          aria-labelledby={titleId}
          aria-describedby={descriptionId}
        >
          <ShieldWarning size="var(--icon-xl)" weight="fill" />
          <h2 id={titleId}>{t("extensions.uiRecovery.title")}</h2>
          <p id={descriptionId}>{t(invalid
            ? "extensions.uiRecovery.invalidDescription"
            : "extensions.uiRecovery.description")}</p>
          {props.error && (
            <p className="extur-error" role="alert">
              {t("extensions.uiRecovery.error")}
            </p>
          )}
          <div className="extur-actions">
            <button
              ref={safeRef}
              type="button"
              className="btn btn-sm btn-primary"
              disabled={props.busy}
              onClick={props.onSafe}
            >
              {t("extensions.uiRecovery.safe")}
            </button>
            {interrupted && (
              <button
                type="button"
                className="btn btn-sm btn-secondary"
                disabled={props.busy}
                onClick={() => props.onOpen(interrupted.extensionId)}
              >
                {t("extensions.uiRecovery.open")}
              </button>
            )}
            {interrupted && props.state.canRetry && (
              <button
                type="button"
                className="btn btn-sm btn-secondary"
                disabled={props.busy}
                onClick={props.onRetry}
              >
                {t("extensions.uiRecovery.retry")}
              </button>
            )}
            {invalid && (
              <button
                type="button"
                className="btn btn-sm btn-secondary"
                disabled={props.busy}
                onClick={props.onDiscard}
              >
                {t("extensions.uiRecovery.discard")}
              </button>
            )}
          </div>
        </div>
      </div>
    </DialogPortal>
  );
}

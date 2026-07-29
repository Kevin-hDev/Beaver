import { useTranslation } from "react-i18next";
import {
  ArrowsClockwise,
  FolderOpen,
  ShieldWarning,
  Trash,
} from "@/components/ui/icons";
import { ConfirmButton } from "@/components/settings/confirm-button";

interface ExtensionActionsProps {
  busy: boolean;
  managed: boolean;
  onOpenSource: () => void;
  onUpdate: () => void;
  onReload: () => void;
  onRemove: () => void;
}

export function ExtensionActions(props: ExtensionActionsProps) {
  const { t } = useTranslation();
  return (
    <>
      {props.managed && (
        <p className="extd-update-warning">
          <ShieldWarning size="var(--icon-sm)" />
          {t("extensions.updateTrustWarning")}
        </p>
      )}
      <div className="extp-actions">
        <button
          type="button"
          className="wk-btn-secondary"
          disabled={props.busy}
          onClick={props.onOpenSource}
        >
          <FolderOpen size="var(--icon-sm)" />{t("extensions.actions.openSource")}
        </button>
        {props.managed && (
          <ConfirmButton
            className="wk-btn-secondary"
            label={<><ArrowsClockwise size="var(--icon-sm)" />{t("extensions.actions.update")}</>}
            confirmLabel={t("extensions.actions.confirmUpdate")}
            onConfirm={props.onUpdate}
            disabled={props.busy}
          />
        )}
        <button
          type="button"
          className="wk-btn-secondary"
          disabled={props.busy}
          onClick={props.onReload}
        >
          <ArrowsClockwise size="var(--icon-sm)" />{t("extensions.actions.reload")}
        </button>
        <ConfirmButton
          className="wk-btn-secondary extd-danger"
          label={<><Trash size="var(--icon-sm)" />{t("extensions.actions.remove")}</>}
          confirmLabel={t("extensions.actions.confirmRemove")}
          onConfirm={props.onRemove}
          disabled={props.busy}
        />
      </div>
    </>
  );
}

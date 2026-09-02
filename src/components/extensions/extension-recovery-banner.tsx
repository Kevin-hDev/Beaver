import { useTranslation } from "react-i18next";
import { ShieldWarning } from "@/components/ui/icons";
import type { ExtensionRecoveryState } from "@/types/extensions";

interface ExtensionRecoveryBannerProps {
  state: ExtensionRecoveryState;
  busy: boolean;
  onOpen: (id: string) => void;
  onKeepDisabled: (id: string) => void;
  onRetry: (id: string) => void;
  onDiscard: () => void;
  onRestore: () => void;
}

export function ExtensionRecoveryBanner(props: ExtensionRecoveryBannerProps) {
  const { t } = useTranslation();
  const { state } = props;
  if (!state.extensionId && !state.markerInvalid && !state.recoverySnapshotAvailable) return null;
  return (
    <section className="extp-recovery" aria-labelledby="extrb-title">
      <ShieldWarning size="var(--icon-lg)" />
      <div>
        <strong id="extrb-title">{t("extensions.recovery.title")}</strong>
        <p>{t(state.markerInvalid
          ? "extensions.recovery.invalid"
          : "extensions.recovery.description")}</p>
        <div className="extp-actions">
          {state.extensionId && (
            <>
              <button type="button" className="btn btn-sm btn-secondary" onClick={() => props.onOpen(state.extensionId!)}>{t("extensions.recovery.openDetail")}</button>
              <button type="button" className="btn btn-sm btn-secondary" disabled={props.busy} onClick={() => props.onKeepDisabled(state.extensionId!)}>{t("extensions.recovery.keepDisabled")}</button>
              {state.canRetry && <button type="button" className="btn btn-sm btn-primary" disabled={props.busy} onClick={() => props.onRetry(state.extensionId!)}>{t("extensions.recovery.retry")}</button>}
            </>
          )}
          {state.markerInvalid && <button type="button" className="btn btn-sm btn-secondary" disabled={props.busy} onClick={props.onDiscard}>{t("extensions.recovery.discard")}</button>}
          {state.recoverySnapshotAvailable && <button type="button" className="btn btn-sm btn-secondary" disabled={props.busy} onClick={props.onRestore}>{t("extensions.recovery.restore")}</button>}
        </div>
      </div>
    </section>
  );
}

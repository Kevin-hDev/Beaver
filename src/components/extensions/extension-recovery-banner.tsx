import { useTranslation } from "react-i18next";
import { ShieldWarning } from "@/components/ui/icons";
import type { ExtensionRecoveryState } from "@/types/extensions";
import { useExtensionUiStartupContext } from "@/hooks/use-extension-ui-startup";

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
  const uiStartup = useExtensionUiStartupContext();
  const { state } = props;
  const uiSafe = uiStartup?.state.showSafeBanner === true;
  if (!uiSafe && !state.extensionId && !state.markerInvalid && !state.recoverySnapshotAvailable) {
    return null;
  }
  return (
    <>
      {uiSafe && (
        <section className="extp-recovery" aria-labelledby="extrb-ui-title">
          <ShieldWarning size="var(--icon-lg)" />
          <div>
            <strong id="extrb-ui-title">{t("extensions.uiRecovery.safeBannerTitle")}</strong>
            <p>{t("extensions.uiRecovery.safeBannerDescription")}</p>
          </div>
        </section>
      )}
      {(state.extensionId || state.markerInvalid || state.recoverySnapshotAvailable) && (
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
      )}
    </>
  );
}

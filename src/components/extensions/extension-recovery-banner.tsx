import { useTranslation } from "react-i18next";
import { ShieldWarning } from "@/components/ui/icons";
import type { ExtensionRecord, ExtensionRecoveryState } from "@/types/extensions";
import { useExtensionUiStartupContext } from "@/hooks/use-extension-ui-startup";

interface ExtensionRecoveryBannerProps {
  state: ExtensionRecoveryState;
  records: ExtensionRecord[];
  busy: boolean;
  onOpen: (id: string) => void;
  onKeepDisabled: (id: string) => void;
  onRetry: (id: string) => void;
  onDiscard: () => void;
  onRestore: () => void;
  onDisableUi: (id: string) => Promise<boolean>;
}

export function ExtensionRecoveryBanner(props: ExtensionRecoveryBannerProps) {
  const { t } = useTranslation();
  const uiStartup = useExtensionUiStartupContext();
  const { state } = props;
  const uiSafe = uiStartup?.state.showSafeBanner === true;
  const incident = uiStartup?.incident ?? null;
  const incidentRecord = incident
    ? props.records.find((record) => record.manifest.id === incident.extensionId)
    : undefined;
  const visibleIncident = incident && incidentRecord?.enabled !== false ? incident : null;
  const disableInterrupted = async (extensionId: string) => {
    if (await props.onDisableUi(extensionId)) {
      uiStartup?.resolveIncident(extensionId);
    }
  };
  if (!uiSafe && !visibleIncident
    && !state.extensionId && !state.markerInvalid && !state.recoverySnapshotAvailable) {
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
      {visibleIncident && (
        <section className="extp-recovery" aria-labelledby="extrb-ui-incident-title">
          <ShieldWarning size="var(--icon-lg)" />
          <div>
            <strong id="extrb-ui-incident-title">
              {t("extensions.uiRecovery.interruptedTitle")}
            </strong>
            <p>{t("extensions.uiRecovery.interruptedDescription", {
              name: incidentRecord?.manifest.name ?? visibleIncident.extensionId,
            })}</p>
            {uiStartup?.error && (
              <p className="extp-recovery-error" role="alert">
                {t("extensions.uiRecovery.error")}
              </p>
            )}
            <div className="extp-actions">
              <button type="button" className="btn btn-sm btn-secondary" onClick={() => props.onOpen(visibleIncident.extensionId)}>
                {t("extensions.recovery.openDetail")}
              </button>
              <button type="button" className="btn btn-sm btn-secondary" disabled={props.busy || uiStartup?.busy} onClick={() => void disableInterrupted(visibleIncident.extensionId)}>
                {t("extensions.recovery.keepDisabled")}
              </button>
              <button type="button" className="btn btn-sm btn-secondary" disabled={props.busy || uiStartup?.busy} onClick={() => void uiStartup?.discardInterrupted(visibleIncident.extensionId)}>
                {t("extensions.uiRecovery.discardInterrupted")}
              </button>
            </div>
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

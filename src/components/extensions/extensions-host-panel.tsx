import { useTranslation } from "react-i18next";
import { ArrowsClockwise, ShieldWarning } from "@/components/ui/icons";
import { SettingsCard } from "@/components/settings/settings-card";
import { extensionErrorKey } from "@/lib/extension-errors";
import { IS_LINUX } from "@/lib/platform";
import { UI_DIAGNOSTIC_CODES } from "@/types/extension-ui-contract.generated";
import type { ExtensionHostStatus } from "@/types/extensions";
import "./extensions-host-panel.css";

interface ExtensionsHostPanelProps {
  host: ExtensionHostStatus;
  loaded: boolean;
  loading: boolean;
  loadError: string | null;
  busy: boolean;
  onRestart: () => void;
  onRecover: () => void;
}

export function ExtensionsHostPanel({
  host,
  loaded,
  loading,
  loadError,
  busy,
  onRestart,
  onRecover,
}: ExtensionsHostPanelProps) {
  const { t } = useTranslation();
  const stopUnconfirmed = host.lastError === "extensions_stop_unconfirmed";
  const activityEmpty = host.activity.activeInterceptors === 0
    && Object.values(host.activity.events).every((value) => value === 0);
  return (
    <>
      <p className="settings-panel-description">{t("extensions.host.description")}</p>
      {loadError && (
        <div className="extp-message extp-message-error" role="alert">
          {t(loadError)}
        </div>
      )}
      {!loaded && loading && <p role="status">{t("extensions.loading")}</p>}
      {loaded && (
        <SettingsCard className="extp-lines">
          <InfoLine label={t("extensions.host.state")} value={t(`extensions.host.states.${host.state}`)} />
          <InfoLine label={t("extensions.host.node")} value={host.nodeVersion ?? t("extensions.host.unavailable")} />
          <InfoLine label={t("extensions.host.jiti")} value={host.jitiVersion || t("extensions.host.unavailable")} />
          <InfoLine label={t("extensions.host.api")} value={host.apiVersion} />
          <InfoLine label={t("extensions.host.active")} value={String(host.activeExtensions)} />
        </SettingsCard>
      )}
      {loaded && !IS_LINUX && activityEmpty && (
        <p className="settings-panel-description">{t("extensions.host.activity.empty")}</p>
      )}
      {loaded && !IS_LINUX && !activityEmpty && (
        <SettingsCard className="extp-lines">
          <InfoLine label={t("extensions.host.activity.interceptors")} value={String(host.activity.activeInterceptors)} />
          <InfoLine label={t("extensions.host.activity.queued")} value={String(host.activity.events.queued)} />
          <InfoLine label={t("extensions.host.activity.delivered")} value={String(host.activity.events.delivered)} />
          <InfoLine label={t("extensions.host.activity.dropped")} value={String(host.activity.events.dropped)} />
          <InfoLine label={t("extensions.host.activity.timedOut")} value={String(host.activity.events.timedOut)} />
          <InfoLine label={t("extensions.host.activity.activeHandlers")} value={String(host.activity.events.activeHandlers)} />
        </SettingsCard>
      )}
      {host.lastError && (
        <div className="extp-message extp-message-error" role="alert">
          {t(extensionErrorKey(host.lastError, "extensions.errors.host"))}
          {stopUnconfirmed && <p>{t("extensions.host.quitAndRestartHint")}</p>}
        </div>
      )}
      {host.diagnostics.length > 0 && (
        <section className="exth-diagnostics">
          <h3>{t("extensions.host.diagnostics")}</h3>
          {host.diagnostics.map((diagnostic) => (
            <div
              className="exth-diagnostic"
              key={`${diagnostic.extensionId}-${diagnostic.stage}-${diagnostic.code}`}
            >
              <code>{diagnostic.extensionId}</code>
              <span>{t(diagnosticMessageKey(diagnostic.code))}</span>
              {diagnostic.file && (
                <small>
                  {diagnostic.file}
                  {diagnostic.line ? `:${diagnostic.line}${diagnostic.column ? `:${diagnostic.column}` : ""}` : ""}
                </small>
              )}
            </div>
          ))}
        </section>
      )}
      <div className="extp-actions">
        <button type="button" className="btn btn-sm btn-secondary" disabled={busy} onClick={onRestart}>
          <ArrowsClockwise size="var(--icon-sm)" />
          {t("extensions.actions.restartHost")}
        </button>
      </div>
      <div className="extp-recovery">
        <ShieldWarning size="var(--icon-lg)" />
        <div>
          <strong>{t("extensions.host.recoveryTitle")}</strong>
          <p>{t("extensions.host.recoveryDescription")}</p>
        </div>
        <button type="button" className="btn btn-sm btn-secondary" disabled={busy} onClick={onRecover}>
          {t("extensions.actions.recovery")}
        </button>
      </div>
    </>
  );
}

function diagnosticMessageKey(code: string): string {
  if (code === "ui_manifest_legacy") {
    return "extensions.diagnostics.codes.ui_manifest_legacy";
  }
  return UI_DIAGNOSTIC_CODES.some((candidate) => candidate === code)
    ? "extensions.diagnostics.uiGeneric"
    : `extensions.diagnostics.codes.${code}`;
}

function InfoLine({ label, value }: { label: string; value: string }) {
  return (
    <div className="extp-info-line">
      <span>{label}</span>
      <code>{value}</code>
    </div>
  );
}

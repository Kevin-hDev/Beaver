import { useTranslation } from "react-i18next";
import { ArrowsClockwise, ShieldWarning } from "@/components/ui/icons";
import { SettingsCard } from "@/components/settings/settings-card";
import { extensionErrorKey } from "@/lib/extension-errors";
import type { ExtensionHostStatus } from "@/types/extensions";
import "./extensions-host-panel.css";

interface ExtensionsHostPanelProps {
  host: ExtensionHostStatus;
  busy: boolean;
  onRestart: () => void;
  onRecover: () => void;
}

export function ExtensionsHostPanel({
  host,
  busy,
  onRestart,
  onRecover,
}: ExtensionsHostPanelProps) {
  const { t } = useTranslation();
  const stopUnconfirmed = host.lastError === "extensions_stop_unconfirmed";
  return (
    <>
      <p className="settings-panel-description">{t("extensions.host.description")}</p>
      <SettingsCard className="extp-lines">
        <InfoLine label={t("extensions.host.state")} value={t(`extensions.host.states.${host.state}`)} />
        <InfoLine label={t("extensions.host.node")} value={host.nodeVersion ?? t("extensions.host.unavailable")} />
        <InfoLine label={t("extensions.host.jiti")} value={host.jitiVersion || t("extensions.host.unavailable")} />
        <InfoLine label={t("extensions.host.api")} value={host.apiVersion} />
        <InfoLine label={t("extensions.host.active")} value={String(host.activeExtensions)} />
      </SettingsCard>
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
            <div className="exth-diagnostic" key={`${diagnostic.extensionId}-${diagnostic.stage}`}>
              <code>{diagnostic.extensionId}</code>
              <span>{t(`extensions.diagnostics.codes.${diagnostic.code}`)}</span>
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

function InfoLine({ label, value }: { label: string; value: string }) {
  return (
    <div className="extp-info-line">
      <span>{label}</span>
      <code>{value}</code>
    </div>
  );
}

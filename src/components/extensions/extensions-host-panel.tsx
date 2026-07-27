import { useTranslation } from "react-i18next";
import { ArrowsClockwise, ShieldWarning } from "@/components/ui/icons";
import type { ExtensionHostStatus } from "@/types/extensions";
import "./extensions-host-panel.css";

interface ExtensionsHostPanelProps {
  host: ExtensionHostStatus;
  onRestart: () => void;
  onRecover: () => void;
}

export function ExtensionsHostPanel({
  host,
  onRestart,
  onRecover,
}: ExtensionsHostPanelProps) {
  const { t } = useTranslation();
  return (
    <div className="extp-content">
      <header className="extp-header">
        <div>
          <h2>{t("extensions.host.title")}</h2>
          <p>{t("extensions.host.description")}</p>
        </div>
      </header>
      <div className="extp-lines">
        <InfoLine label={t("extensions.host.state")} value={t(`extensions.host.states.${host.state}`)} />
        <InfoLine label={t("extensions.host.node")} value={host.nodeVersion ?? t("extensions.host.unavailable")} />
        <InfoLine label={t("extensions.host.jiti")} value={host.jitiVersion} />
        <InfoLine label={t("extensions.host.api")} value={host.apiVersion} />
        <InfoLine label={t("extensions.host.active")} value={String(host.activeExtensions)} />
      </div>
      {host.lastError && (
        <div className="extp-message extp-message-error">{t("extensions.errors.host")}</div>
      )}
      <div className="extp-actions">
        <button type="button" className="wk-btn-secondary" onClick={onRestart}>
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
        <button type="button" className="wk-btn-secondary" onClick={onRecover}>
          {t("extensions.actions.recovery")}
        </button>
      </div>
    </div>
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

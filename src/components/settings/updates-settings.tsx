import { useTranslation } from "react-i18next";
import { BeaverBrandIcon } from "@/components/ui/beaver-brand-icon";
import { OllamaBrandIcon } from "@/components/ui/ollama-brand-icon";
import { UpdateProgressAction } from "@/components/updates/update-progress-action";
import { useUpdates } from "@/hooks/update-context";
import { SettingsCard } from "./settings-card";
import "./updates-settings.css";

export function UpdatesSettings() {
  const { t } = useTranslation();
  const updates = useUpdates();
  const hasUpdates = Boolean(updates.appUpdate || updates.ollamaBinaryUpdate);

  return (
    <div className="ups-root">
      <div className="ups-inner">
        <header className="ups-header">
          <div>
            <h2 className="ups-title">{t("settings.updates.title")}</h2>
            <p className="ups-description">{t("settings.updates.description")}</p>
          </div>
          <button type="button" className="btn btn-sm btn-secondary" disabled={updates.checking} onClick={() => void updates.checkAll(true)}>
            {updates.checking ? t("settings.updates.checking") : t("settings.updates.check")}
          </button>
        </header>

        <SettingsCard className="ups-card">
          <section className="ups-section">
            <h3 className="ups-section-title">{t("settings.updates.installedTitle")}</h3>
            <UpdateRow product="beaver" version={updates.installedAppVersion} />
            <UpdateRow product="ollama" version={updates.installedOllamaVersion} />
          </section>

          {hasUpdates && (
            <section className="ups-section ups-available">
              <h3 className="ups-section-title">{t("settings.updates.availableTitle")}</h3>
              {updates.appUpdate && (
                <UpdateRow
                  product="beaver"
                  version={updates.appUpdate.version}
                  action={updates.appDownloading ? (
                    <UpdateProgressAction percent={updates.appPercent} cancelling={updates.appCancelling} cancelLabel={t("common.cancel")} cancellingLabel={t("updates.cancelling")} onCancel={() => void updates.cancelAppUpdate()} />
                  ) : (
                    <button type="button" className="btn btn-sm btn-primary" disabled={updates.binaryBusy} onClick={() => void updates.downloadAppUpdate(updates.appUpdate!.assetUrl)}>{t("updates.appUpdate")}</button>
                  )}
                />
              )}
              {updates.ollamaBinaryUpdate && (
                <UpdateRow
                  product="ollama"
                  version={updates.ollamaBinaryUpdate.latestVersion}
                  action={updates.ollamaBinaryUpdating ? (
                    <UpdateProgressAction percent={updates.ollamaBinaryPercent} cancelling={updates.ollamaBinaryCancelling} cancelLabel={t("common.cancel")} cancellingLabel={t("updates.cancelling")} onCancel={() => void updates.cancelOllamaBinary()} />
                  ) : (
                    <button type="button" className="btn btn-sm btn-primary" disabled={updates.binaryBusy} onClick={() => void updates.updateOllamaBinary()}>{t("updates.ollamaBinaryUpdate")}</button>
                  )}
                />
              )}
            </section>
          )}
        </SettingsCard>
      </div>
    </div>
  );
}

function UpdateRow({ product, version, action }: { product: "beaver" | "ollama"; version: string | null; action?: React.ReactNode }) {
  return (
    <div className={`ups-row ${action ? "ups-row-has-action" : ""}`}>
      <div className="ups-product">
        {product === "beaver" ? <BeaverBrandIcon size={34} /> : <OllamaBrandIcon size={34} />}
        <span className="ups-name">{product === "beaver" ? "Beaver" : "Ollama"}</span>
      </div>
      {action && <div className="ups-action">{action}</div>}
      <span className="ups-version">{version ? `v${version.replace(/^v/, "")}` : "—"}</span>
    </div>
  );
}

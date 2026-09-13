import { FolderOpen } from "@phosphor-icons/react";

import type { InstallerSnapshot } from "./installer-contract.generated";
import { InstallerBeaver } from "./installer-beaver";
import { t } from "./installer-i18n";

interface Props {
  snapshot: InstallerSnapshot;
  onBrowse: () => void;
  onInstall: () => void;
}

export function InstallerStartScreen({ snapshot, onBrowse, onInstall }: Props) {
  const installed = snapshot.installedVersion;
  return (
    <>
      <main className="binst-body">
        <div className="binst-center">
          <InstallerBeaver size="large" frozen muted={snapshot.beaverRunning} />
          <h1 className="binst-name">Beaver</h1>
          <p className="binst-version">
            {installed
              ? t("installer.versionInstalled", { version: snapshot.version, installed })
              : t("installer.version", { version: snapshot.version })}
          </p>
        </div>
        {installed && !snapshot.beaverRunning && (
          <div className="callout callout-accent binst-notice">
            <span className="callout-title">{t("installer.alreadyInstalledTitle")}</span>
            <span>{t("installer.alreadyInstalledBody", { installed, version: snapshot.version })}</span>
          </div>
        )}
        {snapshot.beaverRunning && (
          <div className="callout callout-warning binst-notice">
            <span className="callout-title">{t("installer.runningTitle")}</span>
            <span>{t("installer.runningBody")}</span>
          </div>
        )}
        <div className="binst-directory">
          <span id="binst-directory-label" className="binst-directory-label">
            {t("installer.destination")}
          </span>
          <div className="binst-directory-row">
            <div
              className="field binst-directory-field relief"
              role="textbox"
              aria-readonly="true"
              aria-labelledby="binst-directory-label"
              tabIndex={0}
            >
              {snapshot.destination}
            </div>
            <button type="button" className="btn btn-sm btn-secondary" onClick={onBrowse}>
              <FolderOpen aria-hidden="true" />
              {t("installer.browse")}
            </button>
          </div>
        </div>
      </main>
      <footer className="binst-footer">
        {snapshot.beaverRunning && <span className="binst-footer-note">{t("installer.waiting")}</span>}
        <button
          type="button"
          className="btn btn-sm btn-primary"
          disabled={snapshot.beaverRunning}
          onClick={onInstall}
        >
          {t(installed ? "installer.reinstall" : "installer.install")}
        </button>
      </footer>
    </>
  );
}

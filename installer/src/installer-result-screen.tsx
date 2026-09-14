import { ArrowClockwise, RocketLaunch } from "@phosphor-icons/react";

import type { InstallerSnapshot } from "./installer-contract.generated";
import { InstallerBeaver } from "./installer-beaver";
import { t } from "./installer-i18n";
import { ProgressHeader, StepList } from "./installer-progress-screen";

interface Props {
  snapshot: InstallerSnapshot;
  durations: number[];
  onClose: () => void;
  onRetry: () => void;
  onLaunch: () => void;
}

export function InstallerResultScreen({ snapshot, durations, onClose, onRetry, onLaunch }: Props) {
  if (snapshot.phase === "completed") {
    return (
      <>
        <main className="binst-body">
          <div className="binst-center">
            <InstallerBeaver size="large" frozen />
            <h1 className="binst-result-title">{t("installer.readyTitle")}</h1>
            <p className="binst-result-note">{t("installer.readyBody")}</p>
          </div>
        </main>
        <footer className="binst-footer">
          <button type="button" className="btn btn-sm btn-primary" onClick={onLaunch}>
            <RocketLaunch aria-hidden="true" />
            {t("installer.launch")}
          </button>
        </footer>
      </>
    );
  }

  const failed = snapshot.phase === "failed";
  const title = t(failed ? "installer.interruptedTitle" : "installer.cancelledTitle");
  return (
    <>
      <main className="binst-body">
        <ProgressHeader snapshot={snapshot} />
        <StepList snapshot={snapshot} durations={durations} />
        <div className={`callout binst-result-callout${failed ? " binst-result-error" : ""}`}>
          <span className="callout-title">{title}</span>
          <span>{failed ? errorMessage(snapshot.errorKey) : t("installer.cancelledBody")}</span>
        </div>
      </main>
      <footer className="binst-footer">
        <button type="button" className="btn btn-sm btn-secondary" onClick={onClose}>
          {t("installer.quit")}
        </button>
        <button type="button" className="btn btn-sm btn-primary" onClick={onRetry}>
          <ArrowClockwise aria-hidden="true" />
          {t("installer.retry")}
        </button>
      </footer>
    </>
  );
}

function errorMessage(key: string | null): string {
  const messages: Record<string, string> = {
    "installer-download-failed": "installer.errors.download",
    "installer-integrity-failed": "installer.errors.integrity",
    "installer-cleanup-failed": "installer.errors.cleanup",
    "installer-invalid-launch": "installer.errors.launch",
    "installer-install-failed": "installer.errors.install",
  };
  return t(messages[key ?? ""] ?? "installer.errors.install");
}

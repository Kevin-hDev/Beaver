import { Check, Warning, X } from "@phosphor-icons/react";
import { useState } from "react";

import type { InstallerSnapshot } from "./installer-contract.generated";
import { InstallerBeaver } from "./installer-beaver";
import { t } from "./installer-i18n";

const stepKeys = ["checking", "downloading", "verifying", "installing", "finishing"];

interface ProgressProps {
  snapshot: InstallerSnapshot;
  durations: number[];
  logs: string[];
  onCancel: () => void;
}

export function InstallerProgressScreen({ snapshot, durations, logs, onCancel }: ProgressProps) {
  const [details, setDetails] = useState(false);
  return (
    <>
      <main className="binst-body">
        <ProgressHeader snapshot={snapshot} />
        <StepList snapshot={snapshot} durations={durations} />
        <div className="binst-log">
          <button
            type="button"
            className="binst-log-toggle"
            aria-expanded={details}
            aria-controls="binst-log-entries"
            onClick={() => setDetails((visible) => !visible)}
          >
            {t(details ? "installer.hideDetails" : "installer.showDetails")}
          </button>
          {details && (
            <div className="binst-log-body" id="binst-log-entries" aria-live="polite">
              {logs.map((entry, index) => (
                <p data-testid="installer-log-entry" key={`${index}-${entry}`}>
                  {entry}
                </p>
              ))}
            </div>
          )}
        </div>
      </main>
      <footer className="binst-footer">
        {!snapshot.canCancel && <span className="binst-footer-note">{t("installer.notCancellable")}</span>}
        {snapshot.canCancel && (
          <button type="button" className="btn btn-sm btn-secondary" onClick={onCancel}>
            {t("installer.cancel")}
          </button>
        )}
      </footer>
    </>
  );
}

export function ProgressHeader({ snapshot }: { snapshot: InstallerSnapshot }) {
  const current = Math.max(1, snapshot.stepIndex);
  const part = snapshot.percent === null ? 0.5 : snapshot.percent / 100;
  const total = Math.min(100, ((current - 1 + part) / snapshot.stepCount) * 100);
  const titleKey =
    snapshot.phase === "failed"
      ? "installer.interruptedTitle"
      : snapshot.phase === "cancelled"
        ? "installer.cancelledTitle"
        : "installer.installingTitle";
  return (
    <div className="binst-progress-header">
      <div className="binst-progress-heading">
        <h1>{t(titleKey, { version: snapshot.version })}</h1>
        <span>{t("installer.stepRank", { current, count: snapshot.stepCount })}</span>
      </div>
      <ProgressBar percent={total} label={t("installer.globalProgress")} />
    </div>
  );
}

export function StepList({
  snapshot,
  durations,
}: {
  snapshot: InstallerSnapshot;
  durations: number[];
}) {
  return (
    <ol className="binst-steps">
      {stepKeys.map((key, index) => {
        const number = index + 1;
        const done = number < snapshot.stepIndex;
        const active = number === snapshot.stepIndex;
        const terminal = active && (snapshot.phase === "failed" || snapshot.phase === "cancelled");
        const measure = done
          ? formatDuration(durations[index])
          : active && snapshot.percent !== null
            ? `${snapshot.percent} %`
            : terminal
              ? t(snapshot.phase === "failed" ? "installer.failedMeasure" : "installer.cancelledMeasure")
              : "";
        return (
          <li
            className={`binst-step${done ? " binst-step-done" : ""}${active ? " binst-step-active" : ""}`}
            key={key}
          >
            <StepMark done={done} active={active} phase={snapshot.phase} />
            <span className="binst-step-name">{t(`installer.steps.${key}`)}</span>
            <span className="binst-step-measure">{measure}</span>
            {active && !terminal && (
              <ProgressBar
                compact
                percent={snapshot.progressMode === "determinate" ? snapshot.percent : null}
                label={t(`installer.steps.${key}`)}
              />
            )}
          </li>
        );
      })}
    </ol>
  );
}

function StepMark({
  done,
  active,
  phase,
}: {
  done: boolean;
  active: boolean;
  phase: InstallerSnapshot["phase"];
}) {
  if (done) return <Check className="binst-step-icon binst-step-icon-ok" aria-hidden="true" />;
  if (active && phase === "failed") {
    return <Warning className="binst-step-icon binst-step-icon-error" aria-hidden="true" />;
  }
  if (active && phase === "cancelled") return <X className="binst-step-icon" aria-hidden="true" />;
  if (active) return <InstallerBeaver size="small" />;
  return <span className="binst-step-dot" aria-hidden="true" />;
}

function ProgressBar({ percent, compact = false, label }: { percent: number | null; compact?: boolean; label: string }) {
  return (
    <div
      className={`binst-progress${compact ? " binst-progress-compact" : ""}${percent === null ? " binst-progress-indeterminate" : ""}`}
      role="progressbar"
      aria-label={label}
      aria-valuemin={0}
      aria-valuemax={100}
      aria-valuenow={percent === null ? undefined : Math.round(percent)}
    >
      <span
        className={percent === null ? "binst-progress-fill operation-progress-indeterminate" : "binst-progress-fill"}
        style={percent === null ? undefined : { width: `${percent}%` }}
      />
    </div>
  );
}

function formatDuration(milliseconds: number | undefined): string {
  return milliseconds === undefined ? "" : `${(milliseconds / 1000).toLocaleString(undefined, { maximumFractionDigits: 1 })} s`;
}

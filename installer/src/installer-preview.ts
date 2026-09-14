import type { InstallerApi } from "./installer-api";
import type { InstallerPhase, InstallerSnapshot } from "./installer-contract.generated";

const screens = new Set<InstallerPhase>([
  "ready",
  "downloading",
  "installing",
  "failed",
  "cancelled",
  "completed",
]);

export function installerPreviewApi(): InstallerApi | undefined {
  const value = new URLSearchParams(window.location.search).get("screen");
  if (!value || !screens.has(value as InstallerPhase)) return undefined;
  const phase = value as InstallerPhase;
  const snapshot: InstallerSnapshot = {
    version: "1.4.2",
    destination: "/Applications",
    installedVersion: phase === "ready" ? null : "1.4.1",
    beaverRunning: false,
    phase,
    stepIndex: phase === "installing" || phase === "completed" ? 4 : 2,
    stepCount: 5,
    progressMode: phase === "downloading" ? "determinate" : "indeterminate",
    percent: phase === "downloading" ? 61 : null,
    canCancel: phase === "downloading",
    outcome: phase === "completed" ? "reinstalled" : null,
    errorKey: phase === "failed" ? "installer-download-failed" : null,
  };
  return {
    snapshot: () => Promise.resolve(snapshot),
    chooseDirectory: () => Promise.resolve(null),
    start: () => Promise.resolve(),
    cancel: () => Promise.resolve({
      sequence: 1,
      snapshot,
      completedStepDurationsMs: [],
      logKey: null,
    }),
    launch: () => Promise.resolve(),
    close: () => Promise.resolve(),
  };
}

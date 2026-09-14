import { useCallback, useEffect, useRef, useState } from "react";

import { installerApi, type InstallerApi } from "./installer-api";
import type { InstallerEvent, InstallerSnapshot } from "./installer-contract.generated";
import { t } from "./installer-i18n";
import { InstallerProgressScreen } from "./installer-progress-screen";
import { InstallerResultScreen } from "./installer-result-screen";
import { InstallerStartScreen } from "./installer-start-screen";

export type { InstallerApi } from "./installer-api";

const allowedLogKeys = new Set([
  "installer.log.checking",
  "installer.log.downloading",
  "installer.log.verifying",
  "installer.log.installing",
  "installer.log.finishing",
]);

export function InstallerApp({ api = installerApi }: { api?: InstallerApi }) {
  const [snapshot, setSnapshot] = useState<InstallerSnapshot | null>(null);
  const [snapshotFailed, setSnapshotFailed] = useState(false);
  const [durations, setDurations] = useState<number[]>([]);
  const [logs, setLogs] = useState<string[]>([]);
  const sequence = useRef(-1);
  const snapshotRequest = useRef(0);

  const loadSnapshot = useCallback(() => {
    const request = ++snapshotRequest.current;
    void api.snapshot()
      .then((value) => {
        if (request === snapshotRequest.current) setSnapshot(value);
      })
      .catch(() => {
        if (request === snapshotRequest.current) setSnapshotFailed(true);
      });
  }, [api]);

  useEffect(() => {
    sequence.current = -1;
    loadSnapshot();
    return () => {
      snapshotRequest.current += 1;
    };
  }, [loadSnapshot]);

  useEffect(() => {
    if (!snapshot?.beaverRunning) return;
    let current = true;
    let timer = 0;
    const poll = () => {
      void api.snapshot()
        .then((value) => {
          if (!current) return;
          setSnapshot(value);
          timer = window.setTimeout(poll, 1000);
        })
        .catch(() => {
          if (current) timer = window.setTimeout(poll, 1000);
        });
    };
    timer = window.setTimeout(poll, 1000);
    return () => {
      current = false;
      window.clearTimeout(timer);
    };
  }, [api, snapshot?.beaverRunning]);

  const receive = useCallback((event: InstallerEvent) => {
    if (event.sequence <= sequence.current) return;
    sequence.current = event.sequence;
    setSnapshot(event.snapshot);
    setDurations(event.completedStepDurationsMs);
    if (event.logKey && allowedLogKeys.has(event.logKey)) {
      const logKey = event.logKey;
      setLogs((entries) => [...entries, t(logKey)].slice(-64));
    }
  }, []);

  const start = useCallback(() => {
    setLogs([]);
    void api.start(receive).catch(() => {
      setSnapshot((value) => value && !["failed", "cancelled", "completed"].includes(value.phase)
        ? { ...value, phase: "failed", canCancel: false, errorKey: "installer-install-failed" }
        : value);
    });
  }, [api, receive]);

  if (snapshotFailed && !snapshot) {
    return (
      <div className="binst-window relief elev-above" data-screen="failed">
        <div className="binst-title">{t("installer.windowTitle")}</div>
        <main className="binst-body">
          <div className="callout binst-result-callout binst-result-error">
            <h1 className="callout-title">{t("installer.interruptedTitle")}</h1>
            <span>{t("installer.errors.install")}</span>
          </div>
        </main>
        <footer className="binst-footer">
          <button type="button" className="btn btn-sm btn-secondary" onClick={() => void api.close()}>
            {t("installer.quit")}
          </button>
          <button type="button" className="btn btn-sm btn-primary" onClick={loadSnapshot}>
            {t("installer.retry")}
          </button>
        </footer>
      </div>
    );
  }

  if (!snapshot) return <div className="binst-window relief elev-above" aria-busy="true" />;

  const working = !["ready", "failed", "cancelled", "completed"].includes(snapshot.phase);
  return (
    <div className="binst-window relief elev-above" data-screen={snapshot.phase}>
      <div className="binst-title">{t("installer.windowTitle")}</div>
      {snapshot.phase === "ready" && (
        <InstallerStartScreen
          snapshot={snapshot}
          onBrowse={() => {
            void api.chooseDirectory().then((value) => {
              if (value) setSnapshot(value);
            }).catch(() => {});
          }}
          onInstall={start}
        />
      )}
      {working && (
        <InstallerProgressScreen
          snapshot={snapshot}
          durations={durations}
          logs={logs}
          onCancel={() => void api.cancel().then(receive).catch(() => {})}
        />
      )}
      {["failed", "cancelled", "completed"].includes(snapshot.phase) && (
        <InstallerResultScreen
          snapshot={snapshot}
          durations={durations}
          onClose={() => void api.close()}
          onRetry={start}
          onLaunch={() => void api.launch()}
        />
      )}
    </div>
  );
}

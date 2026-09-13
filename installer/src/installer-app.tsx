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
  const [durations, setDurations] = useState<number[]>([]);
  const [logs, setLogs] = useState<string[]>([]);
  const sequence = useRef(-1);

  useEffect(() => {
    let current = true;
    sequence.current = -1;
    void api.snapshot().then((value) => {
      if (current) setSnapshot(value);
    });
    return () => {
      current = false;
    };
  }, [api]);

  useEffect(() => {
    if (!snapshot?.beaverRunning) return;
    const timer = window.setInterval(() => void api.snapshot().then(setSnapshot), 1000);
    return () => window.clearInterval(timer);
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
    void api.start(receive).catch(() => undefined);
  }, [api, receive]);

  if (!snapshot) return <div className="binst-window relief elev-above" aria-busy="true" />;

  const working = !["ready", "failed", "cancelled", "completed"].includes(snapshot.phase);
  return (
    <div className="binst-window relief elev-above" data-screen={snapshot.phase}>
      <div className="binst-title">{t("installer.windowTitle")}</div>
      {snapshot.phase === "ready" && (
        <InstallerStartScreen
          snapshot={snapshot}
          onBrowse={() => {
            void api.chooseDirectory().then((destination) => {
              if (destination) setSnapshot((value) => (value ? { ...value, destination } : value));
            });
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

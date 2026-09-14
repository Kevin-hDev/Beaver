import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./voice-probe-panel.css";

const MAX_LEVELS = 100;
const STATUSES = ["idle", "starting", "listening", "stopping", "error"] as const;
type VoiceProbeStatus = (typeof STATUSES)[number];

interface VoiceProbeSnapshot {
  status: VoiceProbeStatus;
  sample_count: number;
  levels: number[];
}

const IDLE: VoiceProbeSnapshot = { status: "idle", sample_count: 0, levels: [] };

function isSnapshot(value: unknown): value is VoiceProbeSnapshot {
  if (!value || typeof value !== "object") return false;
  const snapshot = value as Partial<VoiceProbeSnapshot>;
  return STATUSES.includes(snapshot.status as VoiceProbeStatus)
    && Number.isSafeInteger(snapshot.sample_count)
    && (snapshot.sample_count ?? -1) >= 0
    && Array.isArray(snapshot.levels)
    && snapshot.levels.length <= MAX_LEVELS
    && snapshot.levels.every((level) => Number.isFinite(level) && level >= 0 && level <= 1);
}

export function VoiceProbeGate() {
  const [available, setAvailable] = useState(false);

  useEffect(() => {
    let mounted = true;
    void invoke<boolean>("voice_probe_available")
      .then((result) => {
        if (mounted && result === true) setAvailable(true);
      })
      .catch(() => console.warn("voice_probe_availability_unavailable"));
    return () => {
      mounted = false;
    };
  }, []);

  return available ? <VoiceProbePanel /> : null;
}

function VoiceProbePanel() {
  const { t } = useTranslation();
  const [snapshot, setSnapshot] = useState<VoiceProbeSnapshot>(IDLE);
  const busy = useRef(false);

  useEffect(() => {
    let mounted = true;
    let unlisten: (() => void) | undefined;
    void listen<unknown>("voice-probe-snapshot", (event) => {
      if (mounted && isSnapshot(event.payload)) setSnapshot(event.payload);
    }).then((stopListening) => {
      if (mounted) unlisten = stopListening;
      else stopListening();
    }).catch(() => console.warn("voice_probe_listener_unavailable"));
    return () => {
      mounted = false;
      unlisten?.();
    };
  }, []);

  const start = async () => {
    if (busy.current) return;
    busy.current = true;
    setSnapshot({ ...IDLE, status: "starting" });
    try {
      const result = await invoke<unknown>("voice_probe_start");
      setSnapshot(isSnapshot(result) ? result : { ...IDLE, status: "error" });
    } catch {
      setSnapshot({ ...IDLE, status: "error" });
    } finally {
      busy.current = false;
    }
  };

  const stop = async () => {
    if (busy.current) return;
    busy.current = true;
    setSnapshot((current) => ({ ...current, status: "stopping" }));
    try {
      const result = await invoke<unknown>("voice_probe_stop");
      setSnapshot(isSnapshot(result) ? result : { ...IDLE, status: "error" });
    } catch {
      setSnapshot({ ...IDLE, status: "error" });
    } finally {
      busy.current = false;
    }
  };

  const latestLevel = snapshot.levels[snapshot.levels.length - 1] ?? 0;
  const canStart = snapshot.status === "idle" || snapshot.status === "error";
  const canStop = snapshot.status !== "idle";

  return (
    <section className="vp-panel relief elev-float" aria-label={t("voiceProbe.title")}>
      <div className="vp-heading">
        <strong>{t("voiceProbe.title")}</strong>
        <span role="status" aria-live="polite">{t(`voiceProbe.status.${snapshot.status}`)}</span>
      </div>
      <div
        className="vp-level"
        role="progressbar"
        aria-label={t("voiceProbe.level")}
        aria-valuemin={0}
        aria-valuemax={100}
        aria-valuenow={Math.round(latestLevel * 100)}
      >
        <span style={{ width: `${latestLevel * 100}%` }} />
      </div>
      <span className="vp-samples">{t("voiceProbe.samples", { count: snapshot.sample_count })}</span>
      <div className="vp-actions">
        {canStart && (
          <button className="btn btn-sm btn-primary" type="button" onClick={() => void start()}>
            {t("voiceProbe.start")}
          </button>
        )}
        {canStop && (
          <button className="btn btn-sm btn-secondary" type="button" onClick={() => void stop()}>
            {t("voiceProbe.stop")}
          </button>
        )}
      </div>
    </section>
  );
}

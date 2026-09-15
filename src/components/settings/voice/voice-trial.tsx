import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useVoiceSnapshot } from "@/features/voice/voice-store";
import { dispatchVoiceAction } from "@/features/voice/voice-client";
import { VoiceSignal } from "@/components/agent-local/voice/voice-signal";

export function VoiceTrial() {
  const { t } = useTranslation();
  const snapshot = useVoiceSnapshot();
  const [trialId] = useState(() => crypto.randomUUID());
  const [stoppedAt, setStoppedAt] = useState<number | null>(null);
  const [delayMs, setDelayMs] = useState<number | null>(null);
  const operation = snapshot?.operation?.destination.kind === "trial" && snapshot.operation.destination.trial_id === trialId ? snapshot.operation : null;
  const result = snapshot?.trialResult?.trialId === trialId ? snapshot.trialResult : null;
  useEffect(() => {
    if (!result || stoppedAt === null || delayMs !== null) return;
    const measured = performance.now() - stoppedAt;
    queueMicrotask(() => setDelayMs(measured));
  }, [delayMs, result, stoppedAt]);
  const start = () => dispatchVoiceAction({ action: "start", destination: { kind: "trial", trial_id: trialId }, context_generation: Date.now(), language: null });
  return <section className="vset-trial">
    <h3>{t("voice.settings.try")}</h3>
    {operation ? <div className="vset-trial-line"><VoiceSignal level={operation.level} /><button type="button" className="btn btn-sm btn-secondary" onClick={() => { setStoppedAt(performance.now()); setDelayMs(null); void dispatchVoiceAction({ action: "validate", operation_id: operation.id }); }}>{t("voice.validate")}</button></div>
      : <button type="button" className="btn btn-sm btn-secondary" onClick={() => void start()}>{t("voice.settings.startTrial")}</button>}
    {result && <div className="vset-result"><p>{result.text}</p><small>{t("voice.settings.trialTiming", { compute: (result.computeMs / 1000).toFixed(1), delay: delayMs === null ? "—" : (delayMs / 1000).toFixed(1) })}</small></div>}
  </section>;
}

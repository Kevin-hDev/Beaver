import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useVoiceSnapshot } from "@/features/voice/voice-store";
import { dispatchVoiceAction } from "@/features/voice/voice-client";
import { resolveVoiceLanguage } from "@/features/voice/voice-language-options";
import { VoiceSignal } from "@/components/agent-local/voice/voice-signal";
import { SettingsSelect } from "../settings-select";
import { voiceModelName } from "./voice-model-list";
import type { VoiceCatalogItem, VoiceLanguage, VoiceLanguageMode, VoiceModel } from "@/types/voice.generated";

export function VoiceTrial({ language, languageMode, model, models, onModelChange }: {
  language: VoiceLanguage;
  languageMode: VoiceLanguageMode;
  model: VoiceModel;
  models: VoiceCatalogItem[];
  onModelChange: (model: VoiceModel) => void;
}) {
  const { t, i18n } = useTranslation();
  const snapshot = useVoiceSnapshot();
  const [trialId] = useState(() => crypto.randomUUID());
  const [stoppedAt, setStoppedAt] = useState<number | null>(null);
  const [delayMs, setDelayMs] = useState<number | null>(null);
  const operation = snapshot?.operation?.destination.kind === "trial" && snapshot.operation.destination.trial_id === trialId ? snapshot.operation : null;
  const result = snapshot?.trialResult?.trialId === trialId ? snapshot.trialResult : null;
  const modelInstalled = models.some((item) => item.model === model && item.installed);
  useEffect(() => {
    if (!result || stoppedAt === null || delayMs !== null) return;
    const measured = performance.now() - stoppedAt;
    queueMicrotask(() => setDelayMs(measured));
  }, [delayMs, result, stoppedAt]);
  const start = () => dispatchVoiceAction({ action: "start", destination: { kind: "trial", trial_id: trialId }, context_generation: Date.now(), language: resolveVoiceLanguage(language, i18n.resolvedLanguage ?? i18n.language, languageMode) });
  return <section className="vset-trial">
    <h3>{t("voice.settings.try")}</h3>
    <div className="vset-trial-line">
      {operation ? <><VoiceSignal level={operation.level} tick={operation.captureMs} /><button type="button" className="btn btn-sm btn-secondary" onClick={() => { setStoppedAt(performance.now()); setDelayMs(null); void dispatchVoiceAction({ action: "validate", operation_id: operation.id }); }}>{t("voice.validate")}</button></>
        : <button type="button" className="btn btn-sm btn-secondary" disabled={!modelInstalled} onClick={() => void start()}>{t("voice.settings.startTrial")}</button>}
      <SettingsSelect value={model} options={models.flatMap((item) => item.installed && item.model ? [{ value: item.model, label: voiceModelName(item.model) }] : [])} disabled={Boolean(operation)} placement="above" fitLongestOption onChange={(value) => onModelChange(value as VoiceModel)} />
    </div>
    {result && <div className="vset-result"><p>{result.text}</p><small>{t("voice.settings.trialTiming", { compute: (result.computeMs / 1000).toFixed(1), delay: delayMs === null ? "—" : (delayMs / 1000).toFixed(1) })}</small></div>}
  </section>;
}

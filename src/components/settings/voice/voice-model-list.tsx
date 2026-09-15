import { useTranslation } from "react-i18next";
import { OperationProgressAction } from "@/components/ui/operation-progress-action";
import type { ModelDownloadState } from "@/types/model-download.generated";
import type { VoiceCatalogItem, VoiceModel } from "@/types/voice.generated";
import "./voice-settings.css";

const NAMES: Record<VoiceModel, string> = {
  "parakeet-tdt-v3": "Parakeet TDT v3",
  "cohere-transcribe": "Cohere Transcribe",
  "qwen3-asr06b": "Qwen3-ASR 0.6B",
};

export function voiceModelName(model: VoiceModel) { return NAMES[model]; }

interface Props {
  items: VoiceCatalogItem[];
  selected: VoiceModel;
  downloads: ModelDownloadState[];
  onSelect: (model: VoiceModel) => void;
  onInstall: (id: string) => void;
  onCancel: (id: string) => void;
  onRemove: (id: string) => void;
}

export function VoiceModelList({ items, selected, downloads, onSelect, onInstall, onCancel, onRemove }: Props) {
  const { t, i18n } = useTranslation();
  return <div className="vset-models">{items.map((item) => {
    const model = item.model;
    const download = downloads.find((entry) => entry.kind === "voice" && entry.modelId === item.id && ["queued", "running", "cancelling", "suspended"].includes(entry.status));
    return <article key={item.id} className="vset-model" data-selected={model ? selected === model : false}>
      <button type="button" className="vset-model-choice" disabled={!model} onClick={() => model && onSelect(model)}>
        <strong>{model ? voiceModelName(model) : "Silero VAD"}</strong>
        <span>{t("voice.settings.languageCount", { count: item.languages.length })} · {new Intl.NumberFormat(i18n.language, { style: "unit", unit: "megabyte", maximumFractionDigits: 0 }).format(item.downloadBytes / 1_000_000)}</span>
      </button>
      {download ? <OperationProgressAction compact percent={download.status === "suspended" ? null : download.percent} phaseLabel={t("voice.settings.installing")} cancelling={download.status === "cancelling"} canCancel={download.status !== "suspended"} cancelLabel={t("common.cancel")} cancellingLabel={t("voice.settings.cancelling")} onCancel={() => onCancel(download.id)} />
        : <button type="button" className="btn btn-sm btn-secondary" onClick={() => item.installed ? onRemove(item.id) : onInstall(item.id)}>{t(item.installed ? "voice.settings.remove" : "voice.settings.install")}</button>}
      <details><summary>{t("voice.settings.languages")}</summary><p>{item.languages.map((code) => new Intl.DisplayNames([i18n.language], { type: "language" }).of(code) ?? code).join(" · ")}</p></details>
    </article>;
  })}</div>;
}

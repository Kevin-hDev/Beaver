import { useTranslation } from "react-i18next";
import { ConfirmButton } from "@/components/settings/confirm-button";
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
  onResume: (id: string) => void;
  onRemove: (id: string) => void;
}

export function VoiceModelList({ items, selected, downloads, onSelect, onInstall, onResume, onRemove }: Props) {
  const { t, i18n } = useTranslation();
  return <div className="vset-models">{items.map((item) => {
    const model = item.model;
    if (!model) return null;
    const download = downloads.find((entry) => entry.kind === "voice" && entry.modelId === item.id && ["queued", "running", "cancelling", "suspended"].includes(entry.status));
    return <article key={item.id} className="vset-model" data-selected={selected === model}>
      <button type="button" className="vset-model-choice" onClick={() => onSelect(model)}>
        <strong>{voiceModelName(model)}</strong>
        <span>{t("voice.settings.languageCount", { count: item.languages.length })} · {new Intl.NumberFormat(i18n.language, { style: "unit", unit: "megabyte", maximumFractionDigits: 0 }).format(item.downloadBytes / 1_000_000)}</span>
        {item.speedMultiplier != null && <span>{t("voice.settings.lastSpeed", { speed: new Intl.NumberFormat(i18n.language, { maximumSignificantDigits: 2 }).format(item.speedMultiplier) })}</span>}
      </button>
      {download?.status === "suspended" ? <button type="button" className="btn btn-sm btn-secondary" onClick={() => onResume(download.id)}>{t("modelDownloads.resume")}</button>
        : download ? <span className="vset-language-mode">{t(download.status === "cancelling" ? "voice.settings.cancelling" : "voice.settings.installing")}</span>
        : item.installed
          ? <ConfirmButton className="btn btn-sm btn-destructive" label={t("voice.settings.remove")} confirmLabel={t("settings.confirm.deleteModel")} onConfirm={() => onRemove(item.id)} />
          : <button type="button" className="btn btn-sm btn-secondary" onClick={() => onInstall(item.id)}>{t("voice.settings.install")}</button>}
      <details><summary>{t("voice.settings.languages")}</summary><p>{item.languages.map((code) => new Intl.DisplayNames([i18n.language], { type: "language" }).of(code) ?? code).join(" · ")}</p></details>
    </article>;
  })}</div>;
}

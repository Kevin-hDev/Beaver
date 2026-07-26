import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import rehypeRaw from "rehype-raw";
import rehypeSanitize from "rehype-sanitize";
import { useTranslation } from "react-i18next";
import { SettingsCard } from "@/components/settings/settings-card";
import { TranslationControls } from "@/components/ollama/translation-controls";
import { ModelInstallBtn } from "./model-install-btn";
import type { ForecastModelDetails } from "./model-details-types";
import {
  getForecastModelReadiness,
  getForecastEngineKey,
  getModelCapabilities,
  type ForecastModelEntry,
  type ForecastProviderEntry,
} from "../forecast-model-meta";
import "../../ollama/ollama.css";
import "../../ollama/ollama-details.css";
import "../../ollama/model-profile.css";

interface ModelSpecsProps {
  model: ForecastModelEntry;
  provider: ForecastProviderEntry | null;
  onBack: () => void;
  onRefresh: () => void;
}

interface SpecRow {
  label: string;
  value: string;
  mono?: boolean;
}

export function ModelSpecs({ model, provider, onBack, onRefresh }: ModelSpecsProps) {
  const { t } = useTranslation();
  const [details, setDetails] = useState<ForecastModelDetails | null>(null);
  const [translation, setTranslation] = useState<{
    modelId: string;
    text: string | null;
    lang: string | null;
  } | null>(null);

  useEffect(() => {
    let cancelled = false;
    void invoke<ForecastModelDetails>("get_forecast_model_details", { modelId: model.id })
      .then((value) => {
        if (!cancelled) setDetails(value);
      })
      .catch(() => {
        if (!cancelled) setDetails(null);
      });
    return () => {
      cancelled = true;
    };
  }, [model.id]);

  const translated = translation?.modelId === model.id ? translation.text : null;
  const translatedLang = translation?.modelId === model.id ? translation.lang : null;
  const rows = useMemo(() => buildRows(t, model, provider, details), [t, model, provider, details]);

  return (
    <div className="mp-root">
      <div className="mp-inner">
        <button className="fs-back" onClick={onBack}>
          ← {t("settings.llm.back")}
        </button>
        <div className="mp-header">
          <h2 className="mp-title">{model.display_name}</h2>
          {!model.is_cloud && model.installable && (
            <ModelInstallBtn
              modelId={model.id}
              installed={model.installed}
              runtimeReady={model.runtime_ready === true}
              allowUninstall
              onDone={onRefresh}
            />
          )}
        </div>

        {details?.description_short && (
          <div className="mp-description">{details.description_short}</div>
        )}

        <SettingsCard>
          {rows.map((row, index) => (
            <div
              key={row.label}
              className={`mp-spec-row${index < rows.length - 1 ? " mp-spec-row-border" : ""}`}
            >
              <span className="mp-spec-label">{row.label}</span>
              <span className={row.mono ? "mp-spec-value-mono" : "mp-spec-value"}>
                {row.value}
              </span>
            </div>
          ))}
        </SettingsCard>

        {details?.description_long_markdown && (
          <div className="mp-translation-wrapper">
            <TranslationControls
              modelName={model.id}
              originalText={details.description_long_markdown}
              currentLang={translatedLang}
              onChange={(lang, text) => {
                setTranslation({ modelId: model.id, lang, text });
              }}
            />
          </div>
        )}
      </div>

      {details?.description_long_markdown && (
        <div className="ollama-readme mp-readme">
          <ReactMarkdown remarkPlugins={[remarkGfm]} rehypePlugins={[rehypeRaw, rehypeSanitize]}>
            {translated ?? details.description_long_markdown}
          </ReactMarkdown>
        </div>
      )}
    </div>
  );
}

function buildRows(
  t: (key: string) => string,
  model: ForecastModelEntry,
  provider: ForecastProviderEntry | null,
  details: ForecastModelDetails | null,
): SpecRow[] {
  const modelCaps = getModelCapabilities(model);
  const capabilities = [
    modelCaps.context ? t("forecast.models.capabilities.context") : null,
    modelCaps.futureContext ? t("forecast.models.capabilities.futureContext") : null,
    modelCaps.multivariate
      ? t("forecast.models.capabilities.multivariate")
      : modelCaps.multiSeries
        ? t("forecast.models.capabilities.multiSeries")
        : null,
    modelCaps.probabilistic ? t("forecast.models.capabilities.probabilistic") : null,
    modelCaps.backtesting ? t("forecast.models.capabilities.backtesting") : null,
  ].filter(Boolean).join(", ") || "—";

  return [
    { label: t("forecast.models.modelCapabilities"), value: capabilities },
    { label: t("forecast.models.diskSize"), value: model.is_cloud ? "—" : `${model.size_mb} MB`, mono: true },
    { label: t("forecast.models.parameters"), value: model.params, mono: true },
    { label: t("forecast.models.engine"), value: t(getForecastEngineKey(model)), mono: true },
    { label: t("forecast.models.cpuLabel"), value: model.is_cloud ? "—" : (model.cpu_supported ? t("forecast.models.supported") : t("forecast.models.unsupported")) },
    { label: t("forecast.models.gpuLabel"), value: model.is_cloud ? "—" : (model.gpu_supported ? t("forecast.models.supported") : t("forecast.models.unsupported")) },
    { label: t("forecast.models.horizonMax"), value: String(model.horizon_max), mono: true },
    { label: t("forecast.models.frequencies"), value: model.frequencies, mono: true },
    { label: t("forecast.models.status"), value: modelStatus(t, model, provider) },
    { label: t("forecast.models.licenseLabel"), value: details?.license || "—", mono: true },
    { label: t("forecast.models.libraryLabel"), value: details?.library_name || "—", mono: true },
    { label: t("forecast.models.pipelineLabel"), value: details?.pipeline_tag || "—", mono: true },
  ];
}

function modelStatus(
  t: (key: string) => string,
  model: ForecastModelEntry,
  provider: ForecastProviderEntry | null,
): string {
  if (model.is_cloud) {
    return provider?.configured
      ? t("forecast.models.cloud")
      : t("forecast.models.noKeyConfigured");
  }
  if (!model.installed) return t("forecast.models.uninstalled");
  const readiness = getForecastModelReadiness(model);
  if (readiness === "ready") return t("forecast.models.installed");
  return t(readiness === "update_required"
    ? "forecast.models.updateRequired"
    : "forecast.models.preparationRequired");
}

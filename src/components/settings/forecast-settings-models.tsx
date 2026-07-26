import { useMemo } from "react";
import { useTranslation } from "react-i18next";
import { EmptyState } from "@/components/ui/empty-state";
import { SettingsCard } from "./settings-card";
import { ModelSpecs } from "../forecast/model-browser/model-specs";
import { ModelInstallBtn } from "../forecast/model-browser/model-install-btn";
import {
  getForecastHardwareKey,
  getForecastModelSummaryKey,
  type ForecastModelEntry,
  type ForecastModelGroup,
  type ForecastProviderEntry,
} from "../forecast/forecast-model-meta";
import "./forecast-settings.css";

interface ForecastModelsViewProps {
  families: ForecastModelGroup[];
  providers: ForecastProviderEntry[];
  selectedFamily: ForecastModelGroup | null;
  selectedModel: ForecastModelEntry | null;
  onSelectFamily: (id: string) => void;
  onSelectModel: (id: string) => void;
  onBackToFamily: () => void;
  onBackToFamilies: () => void;
  onRefresh: () => void;
}

export function ForecastModelsView({
  families,
  providers,
  selectedFamily,
  selectedModel,
  onSelectFamily,
  onSelectModel,
  onBackToFamily,
  onBackToFamilies,
  onRefresh,
}: ForecastModelsViewProps) {
  const { t } = useTranslation();
  const provider = useMemo(
    () => (selectedModel
      ? providers.find((item) => item.id === selectedModel.provider_id) ?? null
      : null),
    [providers, selectedModel],
  );

  if (selectedModel) {
    return (
      <ModelSpecs
        model={selectedModel}
        provider={provider}
        onBack={onBackToFamily}
        onRefresh={onRefresh}
      />
    );
  }

  if (selectedFamily) {
    return (
      <FamilyModelList
        group={selectedFamily}
        onSelectModel={onSelectModel}
        onBack={onBackToFamilies}
        onRefresh={onRefresh}
      />
    );
  }

  if (families.length === 0) {
    return (
      <div className="fs-empty">
        <EmptyState message={t("forecast.models.noneAvailable")} />
      </div>
    );
  }

  return (
    <SettingsCard>
      {families.map((group) => (
        <div
          key={group.id}
          role="button"
          tabIndex={0}
          className="fs-family-row"
          onClick={() => onSelectFamily(group.id)}
          onKeyDown={(event) => {
            if (event.key === "Enter" || event.key === " ") {
              event.preventDefault();
              onSelectFamily(group.id);
            }
          }}
        >
          <span className="fs-family-name">{t(group.titleKey)}</span>
          <span className="fs-family-count">{group.models.length}</span>
        </div>
      ))}
    </SettingsCard>
  );
}

interface FamilyModelListProps {
  group: ForecastModelGroup;
  onSelectModel: (id: string) => void;
  onBack: () => void;
  onRefresh: () => void;
}

function FamilyModelList({ group, onSelectModel, onBack, onRefresh }: FamilyModelListProps) {
  const { t } = useTranslation();

  if (group.models.length === 0) {
    return (
      <div className="fs-empty">
        <EmptyState message={t("forecast.models.noneAvailable")} />
      </div>
    );
  }

  return (
    <>
      <button className="fs-back" onClick={onBack}>
        ← {t("settings.llm.back")}
      </button>
      <h3 className="fs-section-title">{t(group.titleKey)}</h3>
      <SettingsCard>
        {group.models.map((model, index) => (
          <ModelListRow
            key={model.id}
            model={model}
            isLast={index >= group.models.length - 1}
            onSelect={() => onSelectModel(model.id)}
            onRefresh={onRefresh}
          />
        ))}
      </SettingsCard>
    </>
  );
}

interface ModelListRowProps {
  model: ForecastModelEntry;
  isLast: boolean;
  onSelect: () => void;
  onRefresh: () => void;
}

function ModelListRow({ model, isLast, onSelect, onRefresh }: ModelListRowProps) {
  const { t } = useTranslation();
  const summaryKey = getForecastModelSummaryKey(model.id);
  const summary = t(summaryKey);
  const summaryText = summary === summaryKey ? model.display_name : summary;

  const meta = model.is_cloud
    ? `☁ ${model.provider_configured ? t("forecast.models.cloud") : t("forecast.models.noKeyConfigured")}`
    : `${model.params} · ${model.size_mb} MB${model.cpu_supported ? " · CPU ✓" : ""}${model.gpu_supported ? " · GPU ✓" : ""}`;

  return (
    <div
      role="button"
      tabIndex={0}
      className={`fs-model-row${isLast ? " fs-row-last" : ""}`}
      onClick={onSelect}
      onKeyDown={(event) => {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          onSelect();
        }
      }}
    >
      <div className="fs-model-info">
        <span className="fs-model-name">{model.display_name}</span>
        <span className="fs-model-summary">{summaryText}</span>
        <span className="fs-model-meta">{meta}</span>
        <span className="fs-model-meta">
          {t(getForecastHardwareKey(model))}
          {!model.is_cloud ? ` · ${t("forecast.models.ram")}: ~${model.ram_mb} MB` : ""}
        </span>
      </div>
      <div className="fs-model-actions">
        {model.is_cloud ? (
          <span className="fs-cloud-badge">☁</span>
        ) : model.installable ? (
          <ModelInstallBtn
            modelId={model.id}
            installed={model.installed}
            runtimeReady={model.runtime_ready === true}
            onDone={onRefresh}
          />
        ) : null}
      </div>
    </div>
  );
}

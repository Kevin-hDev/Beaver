import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { EmptyState } from "@/components/ui/empty-state";
import { showToast } from "@/lib/toast-emitter";
import i18n from "@/i18n";
import { SettingsCard } from "./settings-card";
import { SettingsRow } from "./settings-row";
import { SettingsSelect } from "./settings-select";
import type { ForecastModelEntry } from "../forecast/forecast-model-meta";
import type { ForecastConfigParam, ForecastModelConfig } from "../forecast/model-browser/forecast-config-types";
import {
  buildPayload,
  selectOptions,
  toDraft,
  valueToText,
} from "./forecast-config-helpers";
import { ForecastConfigDeleteAction } from "./forecast-config-delete-action";
import "./forecast-settings.css";

interface ForecastConfigViewProps {
  models: ForecastModelEntry[];
  selectedModel: ForecastModelEntry | null;
  onSelectModel: (id: string) => void;
  onModelsChanged?: () => void;
}

export function ForecastConfigView({
  models,
  selectedModel,
  onSelectModel,
  onModelsChanged = () => undefined,
}: ForecastConfigViewProps) {
  const { t } = useTranslation();
  const [config, setConfig] = useState<ForecastModelConfig | null>(null);
  const [draft, setDraft] = useState<Record<string, string>>({});
  const [editing, setEditing] = useState(false);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    if (!selectedModel) return;
    void invoke<ForecastModelConfig>("get_forecast_model_config", { modelId: selectedModel.id })
      .then((next) => {
        setEditing(false);
        setConfig(next);
        setDraft(toDraft(next.params));
      })
      .catch(() => setConfig(null));
  }, [selectedModel]);

  const modelOptions = useMemo(
    () => models.map((model) => ({ value: model.id, label: model.display_name })),
    [models],
  );

  if (!selectedModel) {
    return (
      <div className="fs-empty">
        <EmptyState message={t("forecast.modelConfig.noModel")} />
      </div>
    );
  }

  const handleSave = async () => {
    if (!config) return;
    setSaving(true);
    try {
      const payload = buildPayload(config.params, draft);
      const next = await invoke<ForecastModelConfig>("set_forecast_model_config", {
        modelId: selectedModel.id,
        values: payload,
      });
      setConfig(next);
      setDraft(toDraft(next.params));
      setEditing(false);
    } catch {
      showToast(i18n.t("errors.saveFailed"), "error");
    } finally {
      setSaving(false);
    }
  };

  const cancelEdit = () => {
    setDraft(toDraft(config?.params ?? []));
    setEditing(false);
  };

  return (
    <>
      <SettingsCard>
        <SettingsRow
          title={t("forecast.modelConfig.title")}
          description={t("forecast.modelConfig.sidebarTitle")}
        >
          <SettingsSelect
            options={modelOptions}
            value={selectedModel.id}
            onChange={onSelectModel}
          />
        </SettingsRow>
      </SettingsCard>

      <div className="fs-edit-actions">
        {!selectedModel.is_cloud && selectedModel.installed && (
          <ForecastConfigDeleteAction
            modelId={selectedModel.id}
            disabled={saving}
            onDeleted={onModelsChanged}
          />
        )}
        {editing && (
          <button className="ollama-btn" onClick={cancelEdit} disabled={saving}>
            {t("forecast.modelConfig.cancel")}
          </button>
        )}
        <button
          className={`ollama-btn ${editing ? "ollama-btn-primary" : ""}`}
          onClick={() => (editing ? void handleSave() : setEditing(true))}
          disabled={saving || !config}
        >
          {saving ? "..." : t(editing ? "forecast.modelConfig.save" : "forecast.modelConfig.edit")}
        </button>
      </div>

      {config && config.params.length > 0 && (
        <SettingsCard>
          {config.params.map((param) => (
            <ConfigParamRow
              key={param.id}
              param={param}
              value={draft[param.id] ?? ""}
              editing={editing}
              onChange={(value) => setDraft((prev) => ({ ...prev, [param.id]: value }))}
            />
          ))}
        </SettingsCard>
      )}

      {config && config.inherited.length > 0 && (
        <SettingsCard>
          {config.inherited.map((param) => (
            <ConfigParamRow
              key={param.id}
              param={param}
              value={valueToText(param.effective_value)}
              editing={false}
              inherited
              onChange={() => undefined}
            />
          ))}
        </SettingsCard>
      )}
    </>
  );
}

interface ConfigParamRowProps {
  param: ForecastConfigParam;
  value: string;
  editing: boolean;
  inherited?: boolean;
  onChange: (value: string) => void;
}

function ConfigParamRow({ param, value, editing, inherited, onChange }: ConfigParamRowProps) {
  const { t } = useTranslation();
  return (
    <SettingsRow
      title={t(param.label_key)}
      description={t(param.description_key)}
      className={inherited ? "fs-inherited" : undefined}
    >
      {param.kind === "boolean" || param.kind === "select" ? (
        <SettingsSelect
          value={value || valueToText(param.default_value)}
          onChange={onChange}
          options={selectOptions(param, t)}
          disabled={!editing}
        />
      ) : (
        <input
          className="pe-input pe-input-value fs-input"
          value={value}
          disabled={!editing}
          placeholder={valueToText(param.default_value)}
          onChange={(event) => onChange(event.target.value)}
        />
      )}
    </SettingsRow>
  );
}

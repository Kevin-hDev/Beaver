import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";
import type {
  ForecastScenario,
  ForecastScenarioAnalysis,
  ForecastScenarioCovariateAdjustment,
} from "./forecast-scenario-types";

type ScenarioMode = "percent_adjustment" | "context_adjustment";

interface UseForecastScenarioFormArgs {
  analysisId: string;
  onAnalysisChanged: () => void;
  setData: (value: ForecastScenarioAnalysis) => void;
  setActiveScenarioId: (value: string | null | ((current: string | null) => string | null)) => void;
  setError: (value: string | null) => void;
  t: (key: string) => string;
}

export function useForecastScenarioForm(args: UseForecastScenarioFormArgs) {
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [adjustment, setAdjustment] = useState("10");
  const [mode, setMode] = useState<ScenarioMode>("percent_adjustment");
  const [targetSeriesId, setTargetSeriesId] = useState("");
  const [contextAdjustments, setContextAdjustments] = useState<ForecastScenarioCovariateAdjustment[]>([]);
  const [editingScenarioId, setEditingScenarioId] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  const draftPreviewVisible = Boolean(
    editingScenarioId ||
      name.trim() ||
      description.trim() ||
      (mode === "context_adjustment" && contextAdjustments.length > 0),
  );

  async function submit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setSaving(true);
    args.setError(null);
    try {
      const request = {
        analysis_id: args.analysisId,
        name,
        description,
        scenario_kind: mode,
        adjustment_percent: Number(adjustment),
        covariate_adjustments: contextAdjustments,
        target_series_id: targetSeriesId || null,
      };
      const updated = editingScenarioId
        ? await invoke<ForecastScenarioAnalysis>("update_forecast_scenario", {
            request: { ...request, scenario_id: editingScenarioId },
          })
        : await invoke<ForecastScenarioAnalysis>("create_forecast_scenario", { request });
      args.setData(updated);
      args.setActiveScenarioId(
        editingScenarioId ?? updated.scenarios[updated.scenarios.length - 1]?.id ?? null,
      );
      reset();
      args.onAnalysisChanged();
    } catch {
      args.setError(
        editingScenarioId ? args.t("forecast.scenarios.updateFailed") : args.t("forecast.scenarios.saveFailed"),
      );
    } finally {
      setSaving(false);
    }
  }

  async function removeScenario(scenarioId: string) {
    setSaving(true);
    args.setError(null);
    try {
      const updated = await invoke<ForecastScenarioAnalysis>("delete_forecast_scenario", {
        analysisId: args.analysisId,
        scenarioId,
      });
      args.setData(updated);
      args.setActiveScenarioId((current) =>
        current === scenarioId ? updated.scenarios[0]?.id ?? null : current,
      );
      if (editingScenarioId === scenarioId) reset();
      args.onAnalysisChanged();
    } catch {
      args.setError(args.t("forecast.scenarios.deleteFailed"));
    } finally {
      setSaving(false);
    }
  }

  function editScenario(scenario: ForecastScenario) {
    args.setActiveScenarioId(scenario.id);
    setEditingScenarioId(scenario.id);
    setName(scenario.name);
    setDescription(scenario.description ?? "");
    setMode(
      scenario.params_modified?.kind === "context_adjustment"
        ? "context_adjustment"
        : "percent_adjustment",
    );
    setAdjustment(String(scenario.params_modified?.adjustment_percent ?? 0));
    setContextAdjustments(scenario.params_modified?.covariate_adjustments ?? []);
    setTargetSeriesId(scenario.params_modified?.target_series_id ?? "");
    args.setError(null);
  }

  function reset() {
    setEditingScenarioId(null);
    setName("");
    setDescription("");
    setAdjustment("10");
    setMode("percent_adjustment");
    setTargetSeriesId("");
    setContextAdjustments([]);
  }

  return {
    name,
    description,
    adjustment,
    mode,
    targetSeriesId,
    contextAdjustments,
    editingScenarioId,
    saving,
    draftPreviewVisible,
    setName,
    setDescription,
    setAdjustment,
    setMode,
    setTargetSeriesId,
    setContextAdjustments,
    submit,
    removeScenario,
    editScenario,
    reset,
  };
}

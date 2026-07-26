import type { ForecastDraftRow } from "../forecast-data";
import {
  buildForecastVariableLines,
  type ForecastVariableLine,
} from "../forecast-variable-lines";
import type {
  ForecastScenarioAnalysis,
  ForecastScenarioCovariateAdjustment,
} from "./forecast-scenario-types";

interface BuildScenarioVariableLinesArgs {
  analysis: ForecastScenarioAnalysis;
  historyValues: number[];
  forecastValues: number[];
  selectedSeries: string;
  activeScenarioId: string | null;
  draftAdjustments: ForecastScenarioCovariateAdjustment[];
  draftTargetSeriesId: string | null;
  showDraftPreview: boolean;
}

export function buildScenarioVariableLines(
  args: BuildScenarioVariableLinesArgs,
): ForecastVariableLine[] {
  const source = resolveScenarioSource(args);
  if (!source || !args.analysis.input_data.rows?.length || !args.analysis.target_column) {
    return [];
  }

  const rows = args.analysis.input_data.rows.map((row) =>
    applyAdjustmentsToRow(
      row,
      source.adjustments,
      args.analysis.target_column ?? "",
      args.analysis.input_data.series_column ?? null,
      source.targetSeriesId,
    ),
  );

  return buildForecastVariableLines({
    rows,
    covariates: source.adjustments.map((item) => item.column),
    targetColumn: args.analysis.target_column,
    seriesColumn: args.analysis.input_data.series_column ?? null,
    selectedSeries: args.selectedSeries,
    historyValues: args.historyValues,
    forecastValues: args.forecastValues,
  });
}

function resolveScenarioSource(args: BuildScenarioVariableLinesArgs) {
  if (args.showDraftPreview && args.draftAdjustments.length > 0) {
    return {
      adjustments: args.draftAdjustments,
      targetSeriesId: args.draftTargetSeriesId,
    };
  }
  const scenario = args.analysis.scenarios.find((item) => item.id === args.activeScenarioId);
  const adjustments = scenario?.params_modified?.covariate_adjustments ?? [];
  if (!adjustments.length) return null;
  return {
    adjustments,
    targetSeriesId: scenario?.params_modified?.target_series_id ?? null,
  };
}

function applyAdjustmentsToRow(
  row: ForecastDraftRow,
  adjustments: ForecastScenarioCovariateAdjustment[],
  targetColumn: string,
  seriesColumn: string | null,
  targetSeriesId: string | null,
) {
  if (hasTargetValue(row[targetColumn])) return row;
  if (!matchesSeries(row, seriesColumn, targetSeriesId)) return row;

  const nextRow = { ...row };
  for (const adjustment of adjustments) {
    const current = readNumber(nextRow[adjustment.column]);
    if (current == null) continue;
    nextRow[adjustment.column] =
      adjustment.mode === "absolute"
        ? adjustment.value
        : current * (1 + adjustment.value / 100);
  }
  return nextRow;
}

function hasTargetValue(value: unknown) {
  if (value == null) return false;
  if (typeof value === "string") return value.trim().length > 0;
  if (typeof value === "number") return Number.isFinite(value);
  if (typeof value === "boolean") return true;
  return true;
}

function matchesSeries(
  row: ForecastDraftRow,
  seriesColumn: string | null,
  targetSeriesId: string | null,
) {
  if (!seriesColumn || !targetSeriesId) return true;
  const value = row[seriesColumn];
  if (typeof value === "string") return value.trim() === targetSeriesId;
  if (typeof value === "number" || typeof value === "boolean") {
    return String(value) === targetSeriesId;
  }
  return false;
}

function readNumber(value: unknown): number | null {
  if (typeof value === "number") return Number.isFinite(value) ? value : null;
  if (typeof value === "string") {
    const parsed = Number(value.trim().replace(",", "."));
    return Number.isFinite(parsed) ? parsed : null;
  }
  if (typeof value === "boolean") return value ? 1 : 0;
  return null;
}

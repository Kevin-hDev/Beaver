import type { TFunction } from "i18next";
import type { ModelBacktestResult } from "./forecast-evaluation-types";
import {
  baselineTranslationKey,
  formatCoverage,
  formatDuration,
  formatMetric,
  rankedResults,
} from "./forecast-evaluation-utils";

interface ForecastEvaluationTableProps {
  results: ModelBacktestResult[];
  currentModel: string;
  t: TFunction;
}

export function ForecastEvaluationTable({
  results,
  currentModel,
  t,
}: ForecastEvaluationTableProps) {
  return (
    <div className="fcwe-table" role="table">
      <div className="fcwe-row fcwe-table-head" role="row">
        <span>{t("forecast.workbench.evaluation.model")}</span>
        <span>MASE</span>
        <span>sMAPE</span>
        <span>MAE</span>
        <span>{t("forecast.workbench.evaluation.coverage")}</span>
        <span>{t("forecast.workbench.evaluation.duration")}</span>
        <span>{t("forecast.workbench.evaluation.status")}</span>
      </div>
      {rankedResults(results).map((result) => (
        <EvaluationRow
          key={`${result.kind}:${result.model_id}`}
          result={result}
          currentModel={currentModel}
          t={t}
        />
      ))}
    </div>
  );
}

function EvaluationRow({
  result,
  currentModel,
  t,
}: {
  result: ModelBacktestResult;
  currentModel: string;
  t: TFunction;
}) {
  const baselineKey = baselineTranslationKey(result.model_id);
  const label = baselineKey ? t(baselineKey) : result.model_id;
  return (
    <div className="fcwe-row" role="row">
      <div className="fcwe-model" role="cell">
        <span className="fcwe-rank">{result.rank ?? "—"}</span>
        <span>
          <strong>{label}</strong>
          <small>
            {result.kind === "baseline"
              ? t("forecast.workbench.evaluation.baseline")
              : t("forecast.workbench.evaluation.modelKind")}
            {result.model_id === currentModel
              ? ` · ${t("forecast.workbench.evaluation.current")}`
              : ""}
          </small>
        </span>
      </div>
      <MetricCell label="MASE" value={formatMetric(result.metrics?.mase)} />
      <MetricCell
        label="sMAPE"
        value={result.metrics ? `${formatMetric(result.metrics.smape)}%` : "—"}
      />
      <MetricCell label="MAE" value={formatMetric(result.metrics?.mae)} />
      <MetricCell
        label={t("forecast.workbench.evaluation.coverage")}
        value={result.calibration
          ? `${formatCoverage(result.calibration.measured_coverage)} / ${formatCoverage(result.calibration.theoretical_coverage)}`
          : "—"}
      />
      <MetricCell
        label={t("forecast.workbench.evaluation.duration")}
        value={formatDuration(result.duration_ms)}
      />
      <div className="fcwe-status" role="cell">
        {statusLabel(result, t)}
      </div>
    </div>
  );
}

function MetricCell({ label, value }: { label: string; value: string }) {
  return (
    <div className="fcwe-metric" role="cell">
      <small>{label}</small>
      <span>{value}</span>
    </div>
  );
}

function statusLabel(result: ModelBacktestResult, t: TFunction) {
  if (result.warning) {
    return t(`forecast.workbench.evaluation.warnings.${warningCategory(result.warning)}`);
  }
  if (result.kind === "baseline") return t("forecast.workbench.evaluation.reference");
  return result.beats_best_baseline
    ? t("forecast.workbench.evaluation.beatsBaseline")
    : t("forecast.workbench.evaluation.missesBaseline");
}

function warningCategory(code: string) {
  if (["insufficient_history", "seasonal_history_too_short", "ets_history_too_short"]
    .includes(code)) return "history";
  if (["cloud_not_configured", "cloud_not_allowed"].includes(code)) return "cloud";
  if (code === "model_not_installed") return "notInstalled";
  if (code === "resources_unavailable") return "resources";
  if (["model_start_failed", "window_failed", "incomplete_predictions", "missing_series"]
    .includes(code)) return "execution";
  return "unavailable";
}

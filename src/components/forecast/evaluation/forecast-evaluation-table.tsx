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
    <div className="data-table-scroll fcwe-table">
      <table className="data-table fcwe-grid">
        <thead>
          <tr>
            <th>{t("forecast.workbench.evaluation.model")}</th>
            <th>MASE</th>
            <th>sMAPE</th>
            <th>MAE</th>
            <th>{t("forecast.workbench.evaluation.coverage")}</th>
            <th>{t("forecast.workbench.evaluation.duration")}</th>
            <th>{t("forecast.workbench.evaluation.status")}</th>
          </tr>
        </thead>
        <tbody>
          {rankedResults(results).map((result) => (
            <EvaluationRow
              key={`${result.kind}:${result.model_id}`}
              result={result}
              currentModel={currentModel}
              t={t}
            />
          ))}
        </tbody>
      </table>
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
    <tr>
      <td className="fcwe-model">
        <div className="fcwe-model-inner">
          <span className="fcwe-rank">{result.rank ?? "—"}</span>
          <span className="fcwe-model-name">
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
      </td>
      <td className="fcwe-metric">{formatMetric(result.metrics?.mase)}</td>
      <td className="fcwe-metric">
        {result.metrics ? `${formatMetric(result.metrics.smape)}%` : "—"}
      </td>
      <td className="fcwe-metric">{formatMetric(result.metrics?.mae)}</td>
      <td className="fcwe-metric">
        {result.calibration
          ? `${formatCoverage(result.calibration.measured_coverage)} / ${formatCoverage(result.calibration.theoretical_coverage)}`
          : "—"}
      </td>
      <td className="fcwe-metric">{formatDuration(result.duration_ms)}</td>
      <td className="fcwe-status">{statusLabel(result, t)}</td>
    </tr>
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

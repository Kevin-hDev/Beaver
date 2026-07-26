import { useState } from "react";
import { useTranslation } from "react-i18next";
import { ForecastChartCard } from "../charts/forecast-chart-card";
import { ForecastEvaluationTable } from "./forecast-evaluation-table";
import { ForecastReliabilityChart } from "./forecast-reliability-chart";
import { buildReliabilityBars } from "./forecast-reliability-data";
import { baselineTranslationKey } from "./forecast-evaluation-utils";
import { useForecastEvaluation } from "./use-forecast-evaluation";
import "./forecast-evaluation.css";

interface ForecastEvaluationViewProps {
  analysisId: string;
  mode: "evaluation" | "comparison";
}

export function ForecastEvaluationView({ analysisId, mode }: ForecastEvaluationViewProps) {
  const { t } = useTranslation();
  const {
    analysis, loading, running, loadFailed, runFailed, run,
    ensembleRunning, ensembleFailed, createEnsemble,
  } =
    useForecastEvaluation(analysisId);
  const [reliabilityResize, setReliabilityResize] = useState(0);
  const evaluation = analysis?.evaluation;
  const successfulModels = evaluation?.results.filter((result) =>
    result.kind === "model" && result.metrics !== null) ?? [];
  const reliabilityBars = evaluation && analysis
    ? buildReliabilityBars(evaluation.results, analysis.model)
    : [];

  if (loading) {
    return <div className="fcwe-state">{t("forecast.workbench.evaluation.loading")}</div>;
  }
  if (loadFailed || !analysis) {
    return <div className="fcwe-state fcwe-state-error">{t("forecast.workbench.evaluation.loadFailed")}</div>;
  }
  return (
    <div className="fcwe-root">
      {runFailed ? (
        <p className="fcwe-error" role="alert">
          {t("forecast.workbench.evaluation.runFailed")}
        </p>
      ) : null}
      {ensembleFailed ? (
        <p className="fcwe-error" role="alert">
          {t("forecast.workbench.evaluation.ensembleFailed")}
        </p>
      ) : null}
      <div className="fcwe-toolbar">
        <div className="fcwe-summary">
          {evaluation ? (
            <>
              <strong>{t("forecast.workbench.evaluation.rollingBacktest")}</strong>
              <span>
                {t("forecast.workbench.evaluation.summary", {
                  windows: evaluation.windows,
                  horizon: evaluation.horizon,
                })}
              </span>
            </>
          ) : (
            <strong>{t("forecast.workbench.evaluation.notRun")}</strong>
          )}
        </div>
        {mode === "evaluation" ? (
          <button className="fcwe-run" type="button" disabled={running} onClick={() => void run()}>
            {running
              ? t("forecast.workbench.evaluation.running")
              : t("forecast.workbench.evaluation.run")}
          </button>
        ) : successfulModels.length >= 2 ? (
          <button
            className="fcwe-run"
            type="button"
            disabled={ensembleRunning}
            onClick={() => void createEnsemble()}
          >
            {ensembleRunning
              ? t("forecast.workbench.evaluation.ensembleRunning")
              : t("forecast.workbench.evaluation.createEnsemble")}
          </button>
        ) : null}
      </div>
      {analysis.ensemble ? (
        <p className="fcwe-ensemble-status">
          {t("forecast.workbench.evaluation.ensembleReady", {
            count: analysis.ensemble.members.length,
          })}
        </p>
      ) : null}
      {evaluation?.warning ? (
        <p className="fcwe-warning">
          {t(`forecast.workbench.evaluation.planWarnings.${evaluation.warning}`, {
            defaultValue: t("forecast.workbench.evaluation.planWarnings.unavailable"),
          })}
        </p>
      ) : null}
      {reliabilityBars.length ? (
        <ForecastChartCard
          title={t("forecast.chartCard.reliability")}
          onExpanded={() => setReliabilityResize((value) => value + 1)}
        >
          <ForecastReliabilityChart
            results={evaluation?.results ?? []}
            currentModel={analysis.model}
            resolveModel={(modelId) => {
              const key = baselineTranslationKey(modelId);
              return key ? t(key) : modelId;
            }}
            resizeSignal={reliabilityResize}
          />
        </ForecastChartCard>
      ) : null}
      {evaluation?.results.length ? (
        <ForecastEvaluationTable
          results={evaluation.results}
          currentModel={analysis.model}
          t={t}
        />
      ) : (
        <div className="fcwe-empty">
          <strong>{t("forecast.workbench.evaluation.emptyTitle")}</strong>
          <p>
            {t(
              mode === "evaluation"
                ? "forecast.workbench.evaluation.emptyEvaluation"
                : "forecast.workbench.evaluation.emptyComparison",
            )}
          </p>
        </div>
      )}
    </div>
  );
}

import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { useForecastResult } from "../use-forecast-result";
import { inferMetricMeta } from "../forecast-view-format";
import { ForecastScenarioMenuSelect } from "./forecast-scenario-menu-select";
import { ForecastAnalysisAccordion } from "./forecast-analysis-accordion";
import { ForecastAnalysisCardGrid } from "./forecast-analysis-card-grid";
import { ForecastAnalysisEventList } from "./forecast-analysis-event-list";
import { ForecastAnalysisVariableList } from "./forecast-analysis-variable-list";
import {
  buildHighlightEvents,
  buildTrendCards,
  buildUncertaintyCards,
  filterAnalysisQuantiles,
  filterAnalysisPoints,
} from "./forecast-analysis-utils";
import {
  buildAdvancedVariableInsights,
  buildDecompositionCards,
  buildDriftCards,
  buildResidualAnomalyEvents,
} from "./forecast-advanced-analysis-utils";
import type { ForecastAnalysisData } from "./forecast-analysis-types";
import "../forecast-sections.css";
import "./forecast-scenario-menu.css";
import "./forecast-analysis.css";
import "./forecast-analysis-lists.css";

interface ForecastAnalysisProps {
  analysisId: string;
}

const DEFAULT_OPEN = {
  trend: true,
  decomposition: false,
  uncertainty: true,
  highlights: true,
  anomalies: false,
  drift: false,
  variables: false,
};

export function ForecastAnalysis({ analysisId }: ForecastAnalysisProps) {
  const { t, i18n } = useTranslation();
  const { data, error } = useForecastResult<ForecastAnalysisData>(
    analysisId,
    t("forecast.analysis.loadFailed"),
  );
  const [selectedSeries, setSelectedSeries] = useState("");
  const [open, setOpen] = useState(DEFAULT_OPEN);

  const computed = useMemo(() => {
    if (!data) return null;
    const seriesId = selectedSeries && data.input_data.series_ids?.includes(selectedSeries)
      ? selectedSeries
      : data.input_data.series_ids?.[0] ?? "";
    const metric = inferMetricMeta(i18n.language, data.target_column, data.name);
    const predictions = filterAnalysisPoints(data.predictions, seriesId);
    const quantiles = filterAnalysisQuantiles(data.predictions, seriesId, data.quantiles);
    return {
      seriesId,
      metric,
      predictions,
      trendCards: buildTrendCards(predictions, i18n.language, metric, t),
      decompositionCards: buildDecompositionCards(data.advanced_analytics, seriesId, t),
      uncertaintyCards: buildUncertaintyCards(quantiles, predictions, i18n.language, metric),
      highlights: buildHighlightEvents(predictions, data.input_summary.end, data.frequency, i18n.language, metric, t),
      anomalies: buildResidualAnomalyEvents(data.advanced_analytics, seriesId, i18n.language, metric, t),
      driftCards: buildDriftCards(data.advanced_analytics, seriesId, t),
      variables: buildAdvancedVariableInsights(data.advanced_analytics, t),
    };
  }, [data, i18n.language, selectedSeries, t]);

  if (error) return <div className="fc-error">{error}</div>;
  if (!data || !computed) return <div className="fc-loading"><div className="fc-skeleton" /></div>;

  const seriesIds = data.input_data.series_ids ?? [];
  return (
    <div className="fca-root">
      {seriesIds.length > 1 && (
        <div className="fca-toolbar">
          <div className="fca-series">
            <ForecastScenarioMenuSelect
              value={computed.seriesId}
              options={seriesIds.map((seriesId) => ({ value: seriesId, label: seriesId }))}
              onChange={setSelectedSeries}
              placeholder={t("forecast.analysis.series")}
            />
          </div>
        </div>
      )}
      <div className="fca-scroll">
        <ForecastAnalysisAccordion
          title={t("forecast.analysis.trend")}
          subtitle={t("forecast.analysis.trendSub")}
          open={open.trend}
          onToggle={() => setOpen((current) => ({ ...current, trend: !current.trend }))}
        >
          <ForecastAnalysisCardGrid cards={computed.trendCards} t={t} />
        </ForecastAnalysisAccordion>
        <ForecastAnalysisAccordion
          title={t("forecast.analysis.decomposition")}
          subtitle={t("forecast.analysis.decompositionSub")}
          open={open.decomposition}
          onToggle={() => setOpen((current) => ({ ...current, decomposition: !current.decomposition }))}
        >
          {computed.decompositionCards.length ? (
            <ForecastAnalysisCardGrid cards={computed.decompositionCards} t={t} />
          ) : (
            <p className="fca-empty-line">{t("forecast.analysis.noDecomposition")}</p>
          )}
        </ForecastAnalysisAccordion>
        <ForecastAnalysisAccordion
          title={t("forecast.analysis.uncertainty")}
          subtitle={t("forecast.analysis.uncertaintySub")}
          open={open.uncertainty}
          onToggle={() => setOpen((current) => ({ ...current, uncertainty: !current.uncertainty }))}
        >
          <ForecastAnalysisCardGrid cards={computed.uncertaintyCards} t={t} />
        </ForecastAnalysisAccordion>
        <ForecastAnalysisAccordion
          title={t("forecast.analysis.highlights")}
          subtitle={t("forecast.analysis.highlightsSub")}
          open={open.highlights}
          onToggle={() => setOpen((current) => ({ ...current, highlights: !current.highlights }))}
        >
          <ForecastAnalysisEventList events={computed.highlights} emptyKey="forecast.analysis.noHighlights" t={t} />
        </ForecastAnalysisAccordion>
        <ForecastAnalysisAccordion
          title={t("forecast.analysis.anomalies")}
          subtitle={t("forecast.analysis.anomaliesSub")}
          open={open.anomalies}
          onToggle={() => setOpen((current) => ({ ...current, anomalies: !current.anomalies }))}
        >
          <ForecastAnalysisEventList events={computed.anomalies} emptyKey="forecast.analysis.noAnomalies" t={t} />
        </ForecastAnalysisAccordion>
        <ForecastAnalysisAccordion
          title={t("forecast.analysis.drift")}
          subtitle={t("forecast.analysis.driftSub")}
          open={open.drift}
          onToggle={() => setOpen((current) => ({ ...current, drift: !current.drift }))}
        >
          {computed.driftCards.length ? (
            <ForecastAnalysisCardGrid cards={computed.driftCards} t={t} />
          ) : (
            <p className="fca-empty-line">{t("forecast.analysis.noDriftData")}</p>
          )}
        </ForecastAnalysisAccordion>
        <ForecastAnalysisAccordion
          title={t("forecast.analysis.variables")}
          subtitle={t("forecast.analysis.variablesSub")}
          open={open.variables}
          onToggle={() => setOpen((current) => ({ ...current, variables: !current.variables }))}
        >
          <ForecastAnalysisVariableList variables={computed.variables} t={t} />
        </ForecastAnalysisAccordion>
      </div>
    </div>
  );
}

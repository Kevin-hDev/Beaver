import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import * as echarts from "echarts/core";
import type { EChartsType } from "echarts/core";
import { LineChart } from "echarts/charts";
import {
  GridComponent,
  MarkLineComponent,
  TooltipComponent,
} from "echarts/components";
import { CanvasRenderer } from "echarts/renderers";
import { useTranslation } from "react-i18next";
import {
  filterForecastSeriesData,
  type ForecastViewResult,
} from "../sections/forecast-view-data";
import { useForecastResult } from "../use-forecast-result";
import { buildForecastChartPalette } from "./forecast-chart-palette";
import {
  buildSeasonalityModel,
  defaultVisibleYears,
  toggleVisibleYear,
  type SeasonalityModel,
} from "./forecast-seasonality-data";
import {
  buildSeasonalityOption,
  seasonalityChipToken,
} from "./forecast-seasonality-option";
import { useForecastThemeRevision } from "./use-forecast-theme-revision";
import "./forecast-seasonality-chart.css";

echarts.use([
  CanvasRenderer,
  LineChart,
  GridComponent,
  TooltipComponent,
  MarkLineComponent,
]);

interface ForecastSeasonalityChartProps {
  analysisId: string;
  /** Active series; falls back to the first series when absent/unknown. */
  seriesId?: string;
  /** Bump to request a resize after the card expand transition. */
  resizeSignal?: number;
}

export function ForecastSeasonalityChart({
  analysisId,
  seriesId,
  resizeSignal = 0,
}: ForecastSeasonalityChartProps) {
  const { t, i18n } = useTranslation();
  const { data, error } = useForecastResult<ForecastViewResult>(
    analysisId,
    t("forecast.noAnalysis"),
  );
  const containerRef = useRef<HTMLDivElement | null>(null);
  const chartRef = useRef<EChartsType | null>(null);
  const lastKeyRef = useRef("");
  const themeRevision = useForecastThemeRevision();
  const [toggled, setToggled] = useState<{ signature: string; visible: number[] } | null>(null);

  const model = useMemo(() => {
    if (!data) return null;
    const ids = data.input_data.series_ids ?? [];
    const active = seriesId && ids.includes(seriesId) ? seriesId : ids[0] ?? "";
    const filtered = filterForecastSeriesData(data, active, []);
    return buildSeasonalityModel(filtered.history, i18n.language, data.frequency);
  }, [data, seriesId, i18n.language]);
  const signature = model?.years.map((year) => year.year).join(",") ?? "";
  const visibleYears = useMemo(
    () =>
      model
        ? toggled && toggled.signature === signature
          ? toggled.visible
          : defaultVisibleYears(model.years)
        : [],
    [model, toggled, signature],
  );

  const applyOption = useCallback((replace: boolean) => {
    if (!chartRef.current || !containerRef.current || !model) return;
    const root = getComputedStyle(containerRef.current);
    chartRef.current.setOption(
      buildSeasonalityOption(model, visibleYears, buildForecastChartPalette(root), {
        indexBase: t("forecast.companion.indexBase"),
      }),
      replace,
    );
  }, [model, visibleYears, t]);

  // The mount-only init effect reads the latest apply through a ref
  // (written inside an effect, never during render).
  const applyRef = useRef(applyOption);
  useEffect(() => {
    applyRef.current = applyOption;
  }, [applyOption]);

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return undefined;
    const ensureChart = () => {
      if (chartRef.current) return;
      if (container.clientWidth <= 0 || container.clientHeight <= 0) return;
      chartRef.current = echarts.init(container, undefined, { renderer: "canvas" });
      applyRef.current(true);
    };
    const observer = new ResizeObserver(() => {
      if (!chartRef.current) ensureChart();
      else chartRef.current.resize();
    });
    observer.observe(container);
    ensureChart();
    return () => {
      observer.disconnect();
      chartRef.current?.dispose();
      chartRef.current = null;
    };
  }, []);

  useEffect(() => {
    const key = `${signature}|${visibleYears.join(",")}|${themeRevision}`;
    const replace = lastKeyRef.current !== key;
    lastKeyRef.current = key;
    applyOption(replace);
  }, [signature, visibleYears, themeRevision, applyOption]);

  useEffect(() => {
    if (resizeSignal > 0) chartRef.current?.resize();
  }, [resizeSignal]);

  const setVisible = (next: number[]) => setToggled({ signature, visible: next });
  const highlight = (year: SeasonalityModel["years"][number], on: boolean) => {
    chartRef.current?.dispatchAction({
      type: on ? "highlight" : "downplay",
      seriesName: String(year.year),
    });
  };

  return (
    <div className="fcwf-companion fcwf-companion-seasonality">
      {model ? (
        <div className="fcse-legend">
          {model.years.map((year) => {
            const active = visibleYears.includes(year.year);
            return (
              <button
                key={year.year}
                type="button"
                className={`fcse-chip ${active ? "is-active" : ""}`}
                aria-pressed={active}
                onClick={() => setVisible(toggleVisibleYear(visibleYears, year.year))}
                onMouseEnter={() => highlight(year, true)}
                onMouseLeave={() => highlight(year, false)}
              >
                <span
                  className="fcse-chip-dot"
                  style={{ background: seasonalityChipToken(year, model.years) }}
                  aria-hidden="true"
                />
                {year.year}
              </button>
            );
          })}
          <button
            type="button"
            className="fcse-action"
            onClick={() => setVisible(model.years.map((year) => year.year))}
          >
            {t("forecast.companion.showAll")}
          </button>
          <button type="button" className="fcse-action" onClick={() => setVisible([])}>
            {t("forecast.companion.showNone")}
          </button>
        </div>
      ) : null}
      <div ref={containerRef} className="fcwf-companion-chart" />
      {error ? (
        <div className="fcwf-companion-empty">{error}</div>
      ) : !model ? (
        <div className="fcwf-companion-empty">
          {t("forecast.companion.insufficientData")}
        </div>
      ) : null}
    </div>
  );
}

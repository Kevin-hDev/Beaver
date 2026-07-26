import { useCallback, useEffect, useMemo, useRef } from "react";
import * as echarts from "echarts/core";
import type { EChartsType } from "echarts/core";
import { BarChart } from "echarts/charts";
import {
  GridComponent,
  MarkLineComponent,
  TooltipComponent,
} from "echarts/components";
import { CanvasRenderer } from "echarts/renderers";
import { useTranslation } from "react-i18next";
import { buildForecastChartPalette } from "../charts/forecast-chart-palette";
import { useForecastThemeRevision } from "../charts/use-forecast-theme-revision";
import type { ModelBacktestResult } from "./forecast-evaluation-types";
import {
  buildReliabilityBars,
  reliabilityMean,
  type ReliabilityBar,
} from "./forecast-reliability-data";
import { buildReliabilityOption } from "./forecast-reliability-option";

echarts.use([
  CanvasRenderer,
  BarChart,
  GridComponent,
  TooltipComponent,
  MarkLineComponent,
]);

interface ForecastReliabilityChartProps {
  results: ModelBacktestResult[];
  currentModel: string;
  resolveModel: (modelId: string) => string;
  /** Bump to request a resize after the card expand transition. */
  resizeSignal?: number;
}

export function ForecastReliabilityChart({
  results,
  currentModel,
  resolveModel,
  resizeSignal = 0,
}: ForecastReliabilityChartProps) {
  const { t } = useTranslation();
  const containerRef = useRef<HTMLDivElement | null>(null);
  const chartRef = useRef<EChartsType | null>(null);
  const lastBarsRef = useRef<ReliabilityBar[] | null>(null);
  const themeRevision = useForecastThemeRevision();
  const lastThemeRevisionRef = useRef(themeRevision);

  const bars = useMemo(
    () => buildReliabilityBars(results, currentModel),
    [results, currentModel],
  );
  const mean = useMemo(() => reliabilityMean(bars), [bars]);
  const labels = useMemo(
    () => ({ mean: t("forecast.companion.mean"), resolveModel }),
    [t, resolveModel],
  );

  const applyOption = useCallback((replace: boolean) => {
    if (!chartRef.current || !containerRef.current || !bars.length) return;
    const root = getComputedStyle(containerRef.current);
    chartRef.current.setOption(
      buildReliabilityOption(bars, mean, buildForecastChartPalette(root), labels),
      replace,
    );
  }, [bars, mean, labels]);

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
    const replace =
      lastBarsRef.current !== bars ||
      lastThemeRevisionRef.current !== themeRevision;
    lastBarsRef.current = bars;
    lastThemeRevisionRef.current = themeRevision;
    applyOption(replace);
  }, [bars, themeRevision, applyOption]);

  useEffect(() => {
    if (resizeSignal > 0) chartRef.current?.resize();
  }, [resizeSignal]);

  return (
    <div className="fcwe-reliability">
      <div ref={containerRef} className="fcwe-reliability-chart" />
    </div>
  );
}

import type { EChartsOption } from "echarts";
import type { ForecastChartPalette } from "../charts/forecast-chart-types";
import type { ReliabilityBar } from "./forecast-reliability-data";

interface ReliabilityOptionLabels {
  mean: string;
  resolveModel: (modelId: string) => string;
}

export function buildReliabilityOption(
  bars: ReliabilityBar[],
  mean: number | null,
  palette: ForecastChartPalette,
  labels: ReliabilityOptionLabels,
): EChartsOption {
  return {
    animation: false,
    grid: { left: 12, right: 16, top: 34, bottom: 24, containLabel: true },
    tooltip: {
      trigger: "axis",
      backgroundColor: palette.tooltipBg,
      borderColor: palette.edge,
      borderWidth: 1,
      padding: [7, 10],
      textStyle: { color: palette.tooltipText, fontFamily: "var(--font-sans)", fontSize: 12, lineHeight: 17 },
      confine: true,
    },
    xAxis: {
      type: "category",
      data: bars.map((bar) => labels.resolveModel(bar.modelId)),
      axisLine: { lineStyle: { color: palette.edge } },
      axisTick: { show: false },
      axisLabel: {
        color: palette.inkMuted,
        fontSize: 11,
        interval: 0,
        rotate: bars.length > 6 ? 30 : 0,
      },
    },
    yAxis: {
      type: "value",
      name: "sMAPE (%)",
      nameTextStyle: { color: palette.inkMuted, fontSize: 11, align: "left" },
      splitLine: { lineStyle: { color: palette.edge, opacity: 0.5 } },
      axisLine: { show: false },
      axisTick: { show: false },
      axisLabel: { color: palette.inkMuted, fontSize: 11 },
    },
    series: [{
      type: "bar" as const,
      data: bars.map((bar) => ({
        value: roundMetric(bar.value),
        itemStyle: {
          color: bar.current
            ? palette.linePredict
            : bar.kind === "baseline"
              ? palette.separator
              : palette.scenarios[0] || palette.inkMuted,
          borderRadius: [3, 3, 0, 0],
        },
      })),
      barMaxWidth: 42,
      markLine: mean != null ? {
        symbol: ["none", "none"],
        silent: true,
        label: {
          show: true,
          formatter: `${labels.mean} ${roundMetric(mean)}`,
          position: "insideEndTop" as const,
          color: palette.inkMuted,
          fontSize: 11,
        },
        lineStyle: { color: palette.separator, type: "dashed" as const, width: 1 },
        data: [{ yAxis: mean }],
      } : undefined,
    }],
  };
}

function roundMetric(value: number): number {
  return Math.round(value * 100) / 100;
}

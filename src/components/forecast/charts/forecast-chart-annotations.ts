import type { ForecastLayerState } from "../forecast-layer-matrix";
import type {
  ForecastChartPalette,
  TimelineEntry,
} from "./forecast-chart-types";

export function buildAnnotationSeries(
  timeline: TimelineEntry[],
  palette: ForecastChartPalette,
  labels: { annotationUser: string; annotationLlm: string },
  layers: ForecastLayerState,
  activeAnnotationId: string | null,
) {
  return [
    annotationSeries(
      "annotation-user",
      labels.annotationUser,
      timeline,
      palette.annotationUser,
      "user",
      layers,
      activeAnnotationId,
    ),
    annotationSeries(
      "annotation-llm",
      labels.annotationLlm,
      timeline,
      palette.annotationLlm,
      "llm",
      layers,
      activeAnnotationId,
    ),
  ];
}

function annotationSeries(
  id: string,
  name: string,
  timeline: TimelineEntry[],
  color: string,
  source: "user" | "llm",
  layers: ForecastLayerState,
  activeAnnotationId: string | null,
) {
  return {
    id,
    name,
    type: "scatter" as const,
    data: timeline.map((entry) => {
      const annotations = entry.annotationValues.filter(
        (annotation) => (
          annotation.source === source &&
          layers[annotation.kind ?? "annotations"] === true
        ),
      );
      const value = markerValue(entry);
      if (!annotations.length || value == null) return null;
      return {
        value: [entry.timestamp, value],
        annotationIds: annotations.map((annotation) => annotation.id),
        symbolSize: annotations.some((annotation) => (
          annotation.id === activeAnnotationId
        )) ? 11 : 8,
      };
    }),
    symbol: "circle" as const,
    symbolSize: 8,
    itemStyle: { color, borderColor: color, borderWidth: 1 },
    emphasis: { scale: 1.25 },
    z: 8,
  };
}

function markerValue(entry: TimelineEntry): number | null {
  if (entry.historyValue != null) return entry.historyValue;
  if (entry.forecastValue != null) return entry.forecastValue;
  if (entry.lowerValue != null && entry.upperValue != null) {
    return (entry.lowerValue + entry.upperValue) / 2;
  }
  return entry.scenarioValues[0]?.value ?? entry.variableValues[0]?.value ?? null;
}

import type { ForecastLayerState } from "../forecast-layer-matrix";

interface Point {
  date: string;
  value: number;
}

export interface ScenarioLine {
  id: string;
  name: string;
  predictions: Point[];
}

export interface VariableLine {
  id: string;
  name: string;
  values: Array<number | null>;
  rawValues: Array<number | null>;
}

type ForecastChartMode = "main" | "scenario" | "comparison" | "notes";

export interface ForecastChartAnnotation {
  id: string;
  date: string;
  text: string;
  source: "user" | "llm";
  kind?: "annotations" | "anomalies" | "quality";
}

export interface TimelineEntry {
  axisLabel: string;
  dateKey: string;
  timestamp: number;
  fullLabel: string;
  historyValue: number | null;
  forecastValue: number | null;
  scenarioValues: { id: string; name: string; value: number }[];
  variableValues: { id: string; name: string; value: number; rawValue: number }[];
  annotationValues: ForecastChartAnnotation[];
  lowerValue: number | null;
  upperValue: number | null;
}

export interface ForecastChartPalette {
  lineHistory: string;
  linePredict: string;
  pointPredict: string;
  band90: string;
  separator: string;
  forecastZone: string;
  areaHistoryFrom: string;
  areaHistoryTo: string;
  annotationUser: string;
  annotationLlm: string;
  edge: string;
  inkMuted: string;
  tooltipBg: string;
  tooltipText: string;
  scenarios: string[];
  variables: string[];
}

export interface ForecastChartOptionArgs {
  history: Point[];
  predictions: Point[];
  scenarios: ScenarioLine[];
  variables: VariableLine[];
  annotations: ForecastChartAnnotation[];
  activeAnnotationId?: string | null;
  zoomWindow: { start: number; end: number };
  chartWidth: number;
  compact: boolean;
  quantiles: { q10: number[]; q90: number[] };
  frequency: string;
  endDate: string;
  locale: string;
  targetColumn?: string;
  fallbackName?: string;
  layers: ForecastLayerState;
  palette: ForecastChartPalette;
  labels: {
    history: string;
    forecast: string;
    confidence: string;
    forecastStart: string;
    annotationUser: string;
    annotationLlm: string;
  };
}

export interface ForecastChartProps {
  history: Point[];
  predictions: Point[];
  scenarios: ScenarioLine[];
  variables: VariableLine[];
  annotations?: ForecastChartAnnotation[];
  activeAnnotationId?: string | null;
  onAnnotationClick?: (annotation: ForecastChartAnnotation) => void;
  mode?: ForecastChartMode;
  compact?: boolean;
  quantiles: { q10: number[]; q90: number[] };
  frequency: string;
  endDate: string;
  locale: string;
  targetColumn?: string;
  fallbackName?: string;
  labels: ForecastChartOptionArgs["labels"];
  layers: ForecastLayerState;
  onZoomWindowChange?: (window: { start: number; end: number }) => void;
  zoomJump?: { start: number; seq: number } | null;
}

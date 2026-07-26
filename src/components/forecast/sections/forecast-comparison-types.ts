export interface ForecastPoint {
  date: string;
  value: number;
  series_id?: string | null;
}

interface ForecastComparisonScenario {
  id: string;
  name: string;
  predictions: ForecastPoint[];
}

export interface ForecastComparisonAnalysis {
  id: string;
  name: string;
  target_column?: string;
  model: string;
  horizon: number;
  frequency: string;
  input_summary: {
    end: string;
  };
  input_data: {
    series_ids?: string[];
    history: ForecastPoint[];
  };
  predictions: ForecastPoint[];
  quantiles: { q10: number[]; q90: number[] };
  scenarios?: ForecastComparisonScenario[];
}

export interface ForecastComparisonMeta {
  id: string;
  name: string;
  created_at: string;
  model: string;
  horizon: number;
  points: number;
  scenarios_count: number;
}

type ForecastComparisonKind = "scenario" | "forecast";

export interface ForecastComparisonOption {
  id: string;
  kind: ForecastComparisonKind;
  label: string;
  meta: string;
  predictions: ForecastPoint[];
}

export interface ForecastComparisonRow {
  index: number;
  date: string;
  baseValue: number;
  compareValue: number;
  delta: number;
  deltaPercent: number;
}

export interface ForecastComparisonStats {
  averageDelta: number;
  maxDelta: number;
  averageDeltaPercent: number;
  direction: "higher" | "lower" | "mixed";
}

export interface ForecastAnalysisPoint {
  date: string;
  value: number;
  series_id?: string | null;
}

export interface ForecastAnalysisData {
  id: string;
  name: string;
  target_column?: string;
  series_column?: string | null;
  model: string;
  horizon: number;
  frequency: string;
  input_summary: { end: string };
  input_data: {
    rows?: Record<string, unknown>[];
    series_ids?: string[];
    history: ForecastAnalysisPoint[];
  };
  covariates_used?: string[];
  predictions: ForecastAnalysisPoint[];
  quantiles: { q10: number[]; q90: number[] };
  metrics: { mape: number | null; mae: number | null; crps: number | null; bias: number | null } | null;
  advanced_analytics?: ForecastAdvancedAnalytics | null;
}

export type AnalyticsStatus = "ready" | "insufficient_data" | "not_applicable";

export interface ForecastAdvancedAnalytics {
  generated_at: string;
  decomposition: ForecastDecomposition[];
  anomalies: ForecastResidualAnomaly[];
  variable_importance: ForecastVariableImportanceReport;
  drift: ForecastDriftReport[];
}

export interface ForecastDecomposition {
  series_id?: string | null;
  status: AnalyticsStatus;
  method: string;
  period: number;
  seasonal_strength: number | null;
}

export interface ForecastResidualAnomaly {
  id: string;
  series_id?: string | null;
  date: string;
  observed: number;
  expected: number;
  residual: number;
  score: number;
  severity: "medium" | "high";
}

export interface ForecastVariableImportanceReport {
  status: AnalyticsStatus;
  method: string;
  reliability: "unavailable" | "low" | "moderate" | "high";
  scope?: "all_series";
  validation_points: number;
  items: Array<{
    name: string;
    score: number;
    normalized_score: number;
    direction: "positive" | "negative" | "neutral";
  }>;
}

export interface ForecastDriftReport {
  series_id?: string | null;
  status: AnalyticsStatus;
  score: number | null;
  mean_shift: number | null;
  variance_ratio: number | null;
  distribution_shift: number | null;
  detected: boolean;
  severity: "unavailable" | "none" | "medium" | "high";
}

export interface AnalysisCard {
  labelKey: string;
  value: string;
  tone?: "neutral" | "warn" | "good";
}

export interface AnalysisEvent {
  id: string;
  label: string;
  value: string;
  meta: string;
  severity?: "low" | "medium" | "high";
}

export interface VariableInsight {
  name: string;
  score: number;
  detail: string;
}

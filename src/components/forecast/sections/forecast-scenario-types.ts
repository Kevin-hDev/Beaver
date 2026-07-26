export interface ForecastScenario {
  id: string;
  name: string;
  description?: string | null;
  predictions: { date: string; value: number; series_id?: string | null }[];
  params_modified?: {
    kind?: string;
    adjustment_percent?: number;
    covariate_adjustments?: ForecastScenarioCovariateAdjustment[];
    target_series_id?: string | null;
  };
}

export interface ForecastScenarioCovariateAdjustment {
  column: string;
  mode: "percent" | "absolute";
  value: number;
}

export interface ForecastScenarioAnalysis {
  target_column?: string;
  frequency: string;
  input_summary: {
    end: string;
  };
  input_data: {
    columns?: string[];
    covariate_columns?: string[];
    rows?: Record<string, unknown>[];
    series_column?: string | null;
    series_ids?: string[];
    history: { date: string; value: number; series_id?: string | null }[];
  };
  covariates_used?: string[];
  predictions: { date: string; value: number; series_id?: string | null }[];
  quantiles: { q10: number[]; q90: number[] };
  scenarios: ForecastScenario[];
}

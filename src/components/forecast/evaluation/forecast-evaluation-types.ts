export interface BacktestMetrics {
  mase: number;
  smape: number;
  mae: number;
  rmse: number;
  bias: number;
  stability: number;
  quantile_loss?: number | null;
}

export interface IntervalCalibration {
  theoretical_coverage: number;
  measured_coverage: number;
  mean_width: number;
  residual_half_width: number;
  sample_count: number;
}

export interface ModelBacktestResult {
  model_id: string;
  kind: "baseline" | "model";
  metrics: BacktestMetrics | null;
  calibration: IntervalCalibration | null;
  duration_ms: number;
  max_memory_mb?: number | null;
  rank: number | null;
  beats_best_baseline: boolean | null;
  warning: string | null;
}

export interface ForecastEvaluation {
  schema_version: number;
  created_at: string;
  horizon: number;
  windows: number;
  warning: string | null;
  results: ModelBacktestResult[];
}

export interface EvaluationAnalysis {
  id: string;
  revision?: number;
  model: string;
  evaluation?: ForecastEvaluation | null;
  ensemble?: {
    validation_status: string;
    members: Array<{ model_id: string; weight: number; backtest_mase: number }>;
  } | null;
}

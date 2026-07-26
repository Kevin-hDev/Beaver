import type { ForecastModelEntry } from "./forecast-model-meta";

const DEFAULT_FIXED_LEVELS = [0.6, 0.8];
const DEFAULT_MIN = 0.6;
const DEFAULT_MAX = 0.8;
const DEFAULT_STEP = 0.01;

export interface ForecastConfidenceControl {
  limited: boolean;
  min: number;
  max: number;
  step: number;
  effective: number;
}

export function buildForecastConfidenceControl(
  model: ForecastModelEntry | null,
  confidence: number,
): ForecastConfidenceControl {
  const configured = model?.interval_capability?.mode === "fixed_grid"
    ? model.interval_capability.supported_confidence_levels
    : model?.interval_support === "central_60_or_80"
      ? DEFAULT_FIXED_LEVELS
      : [];
  const levels = [...configured]
    .filter((level) => Number.isFinite(level) && level > 0 && level < 1)
    .sort((left, right) => left - right);
  const limited = levels.length > 0;
  const min = levels[0] ?? DEFAULT_MIN;
  const max = levels[levels.length - 1] ?? DEFAULT_MAX;
  const configuredStep = model?.interval_capability?.confidence_step;
  const stepCandidate = configuredStep && configuredStep > 0
    ? configuredStep
    : levels.length > 1
      ? (levels[1] ?? max) - min
      : DEFAULT_STEP;
  const step = Number(stepCandidate.toFixed(4));
  return {
    limited,
    min,
    max,
    step,
    effective: limited && !levels.includes(confidence) ? max : confidence,
  };
}

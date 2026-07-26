import type { ForecastContextProfile } from "./forecast-context-profile";
import type { ForecastModelEntry } from "./forecast-model-meta";

export function buildLaunchErrorKey(
  model: ForecastModelEntry | null,
  profile: ForecastContextProfile,
  horizon: number,
): string | null {
  if (model && horizon > Math.min(model.horizon_max, 5_000)) {
    return "forecast.config.validation.horizonTooLong";
  }
  if (
    profile.futureRows > 0 &&
    profile.futureRowsPerSeries != null &&
    profile.futureRowsPerSeries !== horizon
  ) {
    return "forecast.config.validation.futureRowsMismatch";
  }
  if (profile.futureRows > 0 && profile.futureRowsPerSeries == null) {
    return "forecast.config.validation.futureRowsPerSeriesMismatch";
  }
  if (profile.selectedCovariates > 0 && !model?.capabilities?.past_covariates) {
    return "forecast.config.validation.contextUnsupported";
  }
  if (
    profile.selectedCovariates > 0 &&
    profile.futureRows > 0 &&
    !model?.capabilities?.future_covariates
  ) {
    return "forecast.config.validation.futureContextUnsupported";
  }
  return null;
}

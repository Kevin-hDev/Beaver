import type { ForecastModelEntry } from "./forecast-model-meta";

export type ForecastModelReadiness =
  | "not_installed"
  | "update_required"
  | "invalid"
  | "ready"
  | "provider_required"
  | "unsupported";

export function getForecastModelReadiness(
  model: ForecastModelEntry,
): ForecastModelReadiness {
  if (model.readiness_state) return model.readiness_state;
  if (model.is_cloud) {
    return model.provider_configured && model.runnable ? "ready" : "provider_required";
  }
  if (!model.installed) return "not_installed";
  return model.runtime_ready ? "ready" : "update_required";
}

export function isForecastModelVisibleInSelector(model: ForecastModelEntry): boolean {
  if (isForecastModelSelectable(model)) return true;
  return !model.is_cloud && model.installed;
}

export function isForecastModelSelectable(model: ForecastModelEntry): boolean {
  if (!model.runnable) return false;
  return model.is_cloud
    ? Boolean(model.provider_configured)
    : model.installed && model.runtime_ready === true;
}

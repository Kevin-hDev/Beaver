export type ForecastSelectionMode = "manual" | "auto";

export interface ForecastSelectionPolicy {
  mode: ForecastSelectionMode;
  manual_model_id: string | null;
  allow_cloud_in_auto: boolean;
}

export const DEFAULT_FORECAST_SELECTION_POLICY: ForecastSelectionPolicy = {
  mode: "manual",
  manual_model_id: null,
  allow_cloud_in_auto: false,
};

export function isForecastSelectionPolicy(value: unknown): value is ForecastSelectionPolicy {
  if (!value || typeof value !== "object") return false;
  const policy = value as Partial<ForecastSelectionPolicy>;
  const modelId = policy.manual_model_id;
  const validModel = modelId === null || (
    typeof modelId === "string" &&
    modelId.length > 0 &&
    modelId.length <= 80 &&
    /^[a-zA-Z0-9._-]+$/.test(modelId)
  );
  return (policy.mode === "manual" || policy.mode === "auto") &&
    validModel &&
    typeof policy.allow_cloud_in_auto === "boolean";
}

const FORECAST_CAPACITY_REACHED = "forecast-capacity-reached";

export function forecastLaunchErrorKey(error: unknown): string {
  const code = typeof error === "string"
    ? error
    : error instanceof Error ? error.message : "";
  return code === FORECAST_CAPACITY_REACHED
    ? "forecast.errors.capacityReached"
    : "forecast.errors.launchFailed";
}

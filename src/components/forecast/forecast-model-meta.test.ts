import { describe, expect, it } from "vitest";
import {
  isForecastModelConfigurable,
  isForecastModelSelectable,
  isForecastModelVisibleInSelector,
  type ForecastModelEntry,
} from "./forecast-model-meta";

const MODEL: ForecastModelEntry = {
  id: "chronos-bolt-tiny",
  provider_id: "amazon",
  display_name: "Chronos Bolt Tiny",
  params: "9M",
  size_mb: 35,
  ram_mb: 300,
  vram_mb: 100,
  cpu_supported: true,
  gpu_supported: true,
  multivariate: false,
  covariates: false,
  horizon_max: 64,
  frequencies: "D,W,M",
  is_cloud: false,
  installed: true,
  installable: true,
  runtime_ready: false,
  runnable: false,
  readiness_state: "update_required",
};

describe("forecast model availability", () => {
  it("keeps a downloaded legacy model configurable while preparation is pending", () => {
    expect(isForecastModelConfigurable(MODEL)).toBe(true);
    expect(isForecastModelSelectable(MODEL)).toBe(false);
    expect(isForecastModelVisibleInSelector(MODEL)).toBe(true);
  });

  it("does not expose a disabled remote-code model in configuration", () => {
    expect(isForecastModelConfigurable({ ...MODEL, installable: false })).toBe(false);
  });

  it("keeps an invalid installed model visible but not selectable", () => {
    const invalid = { ...MODEL, readiness_state: "invalid" as const };
    expect(isForecastModelVisibleInSelector(invalid)).toBe(true);
    expect(isForecastModelSelectable(invalid)).toBe(false);
  });
});

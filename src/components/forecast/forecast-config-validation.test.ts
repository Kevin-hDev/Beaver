import { describe, it, expect } from "vitest";
import { buildLaunchErrorKey } from "./forecast-config-validation";
import type { ForecastContextProfile } from "./forecast-context-profile";
import type { ForecastModelEntry } from "./forecast-model-meta";

function profile(overrides: Partial<ForecastContextProfile> = {}): ForecastContextProfile {
  return {
    historyRows: 0,
    futureRows: 0,
    seriesCount: 1,
    futureRowsPerSeries: null,
    selectedCovariates: 0,
    futureContextColumns: [],
    ...overrides,
  };
}

function caps(past: boolean, future: boolean) {
  return {
    past_covariates: past,
    future_covariates: future,
    multi_series: false,
    multivariate: false,
    probabilistic: false,
    backtesting_ready: false,
    anomalies_ready: false,
    fine_tuning_ready: false,
  };
}

function model(overrides: Partial<ForecastModelEntry> = {}): ForecastModelEntry {
  return {
    id: "chronos",
    familyId: "chronos-bolt",
    capabilities: caps(false, false),
    horizon_max: 1000,
    ...overrides,
  } as ForecastModelEntry;
}

describe("buildLaunchErrorKey", () => {
  it("retourne null quand tout est valide (pas de futures rows ni covariables)", () => {
    expect(buildLaunchErrorKey(model(), profile(), 3)).toBeNull();
  });

  it("refuse un horizon supérieur à la limite du modèle", () => {
    expect(buildLaunchErrorKey(model({ horizon_max: 12 }), profile(), 13)).toBe(
      "forecast.config.validation.horizonTooLong",
    );
  });

  it("détecte une inadéquation futureRows/horizon", () => {
    const p = profile({ futureRows: 5, futureRowsPerSeries: 5 });
    // horizon = 3 mais futureRowsPerSeries = 5 → mismatch
    expect(buildLaunchErrorKey(model(), p, 3)).toBe(
      "forecast.config.validation.futureRowsMismatch",
    );
  });

  it("accepte futureRows == horizon", () => {
    const p = profile({ futureRows: 3, futureRowsPerSeries: 3 });
    expect(buildLaunchErrorKey(model(), p, 3)).toBeNull();
  });

  it("détecte futureRows sans futureRowsPerSeries (multi-séries)", () => {
    const p = profile({ futureRows: 6, futureRowsPerSeries: null });
    expect(buildLaunchErrorKey(model(), p, 3)).toBe(
      "forecast.config.validation.futureRowsPerSeriesMismatch",
    );
  });

  it("rejette les covariables si le modèle ne supporte pas past_covariates", () => {
    const p = profile({ selectedCovariates: 2 });
    const m = model({ capabilities: caps(false, false) });
    expect(buildLaunchErrorKey(m, p, 3)).toBe(
      "forecast.config.validation.contextUnsupported",
    );
  });

  it("accepte les covariables si le modèle supporte past_covariates", () => {
    const p = profile({ selectedCovariates: 2 });
    const m = model({ capabilities: caps(true, false) });
    expect(buildLaunchErrorKey(m, p, 3)).toBeNull();
  });

  it("rejette future covariates + futureRows si pas de future_covariates", () => {
    const p = profile({ selectedCovariates: 2, futureRows: 3, futureRowsPerSeries: 3 });
    const m = model({ capabilities: caps(true, false) });
    expect(buildLaunchErrorKey(m, p, 3)).toBe(
      "forecast.config.validation.futureContextUnsupported",
    );
  });

  it("accepte future covariates + futureRows si future_covariates supporté", () => {
    const p = profile({ selectedCovariates: 2, futureRows: 3, futureRowsPerSeries: 3 });
    const m = model({ capabilities: caps(true, true) });
    expect(buildLaunchErrorKey(m, p, 3)).toBeNull();
  });

  it("gère model null avec covariables sélectionnées", () => {
    const p = profile({ selectedCovariates: 1 });
    expect(buildLaunchErrorKey(null, p, 3)).toBe(
      "forecast.config.validation.contextUnsupported",
    );
  });
});

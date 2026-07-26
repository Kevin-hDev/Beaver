import { describe, expect, it } from "vitest";
import type { ForecastModelEntry } from "./forecast-model-meta";
import { buildForecastConfidenceControl } from "./forecast-config-confidence";

describe("buildForecastConfidenceControl", () => {
  it("normalise une grille fixe non triée", () => {
    const model = {
      interval_capability: {
        mode: "fixed_grid",
        supported_confidence_levels: [0.9, 0.6, 0.8],
        confidence_step: null,
      },
    } as ForecastModelEntry;

    expect(buildForecastConfidenceControl(model, 0.7)).toEqual({
      limited: true,
      min: 0.6,
      max: 0.9,
      step: 0.2,
      effective: 0.9,
    });
  });

  it("laisse une confiance continue inchangée", () => {
    expect(buildForecastConfidenceControl(null, 0.73).effective).toBe(0.73);
  });
});

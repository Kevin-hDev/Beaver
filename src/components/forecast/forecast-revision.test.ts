import { describe, expect, it } from "vitest";
import { preferNewestForecast } from "./forecast-revision";

describe("preferNewestForecast", () => {
  it("accepte la nouvelle analyse même si sa révision est plus petite", () => {
    const current = { id: "analysis-a", revision: 8 };
    const next = { id: "analysis-b", revision: 1 };

    expect(preferNewestForecast(current, next)).toBe(next);
  });

  it("refuse une ancienne révision de la même analyse", () => {
    const current = { id: "analysis-a", revision: 8 };
    const stale = { id: "analysis-a", revision: 7 };

    expect(preferNewestForecast(current, stale)).toBe(current);
  });
});

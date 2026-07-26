import { describe, expect, it } from "vitest";
import type { TimelineEntry } from "./forecast-chart-types";
import {
  buildYAxisBounds,
  niceStep,
  roundBoundsOutward,
} from "./forecast-chart-y-bounds";

describe("niceStep", () => {
  it("rounds up to 1/2/2.5/5 × 10^n steps", () => {
    expect(niceStep(0.9)).toBe(1);
    expect(niceStep(1.2)).toBe(2);
    expect(niceStep(2.2)).toBe(2.5);
    expect(niceStep(3)).toBe(5);
    expect(niceStep(7)).toBe(10);
    expect(niceStep(250)).toBe(250);
    expect(niceStep(251)).toBe(500);
    expect(niceStep(0.04)).toBeCloseTo(0.05);
  });

  it("falls back to 1 for non-positive input", () => {
    expect(niceStep(0)).toBe(1);
    expect(niceStep(-3)).toBe(1);
    expect(niceStep(Number.NaN)).toBe(1);
  });
});

describe("roundBoundsOutward", () => {
  it("snaps bounds outward to round values", () => {
    expect(roundBoundsOutward(77.6, 102.3)).toEqual({ min: 75, max: 105 });
  });

  it("keeps already-round bounds unchanged", () => {
    expect(roundBoundsOutward(980, 1020)).toEqual({ min: 980, max: 1020 });
  });

  it("handles fractional steps without float noise", () => {
    expect(roundBoundsOutward(0.15, 0.87)).toEqual({ min: 0, max: 1 });
  });

  it("rounds negative bounds outward", () => {
    const bounds = roundBoundsOutward(-12.3, 8.1);
    expect(bounds.min).toBeLessThanOrEqual(-12.3);
    expect(bounds.max).toBeGreaterThanOrEqual(8.1);
    expect(Number.isInteger(bounds.min)).toBe(true);
    expect(Number.isInteger(bounds.max)).toBe(true);
  });
});

describe("buildYAxisBounds", () => {
  const entry = (value: number): TimelineEntry => ({
    axisLabel: "",
    dateKey: "",
    timestamp: 0,
    fullLabel: "",
    historyValue: value,
    forecastValue: null,
    scenarioValues: [],
    variableValues: [],
    annotationValues: [],
    lowerValue: null,
    upperValue: null,
  });

  const layers = { history: true, forecast: true, confidence: true };

  it("pads then rounds outward to nice bounds", () => {
    const timeline = [entry(100), entry(130)];
    const bounds = buildYAxisBounds(timeline, layers);
    expect(bounds).toEqual({ min: 90, max: 140 });
  });

  it("keeps a flat series readable", () => {
    const bounds = buildYAxisBounds([entry(5)], layers);
    expect(bounds).toEqual({ min: 4, max: 6 });
  });

  it("returns null without values", () => {
    expect(buildYAxisBounds([], layers)).toBeNull();
    expect(buildYAxisBounds([entry(1)], { history: false })).toBeNull();
  });
});

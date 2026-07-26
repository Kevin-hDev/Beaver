import { describe, expect, it } from "vitest";
import { FORECAST_CHART_MIN_ZOOM_SPAN } from "./forecast-chart-zoom-utils";
import {
  zoomJumpBarCount,
  zoomJumpCurrentIndex,
  zoomJumpTarget,
  zoomJumpVisible,
} from "./forecast-zoom-jump-utils";

describe("zoomJumpVisible", () => {
  it("is visible at span 50 and below", () => {
    expect(zoomJumpVisible(50)).toBe(true);
    expect(zoomJumpVisible(10)).toBe(true);
  });

  it("is hidden above span 50 or for invalid spans", () => {
    expect(zoomJumpVisible(51)).toBe(false);
    expect(zoomJumpVisible(100)).toBe(false);
    expect(zoomJumpVisible(0)).toBe(false);
    expect(zoomJumpVisible(Number.NaN)).toBe(false);
  });
});

describe("zoomJumpBarCount", () => {
  it("maps zoom level to bar count", () => {
    expect(zoomJumpBarCount(50)).toBe(3);
    expect(zoomJumpBarCount(26)).toBe(3);
    expect(zoomJumpBarCount(25)).toBe(4);
    expect(zoomJumpBarCount(11)).toBe(4);
    expect(zoomJumpBarCount(10)).toBe(5);
    expect(zoomJumpBarCount(60)).toBe(0);
  });

  it("reaches the 5-bar tier at the deepest allowed zoom", () => {
    expect(FORECAST_CHART_MIN_ZOOM_SPAN).toBeLessThanOrEqual(10);
    expect(zoomJumpBarCount(FORECAST_CHART_MIN_ZOOM_SPAN)).toBe(5);
  });
});

describe("zoomJumpTarget", () => {
  it("spreads targets across the max start range", () => {
    expect([0, 1, 2].map((i) => zoomJumpTarget(i, 3, 50))).toEqual([0, 25, 50]);
    const targets = [0, 1, 2, 3, 4].map((i) => zoomJumpTarget(i, 5, 10));
    expect(targets).toEqual([0, 22.5, 45, 67.5, 90]);
  });

  it("degenerates to 0 when span covers everything or count is 1", () => {
    expect(zoomJumpTarget(2, 3, 100)).toBe(0);
    expect(zoomJumpTarget(0, 1, 50)).toBe(0);
  });
});

describe("zoomJumpCurrentIndex", () => {
  it("rounds the current start to the nearest bar", () => {
    expect(zoomJumpCurrentIndex(0, 50, 3)).toBe(0);
    expect(zoomJumpCurrentIndex(25, 50, 3)).toBe(1);
    expect(zoomJumpCurrentIndex(50, 50, 3)).toBe(2);
    expect(zoomJumpCurrentIndex(23, 50, 3)).toBe(1);
  });

  it("clamps out-of-range starts", () => {
    expect(zoomJumpCurrentIndex(90, 50, 3)).toBe(2);
    expect(zoomJumpCurrentIndex(-5, 50, 3)).toBe(0);
  });

  it("degenerates to 0 when there is nothing to pan", () => {
    expect(zoomJumpCurrentIndex(30, 100, 3)).toBe(0);
    expect(zoomJumpCurrentIndex(10, 50, 1)).toBe(0);
  });
});

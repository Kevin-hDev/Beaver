import { describe, expect, it } from "vitest";
import { fitGeometryToWorkAreas } from "./forecast-workbench-geometry";

const SCREEN = { x: 0, y: 0, width: 1_920, height: 1_080 };

describe("fitGeometryToWorkAreas", () => {
  it("restores and clamps a visible window inside its monitor", () => {
    expect(fitGeometryToWorkAreas(
      { x: 1_500, y: 800, width: 1_180, height: 820 },
      [SCREEN],
    )).toEqual({ x: 740, y: 260, width: 1_180, height: 820 });
  });

  it("rejects a window that no longer intersects an available monitor", () => {
    expect(fitGeometryToWorkAreas(
      { x: 4_000, y: 4_000, width: 1_180, height: 820 },
      [SCREEN],
    )).toBeUndefined();
  });
});

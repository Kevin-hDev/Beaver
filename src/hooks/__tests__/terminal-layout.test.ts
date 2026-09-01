import { describe, expect, it } from "vitest";
import { clampTerminalHeight } from "../terminal-layout";

describe("clampTerminalHeight", () => {
  it.each([
    [10, 80],
    [200, 200],
    [999, 400],
    [Number.NaN, 80],
    [Number.POSITIVE_INFINITY, 400],
  ])("borne %s à %s pour un maximum de 400", (requested, expected) => {
    expect(clampTerminalHeight(requested, 400)).toBe(expected);
  });
});

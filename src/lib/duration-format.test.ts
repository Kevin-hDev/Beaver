import { describe, expect, it } from "vitest";
import { formatCompactDuration } from "./duration-format";

describe("formatCompactDuration", () => {
  it("affiche les secondes sous une minute", () => {
    expect(formatCompactDuration(42_400)).toBe("42s");
  });

  it("affiche les minutes et secondes", () => {
    expect(formatCompactDuration(84_000)).toBe("1 min 24s");
  });

  it("affiche les heures et minutes", () => {
    expect(formatCompactDuration(7_500_000)).toBe("2 h 5 min");
  });

  it("affiche les jours et heures", () => {
    expect(formatCompactDuration(183_600_000)).toBe("2 j 3 h");
  });
});

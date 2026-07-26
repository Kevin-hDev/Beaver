import { describe, expect, it } from "vitest";
import {
  buildSeasonalityModel,
  supportsSeasonalityFrequency,
} from "./forecast-seasonality-data";

describe("seasonality frequency cadence", () => {
  it("renders quarterly data on four connected quarter buckets", () => {
    const model = buildSeasonalityModel(
      [
        { date: "2025-01-01", value: 100 },
        { date: "2025-04-01", value: 110 },
        { date: "2025-07-01", value: 120 },
        { date: "2025-10-01", value: 130 },
      ],
      "fr",
      "Q",
    );

    expect(model?.periods).toEqual(["Q1", "Q2", "Q3", "Q4"]);
    expect(model?.years[0].values).toHaveLength(4);
    expect(model?.years[0].values).not.toContain(null);
    expect(model?.years[0].complete).toBe(true);
  });

  it("hides annual seasonality because it has no within-year signal", () => {
    expect(supportsSeasonalityFrequency("Y")).toBe(false);
    expect(buildSeasonalityModel([
      { date: "2022-01-01", value: 100 },
      { date: "2023-01-01", value: 110 },
      { date: "2024-01-01", value: 120 },
    ], "fr", "Y")).toBeNull();
  });
});

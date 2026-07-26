import { describe, expect, it } from "vitest";
import { formatAxisDate } from "./forecast-chart-axis-date";
import { forecastXAxisSplitNumber } from "./forecast-chart-option";

describe("forecastXAxisSplitNumber", () => {
  it("only changes the chart density when a width threshold is crossed", () => {
    expect(forecastXAxisSplitNumber(540)).toBe(4);
    expect(forecastXAxisSplitNumber(600)).toBe(4);
    expect(forecastXAxisSplitNumber(675)).toBe(5);
  });

  it("keeps the density within readable bounds", () => {
    expect(forecastXAxisSplitNumber(0)).toBe(4);
    expect(forecastXAxisSplitNumber(200)).toBe(3);
    expect(forecastXAxisSplitNumber(2000)).toBe(8);
  });
});

describe("formatAxisDate", () => {
  it("shows the year for January 1st ticks (yearly spacing)", () => {
    expect(formatAxisDate(new Date(2024, 0, 1).getTime(), "en")).toBe("2024");
    expect(formatAxisDate(new Date(2017, 0, 1).getTime(), "fr")).toBe("2017");
  });

  it("shows short month and 2-digit year at other month boundaries", () => {
    const value = new Date(2024, 3, 1).getTime();
    expect(formatAxisDate(value, "en")).toBe(
      new Intl.DateTimeFormat("en", { month: "short", year: "2-digit" }).format(new Date(value)),
    );
    expect(formatAxisDate(value, "fr")).toBe(
      new Intl.DateTimeFormat("fr", { month: "short", year: "2-digit" }).format(new Date(value)),
    );
  });

  it("keeps day/month for finer day-level ticks", () => {
    expect(formatAxisDate(new Date(2024, 3, 15).getTime(), "en")).toBe("04/15");
    expect(formatAxisDate(new Date(2024, 0, 15).getTime(), "fr")).toBe("15/01");
  });

  it("returns an empty label for invalid timestamps", () => {
    expect(formatAxisDate(Number.NaN, "en")).toBe("");
  });
});

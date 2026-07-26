import { describe, expect, it } from "vitest";
import {
  buildSeasonalityModel,
  defaultVisibleYears,
  monthNames,
  SEASONALITY_MIN_YEAR_POINTS,
  toggleVisibleYear,
  type SeasonalityYear,
} from "./forecast-seasonality-data";

function point(year: number, month: number, value: number) {
  return { date: new Date(year, month, 15).toISOString(), value };
}

function yearPoints(year: number, months: number[], base = 100) {
  return months.map((month) => point(year, month, base * (1 + month / 100)));
}

describe("buildSeasonalityModel", () => {
  it("groups by calendar year and normalizes the first month to 100", () => {
    const model = buildSeasonalityModel(
      [...yearPoints(2023, [0, 1, 2, 3]), ...yearPoints(2024, [0, 1, 2, 3])],
      "en",
    );

    expect(model?.years).toHaveLength(2);
    expect(model?.years[0].values[0]).toBe(100);
    expect(model?.years[0].values[1]).toBeCloseTo((101 / 100) * 100);
    expect(model?.years[0].values[11]).toBeNull();
  });

  it("skips years with too few points", () => {
    const model = buildSeasonalityModel(
      [
        ...yearPoints(2022, [10, 11]),
        ...yearPoints(2023, [0, 1, 2]),
        ...yearPoints(2024, [0, 1, 2, 3, 4]),
      ],
      "en",
    );

    expect(model?.years.map((entry) => entry.year)).toEqual([2023, 2024]);
    expect(SEASONALITY_MIN_YEAR_POINTS).toBe(3);
  });

  it("skips years whose base value is zero", () => {
    const model = buildSeasonalityModel(
      [point(2023, 0, 0), point(2023, 1, 5), point(2023, 2, 6), ...yearPoints(2024, [0, 1, 2])],
      "en",
    );

    expect(model?.years.map((entry) => entry.year)).toEqual([2024]);
  });

  it("emphasizes the most recent complete year", () => {
    const model = buildSeasonalityModel(
      [
        ...yearPoints(2023, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]),
        ...yearPoints(2024, [0, 1, 2]),
      ],
      "en",
    );

    expect(model?.years[0].complete).toBe(true);
    expect(model?.years[0].emphasized).toBe(true);
    expect(model?.years[1].emphasized).toBe(false);
  });

  it("falls back to the most recent year when none is complete", () => {
    const model = buildSeasonalityModel(
      [...yearPoints(2023, [0, 1, 2]), ...yearPoints(2024, [0, 1, 2, 3])],
      "en",
    );

    expect(model?.years[1].emphasized).toBe(true);
  });

  it("returns null without usable years", () => {
    expect(buildSeasonalityModel([], "en")).toBeNull();
    expect(buildSeasonalityModel(yearPoints(2024, [0, 1]), "en")).toBeNull();
  });

  it("ignores invalid dates", () => {
    const model = buildSeasonalityModel(
      [{ date: "not-a-date", value: 1 }, ...yearPoints(2024, [0, 1, 2])],
      "en",
    );
    expect(model?.years).toHaveLength(1);
  });

  it("aggregates all sub-monthly observations by mean", () => {
    const daily = [
      { date: "2024-01-05", value: 100 },
      { date: "2024-01-20", value: 200 },
      { date: "2024-02-03", value: 150 },
      { date: "2024-02-18", value: 210 },
      { date: "2024-03-07", value: 120 },
    ];
    const model = buildSeasonalityModel(daily, "en", "D");
    expect(model?.years).toHaveLength(1);
    // January mean (150) is the base; February mean is 180 -> 120.
    expect(model?.years[0].values[0]).toBe(100);
    expect(model?.years[0].values[1]).toBeCloseTo((180 / 150) * 100, 5);
    expect(model?.years[0].values[2]).toBeCloseTo((120 / 150) * 100, 5);
  });

  it("keeps the first monthly value for monthly-or-longer frequencies", () => {
    const points = [
      { date: "2024-01-01", value: 100 },
      { date: "2024-01-15", value: 200 },
      { date: "2024-02-01", value: 150 },
      { date: "2024-03-01", value: 120 },
    ];
    const model = buildSeasonalityModel(points, "en", "M");
    expect(model?.years[0].values[0]).toBe(100);
    expect(model?.years[0].values[1]).toBeCloseTo(150, 5);
  });

  it("parses date-only strings without timezone drift", () => {
    const previous = process.env.TZ;
    process.env.TZ = "America/New_York";
    try {
      const model = buildSeasonalityModel(
        [
          { date: "2026-01-01", value: 10 },
          { date: "2026-02-01", value: 20 },
          { date: "2026-03-01", value: 30 },
        ],
        "en",
        "M",
      );
      // Behind-UTC timezones must not shift Jan 1st into December 2025.
      expect(model?.years).toHaveLength(1);
      expect(model?.years[0].year).toBe(2026);
      expect(model?.years[0].values[0]).toBe(100);
      expect(model?.years[0].values[1]).toBeCloseTo(200, 5);
      expect(model?.years[0].values[11]).toBeNull();
    } finally {
      if (previous === undefined) delete process.env.TZ;
      else process.env.TZ = previous;
    }
  });
});

describe("monthNames", () => {
  it("returns 12 locale-aware short month names", () => {
    expect(monthNames("en")).toHaveLength(12);
    expect(monthNames("en")[0]).toBe("Jan");
    expect(monthNames("fr")[0]).toBe("janv.");
  });
});

function year(year: number, complete: boolean): SeasonalityYear {
  return { year, values: [], complete, emphasized: false };
}

describe("defaultVisibleYears", () => {
  it("keeps the last 5 complete years plus the trailing partial year", () => {
    const years = [
      year(2018, true), year(2019, true), year(2020, true), year(2021, true),
      year(2022, true), year(2023, true), year(2024, true), year(2025, false),
    ];
    expect(defaultVisibleYears(years)).toEqual([2020, 2021, 2022, 2023, 2024, 2025]);
  });

  it("keeps all years when there are fewer than 5 complete ones", () => {
    const years = [year(2022, true), year(2023, true), year(2024, false)];
    expect(defaultVisibleYears(years)).toEqual([2022, 2023, 2024]);
  });

  it("does not duplicate the last year when it is complete", () => {
    const years = [year(2023, true), year(2024, true)];
    expect(defaultVisibleYears(years)).toEqual([2023, 2024]);
  });
});

describe("toggleVisibleYear", () => {
  it("removes a visible year", () => {
    expect(toggleVisibleYear([2023, 2024], 2023)).toEqual([2024]);
  });

  it("adds a hidden year in sorted order", () => {
    expect(toggleVisibleYear([2024], 2022)).toEqual([2022, 2024]);
  });
});

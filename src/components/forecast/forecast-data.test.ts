import { describe, expect, it } from "vitest";
import { buildForecastDraftData } from "./forecast-data";

describe("buildForecastDraftData", () => {
  it("conserve la position des colonnes avec un header vide", () => {
    const result = buildForecastDraftData(
      {
        headers: ["date", "", "sales"],
        rows: [["2026-01-01", "note", "12,5"]],
        total_rows: 1,
      },
      "data.xlsx",
    );

    expect(result.columns).toEqual(["date", "column_2", "sales"]);
    expect(result.rows[0]).toEqual({
      date: "2026-01-01",
      column_2: "note",
      sales: 12.5,
    });
  });

  it("refuse les headers dupliqués", () => {
    expect(() =>
      buildForecastDraftData(
        { headers: ["date", "date"], rows: [], total_rows: 0 },
        "data.csv",
      ),
    ).toThrow("forecast_headers_duplicate");
  });

  it("ne transforme pas un nombre ambigu", () => {
    const result = buildForecastDraftData(
      {
        headers: ["value"],
        rows: [["1,234.56"]],
        total_rows: 1,
      },
      "data.csv",
    );
    expect(result.rows[0].value).toBe("1,234.56");
  });

  it("refuse un nombre total de lignes supérieur à la limite", () => {
    expect(() =>
      buildForecastDraftData(
        { headers: ["value"], rows: [], total_rows: 5_001 },
        "data.csv",
      ),
    ).toThrow("forecast_data_too_large");
  });

  it("refuse les cellules trop longues", () => {
    expect(() =>
      buildForecastDraftData(
        { headers: ["value"], rows: [["x".repeat(32_769)]], total_rows: 1 },
        "data.csv",
      ),
    ).toThrow("forecast_cell_too_long");
  });
});

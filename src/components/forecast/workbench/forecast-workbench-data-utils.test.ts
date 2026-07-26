import { describe, expect, it } from "vitest";
import { buildDataPreview, formatDataCell } from "./forecast-workbench-data-utils";

describe("forecast workbench data preview", () => {
  it("bounds externally supplied rows and columns", () => {
    const columns = Array.from({ length: 40 }, (_, index) => `column-${index}`);
    const rows = Array.from({ length: 250 }, (_, index) => ({ value: index }));

    const preview = buildDataPreview({ columns, rows }, "value");

    expect(preview.columns).toHaveLength(32);
    expect(preview.rows).toHaveLength(200);
    expect(preview.totalRows).toBe(250);
    expect(preview.truncated).toBe(true);
  });

  it("rebuilds a readable legacy preview from saved history", () => {
    const preview = buildDataPreview({
      date_column: "month",
      series_column: "store",
      history: [{ date: "2026-01", value: 12, series_id: "north" }],
    }, "sales");

    expect(preview.columns).toEqual(["month", "sales", "store"]);
    expect(preview.rows[0]).toEqual({ month: "2026-01", sales: 12, store: "north" });
  });

  it("removes control characters and bounds long cells", () => {
    const formatted = formatDataCell(`safe\n${"x".repeat(180)}`);

    expect(formatted).not.toContain("\n");
    expect(Array.from(formatted)).toHaveLength(120);
    expect(formatted.endsWith("…")).toBe(true);
  });
});

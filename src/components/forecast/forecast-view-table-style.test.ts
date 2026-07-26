import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const tableCss = readFileSync("src/components/forecast/forecast-view-table.css", "utf8");
const tokensCss = readFileSync("src/styles/tokens.css", "utf8");

describe("forecast predictions table sizing", () => {
  it("uses the shared layout token instead of a local pixel value", () => {
    expect(tokensCss).toContain("--forecast-predictions-table-max-height:");
    expect(tableCss).toContain("var(--forecast-predictions-table-max-height)");
    expect(tableCss).not.toContain("max-height: 340px");
  });
});

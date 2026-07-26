import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const reportCss = readFileSync(
  "src/components/forecast/workbench/forecast-workbench-report.css",
  "utf8",
);

describe("Forecast workbench report layout", () => {
  it("keeps expanded report sections inside a scrollable area", () => {
    expect(reportCss).toMatch(/\.fcwr-root\s*\{[^}]*flex:\s*1;/s);
    expect(reportCss).toMatch(/\.fcwr-analysis\s*\{[^}]*min-height:\s*0;/s);
    expect(reportCss).toMatch(
      /\.fcwr-analysis \.fca-scroll\s*\{[^}]*overflow:\s*auto;/s,
    );
    expect(reportCss).not.toContain("overflow: visible");
  });
});

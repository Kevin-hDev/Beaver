import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const layoutCss = readFileSync(
  "src/components/forecast/workbench/forecast-workbench.css",
  "utf8",
);
const responsiveCss = readFileSync(
  "src/components/forecast/workbench/forecast-workbench-responsive.css",
  "utf8",
);

describe("Forecast workbench responsive container", () => {
  it("measures an ancestor before changing the inner shell layout", () => {
    expect(layoutCss).toMatch(
      /\.fcw-viewport\s*\{[^}]*container-type:\s*inline-size;/s,
    );
    expect(responsiveCss).toMatch(
      /@container[^{}]*\{[\s\S]*\.fcw-shell\s*\{[^}]*grid-template-columns:\s*1fr;/,
    );
  });
});

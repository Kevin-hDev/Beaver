import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const componentCss = readFileSync(
  "src/components/extensions/extension-ui-recovery-dialog.css",
  "utf8",
);
const layoutTokens = readFileSync("src/styles/tokens.css", "utf8");

describe("extension UI recovery dialog focus", () => {
  it("resolves its visible focus geometry from the layout token authority", () => {
    expect(layoutTokens).toMatch(/--focus-visible-ring-width:\s*2px;/u);
    expect(layoutTokens).toMatch(/--focus-visible-ring-offset:\s*2px;/u);
    expect(componentCss).toContain(
      "outline: var(--focus-visible-ring-width) solid var(--pulse);",
    );
    expect(componentCss).toContain(
      "outline-offset: var(--focus-visible-ring-offset);",
    );
    expect(componentCss).not.toMatch(/outline:\s*2px/u);
    expect(componentCss).not.toMatch(/outline-offset:\s*2px/u);
  });
});

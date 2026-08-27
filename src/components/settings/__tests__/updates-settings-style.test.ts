import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const css = readFileSync("src/components/settings/updates-settings.css", "utf8");

describe("mise en page des mises à jour", () => {
  it("aligne les versions à droite sans séparateurs internes", () => {
    expect(css).toContain("grid-template-columns: minmax(0, 1fr) auto;");
    expect(css).not.toMatch(/\.ups-row \+ \.ups-row\s*\{[^}]*border-top/s);
    expect(css).not.toMatch(/\.ups-available\s*\{[^}]*border-top/s);
  });
});

import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const globalCss = readFileSync("src/styles/global.css", "utf8");
const tokensCss = readFileSync("src/styles/tokens.css", "utf8");

describe("Sélection de texte", () => {
  it("est peinte par l'application et non par le système", () => {
    // Sans règle, la couleur venait de l'accent du système : sur le fond crème
    // du thème Clair, le texte sélectionné ne se distinguait pas du reste.
    expect(globalCss).toMatch(/::selection\s*\{\s*background:\s*var\(--text-selection-bg\);/s);
  });

  it("tire sa teinte de l'accent du thème", () => {
    expect(tokensCss).toContain("--text-selection-bg: color-mix(in srgb, var(--pulse) 32%, transparent);");
  });
});

import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const css = readFileSync("src/components/layout/update-notifications.css", "utf8");

describe("croix des notifications de mise à jour", () => {
  it("garde une cible accessible mais réduit le cercle visible de moitié", () => {
    expect(css).toMatch(/\.update-bubble-dismiss\s*\{[^}]*width:\s*40px;[^}]*height:\s*40px;/s);
    expect(css).toMatch(/\.update-bubble-dismiss::before\s*\{[^}]*width:\s*20px;[^}]*height:\s*20px;/s);
    expect(css).toMatch(/\.update-bubble-dismiss\s+svg\s*\{[^}]*position:\s*relative;/s);
  });
});

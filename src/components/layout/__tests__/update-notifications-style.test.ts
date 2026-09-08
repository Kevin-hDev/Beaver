import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const css = readFileSync("src/components/layout/update-notifications.css", "utf8");

describe("croix des notifications de mise à jour", () => {
  it("garde une cible accessible mais réduit le cercle visible de moitié", () => {
    expect(css).toMatch(/\.update-bubble-dismiss\s*\{[^}]*width:\s*40px;[^}]*height:\s*40px;/s);
    expect(css).toMatch(/\.update-bubble-dismiss::before\s*\{[^}]*width:\s*20px;[^}]*height:\s*20px;/s);
    expect(css).toMatch(/\.update-bubble-dismiss\s+svg\s*\{[^}]*position:\s*relative;/s);
  });

  /* La liste défile, donc elle rogne ce qui dépasse d'elle. Sans cette place
     réservée, la croix posée à cheval sur le coin de sa bulle est coupée en
     deux, et rien dans les tests de rendu ne le montre. */
  it("réserve dans la liste la place du débordement de la croix", () => {
    expect(css).toMatch(/\.update-list\s*\{[^}]*--bubble-dismiss-overhang:\s*16px;/s);
    expect(css).toMatch(/\.update-list\s*\{[^}]*padding-top:\s*var\(--bubble-dismiss-overhang\);/s);
    expect(css).toMatch(/\.update-list\s*\{[^}]*padding-left:\s*var\(--bubble-dismiss-overhang\);/s);
    expect(css).toMatch(/\.update-list\s*\{[^}]*margin-top:\s*calc\(-1 \* var\(--bubble-dismiss-overhang\)\);/s);
    expect(css).toMatch(/\.update-list\s*\{[^}]*margin-left:\s*calc\(-1 \* var\(--bubble-dismiss-overhang\)\);/s);
    expect(css).toMatch(/\.update-bubble-dismiss\s*\{[^}]*top:\s*calc\(-1 \* var\(--bubble-dismiss-overhang\)\);/s);
    expect(css).toMatch(/\.update-bubble-dismiss\s*\{[^}]*left:\s*calc\(-1 \* var\(--bubble-dismiss-overhang\)\);/s);
  });
});

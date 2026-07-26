import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

describe("entrée de la mascotte", () => {
  it("charge une page dédiée sans le splash de l'application", () => {
    const html = readFileSync("mascot.html", "utf8");

    expect(html).toContain("/src/mascot-main.tsx");
    expect(html).not.toContain('id="splash"');
    expect(html).not.toContain("castor.svg");
  });
});

import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const indexHtml = readFileSync("index.html", "utf8");

describe("déplacement de la fenêtre pendant le splash", () => {
  it("marque le splash comme zone de déplacement", () => {
    expect(indexHtml).toContain('id="splash"');
    expect(indexHtml).toMatch(/<div id="splash"[^>]*data-tauri-drag-region="deep"/);
  });

  it("couvre toute la fenêtre, donc tout point de départ du geste", () => {
    const start = indexHtml.indexOf("#splash {");
    const rule = indexHtml.slice(start, indexHtml.indexOf("}", start));

    expect(rule).toContain("position: fixed;");
    expect(rule).toContain("inset: 0;");
  });
});

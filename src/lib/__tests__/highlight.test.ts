import { describe, expect, it } from "vitest";
import { highlightLines } from "../highlight";

describe("highlightLines", () => {
  it("ne transforme pas le retour final en ligne vide", () => {
    expect(highlightLines("première\ndeuxième\n", "notes.txt")).toEqual([
      "première",
      "deuxième",
    ]);
  });

  it("conserve une vraie ligne vide avant le retour final", () => {
    expect(highlightLines("première\n\n", "notes.txt")).toEqual([
      "première",
      "",
    ]);
  });

  it("applique la même règle au code coloré", () => {
    expect(highlightLines("const value = 1;\n", "example.ts")).toHaveLength(1);
  });

  it("échappe le HTML avant son affichage dans la preview", () => {
    const [html] = highlightLines("<img src=x onerror=alert(1)>", "notes.txt");

    expect(html).toMatch(/&(?:lt|#x3C);img/);
    expect(html).not.toContain("<img");
  });
});

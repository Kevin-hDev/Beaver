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

  it("échappe une balise script dans un fichier coloré (html)", () => {
    const lines = highlightLines("<script>alert(1)</script>", "page.html");

    for (const html of lines) {
      expect(html).not.toContain("<script");
      expect(html).not.toContain("</script");
    }
  });

  it("échappe un attribut avec gestionnaire d'événement dans un fichier coloré (html)", () => {
    const [html] = highlightLines('<img src="x" onerror="alert(1)">', "page.html");

    expect(html).not.toContain("<img");
    expect(html).not.toMatch(/onerror="alert/);
  });

  it("échappe une chaîne piégée dans un fichier coloré (ts)", () => {
    const payload = 'const s = ""><img src=x onerror=alert(1)>";';
    const [html] = highlightLines(payload, "example.ts");

    expect(html).not.toContain("<img");
    expect(html).toMatch(/&(?:lt|#x3C);/);
    /* Aucune balise autre que les <span> du colorateur ne doit survivre. */
    const withoutSpans = html.replace(/<\/?span[^>]*>/g, "");
    expect(withoutSpans).not.toContain("<");
  });

  it("échappe chaque ligne d'un payload multi-lignes dans un fichier coloré (js)", () => {
    const payload = '"><svg onload=alert(1)>\n<iframe src="javascript:alert(1)"></iframe>';
    const lines = highlightLines(payload, "app.js");

    expect(lines.length).toBeGreaterThan(1);
    for (const html of lines) {
      expect(html).not.toContain("<svg");
      expect(html).not.toContain("<iframe");
      expect(html).not.toContain("</iframe");
    }
  });
});

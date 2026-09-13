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

  it("reprend la couleur d'un commentaire sur chaque ligne qu'il couvre", () => {
    expect(highlightLines("/* un\ndeux */\nconst a = 1;\n", "exemple.ts")).toEqual([
      '<span class="hljs-comment">/* un</span>',
      '<span class="hljs-comment">deux */</span>',
      '<span class="hljs-keyword">const</span> a = <span class="hljs-number">1</span>;',
    ]);
  });

  it("reprend la couleur d'un texte entre accents graves sur plusieurs lignes", () => {
    expect(highlightLines("const s = `a\nb`;\n", "exemple.ts")).toEqual([
      '<span class="hljs-keyword">const</span> s = <span class="hljs-string">`a</span>',
      '<span class="hljs-string">b`</span>;',
    ]);
  });

  it("ferme sur chaque ligne toutes les couleurs qu'elle ouvre", () => {
    const samples: Array<[string, string]> = [
      ["/* un\ndeux\ntrois */\n", "exemple.ts"],
      ["const s = `a\nb\nc`;\n", "exemple.ts"],
      ["/* bloc\n   commentaire */\nfn main() {}\n", "main.rs"],
    ];
    for (const [code, path] of samples) {
      for (const line of highlightLines(code, path)) {
        const opened = line.match(/<span\b/g)?.length ?? 0;
        const closed = line.match(/<\/span>/g)?.length ?? 0;
        expect(opened, `${path} : ${line}`).toBe(closed);
      }
    }
  });

  it("échappe l'apostrophe comme les autres caractères du HTML", () => {
    expect(highlightLines("it's", "notes.txt")).toEqual(["it&#39;s"]);
  });

  it("colorie un bloc de code d'un markdown dans sa langue", () => {
    expect(highlightLines("# Titre\n```ts\nconst a = 1;\n```\n", "plan.md")).toEqual([
      '<span class="hljs-section"># Titre</span>',
      '<span class="hljs-code">```ts</span>',
      '<span class="hljs-keyword">const</span> a = <span class="hljs-number">1</span>;',
      '<span class="hljs-code">```</span>',
    ]);
  });

  it("laisse en texte normal un bloc sans langue connue", () => {
    expect(highlightLines("```text\n<b>x</b>\n```\n", "plan.md")).toEqual([
      '<span class="hljs-code">```text</span>',
      "&lt;b&gt;x&lt;/b&gt;",
      '<span class="hljs-code">```</span>',
    ]);
  });

  it("ne ferme un bloc que par une clôture du même signe, au moins aussi longue, sans texte après", () => {
    const code = (text: string) => `<span class="hljs-code">${text}</span>`;
    expect(highlightLines("````text\n```\nx\n````\nfin\n", "plan.md")).toEqual([
      code("````text"), "```", "x", code("````"), "fin",
    ]);
    expect(highlightLines("```text\n``` x\ny\n```\n", "plan.md")).toEqual([
      code("```text"), "``` x", "y", code("```"),
    ]);
    expect(highlightLines("```text\n~~~\ny\n```\n", "plan.md")).toEqual([
      code("```text"), "~~~", "y", code("```"),
    ]);
  });

  it("ignore une langue de clôture qui porte le nom d'une propriété d'objet", () => {
    for (const language of ["constructor", "__proto__"]) {
      expect(highlightLines("```" + language + "\nx\n```\n", "plan.md")).toEqual([
        `<span class="hljs-code">\`\`\`${language}</span>`,
        "x",
        '<span class="hljs-code">```</span>',
      ]);
    }
  });

  it("échappe une balise script dans un bloc html d'un markdown", () => {
    const lines = highlightLines("```html\n<script>alert(1)</script>\n```", "notes.md");

    for (const html of lines) {
      expect(html).not.toContain("<script");
      expect(html).not.toContain("</script");
    }
  });

  it("ne prend pas du code en ligne en début de ligne pour une clôture", () => {
    expect(highlightLines("Du ```x``` texte\n```y``` suite\n# Titre\n", "plan.md")[2])
      .toBe('<span class="hljs-section"># Titre</span>');
  });

  it("tient un très gros markdown", () => {
    const text = Array.from({ length: 150_000 }, () => "-").join("\n");
    expect(highlightLines(text, "gros.md")).toHaveLength(150_000);
  });
});

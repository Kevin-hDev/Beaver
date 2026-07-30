import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

/* Ce test garde le système de boutons unifié (src/styles/buttons.css).
   Il a déjà été contourné une fois : les classes de domaine portaient un
   commentaire « alias fin du système partagé » tout en redéclarant la
   géométrie complète, et l'application affichait onze boutons gris
   différents. Un commentaire ne se vérifie pas, une assertion si. */

const BUTTONS_CSS = "src/styles/buttons.css";
const buttonsCss = readFileSync(BUTTONS_CSS, "utf8");

/* Les seuls contrôles qui posent leur propre hauteur, parce qu'ils ne sont pas
   des boutons d'interface : les pastilles de fenêtre imposées par macOS, et les
   chevrons logés à l'intérieur du champ de taille de police. */
const GEOMETRY_EXCEPTIONS = new Set([
  ".wc-btn",
  ".fsc-step-btn",
]);

function cssFiles(dir: string): string[] {
  // Les chemins parcourus viennent de l'arborescence du dépôt, pas d'une entrée.
  // eslint-disable-next-line security/detect-non-literal-fs-filename
  return readdirSync(dir).flatMap((entry) => {
    const path = join(dir, entry);
    // eslint-disable-next-line security/detect-non-literal-fs-filename
    if (statSync(path).isDirectory()) return cssFiles(path);
    return path.endsWith(".css") && path !== BUTTONS_CSS ? [path] : [];
  });
}

interface Rule {
  file: string;
  selector: string;
  body: string;
}

function buttonRules(): Rule[] {
  const rules: Rule[] = [];
  for (const file of cssFiles("src")) {
    // eslint-disable-next-line security/detect-non-literal-fs-filename
    const css = readFileSync(file, "utf8").replace(/\/\*[\s\S]*?\*\//g, "");
    for (const match of css.matchAll(/([^{}]+)\{([^{}]*)\}/g)) {
      const selector = match[1].trim().replace(/\s+/g, " ");
      // Le dernier segment décide : `.wk-dialog .btn` cible un bouton,
      // `.btn-row .title` cible un titre posé à côté d'un bouton.
      const target = selector.split(/[\s>]+/).pop() ?? "";
      if (!/(^|[.-])(btn|button)([.:[-]|$)/.test(target)) continue;
      rules.push({ file, selector, body: match[2] });
    }
  }
  return rules;
}

const rules = buttonRules();

describe("système de boutons unifié", () => {
  it("garde la hauteur des boutons dans buttons.css seul", () => {
    // La hauteur vient de .btn-sm / .icon-btn. Une classe de domaine qui la
    // repose fabrique un bouton d'une autre taille sur sa seule page — c'est
    // ce qui donnait des boutons de 28 et de 31,5 px d'un onglet à l'autre.
    const offenders = rules
      .filter(({ body }) => /(^|;)\s*height\s*:/.test(body))
      .filter(({ body }) => !/var\(--btn-height\)/.test(body))
      .filter(({ selector }) => !GEOMETRY_EXCEPTIONS.has(selector))
      .map(({ file, selector }) => `${file} → ${selector}`);

    expect(offenders).toEqual([]);
  });

  it("garde les couleurs du bouton gris dans buttons.css seul", () => {
    // --btn-secondary-bg est la couleur de fond du bouton gris. Un fichier de
    // domaine qui la repose duplique la variante au lieu de la composer.
    const offenders = rules
      .filter(({ body }) => body.includes("--btn-secondary-bg"))
      .map(({ file, selector }) => `${file} → ${selector}`);

    expect(offenders).toEqual([]);
  });

  it("ne laisse aucun survol se réduire à un changement de contour", () => {
    // Un survol qui n'éclaire que la bordure se lit comme un état de focus.
    // Le survol d'un bouton change son fond.
    const offenders = rules
      .filter(({ selector }) => selector.includes(":hover"))
      .filter(({ body }) => /border-color\s*:/.test(body))
      .filter(({ body }) => !/(^|;)\s*background\s*:/.test(body))
      .map(({ file, selector }) => `${file} → ${selector}`);

    expect(offenders).toEqual([]);
  });

  it("tire la hauteur des boutons d'un token unique", () => {
    expect(buttonsCss).toContain("height: var(--btn-height);");
    expect(buttonsCss).not.toMatch(/height:\s*\d+px/);
  });

  it("donne au bouton gris le fond et le survol de la référence Ollama", () => {
    const secondary = buttonsCss.match(/\.btn-secondary\s*\{([^}]*)\}/)?.[1] ?? "";
    const hover = buttonsCss.match(/\.btn-secondary:hover:not\(:disabled\)\s*\{([^}]*)\}/)?.[1] ?? "";

    expect(secondary).toContain("background: var(--btn-secondary-bg);");
    expect(hover).toContain("background: var(--select-bg);");
  });
});

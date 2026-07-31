import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

/* Ce test garde les systèmes partagés de boutons (src/styles/buttons.css) et de
   champs (src/styles/fields.css).

   Il a déjà été contourné une fois : les classes de domaine portaient un
   commentaire « alias fin du système partagé » tout en redéclarant la géométrie
   complète, et l'application affichait onze boutons gris différents. Un
   commentaire ne se vérifie pas, une assertion si. */

const buttonsCss = readFileSync("src/styles/buttons.css", "utf8");
const fieldsCss = readFileSync("src/styles/fields.css", "utf8");
const SHARED = ["src/styles/buttons.css", "src/styles/fields.css"];

/* Les seuls contrôles qui posent leur propre hauteur, parce qu'ils ne sont pas
   des contrôles d'interface au sens du système : les pastilles de fenêtre
   imposées par macOS, et les chevrons logés dans le champ de taille de police. */
const GEOMETRY_EXCEPTIONS = new Set([".wc-btn", ".fsc-step-btn"]);

/* La zone de saisie des conversations garde son habillage propre : ses contrôles
   sont posés sur la bulle du composeur et non sur une page, et l'aligner sur le
   reste de l'application casserait le bloc visuel qu'elle forme (décision du
   propriétaire, 2026-07-30). */
const COMPOSER_PREFIXES = [
  ".ms-", ".rs-", ".perm-mode-", ".cpm-", ".slash-",
  ".project-", ".bs-", ".chat-",
];

function cssFiles(dir: string): string[] {
  // Les chemins parcourus viennent de l'arborescence du dépôt, pas d'une entrée.
  // eslint-disable-next-line security/detect-non-literal-fs-filename
  return readdirSync(dir).flatMap((entry) => {
    const path = join(dir, entry);
    // eslint-disable-next-line security/detect-non-literal-fs-filename
    if (statSync(path).isDirectory()) return cssFiles(path);
    return path.endsWith(".css") && !SHARED.includes(path) ? [path] : [];
  });
}

interface Rule {
  file: string;
  selector: string;
  target: string;
  body: string;
}

function domainRules(): Rule[] {
  const rules: Rule[] = [];
  for (const file of cssFiles("src")) {
    // eslint-disable-next-line security/detect-non-literal-fs-filename
    const css = readFileSync(file, "utf8").replace(/\/\*[\s\S]*?\*\//g, "");
    for (const match of css.matchAll(/([^{}]+)\{([^{}]*)\}/g)) {
      const selector = match[1].trim().replace(/\s+/g, " ");
      // Le dernier segment décide : `.wk-dialog .btn` cible un bouton,
      // `.btn-row .title` cible un titre posé à côté d'un bouton.
      const target = selector.split(/[\s>]+/).pop() ?? "";
      rules.push({ file, selector, target, body: match[2] });
    }
  }
  return rules;
}

const rules = domainRules();

function isComposer(selector: string): boolean {
  return COMPOSER_PREFIXES.some((prefix) => selector.includes(prefix));
}

const buttons = rules.filter(({ target }) => /(^|[.-])(btn|button)([.:[-]|$)/.test(target));
const fields = rules.filter(({ target }) => /(input|field|trigger|-select)([.:[-]|$)/.test(target));
const controls = [...buttons, ...fields].filter(({ selector }) => !isComposer(selector));

function report(subset: Rule[]): string[] {
  return subset.map(({ file, selector }) => `${file} → ${selector}`);
}

describe("boutons", () => {
  it("garde la hauteur dans buttons.css seul", () => {
    // Une classe de domaine qui repose la hauteur fabrique un bouton d'une autre
    // taille sur sa seule page — c'est ce qui donnait 28 px ici et 31,5 px là.
    const offenders = buttons
      .filter(({ body }) => /(^|;)\s*height\s*:/.test(body))
      .filter(({ body }) => !/var\(--btn-height\)/.test(body))
      .filter(({ selector }) => !GEOMETRY_EXCEPTIONS.has(selector));

    expect(report(offenders)).toEqual([]);
  });

  it("garde les couleurs du bouton gris dans buttons.css seul", () => {
    const offenders = buttons.filter(({ body }) => body.includes("--btn-secondary-bg"));

    expect(report(offenders)).toEqual([]);
  });

  it("tire la hauteur d'un token unique", () => {
    expect(buttonsCss).toContain("height: var(--btn-height);");
    expect(buttonsCss).not.toMatch(/height:\s*\d+px/);
  });

  it("donne au bouton gris le fond et le survol de la référence Ollama", () => {
    const secondary = buttonsCss.match(/\.btn-secondary\s*\{([^}]*)\}/)?.[1] ?? "";
    const hover = buttonsCss
      .match(/\.btn-secondary:hover:not\(:disabled\)\s*\{([^}]*)\}/)?.[1] ?? "";

    expect(secondary).toContain("background: var(--btn-secondary-bg);");
    expect(hover).toContain("background: var(--select-bg);");
  });
});

describe("champs et listes dépliantes", () => {
  it("garde les couleurs du champ dans fields.css seul", () => {
    const offenders = fields
      .filter(({ selector }) => !isComposer(selector))
      .filter(({ body }) => body.includes("--field-bg"));

    expect(report(offenders)).toEqual([]);
  });

  it("laisse le contour tranquille au survol comme à la saisie", () => {
    // Ni le survol ni la mise au point ne touchent la bordure : le curseur
    // clignotant dit déjà quel champ écoute le clavier (décision du
    // propriétaire, 2026-07-30).
    const offenders = fields
      .filter(({ selector }) => /:(hover|focus|focus-visible|focus-within)/.test(selector))
      .filter(({ selector }) => !isComposer(selector))
      .filter(({ body }) => /border-color\s*:/.test(body));

    expect(report(offenders)).toEqual([]);
  });

  it("n'habille pas le champ d'un relief de bouton", () => {
    // Le liseré intérieur annonce qu'on peut cliquer. Un champ se creuse.
    expect(fieldsCss).not.toContain("--btn-inner-highlight");
  });
});

describe("contrôles, toutes familles", () => {
  it("ne laisse aucun survol se réduire à un changement de contour", () => {
    // Un survol qui n'éclaire que la bordure se lit comme un état de focus.
    const offenders = controls
      .filter(({ selector }) => selector.includes(":hover"))
      .filter(({ body }) => /border-color\s*:/.test(body))
      .filter(({ body }) => !/(^|;)\s*background\s*:/.test(body));

    expect(report(offenders)).toEqual([]);
  });

  it("ne pose le fond de la fenêtre sur aucun contrôle", () => {
    // --void et --shell habillent la fenêtre et ses panneaux. Un contrôle qui
    // les reprend disparaît dans son support : c'est ce qui rendait le bouton
    // « Gérer » et la liste des Réglages indiscernables du fond.
    const offenders = controls.filter(({ body }) =>
      /background:\s*var\(--(void|shell)\)/.test(body),
    );

    expect(report(offenders)).toEqual([]);
  });

  it("laisse un bouton à icône s'élargir pour sa demande de confirmation", () => {
    // ConfirmButton remplace l'icône par un mot. Sans cette règle, le mot doit
    // tenir dans le carré de --btn-height : la page des chats archivés a perdu
    // cet élargissement pendant l'unification, et le texte s'y écrasait.
    const rule = buttonsCss.match(
      /\.icon-btn\[data-confirming="true"\]\s*\{([^}]*)\}/,
    );

    expect(rule).not.toBeNull();
    expect(rule?.[1]).toMatch(/width:\s*auto/);
  });
});

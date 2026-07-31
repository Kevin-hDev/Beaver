import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

/* Une ligne de navigation ne bouge pas quand on la survole : seules ses
   couleurs changent.

   Les sessions glissaient de 2 px vers la droite au survol, ce qui se lisait
   comme un soulèvement de la ligne sous le curseur — d'autant plus visible que
   ni les projets ni les onglets des Réglages, juste à côté, ne bougeaient. */

const NAV_ROWS = [
  "conv-item",
  "conv-new-btn",
  "conv-project-header",
  "settings-subtab",
  "pers-item",
  "sb-nav-item",
];

function cssFiles(dir: string): string[] {
  // Les chemins parcourus viennent de l'arborescence du dépôt, pas d'une entrée.
  // eslint-disable-next-line security/detect-non-literal-fs-filename
  return readdirSync(dir).flatMap((entry) => {
    const path = join(dir, entry);
    // eslint-disable-next-line security/detect-non-literal-fs-filename
    if (statSync(path).isDirectory()) return cssFiles(path);
    return path.endsWith(".css") ? [path] : [];
  });
}

interface Rule {
  file: string;
  selector: string;
  body: string;
}

const rules: Rule[] = cssFiles("src").flatMap((file) => {
  // eslint-disable-next-line security/detect-non-literal-fs-filename
  const css = readFileSync(file, "utf8").replace(/\/\*[\s\S]*?\*\//g, "");
  return [...css.matchAll(/([^{}]+)\{([^{}]*)\}/g)]
    .filter((match) => !match[1].trim().startsWith("@"))
    .map((match) => ({ file, selector: match[1].trim(), body: match[2] }));
});

/* Le sujet d'un sélecteur est sa dernière portion, celle que la règle habille
   réellement : « .conv-item:hover .conv-session-age » agit sur un enfant, et
   déplacer cet enfant ne déplace pas la ligne. */
function subject(selector: string): string {
  return selector.split(/[\s>+~]+/).filter(Boolean).pop() ?? "";
}

function targetsRow(selector: string): boolean {
  return selector.split(",").some((part) => {
    const last = subject(part);
    return NAV_ROWS.some((row) => {
      if (!last.startsWith(`.${row}`)) return false;
      // Sans ce contrôle, « .conv-item » attraperait aussi « .conv-item-tail ».
      const suite = last.slice(row.length + 1);
      return suite === "" || !/[\w-]/.test(suite[0]);
    });
  });
}

const rowRules = rules.filter((rule) => targetsRow(rule.selector));
const conversationCss = readFileSync("src/components/agent-local/conversation.css", "utf8");
const conversationProjectsCss = readFileSync(
  "src/components/agent-local/conversation-projects.css",
  "utf8",
);
const conversationCollapseCss = readFileSync(
  "src/components/agent-local/conversation-collapse.css",
  "utf8",
);

function report(offenders: Rule[]): string[] {
  return offenders.map((rule) => `${rule.file}: ${rule.selector}`);
}

describe("lignes de navigation", () => {
  it("trouve les règles à garder", () => {
    expect(rowRules.length).toBeGreaterThan(NAV_ROWS.length);
  });

  it("ne déplace aucune ligne au survol", () => {
    const offenders = rowRules
      .filter((rule) => rule.selector.includes(":hover"))
      .filter((rule) => /transform\s*:/.test(rule.body));

    expect(report(offenders)).toEqual([]);
  });

  it("n'anime que les couleurs", () => {
    // « transition: all » anime aussi la géométrie : toute variation de
    // hauteur, de marge ou de largeur se joue alors comme un mouvement.
    const offenders = rowRules.filter((rule) => /transition:\s*all\b/.test(rule.body));

    expect(report(offenders)).toEqual([]);
  });

  it("échange l'âge et le menu sans animation d'opacité WebKit", () => {
    expect(conversationCss).toMatch(
      /\.conv-session-age\s*\{[^}]*visibility:\s*visible;/s,
    );
    expect(conversationCss).toMatch(
      /\.conv-session-age\s*\{[^}]*transition:\s*none;/s,
    );
    expect(conversationProjectsCss).toMatch(
      /\.conv-session-menu-btn\s*\{[^}]*visibility:\s*hidden;/s,
    );
    expect(conversationProjectsCss).toMatch(
      /\.conv-item:hover .conv-session-age\s*\{[^}]*visibility:\s*hidden;/s,
    );
    expect(conversationProjectsCss).toMatch(
      /\.conv-item:hover .conv-session-menu-btn\s*\{[^}]*visibility:\s*visible;/s,
    );
    expect(conversationProjectsCss).not.toMatch(
      /\.conv-session-menu-btn\s*\{[^}]*transition:[^;}]*opacity/s,
    );
  });

  it("affiche les actions de projet sans recomposer les icônes de dossier", () => {
    expect(conversationProjectsCss).toMatch(
      /\.conv-project-actions\s*\{[^}]*visibility:\s*hidden;/s,
    );
    expect(conversationProjectsCss).toMatch(
      /\.conv-project-header:hover .conv-project-actions\s*\{[^}]*visibility:\s*visible;/s,
    );
    expect(conversationProjectsCss).not.toMatch(
      /\.conv-project-actions\s*\{[^}]*transition:/s,
    );
    expect(conversationCollapseCss).toMatch(
      /\.conv-folder-icon\s*\{\s*transition:\s*color 400ms ease;\s*\}/s,
    );
    expect(conversationCollapseCss).not.toMatch(
      /\.conv-folder-icon\s*\{[^}]*\b(?:opacity|transform)\b/s,
    );
  });
});

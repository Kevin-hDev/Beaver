import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

/* La gouttière d'une liste se pose sur le conteneur, en marge interne.

   Posée à l'envers — en marge externe sur la ligne — elle déborde dès que la
   ligne mesure toute la largeur de son conteneur : « width: 100% » couvre déjà
   la carte entière, et la marge la décale en plus. Le fond de survol sortait
   ainsi de quatre pixels sur la droite dans la liste des fournisseurs
   configurés, et se faisait rogner d'autant dans celle des sujets de mémoire,
   dont le conteneur coupait ce qui dépassait.

   Relevé du 4 septembre 2026 : onze règles posaient la gouttière en marge
   externe, quatre la combinaient avec la largeur pleine. Ce test ne juge pas la
   marge externe seule — une ligne qui se rétrécit d'elle-même s'en accommode —
   mais interdit le mélange des deux, qui est le défaut. */

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

const offenders: string[] = [];

for (const file of cssFiles("src")) {
  // eslint-disable-next-line security/detect-non-literal-fs-filename
  const css = readFileSync(file, "utf8").replace(/\/\*[\s\S]*?\*\//g, "");
  for (const rule of css.matchAll(/([^{}]+)\{([^{}]*)\}/g)) {
    const body = rule[2];
    const gutterMargin = /(^|[\s;])margin[^:;]*:[^;]*--list-gutter/.test(body);
    const fullWidth = /(^|[\s;])width:\s*100%/.test(body);
    if (gutterMargin && fullWidth) {
      offenders.push(`${file} — ${rule[1].trim().split("\n").pop()?.trim()}`);
    }
  }
}

describe("gouttière de liste", () => {
  it("n'est jamais une marge externe sur une ligne pleine largeur", () => {
    expect(offenders, offenders.join("\n")).toEqual([]);
  });

  it("est portée en marge interne par la carte des listes de réglages", () => {
    const card = readFileSync("src/components/settings/settings-card.css", "utf8");
    expect(card).toContain(".settings-card-list {");
    expect(card).toContain("padding: var(--list-gutter);");
  });
});

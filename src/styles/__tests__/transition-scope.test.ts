import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

/* Une transition dit toujours ce qu'elle anime.

   « transition-duration » posé seul laisse « transition-property » à sa valeur
   par défaut, « all » : la couleur s'anime, mais la position, la hauteur et les
   marges aussi. Le moindre changement de mise en page se joue alors comme un
   glissement.

   C'est ce qui faisait bouger toute la liste des conversations : le libellé de
   section ouvrait chaque groupe et animait sa géométrie sur 400 ms, si bien que
   les sessions et leurs icônes glissaient avec lui. */

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

/* Le bloc « mouvement réduit » coupe volontairement toutes les transitions de
   la page : c'est le seul endroit où viser « all » est l'intention. */
function isUniversal(selector: string): boolean {
  return selector === "*" || selector.startsWith("*::");
}

const named = new Set<string>();
const timed: { file: string; selector: string }[] = [];

for (const file of cssFiles("src")) {
  // eslint-disable-next-line security/detect-non-literal-fs-filename
  const css = readFileSync(file, "utf8").replace(/\/\*[\s\S]*?\*\//g, "");
  for (const rule of css.matchAll(/([^{}]+)\{([^{}]*)\}/g)) {
    const selector = rule[1].trim();
    if (selector.startsWith("@")) continue;

    const declarations = rule[2].split(";").map((line) => line.trim());
    const namesProperty = declarations.some((line) =>
      /^transition(-property)?\s*:/.test(line),
    );
    const setsTiming = declarations.some((line) =>
      /^transition-(duration|delay)\s*:/.test(line),
    );

    for (const part of selector.split(",")) {
      const key = part.trim();
      if (namesProperty) named.add(key);
      if (setsTiming && !isUniversal(key)) timed.push({ file, selector: key });
    }
  }
}

describe("portée des transitions", () => {
  it("trouve des règles à vérifier", () => {
    expect(timed.length).toBeGreaterThan(0);
    expect(named.size).toBeGreaterThan(0);
  });

  it("ne laisse aucune durée sans propriété nommée", () => {
    const offenders = timed
      .filter(({ selector }) => !named.has(selector))
      .map(({ file, selector }) => `${file}: ${selector}`);

    expect(offenders).toEqual([]);
  });
});

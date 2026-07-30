import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

/* Quand le dépliage au survol est désactivé, la barre garde sa largeur — mais
   les règles de dépliage, elles, restent posées par le CSS. Il faut donc que le
   mode verrouillé les neutralise toutes.

   Une seule oubliée suffit : la largeur manquait, et le fond de l'élément actif
   s'élargissait de 24 à 31,5 px puis se décalait de 3,8 px au survol, alors que
   rien d'autre ne bougeait à l'écran. */

const css = readFileSync("src/components/layout/app-layout.css", "utf8");

/* Les règles sont relevées par découpage du fichier, pas par une expression
   construite à partir du sélecteur : celui-ci contient des caractères qui ont
   un sens en expression régulière. */
const RULES = new Map(
  [...css.replace(/\/\*[\s\S]*?\*\//g, "").matchAll(/([^{}]+)\{([^{}]*)\}/g)].map((match) => [
    match[1].trim().replace(/\s+/g, " "),
    match[2],
  ]),
);

function declarations(selector: string): Set<string> {
  return new Set(
    (RULES.get(selector) ?? "")
      .split(";")
      .map((line) => line.split(":")[0].trim())
      .filter(Boolean),
  );
}

const deplie = declarations(".group\\/sb:hover .sb-nav-item");
const verrouille = declarations(".sb-locked:hover .sb-nav-item");
const repos = declarations(".group\\/sb:not(:hover) .sb-nav-item");

describe("barre latérale, dépliage désactivé", () => {
  it("trouve les trois règles de survol", () => {
    expect(deplie.size).toBeGreaterThan(0);
    expect(verrouille.size).toBeGreaterThan(0);
    expect(repos.size).toBeGreaterThan(0);
  });

  it("neutralise chaque propriété posée par le dépliage", () => {
    const oubliees = [...deplie].filter((prop) => !verrouille.has(prop));

    expect(oubliees).toEqual([]);
  });

  it("rend le centrage que le survol retire", () => {
    // Au repos, l'élément est centré par des marges automatiques. Le survol
    // fait tomber cette règle : le mode verrouillé doit reposer le centrage,
    // sans quoi l'élément glisse vers la gauche.
    const manquantes = [...repos].filter((prop) => !verrouille.has(prop));

    expect(manquantes).toEqual([]);
  });
});

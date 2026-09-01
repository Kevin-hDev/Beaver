import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

/* WebKit — le moteur de rendu de l'application — interpole une piste de grille
   en `fr` par paliers : le mouvement hache au début du dépliement et à la fin
   du repliement, là où la courbe est la plus lente. <Collapsible> anime une
   hauteur en pixels pour cette raison, et fait autorité. */

function sheetsUnder(root: string): string[] {
  const found: string[] = [];
  // eslint-disable-next-line security/detect-non-literal-fs-filename -- descente sous une racine fixe
  for (const entry of readdirSync(root)) {
    const path = join(root, entry);
    // eslint-disable-next-line security/detect-non-literal-fs-filename -- descente sous une racine fixe
    if (statSync(path).isDirectory()) found.push(...sheetsUnder(path));
    else if (entry.endsWith(".css")) found.push(path);
  }
  return found;
}

describe("Blocs repliables de forecast", () => {
  it("n'anime aucune piste de grille", () => {
    const offenders = sheetsUnder("src/components/forecast").filter((path) =>
      // eslint-disable-next-line security/detect-non-literal-fs-filename -- chemins issus de la descente ci-dessus
      /transition:[^;]*grid-template-rows/.test(readFileSync(path, "utf8")),
    );
    expect(offenders).toEqual([]);
  });

  it("confie chaque zone repliable à la primitive", () => {
    const users = [
      "src/components/forecast/forecast-view-filter-items.tsx",
      "src/components/forecast/sections/forecast-analysis-accordion.tsx",
      "src/components/forecast/sections/forecast-view.tsx",
      "src/components/forecast/charts/forecast-chart-card.tsx",
    ];
    for (const path of users) {
      // eslint-disable-next-line security/detect-non-literal-fs-filename -- liste fixe déclarée ci-dessus
      const source = readFileSync(path, "utf8");
      expect(source, path).toContain('from "@/components/ui/collapsible"');
    }
  });
});

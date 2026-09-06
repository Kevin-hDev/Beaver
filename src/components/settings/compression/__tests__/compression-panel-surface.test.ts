import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

/* Contrat de dialogue : le panneau prend --popover-bg, et le voile qui le porte
   bascule ce jeton à l'opaque. Les deux vont ensemble — un panneau conforme
   sous un voile non inscrit hérite du verre translucide de body et laisse lire
   la page à travers. C'est ce qui est arrivé ici : la liste des voiles date du
   30 août, ce panneau du 31, et il ne s'y était pas inscrit. */

const PANEL = readFileSync("src/components/settings/compression/compression-panel.css", "utf8");
const LAYOUT = readFileSync("src/components/layout/app-layout.css", "utf8");

function bloc(css: string, selecteur: string): string {
  const debut = css.indexOf(`\n${selecteur} {`);
  expect(debut, `${selecteur} absent`).toBeGreaterThan(-1);
  return css.slice(debut, css.indexOf("}", debut));
}

describe("surface des dialogues de compression", () => {
  it.each([".cpa-dialog", ".cpd-dialog"])(
    "%s prend le fond du contrat plutôt qu'une couleur à lui",
    (selecteur) => {
      const regle = bloc(PANEL, selecteur);
      expect(regle).toContain("background: var(--popover-bg,");
    },
  );

  it("inscrit les deux voiles dans la liste qui bascule les jetons à l'opaque", () => {
    const debut = LAYOUT.indexOf(".wk-dialog-overlay,");
    const liste = LAYOUT.slice(debut, LAYOUT.indexOf("}", debut));
    expect(liste).toContain(".cpa-overlay,");
    expect(liste).toContain(".cpd-overlay,");
    expect(liste).toContain("--popover-bg: var(--shell-opaque);");
  });

  /* Le jeton vaut none sur Linux et Windows, volontairement : une valeur écrite
     en dur y ajoutait un flou que personne n'a demandé. */
  it("laisse le flou du voile au jeton du système", () => {
    const regle = bloc(PANEL, ".cpa-backdrop-dismiss,\n.cpd-backdrop-dismiss");
    expect(regle).toContain("backdrop-filter: var(--dialog-overlay-filter, none);");
    expect(regle).not.toMatch(/backdrop-filter:\s*blur\(/);
  });
});

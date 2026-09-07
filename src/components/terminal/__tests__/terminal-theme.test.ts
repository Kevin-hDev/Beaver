import { describe, expect, it } from "vitest";
import { toXtermColor } from "../terminal-theme";

/* xterm n'analyse lui-même que `#rrggbb[aa]` et `rgb()/rgba()` séparés par des
   virgules. Tout le reste passe par un essai sur un canevas qui rejette les
   couleurs translucides — et les jetons de Beaver se calculent en
   `color(srgb …)`, parce qu'ils sont faits au `color-mix`. La sélection du
   terminal était donc refusée en silence et remplacée par le blanc d'usine. */

describe("couleurs données à xterm", () => {
  it("traduit une couleur translucide, celle de la sélection", () => {
    expect(toXtermColor("color(srgb 1 0.541176 0.298039 / 0.32)")).toBe(
      "rgba(255, 138, 76, 0.32)",
    );
  });

  it("traduit une couleur opaque sans inventer d'opacité", () => {
    expect(toXtermColor("color(srgb 0.2 0.4 0.6)")).toBe("rgb(51, 102, 153)");
  });

  it("accepte l'opacité écrite en pourcentage", () => {
    expect(toXtermColor("color(srgb 0 0 0 / 50%)")).toBe("rgba(0, 0, 0, 0.5)");
  });

  it("traduit l'écriture moderne séparée par des espaces", () => {
    expect(toXtermColor("rgb(255 138 76 / 0.32)")).toBe("rgba(255, 138, 76, 0.32)");
  });

  it("laisse intactes les écritures que xterm sait déjà lire", () => {
    expect(toXtermColor("rgb(255, 138, 76)")).toBe("rgb(255, 138, 76)");
    expect(toXtermColor("rgba(255, 138, 76, 0.32)")).toBe("rgba(255, 138, 76, 0.32)");
    expect(toXtermColor("#ff8a4c")).toBe("#ff8a4c");
  });

  /* Le format retenu doit passer la propre expression de xterm : des entiers,
     séparés par des virgules, opacité entre 0 et 1. */
  it("produit une écriture que xterm analyse sans passer par le canevas", () => {
    /* Copie exacte de l'expression de xterm (Color.ts), appliquée aux seules
       chaînes que ce test produit lui-même : c'est ce qu'on vérifie, pas ce
       qu'on exécute. */
    const xterm =
      // eslint-disable-next-line security/detect-unsafe-regex -- expression de xterm reproduite telle quelle
      /rgba?\(\s*(\d{1,3})\s*,\s*(\d{1,3})\s*,\s*(\d{1,3})\s*(,\s*(0|1|\d?\.(\d+))\s*)?\)/;

    expect(toXtermColor("color(srgb 1 0.541176 0.298039 / 0.32)")).toMatch(xterm);
    expect(toXtermColor("color(srgb 0.2 0.4 0.6)")).toMatch(xterm);
  });
});

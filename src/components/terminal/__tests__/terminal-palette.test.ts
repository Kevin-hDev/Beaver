import { describe, expect, it } from "vitest";
import { ansiColors, backgroundOf, THEMES, type Rgb } from "../test-utils/palette-resolve";

/* Les seize couleurs du terminal sont dérivées des jetons des six thèmes. Deux
   d'entre eux donnaient la même valeur au bleu et au cyan, un troisième les
   rapprochait à vingt : des couleurs de sortie distinctes s'affichaient à
   l'identique, et rien ne le signalait. */

function distance(a: Rgb, b: Rgb): number {
  return Math.abs(a.r - b.r) + Math.abs(a.g - b.g) + Math.abs(a.b - b.b);
}

/* Somme des écarts sur les trois canaux. En deçà, deux teintes se confondent à
   l'œil dans un texte en petit corps. */
const MIN_DISTANCE = 24;

/* Le noir ANSI est volontairement proche du fond — c'est son rôle. Les quinze
   autres doivent s'en détacher assez pour rester lisibles. */
const MIN_AGAINST_BACKGROUND = 60;

describe("les seize couleurs du terminal", () => {
  it.each(THEMES)("sont toutes résolues dans le thème %s", (theme) => {
    expect(ansiColors(theme).size).toBe(16);
    expect(backgroundOf(theme)).not.toBeNull();
  });

  it.each(THEMES)("ne se confondent deux à deux dans aucun thème (%s)", (theme) => {
    const colors = [...ansiColors(theme)];
    const tooClose: string[] = [];

    for (let i = 0; i < colors.length; i += 1) {
      for (let j = i + 1; j < colors.length; j += 1) {
        const [nameA, a] = colors[i];
        const [nameB, b] = colors[j];
        if (distance(a, b) < MIN_DISTANCE) {
          tooClose.push(`${nameA} ≈ ${nameB} (${Math.round(distance(a, b))})`);
        }
      }
    }

    expect(tooClose).toEqual([]);
  });

  it.each(THEMES)("se détachent du fond dans le thème %s", (theme) => {
    const background = backgroundOf(theme)!;
    const unreadable: string[] = [];

    for (const [name, color] of ansiColors(theme)) {
      if (name === "black") continue;
      if (distance(color, background) < MIN_AGAINST_BACKGROUND) {
        unreadable.push(`${name} (${Math.round(distance(color, background))})`);
      }
    }

    expect(unreadable).toEqual([]);
  });
});
